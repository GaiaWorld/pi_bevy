use std::sync::Arc;

use pi_world::prelude::{FromWorld, World};
use crate::render_windows::RenderWindow;
// use bevy_ecs::world::FromWorld;
// use bevy_ecs::prelude::FromWorld;
// use bevy_ecs::system::Resource;
use pi_async_rt::prelude::*;
use pi_render::{renderer::vertex_buffer::{NotUpdatableBufferRange, VertexBufferAllocator}, rhi::{buffer::Buffer, buffer_alloc::BufferAlloter, device::RenderDevice, RenderQueue}};
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


pub struct InstanceCacheBuffer {
    vertices: Vec<u8>,
    used_bytes: usize,
    buffer: (Arc<NotUpdatableBufferRange>, u32, u32),
}

pub struct PiInstanceBufferAllocator {
    list: Vec<InstanceCacheBuffer>,
    used_index: usize,
    /// 单个 Mesh 的实例化最多使用多少字节数据
    /// 当 运行时超过该数据时 对数据进行截取
    one_mesh_max_instance_bytes: u32,
}
impl PiInstanceBufferAllocator {
    pub fn one_mesh_max_instance_bytes(&self) -> usize {
        self.one_mesh_max_instance_bytes as usize
    }
    pub fn check(&self, buffer: &Buffer) -> bool {
        let mut result = true;
        self.list.iter().for_each(|item| {
            if result {
                result = buffer.id() == item.buffer.0.buffer().id();
            }
        });
        return result;
    }
    pub fn new(one_mesh_max_instance_bytes: u32, allocator: &mut VertexBufferAllocator, device: &RenderDevice, queue: &RenderQueue) -> Self {
        let mut data = Vec::with_capacity(one_mesh_max_instance_bytes as usize);
        for _ in 0..one_mesh_max_instance_bytes {
            data.push(0);
        }
        
        // log::error!("InstanceBufferAllocator {}", data.len());
        let buffer = allocator.create_not_updatable_buffer_pre(device, queue, &data, None).unwrap();

        let first = InstanceCacheBuffer {
            vertices: vec![],
            used_bytes: 0,
            buffer: (buffer, 0, one_mesh_max_instance_bytes),
        };
        Self {
            list: vec![first],
            used_index: 0,
            one_mesh_max_instance_bytes,
        }
    }
    pub fn instance_initial_buffer(&self) -> (Arc<NotUpdatableBufferRange>, u32, u32) {
        (self.list[0].buffer.0.clone(), 0, 0)
    }
    /// 默认都是 f32
    pub fn collect(&mut self, data: &[u8], bytes_per_instance: u32, allocator: &mut VertexBufferAllocator, device: &RenderDevice, queue: &RenderQueue) -> Option<(Arc<NotUpdatableBufferRange>, u32, u32)> {
        let max_count = self.one_mesh_max_instance_bytes / bytes_per_instance;
        let byte_size = data.len().min((max_count * bytes_per_instance) as usize);
        let bytes = &data[0..byte_size];

        if let Some(buffer) = self.list.get_mut(self.used_index) {
            if buffer.vertices.len() + buffer.used_bytes + bytes.len() > buffer.buffer.2 as usize {
                buffer.used_bytes = buffer.vertices.len();
                if buffer.vertices.len() > 0 {
                    queue.write_buffer(buffer.buffer.0.buffer(), 0, &buffer.vertices);
                    buffer.vertices.clear();
                }
                self.used_index += 1;
            }
        };
        if let Some(buffer) = self.list.get_mut(self.used_index)  {
            let start = buffer.vertices.len();
            bytes.iter().for_each(|v| { buffer.vertices.push(*v) });
            return Some((buffer.buffer.0.clone(), start as u32, buffer.vertices.len() as u32));
        } else {
            let mut data = Vec::with_capacity(self.one_mesh_max_instance_bytes as usize);
            bytes.iter().for_each(|v| { data.push(*v) });
            let vertices = data.clone();

            for _ in byte_size..self.one_mesh_max_instance_bytes as usize {
                data.push(0);
            }
            // log::error!("InstanceBufferAllocator Collect {}", data.len());
            if let Some(buffer) = allocator.create_not_updatable_buffer_pre(device, queue, &data, None) {
                self.list.push(InstanceCacheBuffer {
                    vertices,
                    used_bytes: 0,
                    buffer: (buffer, 0, self.one_mesh_max_instance_bytes as u32),
                    // key: KeyVertexBuffer::from(self.used_index.to_string().as_str()),
                });
                let buffer = self.list.get_mut(self.used_index).unwrap();
                return Some(
                    (buffer.buffer.0.clone(), 0, buffer.vertices.len() as u32)
                );
            } else {
                return None;
            }
        };
    }
    pub fn upload(&mut self, queue: &RenderQueue) {
        for idx in 0..(self.used_index + 1) {
            if let Some(buffer) = self.list.get_mut(idx) {
                if buffer.vertices.len() > 0 {
                    queue.write_buffer(buffer.buffer.0.buffer(), 0, &buffer.vertices);
                    buffer.vertices.clear();
                }
                buffer.used_bytes = 0;
            }
        }
        self.used_index = 0;
    }
}
