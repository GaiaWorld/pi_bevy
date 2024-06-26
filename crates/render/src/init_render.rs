use std::mem::transmute;

use crate::PiFirstSurface;
use crate::{
    graph::graph::RenderGraph, PiAdapterInfo, PiRenderDevice, PiRenderGraph, PiRenderInstance,
    PiRenderOptions, PiRenderQueue,
};
use bevy_ecs::prelude::With;
use bevy_ecs::world::World;
use bevy_window::{PrimaryWindow, HandleWrapper};
use log::debug;
use pi_async_rt::prelude::{AsyncRuntime, AsyncRuntimeExt};
use pi_render::rhi::texture::PiRenderDefault;
use pi_render::rhi::{
    device::{RenderDevice, initialize_renderer},
    options::RenderOptions,
    RenderInstance, RenderQueue,
};
use pi_assets::allocator::Allocator;
use wgpu::{InstanceFlags, Gles3MinorVersion};

pub(crate) fn init_render<A: AsyncRuntime + AsyncRuntimeExt>(
    world: &mut World,
    rt: &A,
) -> (HandleWrapper, wgpu::PresentMode) {
    let options = world.resource::<PiRenderOptions>().0.clone();
    // let windows = world.resource_mut::<bevy::prelude::Windows>();

    let mut primary_window = world.query_filtered::<&HandleWrapper, With<PrimaryWindow>>();
    let primary_window_handle = primary_window.single(world).clone();
    let mode = options.present_mode;

    init_render_impl(world, rt, &primary_window_handle, options);

    (primary_window_handle.clone(), mode)
}

// 初始化 渲染环境 的 System
//
// A 的 类型 见 plugin 模块
//   + wasm 环境 是 SingleTaskRuntime
//   + 否则 是 MultiTaskRuntime
//
fn init_render_impl<A: AsyncRuntime + AsyncRuntimeExt>(
    world: &mut World,
    rt: &A,
    window: &HandleWrapper,
    options: RenderOptions,
) {
    let backends = options.backends;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        flags: InstanceFlags::VALIDATION,
        gles_minor_version: Gles3MinorVersion::Automatic,
    });

    let surface = window.handle.create_surface(&instance);

    let mut allocator = world.get_resource_mut::<pi_bevy_asset::Allocator>().unwrap();
    let allocator1 = &mut allocator.0;
    let allocator1: &'static mut Allocator = unsafe {transmute(allocator1)};
    let SetupResult {
        surface,
        instance,
        device,
        queue,
        adapter_info,
    } = rt
        .block_on(setup_render_context(instance, surface, options, allocator1))
        .unwrap();

    let instance = instance.unwrap();
    let device = device.unwrap();
    let queue = queue.unwrap();
    let adapter_info = adapter_info.unwrap();

    let rg = RenderGraph::new(device.clone(), queue.clone());

    // 注：之所以写到这里，是因为 Bevy 的 内置类型 不能 传到 pi_async 的 future中。
    world.insert_resource(PiFirstSurface(surface));
    world.insert_resource(PiRenderGraph(rg));
    world.insert_resource(PiRenderInstance(instance));
    world.insert_resource(PiRenderDevice(device));
    world.insert_resource(PiRenderQueue(queue));
    world.insert_resource(PiAdapterInfo(adapter_info));
}

#[derive(Default)]
struct SetupResult<'a> {
    pub surface: Option<wgpu::Surface<'a>>,
    pub instance: Option<RenderInstance>,
    pub device: Option<RenderDevice>,
    pub queue: Option<RenderQueue>,
    pub adapter_info: Option<wgpu::AdapterInfo>,
}

