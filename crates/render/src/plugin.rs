use crate::{
    init_render::init_render, render_windows::RenderWindow, system::run_frame_system,
    PiAsyncRuntime, PiClearOptions, PiRenderDevice, PiRenderOptions, PiRenderWindow,
    PiSafeAtlasAllocator, PiScreenTexture,
};
use bevy::app::{App, Plugin};

use bevy::ecs::system::Res;
use bevy::prelude::{IntoSystemConfig, Resource, SystemSet};
use pi_assets::asset::GarbageEmpty;
use pi_async_rt::prelude::*;
use pi_bevy_asset::{Allocator, AssetConfig, AssetDesc, ShareAssetMgr, ShareHomogeneousMgr};
use pi_render::renderer::sampler::SamplerRes;
use pi_render::{
    components::view::target_alloc::{SafeAtlasAllocator, UnuseTexture},
    rhi::{
        asset::{RenderRes, TextureRes},
        bind_group::BindGroup,
        buffer::Buffer,
        pipeline::RenderPipeline,
    },
};
use std::mem::size_of;
use wgpu::TextureView;

/// ================ 阶段标签 ================

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct PiRenderSystemSet;

#[derive(Debug, Default, Resource, Clone, Copy)]
pub enum FrameState {
    #[default]
    Active,
    UnActive,
}

pub fn should_run(state: Res<FrameState>) -> bool {
    if let FrameState::Active = *state {
        true
    } else {
        false
    }
}

/// ================ 插件 ================

#[derive(Default)]
pub struct PiRenderPlugin {
    pub frame_init_state: FrameState,
}

impl Plugin for PiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.frame_init_state);

        app.insert_resource(PiScreenTexture::default());

        if app.world.get_resource::<PiRenderOptions>().is_none() {
            app.insert_resource(PiRenderOptions::default());
        }
        if app.world.get_resource::<PiClearOptions>().is_none() {
            app.insert_resource(PiClearOptions::default());
        }

        // app.add_stage_after(CoreStage::Last, PiRenderStage, SystemStage::parallel().with_run_criteria(should_run));

        #[cfg(target_arch = "wasm32")]
        let rt = {
            app.add_system(
                run_frame_system::<
                    pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRuntime,
                >
                    .in_set(PiRenderSystemSet)
                    .run_if(should_run),
            );

            create_single_runtime()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let rt = {
            app.add_system(
                run_frame_system::<MultiTaskRuntime>
                    .in_set(PiRenderSystemSet)
                    .run_if(should_run),
            );

            create_multi_runtime()
            // create_single_runtime()
        };

        let (
            share_texture_res,
            share_unuse,
            buffer_res,
            sampler_res,
            bind_group_res,
            texture_res,
            pipeline_res,
        ) = {
            let w = app.world.cell();
            let mut allocator = w.get_resource_mut::<Allocator>().unwrap();
            let asset_config = w.get_resource::<AssetConfig>().unwrap();
            (
                ShareAssetMgr::<RenderRes<TextureView>>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 30 * 1024 * 1024,
                        max: 600 * 1024 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareHomogeneousMgr::<RenderRes<UnuseTexture>>::new_with_config(
                    pi_assets::homogeneous::GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 10 * size_of::<UnuseTexture>(),
                        max: 20 * size_of::<UnuseTexture>(),
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareAssetMgr::<RenderRes<Buffer>>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 10 * 1024 * 1024,
                        max: 50 * 1024 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareAssetMgr::<SamplerRes>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 10 * 1024,
                        max: 20 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareAssetMgr::<RenderRes<BindGroup>>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 5 * 1024 * 1024,
                        max: 10 * 1024 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareAssetMgr::<TextureRes>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 10 * 1024 * 1024,
                        max: 600 * 1024 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
                ShareAssetMgr::<RenderRes<RenderPipeline>>::new_with_config(
                    GarbageEmpty(),
                    &AssetDesc {
                        ref_garbage: false,
                        min: 5 * 1024 * 1024,
                        max: 10 * 1024 * 1024,
                        timeout: 10 * 60 * 1000,
                    },
                    &asset_config,
                    &mut allocator,
                ),
            )
        };

        app.insert_resource(share_texture_res.clone());
        app.insert_resource(share_unuse.clone());

        app.insert_resource(PiAsyncRuntime(rt.clone()));

        // 添加资源管理器单例
        app.insert_resource(buffer_res);

        app.insert_resource(sampler_res);

        app.insert_resource(bind_group_res);

        app.insert_resource(texture_res);
        app.insert_resource(pipeline_res);
        // app.insert_resource(AssetMgr::<RenderRes<Program>>::new(
        // 	GarbageEmpty(),
        // 	false,
        // 	60 * 1024 * 1024,
        // 	3 * 60 * 1000,
        // ));

        let (wrapper, present_mode) = init_render(&mut app.world, &rt);

        app.insert_resource(PiRenderWindow(RenderWindow::new(wrapper, present_mode)));

        let device = app.world.get_resource::<PiRenderDevice>().unwrap();
        app.insert_resource(PiSafeAtlasAllocator(SafeAtlasAllocator::new(
            device.0.clone(),
            share_texture_res.0,
            share_unuse.0,
        )));
    }
}

#[cfg(target_arch = "wasm32")]
fn create_single_runtime() -> pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRuntime
{
    let mut runner = pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRunner::new();
    let rt = runner.get_runtime();

    rt
}

#[cfg(not(target_arch = "wasm32"))]
fn create_multi_runtime() -> MultiTaskRuntime {
    let rt = AsyncRuntimeBuilder::default_multi_thread(Some("pi_bevy_render"), None, None, None);

    rt
}

#[cfg(not(target_arch = "wasm32"))]
fn create_single_runtime() -> SingleTaskRuntime {
    let runner = SingleTaskRunner::default();
    let rt = runner.into_local();

    rt
}
