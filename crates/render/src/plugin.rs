use std::mem::size_of;

use crate::{
    system::{init_render_system, run_frame_system},
    PiAsyncRuntime, PiRenderWindows, PiSingleTaskRunner,
};
use bevy::prelude::{App, CoreStage, Plugin, StageLabel, SystemStage};
use pi_async::prelude::{
    AsyncRuntimeBuilder, MultiTaskRuntime, SingleTaskRunner, SingleTaskRuntime,
};
use pi_bevy_assert::{ShareAssetMgr, ShareHomogeneousMgr};
use pi_render::{rhi::{asset::{RenderRes, TextureRes}, buffer::Buffer, bind_group::BindGroup, pipeline::RenderPipeline}, components::view::target_alloc::UnuseTexture};
use wgpu::TextureView;
use pi_assets::{asset::GarbageEmpty, homogeneous::HomogeneousMgr};

/// ================ 阶段标签 ================

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct PiRenderStage;

/// ================ 插件 ================

pub struct PiRenderPlugin;

impl Plugin for PiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(CoreStage::Last, PiRenderStage, SystemStage::parallel());

        #[cfg(target_arch = "wasm32")]
        let (rt, runner) = {
            app.add_startup_system(init_render_system::<SingleTaskRuntime>);
            app.add_system_to_stage(PiRenderStage, run_frame_system::<SingleTaskRuntime>);

            create_single_runtime()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (rt, runner) = {
            app.add_system_to_stage(PiRenderStage, run_frame_system::<MultiTaskRuntime>);
            app.add_startup_system(init_render_system::<MultiTaskRuntime>);

            create_multi_runtime()
        };

        app.insert_resource(PiSingleTaskRunner(runner))
            .insert_resource(PiAsyncRuntime(rt))
            .insert_resource(PiRenderWindows::default());
		
		
		// 添加资源管理器单例
		app.insert_resource(ShareAssetMgr::<RenderRes<Buffer>>::new(GarbageEmpty(), false, 20 * 1024 * 1024, 3 * 60 * 1000));
		app.insert_resource(ShareAssetMgr::<RenderRes<BindGroup>>::new(GarbageEmpty(), false, 5 * 1024, 3 * 60 * 1000));
		app.insert_resource(ShareAssetMgr::<RenderRes<TextureView>>::new(
			GarbageEmpty(),
			false,
			60 * 1024 * 1024,
			3 * 60 * 1000,
		));
		app.insert_resource(ShareAssetMgr::<TextureRes>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 3 * 60 * 1000));
		app.insert_resource(ShareAssetMgr::<RenderRes<RenderPipeline>>::new(
			GarbageEmpty(),
			false,
			60 * 1024 * 1024,
			3 * 60 * 1000,
		));
		// app.insert_resource(AssetMgr::<RenderRes<Program>>::new(
		// 	GarbageEmpty(),
		// 	false,
		// 	60 * 1024 * 1024,
		// 	3 * 60 * 1000,
		// ));
		app.insert_resource(ShareHomogeneousMgr::<RenderRes<UnuseTexture>>::new(
			pi_assets::homogeneous::GarbageEmpty(),
			10 * size_of::<UnuseTexture>(),
			size_of::<UnuseTexture>(),
			3 * 60 * 1000,
		));
    }
}

fn create_single_runtime() -> (SingleTaskRuntime, Option<SingleTaskRunner<()>>) {
    let mut runner = SingleTaskRunner::default();

    let runtime = runner.startup().unwrap();

    (runtime, Some(runner))
}

fn create_multi_runtime() -> (MultiTaskRuntime, Option<SingleTaskRunner<()>>) {
    let rt = AsyncRuntimeBuilder::default_multi_thread(Some("pi_bevy_render"), None, None, None);

    (rt, None)
}
