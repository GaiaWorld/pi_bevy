//! 渲染图 模块
//!
//! 主要类
//!     + struct RenderContext
//!
pub mod graph;
pub mod node;
pub mod param;
// pub mod sub_graph_node;
pub(crate) mod state_pool;

use pi_render::rhi::{device::RenderDevice, RenderQueue};

/// 渲染图 执行过程中 遇到的 相关错误信息
pub use pi_render::depend_graph::GraphError;

pub use node::{NodeId, NodeLabel};
use pi_share::ShareRefCell;

/// 渲染图 执行过程需要的环境
#[derive(Clone)]
pub struct RenderContext {
    /// 渲染 设备，用于 创建资源
    pub device: RenderDevice,

    /// 异步队列
    pub queue: RenderQueue,

    /// webgl 环境下 使用
	#[allow(dead_code)]
    commands: ShareRefCell<Option<ShareRefCell<wgpu::CommandEncoder>>>,
}