/// 初始化 渲染 环境
async fn setup_render_context<'a>(
    instance: RenderInstance,
    surface: wgpu::Surface<'a>,
    options: RenderOptions,
    alloter: &mut Allocator,
) -> SetupResult<'a> {
    let request_adapter_options = wgpu::RequestAdapterOptions {
        power_preference: options.power_preference,
        compatible_surface: Some(&surface),
        ..Default::default()
    };
    let (device, queue, adapter_info) =
        initialize_renderer(&instance, &options, &request_adapter_options, alloter).await;

    let config = wgpu::SurfaceConfiguration {
        format: wgpu::TextureFormat::pi_render_default(),
        width: 1,
        height: 1,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
    debug!("Configured wgpu adapter Features: {:#?}", device.features());

    SetupResult {
        surface: Some(surface),
        instance: Some(instance),
        device: Some(device),
        queue: Some(queue),
        adapter_info: Some(adapter_info),
    }
}

// /// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
// /// for the specified backend.
// async fn initialize_renderer(
//     instance: &wgpu::Instance,
//     options: &RenderOptions,
//     request_adapter_options: &wgpu::RequestAdapterOptions<'_, '_>,
// ) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
//     let adapter = instance
//         .request_adapter(request_adapter_options)
//         .await
//         .expect("Unable to find a GPU! Make sure you have installed required drivers!");

//     let adapter_info = adapter.get_info();
//     println!("initialize_renderer, adapter_info={:?}, options={:?}", adapter_info, options);

//     // #[cfg(feature = "trace")]
//     // let trace_path = {
//     //     let path = std::path::Path::new("wgpu_trace");
//     //     // ignore potential error, wgpu will log it
//     //     let _ = std::fs::create_dir(path);
//     //     Some(path)
//     // };

//     // #[cfg(not(feature = "trace"))]
//     let trace_path = None;

//     // Maybe get features and limits based on what is supported by the adapter/backend
//     let mut features = wgpu::Features::empty();
//     let mut limits = options.limits.clone();
//     if matches!(options.priority, RenderPriority::Functionality) {
//         features = adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
//         if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
//             // `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
//             // discrete GPUs due to having to transfer data across the PCI-E bus and so it
//             // should not be automatically enabled in this case. It is however beneficial for
//             // integrated GPUs.
//             features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
//         }
//         limits = adapter.limits();
//     }

//     // Enforce the disabled features
//     if let Some(disabled_features) = options.disabled_features {
//         features -= disabled_features;
//     }
//     // NOTE: |= is used here to ensure that any explicitly-enabled features are respected.
//     features |= options.features;

