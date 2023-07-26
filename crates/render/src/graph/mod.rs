//! 渲染图 模块
//!
//! 主要类
//!     + struct RenderContext
//!
pub mod graph;
pub mod node;
pub mod param;
pub(crate) mod state_pool;

use pi_render::rhi::{device::RenderDevice, RenderQueue};

/// 渲染图 执行过程中 遇到的 相关错误信息
pub use pi_render::depend_graph::GraphError;

pub use node::{NodeId, NodeLabel};
use pi_share::{Share, ShareCell};
use wgpu::CommandBuffer;

/// 渲染图 执行过程需要的环境
#[derive(Clone)]
pub struct RenderContext {
    /// 渲染 设备，用于 创建资源
    pub device: RenderDevice,

    /// 异步队列
    pub queue: RenderQueue,

    /// webgl 环境下的 finish_buffer
    webgl_cmd_buffers: CmdBuffers,
}

#[derive(Clone)]
pub(crate) struct CmdBuffers(Share<ShareCell<Vec<CommandBuffer>>>);

impl CmdBuffers {
    pub(crate) fn push(&self, cmd: CommandBuffer) {
        self.0.borrow_mut().push(cmd);
    }

    pub(crate) fn replace_with_new_buffer(&mut self) -> Vec<CommandBuffer> {
        let mut new = Vec::new();

        std::mem::swap(&mut new, &mut *self.0.borrow_mut());
        
        new
    }
}

impl Default for CmdBuffers {
    fn default() -> Self {
        Self(Share::new(ShareCell::new(Vec::new())))
    }
}
