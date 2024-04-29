use pi_world::prelude::{FromWorld, World};
use crate::render_windows::RenderWindow;
// use bevy_ecs::world::FromWorld;
// use bevy_ecs::prelude::FromWorld;
// use bevy_ecs::system::Resource;
use pi_async_rt::prelude::*;
use pi_render::rhi::buffer_alloc::BufferAlloter;
use pi_share::Share;
use wgpu::BufferUsages;
use derive_deref::{Deref, DerefMut};
/// ================ 单例 ================

#[derive(Deref, DerefMut)]
pub struct PiSafeAtlasAllocator(pub pi_render::components::view::target_alloc::SafeAtlasAllocator);

// lazy_static::lazy_static! {

// 	static ref VERTEX_USAGES: BufferUsages = BufferUsages::COPY_DST | BufferUsages::VERTEX;
// }

// pub const VERTEX_USAGES: BufferUsages = BufferUsages::from_bits_truncate(BufferUsages::COPY_DST.bits() | BufferUsages::VERTEX.bits());
// pub const INDEX_USAGES: BufferUsages = BufferUsages::from_bits_truncate(BufferUsages::COPY_DST.bits() | BufferUsages::INDEX.bits());

pub const VERTEX_USAGES: u32 = BufferUsages::COPY_DST.bits() | BufferUsages::VERTEX.bits();
pub const INDEX_USAGES: u32 = BufferUsages::COPY_DST.bits() | BufferUsages::INDEX.bits();

/// 顶点buffer分配器
pub type PiVertexBufferAlloter = PiBufferAlloter<{VERTEX_USAGES}>;

/// 顶点buffer分配器
pub type PiIndexBufferAlloter = PiBufferAlloter<{INDEX_USAGES}>;

/// buffer分配器
#[derive(Deref, DerefMut)]
pub struct PiBufferAlloter<const B: u32>(BufferAlloter);

impl<const B: u32> FromWorld for PiBufferAlloter<B> {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_single_res::<PiRenderDevice>().unwrap();
        let queue = world.get_single_res::<PiRenderQueue>().unwrap();
		Self(BufferAlloter::new((**device).clone(), (**queue).clone(), 4096, BufferUsages::from_bits_truncate(B)))
    }
}

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Deref, DerefMut)]
pub struct PiRenderWindow(pub RenderWindow);

/// 异步 运行时
/// A 的 类型 见 plugin 模块
///   + wasm 环境 是 SingleTaskRuntime
///   + 否则 是 MultiTaskRuntime
///
#[derive(Deref, DerefMut)]
pub struct PiAsyncRuntime<A: AsyncRuntime + AsyncRuntimeExt>(pub A);

/// 渲染 Instance，等价于 wgpu::Instance
#[derive( Deref, DerefMut)]
pub struct PiRenderInstance(pub pi_render::rhi::RenderInstance);

// TODO Send问题， 临时解决
unsafe impl Send for PiRenderInstance {}
unsafe impl Sync for PiRenderInstance {}


/// 渲染 Options，等价于 wgpu::Options
#[derive(Deref, DerefMut, Default)]
pub struct PiRenderOptions(pub pi_render::rhi::options::RenderOptions);

/// 渲染 设备，等价于 wgpu::RenderDevice
#[derive(Deref, DerefMut)]
pub struct PiRenderDevice(pub pi_render::rhi::device::RenderDevice);

// TODO Send问题， 临时解决
unsafe impl Send for PiRenderDevice {}
unsafe impl Sync for PiRenderDevice {}

/// 渲染 队列，等价于 wgpu::RenderQueue
#[derive(Deref, DerefMut)]
pub struct PiRenderQueue(pub pi_render::rhi::RenderQueue);

// TODO Send问题， 临时解决
unsafe impl Send for PiRenderQueue {}
unsafe impl Sync for PiRenderQueue {}

/// AdapterInfo，wgpu::AdapterInfo
#[derive(Deref, DerefMut)]
pub struct PiAdapterInfo(pub pi_render::rhi::AdapterInfo);

/// 渲染图，等价于 RenderGraph
#[derive( Deref, DerefMut)]
pub struct PiRenderGraph(pub super::graph::graph::RenderGraph);

// TODO Send问题， 临时解决
unsafe impl Send for PiRenderGraph {}
unsafe impl Sync for PiRenderGraph {}

/// 交换链对应的屏幕纹理
#[derive(Default,  Deref, DerefMut)]
pub struct PiScreenTexture(pub Option<pi_render::rhi::texture::ScreenTexture>);

// TODO Send问题， 临时解决
unsafe impl Send for PiScreenTexture {}
unsafe impl Sync for PiScreenTexture {}

/// 清屏 参数
#[derive(Default,  Deref, DerefMut, Debug, Clone)]
pub struct PiClearOptions(pub ClearOptions);

/// 用于处理 初始化 的Surface 和 prepare_windows 的 关系
#[derive( Deref, DerefMut)]
pub(crate) struct PiFirstSurface(pub(crate) Option<wgpu::Surface<'static>>);

// TODO Send问题， 临时解决
unsafe impl Send for PiFirstSurface {}
unsafe impl Sync for PiFirstSurface {}

#[derive(Clone, Debug)]
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


// 纹理key分配器
#[derive(Debug, Default,  Clone, Deref)]
pub struct TextureKeyAlloter(pub Share<pi_key_alloter::KeyAlloter>);
