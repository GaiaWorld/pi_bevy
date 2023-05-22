use std::marker::PhantomData;
use std::mem::transmute;

use crate::render_windows::RenderWindow;
use bevy::prelude::{Deref, DerefMut, FromWorld};
use bevy::ecs::system::Resource;
use pi_async::prelude::*;
use pi_render::rhi::buffer_alloc::BufferAlloter;
use wgpu::BufferUsages;

/// ================ 单例 ================

#[derive(Resource, Deref, DerefMut)]
pub struct PiSafeAtlasAllocator(pub pi_render::components::view::target_alloc::SafeAtlasAllocator);

// lazy_static::lazy_static! {

// 	static ref VERTEX_USAGES: BufferUsages = BufferUsages::COPY_DST | BufferUsages::VERTEX;
// }

pub const VERTEX_USAGES: BufferUsages = BufferUsages::from_bits_truncate(BufferUsages::COPY_DST.bits() | BufferUsages::VERTEX.bits());
pub const INDEX_USAGES: BufferUsages = BufferUsages::from_bits_truncate(BufferUsages::COPY_DST.bits() | BufferUsages::INDEX.bits());

/// 顶点buffer分配器
pub type PiVertexBufferAlloter = PiBufferAlloter<{VERTEX_USAGES}>;

/// 顶点buffer分配器
pub type PiIndexBufferAlloter = PiBufferAlloter<{INDEX_USAGES}>;

/// buffer分配器
#[derive(Resource, Deref, DerefMut)]
pub struct PiBufferAlloter<const B: BufferUsages>(BufferAlloter);

impl<const B: BufferUsages> FromWorld for PiBufferAlloter<B> {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let device = world.get_resource::<PiRenderDevice>().unwrap();
        let queue = world.get_resource::<PiRenderQueue>().unwrap();
		Self(BufferAlloter::new((**device).clone(), (**queue).clone(), 4096, B))
    }
}

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
