//!
//! pi_render 的 bevy 封装
//!

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
mod plugin;
mod resource;
mod system;
mod util;

// 渲染 插件
/// 渲染图
pub use graph::*;
pub use plugin::*;
// 渲染 相关 单例
pub use resource::*;
