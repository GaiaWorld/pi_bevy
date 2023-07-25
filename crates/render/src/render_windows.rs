use bevy::window::RawHandleWrapper;
use pi_render::rhi::{
    device::RenderDevice,
    texture::{PiRenderDefault, ScreenTexture},
    PresentMode, RenderInstance,
};
use pi_share::Share;
use wgpu::{Surface, TextureFormat};

pub struct RenderWindow {
    pub width: u32,
    pub height: u32,
    handle: RawHandleWrapper,
    present_mode: PresentMode,
}

impl RenderWindow {
    pub fn new(handle: RawHandleWrapper, present_mode: PresentMode) -> Self {
        Self {
            handle,
            present_mode,
            width: 0,
            height: 0,
        }
    }
}

pub fn prepare_window(
    window: &mut RenderWindow,
    first_surface: Option<Surface>, // 用于衔接 初始化的surface 和 这里的代码
    view: &mut Option<ScreenTexture>,
    device: &RenderDevice,
    instance: &RenderInstance,
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    let is_first = view.is_none();
    if is_first {
        let surface = if first_surface.is_none() {
            log::info!("prepare_window, first_surface is none, create new surface");

            let handle = unsafe { window.handle.get_handle() };
            unsafe { instance.create_surface(&handle).unwrap() }
        } else {
            first_surface.unwrap()
        };

        let surface = Share::new(surface);

        *view = Some(ScreenTexture::with_surface(surface));
    }

    let view = view.as_mut().unwrap();

    let config = wgpu::SurfaceConfiguration {
        format: TextureFormat::pi_render_default(),
        width,
        height,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        present_mode: window.present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };

    let is_size_changed = width != window.width || height != window.height;
    if is_size_changed {
        window.width = width;
        window.height = height;
    }
    // 记得 第一次 需要 Config
    if is_first || is_size_changed {
        device.configure_surface(view.surface(), &config);
    }

    // 每帧 都要 设置 新的 SuraceTexture
    view.next_frame(device, &config);

    Ok(())
}
