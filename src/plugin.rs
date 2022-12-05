use crate::{
    system::{init_render_system, run_frame_system},
    PiAsyncRuntime, PiRenderWindows, PiSingleTaskRunner,
};
use bevy::prelude::{App, CoreStage, Plugin, StageLabel, SystemStage};
use pi_async::prelude::{
    AsyncRuntimeBuilder, MultiTaskRuntime, SingleTaskRunner, SingleTaskRuntime,
};
use pi_share::{Share, ShareCell};

/// ================ 阶段标签 ================

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct PiRenderStage;

/// ================ 插件 ================

pub struct PiRenderPlugin;

impl Plugin for PiRenderPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        let (rt, runner) = {
            app.add_startup_system(init_render_system::<SingleTaskRuntime>);
            app.add_system_to_stage(PiRenderStage, run_frame_system::<SingleTaskRuntime>);

            create_single_runtime()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (rt, runner) = {
            app.add_startup_system(init_render_system::<MultiTaskRuntime>);
            app.add_system_to_stage(PiRenderStage, run_frame_system::<MultiTaskRuntime>);

            create_multi_runtime()
        };

        app.insert_resource(PiSingleTaskRunner(Share::new(ShareCell::new(runner))))
            .insert_resource(PiAsyncRuntime(rt))
            .insert_resource(PiRenderWindows::default())
            .add_stage_after(CoreStage::Last, PiRenderStage, SystemStage::parallel());
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
