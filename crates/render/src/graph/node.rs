//! 利用 DependGraph 实现 渲染图
use super::{
    param::{InParam, OutParam},
    RenderContext,
};
use bevy_ecs::{system::{SystemParam, SystemState}, world::World};
use pi_futures::BoxFuture;
use pi_render::depend_graph::node::DependNode;
use pi_share::{Share, ShareRefCell, ThreadSync};
use tracing::Instrument;
use wgpu::CommandEncoder;

pub use pi_render::depend_graph::node::{NodeId, NodeLabel, ParamUsage};

/// 渲染节点，给 外部 扩展 使用
pub trait Node: 'static + ThreadSync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// Bevy 系统参数
    type Param: SystemParam + 'static;

    /// 构建，当渲染图 构建时候，会调用一次
    /// 一般 用于 准备 渲染 资源的 创建
    fn build<'a>(
        &'a mut self,
        _world: &'a World,
        _param: &'a mut SystemState<Self::Param>,
        _context: RenderContext,
        _usage: &'a ParamUsage,
    ) -> Result<(), String> {
        Ok(())
    }

    /// 执行，每帧会调用一次
    fn run<'a>(
        &'a mut self,
        world: &'a World,
        param: &'a mut SystemState<Self::Param>,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>>;
}

// ====================== crate内 使用的 数据结构

pub(crate) struct NodeImpl<I, O, R, P>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Param = P, Input = I, Output = O>,
    P: SystemParam + 'static,
{
    node: R,
    state: Option<SystemState<P>>,
    context: RenderContext,
}

impl<I, O, R, P> NodeImpl<I, O, R, P>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Param = P, Input = I, Output = O>,
    P: SystemParam + 'static,
{
    #[inline]
    pub(crate) fn new(node: R, context: RenderContext) -> Self {
        Self {
            node,
            context,
            state: None,
        }
    }
}

impl<I, O, R, P> DependNode<World> for NodeImpl<I, O, R, P>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Param = P, Input = I, Output = O>,
    P: SystemParam + 'static,
{
    type Input = I;
    type Output = O;

    #[inline]
    fn build<'a>(&'a mut self, world: &'a World, usage: &'a ParamUsage) -> Result<(), String> {
        if self.state.is_none() {
            let w_ptr = world as *const World as usize;
            let world = unsafe { std::mem::transmute(w_ptr) };
            self.state = Some(SystemState::new(world));
        }

        self.node.build(
            world,
            self.state.as_mut().unwrap(),
            self.context.clone(),
            usage,
        )
    }

    #[inline]
    fn run<'a>(
        &'a mut self,
        world: &'a World,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        let context = self.context.clone();
		let mut task = async move {
            // 每节点 一个 CommandEncoder
            let commands = self
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let commands = ShareRefCell::new(commands);

            let output = self
                .node
                .run(
                    world,
                    self.state.as_mut().unwrap(),
                    context,
                    commands.clone(),
                    input,
                    usage,
                );
			#[cfg(feature = "trace")]
			let output = output.instrument(tracing::info_span!("GraphNode run"));
			let output = output.await.unwrap();

            // CommandEncoder --> CommandBuffer
            let commands = Share::try_unwrap(commands.0).unwrap();
            let commands = commands.into_inner();

            // CommandBuffer --> Queue
			#[cfg(not(feature = "trace"))]
            self.context.queue.submit(vec![commands.finish()]);

			#[cfg(feature = "trace")]
			async {
				self.context.queue.submit(vec![commands.finish()]);
			}.instrument(tracing::info_span!("submit")).await;

            Ok(output)
        };

        Box::pin(task)
    }
}
