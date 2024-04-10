//! 利用 DependGraph 实现 渲染图
use std::sync::atomic::{AtomicBool, Ordering};

use crate::state_pool::SystemStatePool;

use super::{
    param::{InParam, OutParam},
    RenderContext,
};
use bevy_ecs::{
    system::{SystemParam, SystemState},
    world::World,
};
use derive_deref::{Deref, DerefMut};
use pi_map::vecmap::VecMap;
use pi_async_rt::prelude::AsyncRuntime;
use pi_futures::BoxFuture;
use pi_render::depend_graph::node::DependNode;
use pi_share::{Share, ShareMutex, ShareRefCell, ThreadSync};
#[cfg(feature = "trace")]
use tracing::Instrument;
use wgpu::CommandEncoder;

pub use pi_render::depend_graph::node::{NodeId, NodeLabel, ParamUsage};

/// 渲染节点，给 外部 扩展 使用
pub trait Node: 'static + ThreadSync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// Bevy Build 系统参数
    type BuildParam: SystemParam + 'static;

	/// Bevy Run 系统参数
    type RunParam: SystemParam + 'static;

	fn build<'a>(
        &'a mut self,
        world: &'a mut World,
        param: &'a mut SystemState<Self::BuildParam>,
        context: RenderContext,
		input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId,
		from: &'a [NodeId],
		to: &'a [NodeId],
    ) -> Result<Self::Output, String>;

	// 节点被使用完毕后(所有出度节点的build方法执行完成)， 会调用此方法
	fn reset<'a>(
        &'a mut self,
    ) {}

    /// 执行，每帧会调用一次
    fn run<'a>(
        &'a mut self,
        world: &'a World,
        param: &'a mut SystemState<Self::RunParam>,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId,
		from: &'a [NodeId],
		to: &'a [NodeId],
    ) -> BoxFuture<'a, Result<(), String>>;
}

// ====================== crate内 使用的 数据结构

pub(crate) struct NodeImpl<I, O, R, BP, RP>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
    BP: SystemParam + 'static,
	RP: SystemParam + 'static,
{
    node: R,
    state_pool: SystemStatePool,
    build_state: Option<SystemState<BP>>,
	run_state: Option<SystemState<RP>>,
    context: RenderContext,
}

impl<I, O, R, BP, RP> NodeImpl<I, O, R, BP, RP>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
    BP: SystemParam + 'static,
	RP: SystemParam + 'static,
{
    #[inline]
    pub(crate) fn new(node: R, context: RenderContext, state_pool: SystemStatePool) -> Self {
        Self {
            node,
            context,
            state_pool,
            build_state: None,
			run_state: None,
        }
    }
}

pub struct NodeContext {
    world: &'static mut World,
    pub async_tasks: Box<dyn AsyncQueue>,
}

impl NodeContext {
    pub fn new(world: &'static mut World, async_tasks: Box<dyn AsyncQueue>) -> Self {
        NodeContext { world, async_tasks }
    }

    pub fn world(&self) -> &World {
        &*self.world
    }

	pub fn world_mut(&mut self) -> &mut World {
        &mut *self.world
    }
}

impl<I, O, R, BP, RP> Drop for NodeImpl<I, O, R, BP, RP>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
    BP: SystemParam + 'static,
	RP: SystemParam + 'static,
{
    fn drop(&mut self) {
        // 将 state 拿出来，扔到 state_pool 中
        if let Some(state) = self.build_state.take() {
            self.state_pool.set(state);
        }
		if let Some(state) = self.run_state.take() {
            self.state_pool.set(state);
        }
    }
}

impl<I, O, R, BP, RP> DependNode<NodeContext> for NodeImpl<I, O, R, BP, RP>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<BuildParam = BP, RunParam = RP, Input = I, Output = O>,
    BP: SystemParam + 'static,
	RP: SystemParam + 'static,
{
    type Input = I;
    type Output = O;

    #[inline]
    fn build<'a>(
        &'a mut self,
        context: &'a mut NodeContext,
		input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId,
		from: &'a [NodeId],
		to: &'a [NodeId],
    ) -> Result<O, String> {
		let world = context.world_mut();
        if self.build_state.is_none() {
            self.build_state = self.state_pool.get();
            if self.build_state.is_none() {
                self.build_state = Some(SystemState::new(world));
            }
        }

		if self.run_state.is_none() {
            self.run_state = self.state_pool.get();
            if self.run_state.is_none() {
                self.run_state = Some(SystemState::new(world));
            }
        }

		let c = self.context.clone();

        let r = self.node.build(
			world,
			self.build_state.as_mut().unwrap(),
			c,
			input,
			usage,
			id,
			from,
			to,
        );
        self.state_pool.set(self.build_state.take().unwrap());
        r
    }

    #[inline]
    fn run<'a>(
        &'a mut self,
		index: usize,
        c: &'a NodeContext,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId,
		from: &'a [NodeId],
		to: &'a [NodeId],
    ) -> BoxFuture<'a, Result<(), String>> {
        let context = self.context.clone();
        let task = async move {
            #[cfg(all(not(feature = "webgl"),not(feature = "single_thread")))]
            let commands = {
                // 每节点 一个 CommandEncoder
                let commands = self
                    .context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let commands = ShareRefCell::new(commands);

                commands
            };

            #[cfg(any(feature = "webgl",feature = "single_thread"))]
            let commands = self.context.commands.0.borrow().as_ref().unwrap().clone();

			// pi_hal::runtime::LOGS.lock().0.push("node run before".to_string());
            let output = self.node.run(
                c.world(),
                self.run_state.as_mut().unwrap(),
                context,
                commands.clone(),
                input,
                usage,
				id,
				from,
				to,
            );

            #[cfg(feature = "trace")]
            let output = output.instrument(tracing::info_span!("GraphNode run"));

            let output = output.await.unwrap();
			// pi_hal::runtime::LOGS.lock().0.push("node run end".to_string());

            #[cfg(all(not(feature = "webgl"),not(feature = "single_thread")))]
            {
                // CommandEncoder --> CommandBuffer
                let commands = Share::try_unwrap(commands.0).unwrap();
                let commands = commands.into_inner();

                // CommandBuffer --> Queue
                let queue = self.context.queue.clone();

                let submit_task = async move {
					// pi_hal::runtime::LOGS.lock().0.push("submit before".to_string());
					// log::warn!("submit before===========");
                    queue.submit(vec![commands.finish()]);
					// pi_hal::runtime::LOGS.lock().0.push("submit end".to_string());
					// log::warn!("submit after===========");
                };

                #[cfg(feature = "trace")]
                let submit_task = submit_task.instrument(tracing::info_span!("submite"));

                c.async_tasks.push(index, Box::pin(submit_task));
            }
            Ok(output)
        };
        Box::pin(task)
    }

    fn reset<'a>(
            &'a mut self,
        ) {
        self.node.reset()
    }
}

