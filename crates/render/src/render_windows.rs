use bevy::window::RawHandleWrapper;
use pi_render::rhi::{
    device::RenderDevice,
    texture::{PiRenderDefault, ScreenTexture},
    PresentMode, RenderInstance,
};
use pi_share::Share;
use wgpu::TextureFormat;

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
    view: &mut Option<ScreenTexture>,
    device: &RenderDevice,
    instance: &RenderInstance,
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    let is_first = view.is_none();
    if is_first {
        let surface = unsafe {
            let handle = window.handle.get_handle();
            instance.create_surface(&handle).unwrap()
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
        view_formats: vec![wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba8UnormSrgb],
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
