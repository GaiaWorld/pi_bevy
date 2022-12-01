use bevy::prelude::*;
use pi_async::rt::AsyncRuntime;

use crate::{
    init_render_system, run_frame_system, PiAsyncRuntime, PiRenderTargets, PiRenderWindows,
    PiTextureViews,
};

// 阶段
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct PiRenderStage;

/// ================ 插件 ================

pub struct PiRenderPlugin<A: 'static + AsyncRuntime + Send> {
    rt: A,
}

impl<A: 'static + AsyncRuntime + Send> PiRenderPlugin<A> {
    pub fn new(rt: A) -> Self {
        Self { rt }
    }
}

impl<A: 'static + AsyncRuntime + Send> PiRenderPlugin<A> {}

impl<A: 'static + AsyncRuntime + Send> Plugin for PiRenderPlugin<A> {
    fn build(&self, app: &mut App) {
        app.insert_resource(PiTextureViews::default())
            .insert_resource(PiRenderTargets::default())
            .insert_resource(PiRenderWindows::default())
            .insert_resource(PiAsyncRuntime(self.rt.clone()))
            .add_startup_system(init_render_system::<A>)
            .add_stage_after(CoreStage::Last, PiRenderStage, SystemStage::parallel())
            .add_system_to_stage(PiRenderStage, run_frame_system::<A>);
    }
}
