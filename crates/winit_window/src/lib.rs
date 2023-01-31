use std::sync::Arc;

use bevy_window::{Windows, WindowCreated, WindowId, WindowDescriptor, RawHandleWrapper};
use glam::IVec2;
use winit::{window::{Window, WindowBuilder}, dpi::{Size, PhysicalSize}};
use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::HasRawDisplayHandle;
use bevy_app::Plugin;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch="wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;

#[cfg(not(target_arch="wasm32"))]
pub struct WinitPlugin {
	descript: WindowDescribe,
}
// 可能不安全， TODO
#[cfg(not(target_arch="wasm32"))]
unsafe impl Send for WinitPlugin {}
#[cfg(not(target_arch="wasm32"))]
unsafe impl Sync for WinitPlugin {}

#[cfg(not(target_arch="wasm32"))]
impl WinitPlugin {
	pub fn new(window: Arc<Window>, window_id: WindowId) -> Self {
		Self {
			descript: WindowDescribe::new(window, window_id),
		}
	}

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		Self{descript: self.descript.with_size(width, height)}
	}
}


#[cfg(not(target_arch="wasm32"))]
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        self.descript.build(app);
    }
}


#[cfg(target_arch="wasm32")]
pub struct WinitPlugin {
	canvas: HtmlCanvasElement,
	window_id: WindowId,
	size: Option<(u32, u32)>,
}

// wasm平台下强行实现Send和Sync，wasm是单线程，不会出现安全隐患
#[cfg(target_arch="wasm32")]
unsafe impl Send for WinitPlugin {}
#[cfg(target_arch="wasm32")]
unsafe impl Sync for WinitPlugin {}

#[cfg(target_arch="wasm32")]
impl WinitPlugin {
	pub fn new(canvas: HtmlCanvasElement, window_id: WindowId) -> Self {
		Self {
			canvas,
			window_id,
			size: None
		}
	}

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		self.size = Some((width, height));
		self
	}
}

#[cfg(target_arch="wasm32")]
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut bevy_app::App) {
		let event_loop = winit::event_loop::EventLoop::new();
		let window = Arc::new(WindowBuilder::new().with_canvas(Some(self.canvas.clone())).build(&event_loop).unwrap());

		let describe = WindowDescribe {
			window,
			window_id: self.window_id,
			size: self.size.clone(),
		};
        describe.build(app);
    }
}

pub struct WindowDescribe {
	window: Arc<Window>,
	window_id: WindowId,
	size: Option<(u32, u32)>,
}

impl WindowDescribe {
	pub fn new(window: Arc<Window>, window_id: WindowId) -> Self {
		Self {
			window,
			size: None,
			window_id
		}
	}

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		self.size = Some((width, height));
		self
	}

	fn build(&self, app: &mut bevy_app::App) {
		let world = app.world.cell();
        let mut windows = world.resource_mut::<Windows>();
		let winit_window = &*self.window;
		if let Some(size) = self.size {
			winit_window.set_inner_size(PhysicalSize{width: size.0, height: size.1});
		}

		let mut window_descriptor = WindowDescriptor::default();
		let inner_size = winit_window.inner_size();
		window_descriptor.width = inner_size.width as f32;
		window_descriptor.height = inner_size.height as f32;
		let position = winit_window
            .outer_position()
            .ok()
            .map(|position| IVec2::new(0, 0));
        let scale_factor = winit_window.scale_factor();
        let raw_handle = RawHandleWrapper {
            window_handle: winit_window.raw_window_handle(),
            display_handle: winit_window.raw_display_handle(),
        };
		let window = bevy_window::Window::new(
            self.window_id,
            &window_descriptor,
            inner_size.width,
            inner_size.height,
            scale_factor,
            position,
            Some(raw_handle),
        );

		#[cfg(not(any(target_os = "windows", target_feature = "x11")))]
        world.send_event(bevy_window::WindowResized {
            id: self.window_id,
            width: window.width(),
            height: window.height(),
        });
        windows.add(window);
        world.send_event(bevy_window::WindowCreated {
            id: self.window_id,
        });
    }
}
