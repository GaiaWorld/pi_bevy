use bevy::prelude::{Resource, DerefMut, Deref};
use pi_async::rt::AsyncRuntime;
use pi_share::{Share, ShareCell};

/// ================ 单例 ================

// winit 窗口
#[derive(Resource)]
pub struct PiWinitWindow(pub Share<winit::window::Window>);

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Resource, Deref, DerefMut)]
pub struct PiAsyncRuntime<A: AsyncRuntime>(pub A);

/// 渲染 Instance，等价于 wgpu::Instance
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderInstance(pub Share<pi_render::rhi::RenderInstance>);

/// 渲染 Options，等价于 wgpu::Options
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderOptions(pub Share<pi_render::rhi::options::RenderOptions>);

/// 渲染 设备，等价于 wgpu::RenderDevice
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderDevice(pub pi_render::rhi::device::RenderDevice);

/// 渲染 队列，等价于 wgpu::RenderQueue
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderQueue(pub pi_render::rhi::RenderQueue);

/// AdapterInfo，wgpu::AdapterInfo
#[derive(Resource, Deref, DerefMut)]
pub struct PiAdapterInfo(pub Share<pi_render::rhi::AdapterInfo>);

/// 渲染图，等价于 RenderGraph
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderGraph(pub Share<ShareCell<super::graph::graph::RenderGraph>>);

/// 渲染窗口
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiRenderWindows(
    pub Share<ShareCell<pi_render::components::view::render_window::RenderWindows>>,
);

/// 交换链对应的屏幕纹理
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiScreenTexture(pub Share<ShareCell<Option<pi_render::rhi::texture::ScreenTexture>>>);

/// 用于 wasm 的 单线程 Runner
#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct PiSingleTaskRunner(pub Option<pi_async::prelude::SingleTaskRunner<()>>);
