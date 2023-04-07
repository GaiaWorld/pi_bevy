

//!
//! pi_render 的 bevy 封装
//!
//!
//! 将 pi_render 封装成 Bevy插件，使用方式 如下
//!
//! use bevy_app::App;
//! use bevy_log::LogPlugin;
//! use bevy_window::WindowPlugin;
//! use bevy_winit::WinitPlugin;
//! use pi_bevy_render_plugin::{ClearOptions, PiClearOptions, PiRenderPlugin};
//!
//! fn main() {
//!    App::new()
//!        .insert_resource(PiClearOptions(ClearOptions {
//!            color: wgpu::Color::GREEN,
//!            ..Default::default()
//!        }))
//!        .add_plugin(LogPlugin::default())
//!        .add_plugin(WindowPlugin::default())
//!        .add_plugin(WinitPlugin::default())
//!       .add_plugin(PiRenderPlugin::default())
//!        .run();
//! }
//!

mod async_queue;
mod clear_node;
mod graph;
mod init_render;
mod plugin;
mod render_windows;
mod resource;
mod system;
pub mod component;

/// 渲染图
pub use graph::*;
use pi_render::{components::view::target_alloc::ShareTargetView};
/// 渲染 插件
pub use plugin::*;
use render_derive::NodeParam;
/// 单例
pub use resource::*;

/// 标签
pub use clear_node::CLEAR_WIDNOW_NODE;

#[derive(Default, Clone, NodeParam)]
pub struct SimpleInOut {
    pub target: Option<ShareTargetView>,
}