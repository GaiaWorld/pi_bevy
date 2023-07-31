use std::sync::Arc;

use bevy::{
    app::App,
    prelude::{Res, ResMut, Resource},
};
use pi_bevy_asset::{AssetConfig, PiAssetPlugin};
use pi_bevy_render_plugin::{
    ClearOptions, PiClearOptions, PiRenderDevice, PiRenderInstance, PiRenderPlugin, PiRenderWindow,
    PiScreenTexture,
};
use pi_bevy_winit_window::WinitPlugin;
use pi_render::rhi::texture::{PiRenderDefault, ScreenTexture};
use wgpu::TextureFormat;
use winit::event::{Event, WindowEvent};

pub const FILTER: &'static str = "wgpu=warn";

#[derive(Resource, Default)]
pub struct CheckSurfaceCmd(Vec<u32>);

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
    let mut app = App::default();
    app.insert_resource(CheckSurfaceCmd::default());

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::Window::new(&event_loop).unwrap();
    let window = Arc::new(window);
    let mut is_first = true;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                app.update();
            }
            Event::Resumed => {
                println!("Resumed");
                if is_first {
                    app.add_plugin(WinitPlugin::new(window.clone()))
                        .insert_resource(PiClearOptions(ClearOptions {
                            color: wgpu::Color::GREEN,
                            ..Default::default()
                        }))
                        .add_plugin(PiAssetPlugin {
                            total_capacity: 256 * 1024 * 1024,
                            asset_config: AssetConfig::default(),
                        })
                        .add_plugin(PiRenderPlugin::default());
                    app.add_system(WindowStateChangeSys::sys);
                    is_first = false;
                } else {
                    app.world.resource_mut::<CheckSurfaceCmd>().0.push(2);
                }
            }
            Event::Suspended => {
                println!("Suspended");
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

pub struct WindowStateChangeSys;

impl WindowStateChangeSys {
    pub fn sys(
        instance: Res<PiRenderInstance>,
        device: Res<PiRenderDevice>,
        window: Res<PiRenderWindow>,
        mut states: ResMut<CheckSurfaceCmd>,
        mut view: ResMut<PiScreenTexture>,
    ) {
        // println!("WindowStateChangeCmd!!! sys:");
        states.0.drain(..).for_each(|state| {
            println!("WindowStateChangeCmd!!! state: {:?}", state);
            match state {
                1 => {
                    view.0.take();
                }
                2 => {
                    println!("==============1");
                    let surface = unsafe {
                        let handle = window.0.handle.get_handle();
                        // println!("")
                        instance.0.create_surface(&handle).unwrap()
                    };
                    println!("==============2");
                    let config = wgpu::SurfaceConfiguration {
                        format: TextureFormat::pi_render_default(),
                        width: window.width,
                        height: window.height,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        present_mode: window.0.present_mode,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: vec![],
                    };
                    device.configure_surface(&surface, &config);
                    view.0 = Some(ScreenTexture::with_surface(surface));
                }
                _ => panic!("========="),
            }
        })
    }
}
