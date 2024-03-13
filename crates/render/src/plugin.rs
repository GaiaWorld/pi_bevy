use crate::{
    init_render::init_render, render_windows::RenderWindow, system::run_frame_system,
    PiAsyncRuntime, PiClearOptions, PiRenderDevice, PiRenderOptions, PiRenderWindow,
    PiSafeAtlasAllocator, PiScreenTexture,
};
use bevy_app::{App, Plugin, PostUpdate};

use bevy_ecs::prelude::IntoSystemConfigs;
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
pub use bevy_window::{should_run, FrameState};

/// ================ 阶段标签 ================

pub use bevy_window::FrameSet as PiRenderSystemSet;

/// ================ 插件 ================

#[derive(Default)]
pub struct PiRenderPlugin {
    pub frame_init_state: FrameState,
}

impl Plugin for PiRenderPlugin {
    fn build(&self, app: &mut App) {
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
            app.add_systems(
				PostUpdate,
                run_frame_system::<
                    pi_async_rt::rt::serial_local_compatible_wasm_runtime::LocalTaskRuntime,
                >
                    .in_set(PiRenderSystemSet)
                    .run_if(should_run),
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
		app.add_systems(
			PostUpdate,
			run_frame_system::<MultiTaskRuntime>
				.in_set(PiRenderSystemSet)
				.run_if(should_run),
		);

		#[cfg(all(not(target_arch = "wasm32"), feature = "single_thread"))]
		app.add_systems(
			PostUpdate,
			run_frame_system::<pi_async_rt::prelude::SingleTaskRuntime>
				.in_set(PiRenderSystemSet)
				.run_if(should_run),
		);

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
