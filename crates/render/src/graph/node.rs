//! 利用 DependGraph 实现 渲染图
use std::{collections::VecDeque, sync::atomic::{AtomicBool, Ordering}};

use super::{
    param::{InParam, OutParam},
    RenderContext,
};
use bevy::ecs::{system::{SystemParam, SystemState}, world::World};
use pi_async::rt::{AsyncRuntime, AsyncValue};
use pi_futures::BoxFuture;
use pi_render::depend_graph::node::DependNode;
use pi_share::{Share, ShareRefCell, ThreadSync, ShareMutex};
use crossbeam::queue::SegQueue;
use wgpu::CommandEncoder;
#[cfg(feature = "trace")]
use tracing::Instrument;

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

pub struct NodeContext {
	world: &'static World,
	pub async_tasks: Box<dyn AsyncQueue>,
}

impl NodeContext {
	pub fn new(world: &'static World, async_tasks: Box<dyn AsyncQueue>) -> Self {
		NodeContext {
			world, 
			async_tasks
		}
	}

	pub fn world(&self) -> &World {
		&*self.world
	}
}

impl<I, O, R, P> DependNode<NodeContext> for NodeImpl<I, O, R, P>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Param = P, Input = I, Output = O>,
    P: SystemParam + 'static,
{
    type Input = I;
    type Output = O;

    #[inline]
    fn build<'a>(&'a mut self, context: &'a NodeContext, usage: &'a ParamUsage) -> Result<(), String> {
        if self.state.is_none() {
            let w_ptr = context.world() as *const World as usize;
            let world = unsafe { std::mem::transmute(w_ptr) };
            self.state = Some(SystemState::new(world));
        }

        self.node.build(
            context.world,
            self.state.as_mut().unwrap(),
            self.context.clone(),
            usage,
        )
    }

    #[inline]
    fn run<'a>(
        &'a mut self,
        c: &'a NodeContext,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        let context = self.context.clone();
        let task = async move {
            // 每节点 一个 CommandEncoder
            let commands = self
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let commands = ShareRefCell::new(commands);

            let output = self.node.run(
                c.world(),
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
			let queue = self.context.queue.clone();
			let submit_task = async move {
				queue.submit(vec![commands.finish()]);
			};
			#[cfg(feature = "trace")]
			let submit_task = submit_task.instrument(tracing::info_span!("submite"));
			c.async_tasks.push(Box::pin(submit_task));

            Ok(output)
        };

        Box::pin(task)
    }
}

pub trait AsyncQueue: Send + Sync + 'static {
	fn push(&self, task: BoxFuture<'static, ()>);
}

#[derive(Clone)]
pub struct AsyncTaskQueue<A: AsyncRuntime> {
	pub queue: Share<ShareMutex<VecDeque<BoxFuture<'static, ()>>>>,
	pub is_runing: Share<AtomicBool>,
	pub rt: A
}

impl<A: AsyncRuntime> AsyncQueue for AsyncTaskQueue<A> {
	fn push(&self, task: BoxFuture<'static, ()>) {
		fn run<A: AsyncRuntime>(queue: Share<ShareMutex<VecDeque<BoxFuture<'static, ()>>>>, rt: A, is_runing: Share<AtomicBool>) {
			let mut t = queue.lock().pop_front();
			if let Some(task) = t {
				let rt1 = rt.clone();
				rt.spawn(rt.alloc(), async move {
					task.await;
					run(queue, rt1, is_runing);
				});
			} else {
				is_runing.store(false, Ordering::Relaxed);
			}
		}
		
		
        let is_start_run = {
			let mut lock = self.queue.lock();
			let is_start_run = lock.is_empty() && self.is_runing.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok();
            lock.push_back(task);
            is_start_run
        };

        if is_start_run {
            // 第一个元素，挨个 执行一次
            run(self.queue.clone(), self.rt.clone(), self.is_runing.clone());
        }
    }
}
