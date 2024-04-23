// use bevy_app::Plugin;
use bevy_window::{
    PrimaryWindow, WindowCreated, WindowPosition, WindowResolution, CreateSurface, HandleWrapper,
};
// use glam::IVec2;
use nalgebra::Vector2;
use pi_world::prelude::App;
use pi_world_extend_plugin::plugin::Plugin;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
pub struct WinitPlugin {
    descript: WindowDescribe,
}
// 可能不安全， TODO
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Send for WinitPlugin {}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Sync for WinitPlugin {}

#[cfg(not(target_arch = "wasm32"))]
impl WinitPlugin {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            descript: WindowDescribe::new(window),
        }
    }

    pub fn with_size(self, width: u32, height: u32) -> Self {
        Self {
            descript: self.descript.with_size(width, height),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        self.descript.build(app);
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WinitPlugin {
    canvas: Arc<Window>,
    size: Option<(u32, u32)>,
}

// wasm平台下强行实现Send和Sync，wasm是单线程，不会出现安全隐患
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WinitPlugin {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WinitPlugin {}

#[cfg(target_arch = "wasm32")]
impl WinitPlugin {
    pub fn new(window: Arc<Window>) -> Self {
        Self { canvas: window, size: None }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }
}

#[cfg(target_arch = "wasm32")]
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // let event_loop = winit::event_loop::EventLoop::new();
        // let window = Arc::new(
        //     winit::window::WindowBuilder::new()
        //         .with_canvas(Some(self.canvas.clone()))
        //         .build(&event_loop)
        //         .unwrap(),
        // );

        let describe = WindowDescribe {
            window: self.canvas.clone(),
            size: self.size.clone(),
        };
        describe.build(app);
    }
}

pub struct WindowDescribe {
    window: Arc<Window>,
    size: Option<(u32, u32)>,
}

pub struct WindowWrapper(Arc<Window>);

impl CreateSurface for WindowWrapper {
    fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface<'static> {
		instance.create_surface(self.0.clone()).unwrap()
    }
}

impl WindowDescribe {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window, size: None }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    fn build(&self, app: &mut App) {
        let winit_window = &*self.window;
        if let Some(size) = self.size {
            winit_window.set_inner_size(PhysicalSize {
                width: size.0,
                height: size.1,
            });
        }

        let inner_size = winit_window.inner_size();
        let scale_factor = winit_window.scale_factor();
        let raw_handle = bevy_window::HandleWrapper {
            handle: Arc::new(WindowWrapper(self.window.clone())),
        };
        let mut window = bevy_window::prelude::Window::default();
        window.resolution = WindowResolution::new(
            inner_size.width as f32 / scale_factor as f32,
            inner_size.height as f32 / scale_factor as f32,
        );
        window.resolution.set_scale_factor(scale_factor);
        window.position = match winit_window.outer_position().map(|r| Vector2::new(r.x, r.y)) {
            Ok(r) => WindowPosition::At(r),
            _ => WindowPosition::Automatic,
        };
        let i = app.world.make_inserter::<(bevy_window::prelude::Window, HandleWrapper, PrimaryWindow)>();
        let primary = i.insert((window, raw_handle, PrimaryWindow));
		log::warn!("zzzzzzzzzzzzzz==================");

        // TODO?
        #[cfg(not(any(target_os = "windows", target_feature = "x11")))]
        app.world.send_event(bevy_window::WindowResized {
            window: primary,
            width: inner_size.width as f32,
            height: inner_size.height as f32,
        });

        // windows.add(window);
        // todo
        // app.world.send_event(WindowCreated { window: primary });
        // world.send_event(bevy_window::WindowCreated { id: self.window_id });
    }
}

// pub fn update_window_handle(world: &mut World, window: &Window) -> HandleWrapper {
//     let mut primary_window = world.query_filtered::<&mut HandleWrapper, With<PrimaryWindow>>();

//     let mut r = primary_window.single_mut(world);

//     r.window_handle = window.window_handle();
//     r.display_handle = window.display_handle();

//     r.clone()
// }
