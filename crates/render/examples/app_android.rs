use std::sync::Arc;

use bevy_ecs::prelude::Resource;
use bevy_log::LogPlugin;
use bevy_app::App;
use pi_bevy_asset::{AssetConfig, PiAssetPlugin};
use pi_bevy_render_plugin::{
    ClearOptions, PiClearOptions, PiRenderOptions, PiRenderPlugin, PiRenderWindow, PiScreenTexture,
};
use pi_bevy_winit_window::{update_window_handle, WinitPlugin};
use pi_render::rhi::options::RenderOptions;
use winit::event::{Event, WindowEvent};

pub const FILTER: &'static str = "wgpu=warn";

#[derive(Resource, Default)]
pub struct CheckSurfaceCmd(Vec<u32>);

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
    let mut app = App::default();

    app.add_plugins(LogPlugin::default());
    app.insert_resource(CheckSurfaceCmd::default());

    let event_loop = winit::event_loop::EventLoop::new();

    let mut is_resume = false;

    let window = winit::window::Window::new(&event_loop).unwrap();
    let window = Arc::new(window);

    let mut is_first = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                if is_resume {
                    // println!("App.update, thread id = {:?}", std::thread::current().id());
                    app.update();
                }
            }
            Event::Resumed => {
                is_resume = true;
                println!("Resumed, thread id = {:?}", std::thread::current().id());
                if !is_first {
                    let w = update_window_handle(&mut app.world, &window);

                    app.world
                        .resource_mut::<PiRenderWindow>()
                        .0
                        .update_handle(w);
                } else {
                    is_first = false;

                    let option = PiRenderOptions(RenderOptions {
                        backends: wgpu::Backends::PRIMARY,

                        limits: wgpu::Limits::downlevel_webgl2_defaults(),

                        ..Default::default()
                    });

                    app.add_plugins(WinitPlugin::new(window.clone()))
                        .insert_resource(option)
                        .insert_resource(PiClearOptions(ClearOptions {
                            color: wgpu::Color::GREEN,
                            ..Default::default()
                        }))
                        .add_plugins(PiAssetPlugin {
                            total_capacity: 256 * 1024 * 1024,
                            asset_config: AssetConfig::default(),
                        })
                        .add_plugins(PiRenderPlugin::default());
                }
            }
            Event::Suspended => {
                is_resume = false;
                println!("Suspended, thread id = {:?}", std::thread::current().id());
                app.world.resource_mut::<PiScreenTexture>().0.take();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }
    });
}
