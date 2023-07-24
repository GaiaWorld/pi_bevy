use bevy::{
    app::App,
    window::{Window, WindowResolution},
    winit::WinitPlugin,
};
use pi_bevy_asset::{PiAssetPlugin, AssetConfig};
use pi_bevy_render_plugin::{ClearOptions, PiClearOptions, PiRenderPlugin};

pub const FILTER: &'static str = "wgpu=warn";

fn main() {
    let mut app = App::default();

    let mut window = Window::default();
    window.resolution = WindowResolution::new(1024.0, 768.0);

    let mut window_plugin = bevy::window::WindowPlugin::default();
    window_plugin.primary_window = Some(window);

    app.add_plugin(bevy::log::LogPlugin {
        filter: FILTER.to_string(),
        level: bevy::log::Level::INFO,
    })
    .insert_resource(PiClearOptions(ClearOptions {
        color: wgpu::Color::GREEN,
        ..Default::default()
    }))
    .add_plugin(bevy::a11y::AccessibilityPlugin)
    .add_plugin(bevy::input::InputPlugin::default())
    .add_plugin(window_plugin)
    .add_plugin(WinitPlugin::default())
    .add_plugin(PiAssetPlugin {
        total_capacity: 1024 * 1024 * 1024,
        asset_config: AssetConfig::default(),
    })
    .add_plugin(PiRenderPlugin::default());

    app.run();
}
