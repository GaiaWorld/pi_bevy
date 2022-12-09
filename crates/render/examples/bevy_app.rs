use bevy_app::App;
use bevy_log::LogPlugin;
use bevy_window::WindowPlugin;
use bevy_winit::WinitPlugin;
use pi_bevy_render_plugin::{ClearOptions, PiClearOptions, PiRenderPlugin};

fn main() {
    App::new()
        .insert_resource(PiClearOptions(ClearOptions {
            color: wgpu::Color::GREEN,
            ..Default::default()
        }))
        .add_plugin(LogPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_plugin(PiRenderPlugin::default())
        .run();
}
