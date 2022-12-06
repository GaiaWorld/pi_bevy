use bevy_app::{Plugin, App};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::system::Resource;
use pi_postprocess::{postprocess_pipeline::PostProcessMaterialMgr, postprocess_geometry::PostProcessGeometryManager};

#[derive(Deref, DerefMut, Resource)]
pub struct PiPostProcessMaterialMgr(pub PostProcessMaterialMgr);

#[derive(Deref, DerefMut, Resource)]
pub struct PiPostProcessGeometryManager(pub PostProcessGeometryManager);

pub struct PiPostProcessPlugin;

impl Plugin for PiPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PiPostProcessMaterialMgr(PostProcessMaterialMgr::default()))
            .insert_resource(PiPostProcessGeometryManager(PostProcessGeometryManager::default()));
    }
}
