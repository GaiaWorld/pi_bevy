use bevy::prelude::{Deref, DerefMut, Resource};
use pi_async::rt::AsyncRuntime;
use pi_share::Share;

/// ================ 单例 ================

// winit 窗口
#[derive(Resource, Deref, DerefMut)]
pub struct PiWinitWindow(pub Share<winit::window::Window>);

#[derive(Resource, Deref, DerefMut)]
pub struct PiSafeAtlasAllocator(pub pi_render::components::view::target_alloc::SafeAtlasAllocator);

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Resource, Deref, DerefMut)]
pub struct PiAsyncRuntime<A: AsyncRuntime>(pub A);

/// 用于 wasm 的 单线程 Runner
#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct PiSingleTaskRunner(pub Option<pi_async::prelude::SingleTaskRunner<()>>);

/// 渲染 Instance，等价于 wgpu::Instance
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderInstance(pub pi_render::rhi::RenderInstance);

/// 渲染 Options，等价于 wgpu::Options
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderOptions(pub pi_render::rhi::options::RenderOptions);

/// 渲染 设备，等价于 wgpu::RenderDevice
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderDevice(pub pi_render::rhi::device::RenderDevice);

/// 渲染 队列，等价于 wgpu::RenderQueue
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderQueue(pub pi_render::rhi::RenderQueue);

/// AdapterInfo，wgpu::AdapterInfo
#[derive(Resource, Deref, DerefMut)]
pub struct PiAdapterInfo(pub pi_render::rhi::AdapterInfo);

/// 渲染图，等价于 RenderGraph
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderGraph(pub super::graph::graph::RenderGraph);

/// 渲染窗口
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiRenderWindows(pub pi_render::components::view::render_window::RenderWindows);

/// 交换链对应的屏幕纹理
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiScreenTexture(pub Option<pi_render::rhi::texture::ScreenTexture>);