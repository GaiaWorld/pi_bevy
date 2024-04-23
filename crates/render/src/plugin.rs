use crate::system::build_graph;
use crate::TextureKeyAlloter;
use crate::{
    init_render::init_render, render_windows::RenderWindow, system::run_frame_system,
    PiAsyncRuntime, PiClearOptions, PiRenderDevice, PiRenderOptions, PiRenderWindow,
    PiSafeAtlasAllocator, PiScreenTexture,
};
// use bevy_app::{App, Plugin, PostUpdate, Update};

// use bevy_ecs::prelude::IntoSystemConfigs;
// use bevy_ecs::schedule::{SystemSet, IntoSystemSetConfig, IntoSystemSetConfigs};
pub use bevy_window::{should_run, FrameState};
use pi_assets::asset::GarbageEmpty;
use pi_async_rt::prelude::*;
use pi_bevy_asset::{Allocator, AssetConfig, AssetDesc, ShareAssetMgr, ShareHomogeneousMgr};
use pi_render::renderer::sampler::SamplerRes;
use pi_render::{
    components::view::target_alloc::{SafeAtlasAllocator, UnuseTexture},
    rhi::{
        asset::{AssetWithId, RenderRes, TextureRes},
        bind_group::BindGroup,
        buffer::Buffer,
        pipeline::RenderPipeline,
    },
};
use pi_world::prelude::App;
use pi_world_extend_plugin::plugin::Plugin;
use std::mem::size_of;
use wgpu::TextureView;

/// ================ 阶段标签 ================
pub use bevy_window::FrameSet as PiRenderSystemSet;

/// 图构建系统集（一些资源更新显存可能需要在图构建之后）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GraphBuild;

/// 图运行系统集（一些资源更新显存可能需要在图构建之后）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GraphRun;

/// 帧数据准备（实际上就是在FrameDataPrepare系统集中的system，添加了FrameState::Active的运行条件）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FrameDataPrepare;

/// ================ 插件 ================

#[derive(Default)]
pub struct PiRenderPlugin {
    pub frame_init_state: FrameState,
}

impl Plugin for PiRenderPlugin {
    fn build(&self, app: &mut App) {
        // app
        // 	.configure_set(Update, FrameDataPrepare.run_if(should_run))
        // 	.configure_set(PostUpdate, PiRenderSystemSet.run_if(should_run))
        // 	.configure_set(PostUpdate, GraphBuild.in_set(PiRenderSystemSet))
        // 	.configure_set(PostUpdate, GraphRun.in_set(PiRenderSystemSet))
        // 	.configure_sets(PostUpdate, (GraphBuild, GraphRun).chain());
        // std::thread::spawn(move || {
        // 	loop {
        // 		{
        // 			let mut l = pi_hal::runtime::LOGS.lock();
        // 			if l.0.len() > 0 {
        // 				log::warn!("{}", l.0.join("\n"));
        // 				l.0.clear();
        // 			}
        // 		}

        // 		std::thread::sleep(std::time::Duration::from_millis(2000));
        // 	}
        // });

        app.world.register_single_res(self.frame_init_state);

        app.world.register_single_res(PiScreenTexture::default());

        if app.world.get_single_res::<PiRenderOptions>().is_none() {
            app.world.register_single_res(PiRenderOptions::default());
        }
        if app.world.get_single_res::<PiClearOptions>().is_none() {
            app.world.register_single_res(PiClearOptions::default());
        }

        // app.add_stage_after(CoreStage::Last, PiRenderStage, SystemStage::parallel().with_run_criteria(should_run));

        #[cfg(target_arch = "wasm32")]
        let rt = {
            app.schedule.add_system(
                run_frame_system::<
                    pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRuntime,
                >,
            );
            app.schedule.add_system(
                build_graph::<
                    pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRuntime,
                >,
            );
            pi_hal::runtime::RENDER_RUNTIME.clone()
            // create_single_runtime()
        };

        #[cfg(all(not(target_arch = "wasm32"), not(feature = "single_thread")))]
        let rt = pi_hal::runtime::RENDER_RUNTIME.clone();
        #[cfg(all(not(target_arch = "wasm32"), feature = "single_thread"))]
        let rt = pi_hal::runtime::RENDER_RUNTIME.clone();
        // let rt = create_single_runtime();
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "single_thread")))]
        app.schedule
            .add_system(run_frame_system::<MultiTaskRuntime>);
        app.schedule.add_system(build_graph::<MultiTaskRuntime>);

        #[cfg(all(not(target_arch = "wasm32"), feature = "single_thread"))]
        app.schedule
            .add_system(run_frame_system::<pi_async_rt::prelude::SingleTaskRuntime>);
        app.schedule
            .add_system(build_graph::<pi_async_rt::prelude::SingleTaskRuntime>);

        let (
            share_texture_res,
            share_unuse,
            buffer_res,
            sampler_res,
            bind_group_res,
            texture_res,
            texture_asset_res,
            pipeline_res,
        ) = {
            // let w = &mut app.world;
            let mut unsafe_world = app.world.unsafe_world();
            let mut allocator = unsafe_world.get_single_res_mut::<Allocator>().unwrap();
            let unsafe_world = app.world.unsafe_world();
            let asset_config = unsafe_world.get_single_res::<AssetConfig>().unwrap();

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
                ShareAssetMgr::<AssetWithId<TextureRes>>::new_with_config(
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

        app.world.register_single_res(share_texture_res.clone());
        app.world.register_single_res(share_unuse.clone());

        app.world.register_single_res(PiAsyncRuntime(rt.clone()));

        // 添加资源管理器单例
        app.world.register_single_res(buffer_res);

        app.world.register_single_res(sampler_res);

        app.world.register_single_res(bind_group_res);

        app.world.register_single_res(texture_res);

        app.world.register_single_res(texture_asset_res.clone());
        app.world.register_single_res(pipeline_res);
        // app.insert_resource(AssetMgr::<RenderRes<Program>>::new(
        // 	GarbageEmpty(),
        // 	false,
        // 	60 * 1024 * 1024,
        // 	3 * 60 * 1000,
        // ));

        let (wrapper, present_mode) = init_render(&mut app.world, &rt);

        app.world
            .register_single_res(PiRenderWindow(RenderWindow::new(wrapper, present_mode)));
        let texture_key_alloter = TextureKeyAlloter::default();
        app.world.register_single_res(texture_key_alloter.clone());

        let device = app.world.get_single_res::<PiRenderDevice>().unwrap();
        app.world
            .register_single_res(PiSafeAtlasAllocator(SafeAtlasAllocator::new(
                device.0.clone(),
                texture_asset_res.0,
                share_unuse.0,
                texture_key_alloter.0.clone(),
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

// #[cfg(not(target_arch = "wasm32"))]
// fn create_multi_runtime() -> MultiTaskRuntime {
//     let rt = AsyncRuntimeBuilder::default_multi_thread(Some("pi_bevy_render"), None, None, None);
//     rt
// }

#[cfg(all(not(target_arch = "wasm32"), feature = "single_thread"))]
fn create_single_runtime() -> SingleTaskRuntime {
    let runner = SingleTaskRunner::default();
    let rt = runner.into_local();

    rt
}
