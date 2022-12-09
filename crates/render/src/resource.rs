use crate::render_windows::RenderWindow;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::system::Resource;
use pi_async::prelude::*;

/// ================ 单例 ================

#[derive(Resource, Deref, DerefMut)]
pub struct PiSafeAtlasAllocator(pub pi_render::components::view::target_alloc::SafeAtlasAllocator);

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderWindow(pub RenderWindow);

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Resource, Deref, DerefMut)]
pub struct PiAsyncRuntime<A: AsyncRuntime + AsyncRuntimeExt>(pub A);

/// 渲染 Instance，等价于 wgpu::Instance
#[derive(Resource, Deref, DerefMut)]
pub struct PiRenderInstance(pub pi_render::rhi::RenderInstance);

/// 渲染 Options，等价于 wgpu::Options
#[derive(Resource, Deref, DerefMut, Default)]
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

/// 交换链对应的屏幕纹理
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiScreenTexture(pub Option<pi_render::rhi::texture::ScreenTexture>);

/// 清屏 参数
#[derive(Default, Resource, Deref, DerefMut)]
pub struct PiClearOptions(pub ClearOptions);

#[derive(Clone)]
pub struct ClearOptions {
    pub color: wgpu::Color,
    pub depth: Option<f32>,
    pub stencil: Option<u32>,
}

impl Default for ClearOptions {
    fn default() -> Self {
        Self {
            color: wgpu::Color::WHITE,
            depth: Some(1.0),
            stencil: None,
        }
    }
}
