use bevy::prelude::*;
use pi_async::rt::AsyncRuntime;

/// ================ 资源 ================

/// 异步 运行时
#[derive(Resource)]
pub struct PiAsyncRuntime<A: 'static + AsyncRuntime + Send>(pub A);

/// 渲染 Instance，等价于 wgpu::Instance
#[derive(Resource)]
pub struct PiRenderInstance(pub pi_render::rhi::RenderInstance);

/// 渲染 Options，等价于 wgpu::Options
#[derive(Resource)]
pub struct PiRenderOptions(pub pi_render::rhi::options::RenderOptions);

/// 渲染 设备，等价于 wgpu::RenderDevice
#[derive(Resource)]
pub struct PiRenderDevice(pub pi_render::rhi::device::RenderDevice);

/// 渲染 队列，等价于 wgpu::RenderQueue
#[derive(Resource)]
pub struct PiRenderQueue(pub pi_render::rhi::RenderQueue);

/// AdapterInfo，wgpu::AdapterInfo
#[derive(Resource)]
pub struct PiAdapterInfo(pub pi_render::rhi::AdapterInfo);

/// 渲染图，等价于 RenderGraph
#[derive(Resource)]
pub struct PiRenderGraph(pub super::graph::graph::RenderGraph);

/// TextureViews
#[derive(Default, Resource)]
pub struct PiTextureViews(pub pi_render::components::view::target::TextureViews);

/// 渲染目标
#[derive(Default, Resource)]
pub struct PiRenderTargets(pub pi_render::components::view::target::RenderTargets);

/// 渲染窗口
#[derive(Default, Resource)]
pub struct PiRenderWindows(pub pi_render::components::view::render_window::RenderWindows);
