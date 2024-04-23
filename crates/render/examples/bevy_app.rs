use std::sync::Arc;

// use bevy_app::App;
// use bevy_log::LogPlugin;
use pi_bevy_asset::{AssetConfig, PiAssetPlugin};
use pi_bevy_render_plugin::{ClearOptions, PiClearOptions, PiRenderPlugin};
use pi_bevy_winit_window::WinitPlugin;
use pi_world::prelude::App;
use pi_world_extend_plugin::plugin_group::WorldPluginExtent;

pub const FILTER: &'static str = "wgpu=warn";

fn main() {
    let mut app = App::new();

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::Window::new(&event_loop).unwrap();

    app.add_plugins(WinitPlugin::new(Arc::new(window)));
    app.world.register_single_res(PiClearOptions(ClearOptions {
        color: wgpu::Color::GREEN,
        ..Default::default()
    }));
    // app.add_plugins(bevy_a11y::AccessibilityPlugin);
    // app.add_plugins(bevy_input::InputPlugin::default());
    app.add_plugins(PiAssetPlugin {
        total_capacity: 256 * 1024 * 1024,
        asset_config: AssetConfig::default(),
    });
    app.add_plugins(PiRenderPlugin::default());

    app.initialize();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::MainEventsCleared => {
                // println!("App.update, thread id = {:?}", std::thread::current().id());
                app.run();
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
