//! RenderGraph

use super::{
    node::{Node, NodeId, NodeImpl, NodeLabel},
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use bevy::{ecs::system::SystemParam, prelude::World};
use pi_async::prelude::AsyncRuntime;
use pi_render::{
    depend_graph::graph::DependGraph,
    rhi::{device::RenderDevice, RenderQueue},
};
use std::borrow::Cow;

/// 渲染图
pub struct RenderGraph {
    device: RenderDevice,
    queue: RenderQueue,

    imp: DependGraph<World>,
}

/// 渲染图的 拓扑信息 相关 方法
impl RenderGraph {
    /// 创建
    #[inline]
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        Self {
            device,
            queue,
            imp: Default::default(),
        }
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
        };

        let node = NodeImpl::<I, O, R, P>::new(node, context);
        self.imp.add_node(name, node)
    }

    /// 移除 节点
    #[inline]
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<(), GraphError> {
        self.imp.remove_node(label)
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
        self.imp.run(rt, world).await
    }
}
