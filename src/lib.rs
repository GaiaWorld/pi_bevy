
mod graph;
mod plugin;
mod resource;
mod system;

use thiserror::Error;

/// 渲染图
pub use graph::*;
// 渲染 插件
pub use plugin::*;
// 渲染 相关 Resource
pub use resource::*;
// 渲染 相关 Stage 和 System
pub use system::*;

#[derive(Error, Debug)]
pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}