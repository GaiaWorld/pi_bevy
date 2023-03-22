

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

/// 渲染图
pub use graph::*;
use pi_hash::{XHashSet, XHashMap};
use pi_render::{components::view::target_alloc::ShareTargetView, depend_graph::param::{InParam, OutParam}};
/// 渲染 插件
pub use plugin::*;
/// 单例
pub use resource::*;

/// 标签
pub use clear_node::CLEAR_WIDNOW_NODE;

#[derive(Default, Clone)]
pub struct SimpleInOut {
    target: Option<ShareTargetView>,
}
impl InParam for SimpleInOut {
    fn can_fill<O: pi_render::depend_graph::param::OutParam + ?Sized>(
        &self,
        map: &mut XHashMap<std::any::TypeId, Vec<NodeId>>,
        pre_id: NodeId,
        out_param: &O,
    ) -> bool {
        true
    }

    fn fill_from<O: pi_render::depend_graph::param::OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &O) -> bool {
        true
    }
}
impl OutParam for SimpleInOut {
    fn can_fill(&self, set: &mut Option<&mut XHashSet<std::any::TypeId>>, ty: std::any::TypeId) -> bool {
        true
    }

    fn fill_to(&self, this_id: NodeId, to: &mut dyn pi_render::depend_graph::param::Assign, ty: std::any::TypeId) -> bool {
        true
    }
}