pub trait AsyncQueue: Send + Sync + 'static {
	/// 在指定索引处添加一个任务
    fn push(&self, index: usize, task: BoxFuture<'static, ()>);
	fn reset(&self);
}

#[derive(Clone)]
pub struct AsyncTaskQueue<A: AsyncRuntime> {
    pub queue: ShareTaskQueue,
    pub is_runing: Share<AtomicBool>,
    pub rt: A,
}

pub struct TaskQueue {
	list: VecMap<BoxFuture<'static, ()>>,
	index: usize, // 下一次要执行的任务索引
}

impl TaskQueue {
	pub fn new() -> Self {
		Self {
			list: VecMap::new(),
			index: 0,
		}
	}
}

#[derive(Clone, Deref, DerefMut)]
pub struct ShareTaskQueue(pub Share<ShareMutex<TaskQueue>>);

// 强制实现send和sync， 否则wasm上不能运行 TODO
unsafe impl Send for ShareTaskQueue {}
unsafe impl Sync for ShareTaskQueue {}


impl<A: AsyncRuntime> AsyncQueue for AsyncTaskQueue<A> {
	fn reset(&self) {
		let mut lock = self.queue.0.lock().unwrap();
		lock.index = 0;
	}
    /// 在指定索引处添加一个任务
	/// 任务从索引0处开始从小到大按顺序运行
	/// 若添加的任务索引高于下一个要执行的任务索引， 则将任务放入队列，但不立即执行， 等待前置任务就绪
    fn push(&self, index: usize, task: BoxFuture<'static, ()>) {
        // 依次 处理 队列
        fn run<A: AsyncRuntime>(queue: ShareTaskQueue, rt: A, is_runing: Share<AtomicBool>) {
			
			// pi_hal::runtime::LOGS.lock().0.push("AsyncQueue lock before".to_string());
            
			// pi_hal::runtime::LOGS.lock().0.push(format!("AsyncQueue lock after".to_string(), t.is_some()));
			let task = {
				let mut t1 = queue.0.lock().unwrap();
				let index = t1.index;
				let t = t1.list.remove(index);
				// log::warn!("AsyncQueue run==========={:?}", index);
				match t {
					Some(r) => {
						t1.index += 1;
						r
					},
					None => {
						// 当前位置不存在任务， 则停止执行
						is_runing.store(false, Ordering::Relaxed);
						return;
					}
				}
				
			};
            let rt1 = rt.clone();
			// 运行时 处理，但 不等待
			let _ = rt.spawn(async move {
				// pi_hal::runtime::LOGS.lock().0.push("t lock before".to_string());
				// log::warn!("t before===========");
				task.await;
				// pi_hal::runtime::LOGS.lock().0.push("t lock after".to_string());
				// log::warn!("t after===========");
				run(queue, rt1, is_runing);
			});
        }

        // 当 队列为空，而且 里面的东西已经执行完的时候，才会去 推 队列
		// pi_hal::runtime::LOGS.lock().0.push("AAsyncQueue1 push_back before".to_string());
        let is_start_run = {
			
			// pi_hal::runtime::LOGS.lock().0.push("AsyncQueue1 lock before".to_string());
            let mut lock = self.queue.0.lock().unwrap();
			lock.list.insert(index, task);
			// log::warn!("AsyncQueue push task==========={:?}", index);
			// pi_hal::runtime::LOGS.lock().0.push("AsyncQueue1 lock after".to_string());
            let is_start_run = lock.index == index && self
				.is_runing
				.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
				.is_ok();

            is_start_run
        };
		// pi_hal::runtime::LOGS.lock().0.push(format!("AAsyncQueue1 push_back after, {:?}", is_start_run));

        if is_start_run {
            // 第一个元素，挨个 执行一次
            run(self.queue.clone(), self.rt.clone(), self.is_runing.clone());
        }
    }
}
