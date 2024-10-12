//! RenderGraph

use super::{
    node::{Node, NodeId, NodeImpl, NodeLabel},
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use crate::{
    clear_node::ClearNode,
    node::{AsyncTaskQueue, NodeContext, ShareTaskQueue, TaskQueue},
    state_pool::SystemStatePool,
    CLEAR_WIDNOW_NODE,
};
// use bevy_ecs::{system::SystemParam, world::World};
use pi_async_rt::prelude::AsyncRuntime;
use pi_render::{
    depend_graph::{graph::DependGraph, graph_data::NGraph},
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_share::{Share, ShareMutex, ShareRefCell};
use pi_world::{prelude::SystemParam, world::World};
use std::{borrow::Cow, mem::transmute, sync::atomic::AtomicBool};
use pi_null::Null;
/// 渲染图
pub struct RenderGraph {
    device: RenderDevice,
    queue: RenderQueue,
    commands: ShareRefCell<Option<ShareRefCell<wgpu::CommandEncoder>>>,

    state_pool: SystemStatePool,

    node_count: u32,

    imp: DependGraph<NodeContext>,

    async_submit_queue: ShareTaskQueue,
}
#[cfg(all(not(feature = "webgl"),not(feature = "single_thread")))]
use crate::node::AsyncQueue;
#[cfg(all(not(feature = "webgl"),not(feature = "single_thread")))]
use pi_async_rt::prelude::AsyncValueNonBlocking;

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
            commands: ShareRefCell::new(None),

            node_count: 0,
            imp: Default::default(),
            state_pool: SystemStatePool::default(),
            async_submit_queue: ShareTaskQueue(Share::new(ShareMutex::new(TaskQueue::new()))),
        };

        // 一开始，就将 Clear 扔到 graph
        // 注：每帧 必须运行一次 窗口的 清屏，否则 wgpu 会报错
        let clear_node = ClearNode;
        graph.add_node(CLEAR_WIDNOW_NODE, clear_node, NodeId::null()).unwrap();
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
    pub fn add_node<I, O, R, BP, RP>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
		parent_graph_id: NodeId,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
        BP: SystemParam + 'static,
		RP: SystemParam + 'static,
    {
        let context = RenderContext {
            device: self.device.clone(),
            queue: self.queue.clone(),
            commands: self.commands.clone(),
        };

        let node = NodeImpl::<I, O, R, BP, RP>::new(node, context, self.state_pool.clone());
        let r = self.imp.add_node(name, node, parent_graph_id, true);

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

	/// 添加一个不运行的节点
    #[inline]
    pub fn add_node_not_run<I, O, R, BP, RP>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
		parent_graph_id: NodeId,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
        BP: SystemParam + 'static,
		RP: SystemParam + 'static,
    {
        let context = RenderContext {
            device: self.device.clone(),
            queue: self.queue.clone(),
            commands: self.commands.clone(),
        };

        let node = NodeImpl::<I, O, R, BP, RP>::new(node, context, self.state_pool.clone());
        let r = self.imp.add_node(name, node, parent_graph_id, false);

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

	/// 添加 名为 name 的 节点
	#[inline]
	pub fn add_sub_graph(
		&mut self,
		name: impl Into<Cow<'static, str>>,
	) -> Result<NodeId, GraphError>{
		let r = self.imp.add_sub_graph(name);
		if r.is_ok() {
			let id = *r.as_ref().unwrap();
			// 清屏节点 在 所有节点 之前
			self.add_depend(CLEAR_WIDNOW_NODE, id).unwrap();
		}
		r
	}

	/// 设置子图的父图， 只能在该图与其他节点创建连接关系之前设置， 否则设置不成功
	pub fn set_sub_graph_parent(&mut self, k: NodeId, parent_graph_id: NodeId) {
		if !parent_graph_id.is_null() {
			let _ = self.remove_depend(CLEAR_WIDNOW_NODE, k);
		}

		self.imp.set_sub_graph_parent(k, parent_graph_id);
	}

    /// 移除 节点
    #[inline]
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<NodeId, GraphError> {
        let r = self.imp.remove(label);
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

	/// 取到入度节点
	#[inline]
	pub fn before_nodes(
        &self,
        label: impl Into<NodeLabel>
	) -> Result<&[NodeId], GraphError> {
        self.imp.before_nodes(label)
    }

	/// 取到出现度节点
	#[inline]
	pub fn after_nodes(
        &self,
        label: impl Into<NodeLabel>
	) -> Result<&[NodeId], GraphError> {
        self.imp.after_nodes(label)
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
    // #[inline]
    // pub fn build(&mut self) -> Result<(), GraphError> {
    //     self.imp.build()
    // }

    /// 执行 渲染
    #[inline]
    pub async fn run<'a, A: AsyncRuntime>(
        &'a mut self,
        rt: &'a A,
        world: &'static mut World,
    ) -> Result<(), GraphError> {
        let async_submit_queue = self.async_submit_queue.clone();

        let task_queue = AsyncTaskQueue {
            queue: async_submit_queue,
            is_runing: Share::new(AtomicBool::new(false)),
            rt: rt.clone(),
        };

		#[cfg(any(feature = "webgl",feature = "single_thread"))]
		{
			let commands = {
				let c = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
				Some(ShareRefCell::new(c))
			};
			*self.commands.0.borrow_mut() = commands;
		}

        // 运行 渲染图
        let mut context = NodeContext::new(world, Box::new(task_queue.clone()));
        let ret = self
            .imp
            .run(rt, unsafe {
                transmute::<_, &'static mut NodeContext>(&mut context)
            })
            .await;
		// 图执行完毕， 通知外部可进行下一步渲染
		
		let node_count = self.imp.can_run_count();
        // 用 异步值 等待 队列的 提交 全部完成
        #[cfg(all(not(feature = "webgl"),not(feature = "single_thread")))]
        {
            let wait: AsyncValueNonBlocking<()> = AsyncValueNonBlocking::new();
            let wait1 = wait.clone();
            task_queue.push(node_count, Box::pin(async move {
                wait1.set(());
            }));
            wait.await;
			task_queue.reset();
        }

        #[cfg(any(feature = "webgl",feature = "single_thread"))]
        {
            let mut cmd_ref = self.commands.0.borrow_mut();
			let r = std::mem::replace(&mut *cmd_ref, None);
			let cmd = Share::into_inner(r.unwrap().0).unwrap().into_inner();
            self.queue.submit(vec![cmd.finish()]);
        }

        ret
    }

	/// 执行 构建
    #[inline]
    pub fn build<'a, A: AsyncRuntime>(
        &'a mut self,
        rt: &'a A,
        world: &'static mut World,
    ) -> Result<(), GraphError> {
        let async_submit_queue = self.async_submit_queue.clone();

        let task_queue = AsyncTaskQueue {
            queue: async_submit_queue,
            is_runing: Share::new(AtomicBool::new(false)),
            rt: rt.clone(),
        };


        // 运行 渲染图
        let mut context = NodeContext::new(world, Box::new(task_queue.clone()));
        let ret = self
            .imp
            .build(unsafe {
                transmute::<_, &'static mut NodeContext>(&mut context)
            });
        ret
    }

	/// 更新图
    #[inline]
    pub fn update(&mut self) -> Result<(), GraphError> {
		self
            .imp
            .update()
    }

	pub fn schedule_graph(&self) -> &NGraph<NodeId, ()> {
		self
            .imp
            .schedule_graph()
	}
}
