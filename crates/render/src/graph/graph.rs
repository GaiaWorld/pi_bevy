//! RenderGraph

use super::{
    node::{Node, NodeId, NodeImpl, NodeLabel},
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use crate::{
    clear_node::ClearNode,
    node::{AsyncQueue, AsyncTaskQueue, NodeContext, TaskQueue},
    state_pool::SystemStatePool,
    CmdBuffers, CLEAR_WIDNOW_NODE,
};
use bevy::ecs::{system::SystemParam, world::World};
use pi_async_rt::prelude::{AsyncRuntime, AsyncValueNonBlocking};
use pi_render::{
    depend_graph::graph::DependGraph,
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_share::{Share, ShareMutex};
use std::{borrow::Cow, collections::VecDeque, mem::transmute, sync::atomic::AtomicBool};
/// 渲染图
pub struct RenderGraph {
    device: RenderDevice,
    queue: RenderQueue,
    webgl_cmd_buffers: CmdBuffers,

    state_pool: SystemStatePool,

    node_count: u32,

    imp: DependGraph<NodeContext>,

    async_submit_queue: TaskQueue,
}

/// 渲染图的 拓扑信息 相关 方法
impl RenderGraph {
    /// + Debug 模式
    ///     - windwos 非 wasm32 平台，运行目录 生成 dump_graphviz.dot
    ///     - 其他 平台，返回 字符串
    /// + Release 模式：返回 空串
    pub fn dump_graphviz(&self) -> String {
        self.imp.dump_graphviz()
    }

    /// 创建
    #[inline]
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mut graph = Self {
            device,
            queue,
            webgl_cmd_buffers: CmdBuffers::default(),

            node_count: 0,
            imp: Default::default(),
            state_pool: SystemStatePool::default(),
            async_submit_queue: TaskQueue(Share::new(ShareMutex::new(VecDeque::new()))),
        };

        // 一开始，就将 Clear 扔到 graph
        // 注：每帧 必须运行一次 窗口的 清屏，否则 wgpu 会报错
        let clear_node = ClearNode;
        graph.add_node(CLEAR_WIDNOW_NODE, clear_node).unwrap();
        graph.set_finish(CLEAR_WIDNOW_NODE, true).unwrap();

        graph
    }

    /// 查 指定节点 的 前驱节点
    #[inline]
    pub fn get_prev_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.imp.get_prev_ids(id)
    }

    /// 查 指定节点 的 后继节点
    #[inline]
    pub fn get_next_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.imp.get_next_ids(id)
    }

    /// 添加 名为 name 的 节点
    #[inline]
    pub fn add_node<I, O, R, P>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: Node<Param = P, Input = I, Output = O>,
        P: SystemParam + 'static,
    {
        let context = RenderContext {
            device: self.device.clone(),
            queue: self.queue.clone(),
            webgl_cmd_buffers: self.webgl_cmd_buffers.clone(),
        };

        let node = NodeImpl::<I, O, R, P>::new(node, context, self.state_pool.clone());
        let r = self.imp.add_node(name, node);

        if r.is_ok() {
            self.node_count += 1;

            if self.node_count > 1 {
                let id = *r.as_ref().unwrap();
                // 清屏节点 在 所有节点 之前
                self.add_depend(CLEAR_WIDNOW_NODE, id).unwrap();
                // // 两个以上的节点，清屏节点设置为 非终止节点
                // self.set_finish(CLEAR_WIDNOW_NODE, false).unwrap();
            }
        }
        r
    }

    /// 移除 节点
    #[inline]
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<(), GraphError> {
        let r = self.imp.remove_node(label);
        if r.is_ok() {
            self.node_count -= 1;
            if self.node_count == 1 {
                self.set_finish(CLEAR_WIDNOW_NODE, true).unwrap();
            }
        }
        r
    }

    /// 建立 Node 的 依赖关系
    /// 执行顺序 `before` 先于 `after`
    #[inline]
    pub fn add_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        after_label: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        self.imp.add_depend(before_label, after_label)
    }

    /// 移除依赖
    #[inline]
    pub fn remove_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        after_label: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        self.imp.remove_depend(before_label, after_label)
    }

    /// 设置 是否 是 最终节点
    #[inline]
    pub fn set_finish(
        &mut self,
        label: impl Into<NodeLabel>,
        is_finish: bool,
    ) -> Result<(), GraphError> {
        self.imp.set_finish(label, is_finish)
    }
}

/// 渲染图的 执行 相关
impl RenderGraph {
    #[inline]
    pub fn build(&mut self) -> Result<(), GraphError> {
        self.imp.build()
    }

    /// 执行 渲染
    #[inline]
    pub async fn run<'a, A: AsyncRuntime>(
        &'a mut self,
        rt: &'a A,
        world: &'static World,
    ) -> Result<(), GraphError> {
        let async_submit_queue = self.async_submit_queue.clone();

        let task_queue = AsyncTaskQueue {
            queue: async_submit_queue,
            is_runing: Share::new(AtomicBool::new(false)),
            rt: rt.clone(),
        };

        // 运行 渲染图
        let context = NodeContext::new(world, Box::new(task_queue.clone()));
        let ret = self
            .imp
            .run(rt, unsafe {
                transmute::<_, &'static NodeContext>(&context)
            })
            .await;

        // 用 异步值 等待 队列的 提交 全部完成
        #[cfg(not(feature = "webgl"))]
        {
            let wait: AsyncValueNonBlocking<()> = AsyncValueNonBlocking::new();
            let wait1 = wait.clone();
            task_queue.push(Box::pin(async move {
                wait1.set(());
            }));
            wait.await;
        }

        #[cfg(feature = "webgl")]
        {
            let cmds = webgl_cmd_buffers.replace_with_new_buffer();
            self.queue.submit(cmds);
        }

        ret
    }
}
