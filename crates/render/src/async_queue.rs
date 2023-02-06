//!
//!  异步队列
//!

use bevy_ecs::system::Command;
use pi_render::rhi::RenderQueue;
use std::{collections::VecDeque, sync::Mutex};
use wgpu::CommandBuffer;

// 异步队列，允许 多线程并发 访问
pub(crate) struct AsyncQueue {
    render_queue: RenderQueue,
    buffers: Mutex<VecDeque<CommandBuffer>>,
}

impl AsyncQueue {
    #[inline]
    pub(crate) fn new(render_queue: RenderQueue) -> Self {
        Self {
            render_queue,
            buffers: Mutex::new(Default::default()),
        }
    }

    pub(crate) async fn push_back(&mut self, cmd: CommandBuffer) {
        let is_empty = {
            let mut buffers = self.buffers.lock().unwrap();

            let is_empty = buffers.is_empty();
            buffers.push_back(cmd);

            is_empty
        };

        if is_empty {
            // 第一个元素，挨个 执行一次
            self.run().await;
        }
    }

    pub(crate) async fn run(&mut self) {
        while let Some(cmd) = self.pop_front() {
            #[cfg(not(feature = "trace"))]
            async {
                self.render_queue.submit(vec![cmd]);
            }
            .await;

            #[cfg(feature = "trace")]
            async {
                self.render_queue.submit(vec![cmd]);
            }
            .instrument(tracing::info_span!("submit"))
            .await;
        }
    }

    #[inline]
    fn pop_front(&mut self) -> Option<CommandBuffer> {
        let mut buffers = self.buffers.lock().unwrap();

        buffers.pop_front()
    }
}
