use std::sync::Arc;

use bevy::{app::App, log::LogPlugin};
use pi_bevy_asset::{AssetConfig, PiAssetPlugin};
use pi_bevy_render_plugin::{ClearOptions, PiClearOptions, PiRenderPlugin};
use pi_bevy_winit_window::WinitPlugin;

pub const FILTER: &'static str = "wgpu=warn";

fn main() {
    let mut app = App::default();

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::Window::new(&event_loop).unwrap();

    app
    .add_plugin(LogPlugin::default())
    .add_plugin(WinitPlugin::new(Arc::new(window)))
    .insert_resource(PiClearOptions(ClearOptions {
        color: wgpu::Color::GREEN,
        ..Default::default()
    }))
    .add_plugin(bevy::a11y::AccessibilityPlugin)
    .add_plugin(bevy::input::InputPlugin::default())
    .add_plugin(PiAssetPlugin {
        total_capacity: 256 * 1024 * 1024,
        asset_config: AssetConfig::default(),
    })
    .add_plugin(PiRenderPlugin::default());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::MainEventsCleared => {
                // println!("App.update, thread id = {:?}", std::thread::current().id());
                app.update();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }
    });

}
