//!
//! pi_render 的 bevy 封装
//!

mod clear_node;
///
/// 将 pi_render 封装成 Bevy插件，使用方式 如下
///
/// use bevy::prelude::*;
/// use pi_render_bevy::PiRenderPlugin;
///
/// fn main() {
///     App::new()
///      .add_plugins(DefaultPlugins)
///      .add_plugin(PiRenderPlugin)
///      .run();
/// }
///
mod graph;
mod init_render;
mod plugin;
mod render_windows;
mod resource;
mod system;

/// 渲染图
pub use graph::*;
/// 渲染 插件
pub use plugin::*;
/// 单例
pub use resource::*;

/// 标签
pub use clear_node::CLEAR_WIDNOW_NODE;