//     // Enforce the limit constraints
//     if let Some(constrained_limits) = options.constrained_limits.as_ref() {
//         // NOTE: Respect the configured limits as an 'upper bound'. This means for 'max' limits, we
//         // take the minimum of the calculated limits according to the adapter/backend and the
//         // specified max_limits. For 'min' limits, take the maximum instead. This is intended to
//         // err on the side of being conservative. We can't claim 'higher' limits that are supported
//         // but we can constrain to 'lower' limits.
//         limits = wgpu::Limits {
//             max_texture_dimension_1d: limits
//                 .max_texture_dimension_1d
//                 .min(constrained_limits.max_texture_dimension_1d),
//             max_texture_dimension_2d: limits
//                 .max_texture_dimension_2d
//                 .min(constrained_limits.max_texture_dimension_2d),
//             max_texture_dimension_3d: limits
//                 .max_texture_dimension_3d
//                 .min(constrained_limits.max_texture_dimension_3d),
//             max_texture_array_layers: limits
//                 .max_texture_array_layers
//                 .min(constrained_limits.max_texture_array_layers),
//             max_bind_groups: limits
//                 .max_bind_groups
//                 .min(constrained_limits.max_bind_groups),
//             max_dynamic_uniform_buffers_per_pipeline_layout: limits
//                 .max_dynamic_uniform_buffers_per_pipeline_layout
//                 .min(constrained_limits.max_dynamic_uniform_buffers_per_pipeline_layout),
//             max_dynamic_storage_buffers_per_pipeline_layout: limits
//                 .max_dynamic_storage_buffers_per_pipeline_layout
//                 .min(constrained_limits.max_dynamic_storage_buffers_per_pipeline_layout),
//             max_sampled_textures_per_shader_stage: limits
//                 .max_sampled_textures_per_shader_stage
//                 .min(constrained_limits.max_sampled_textures_per_shader_stage),
//             max_samplers_per_shader_stage: limits
//                 .max_samplers_per_shader_stage
//                 .min(constrained_limits.max_samplers_per_shader_stage),
//             max_storage_buffers_per_shader_stage: limits
//                 .max_storage_buffers_per_shader_stage
//                 .min(constrained_limits.max_storage_buffers_per_shader_stage),
//             max_storage_textures_per_shader_stage: limits
//                 .max_storage_textures_per_shader_stage
//                 .min(constrained_limits.max_storage_textures_per_shader_stage),
//             max_uniform_buffers_per_shader_stage: limits
//                 .max_uniform_buffers_per_shader_stage
//                 .min(constrained_limits.max_uniform_buffers_per_shader_stage),
//             max_uniform_buffer_binding_size: limits
//                 .max_uniform_buffer_binding_size
//                 .min(constrained_limits.max_uniform_buffer_binding_size),
//             max_storage_buffer_binding_size: limits
//                 .max_storage_buffer_binding_size
//                 .min(constrained_limits.max_storage_buffer_binding_size),
//             max_vertex_buffers: limits
//                 .max_vertex_buffers
//                 .min(constrained_limits.max_vertex_buffers),
//             max_vertex_attributes: limits
//                 .max_vertex_attributes
//                 .min(constrained_limits.max_vertex_attributes),
//             max_vertex_buffer_array_stride: limits
//                 .max_vertex_buffer_array_stride
//                 .min(constrained_limits.max_vertex_buffer_array_stride),
//             max_push_constant_size: limits
//                 .max_push_constant_size
//                 .min(constrained_limits.max_push_constant_size),
//             min_uniform_buffer_offset_alignment: limits
//                 .min_uniform_buffer_offset_alignment
//                 .max(constrained_limits.min_uniform_buffer_offset_alignment),
//             min_storage_buffer_offset_alignment: limits
//                 .min_storage_buffer_offset_alignment
//                 .max(constrained_limits.min_storage_buffer_offset_alignment),
//             max_inter_stage_shader_components: limits
//                 .max_inter_stage_shader_components
//                 .min(constrained_limits.max_inter_stage_shader_components),
//             max_compute_workgroup_storage_size: limits
//                 .max_compute_workgroup_storage_size
//                 .min(constrained_limits.max_compute_workgroup_storage_size),
//             max_compute_invocations_per_workgroup: limits
//                 .max_compute_invocations_per_workgroup
//                 .min(constrained_limits.max_compute_invocations_per_workgroup),
//             max_compute_workgroup_size_x: limits
//                 .max_compute_workgroup_size_x
//                 .min(constrained_limits.max_compute_workgroup_size_x),
//             max_compute_workgroup_size_y: limits
//                 .max_compute_workgroup_size_y
//                 .min(constrained_limits.max_compute_workgroup_size_y),
//             max_compute_workgroup_size_z: limits
//                 .max_compute_workgroup_size_z
//                 .min(constrained_limits.max_compute_workgroup_size_z),
//             max_compute_workgroups_per_dimension: limits
//                 .max_compute_workgroups_per_dimension
//                 .min(constrained_limits.max_compute_workgroups_per_dimension),
//             max_buffer_size: limits
//                 .max_buffer_size
//                 .min(constrained_limits.max_buffer_size),
//             max_bindings_per_bind_group: limits
//                 .max_bindings_per_bind_group
//                 .min(constrained_limits.max_bindings_per_bind_group),
//             max_non_sampler_bindings: limits
// 				.max_non_sampler_bindings
// 				.min(constrained_limits.max_non_sampler_bindings),
//         };
//     }

//     let (device, queue) = adapter
//         .request_device(
//             &wgpu::DeviceDescriptor {
//                 label: options.device_label.as_ref().map(|a| a.as_ref()),
//                 required_features: features,
//                 required_limits: limits,
//             },
//             trace_path,
//         )
//         .await
//         .unwrap();
//     let device = Share::new(device);
//     let queue = Share::new(queue);

//     (RenderDevice::from(device), queue, adapter_info)
// }
