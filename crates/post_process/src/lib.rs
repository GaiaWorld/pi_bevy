use bevy::app::{Plugin, App};
use bevy::ecs::system::Resource;
use pi_assets::mgr::AssetMgr;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_render_plugin::{PiRenderDevice, PiRenderQueue};
use pi_postprocess::image_effect::*;
use pi_render::renderer::sampler::SamplerRes;
use pi_render::renderer::vertex_buffer::VertexBufferAllocator;
use pi_render::rhi::RenderQueue;
use pi_render::rhi::device::RenderDevice;
use pi_share::Share;
// use pi_postprocess::{postprocess_pipeline::PostProcessMaterialMgr, postprocess_geometry::PostProcessGeometryManager};

// #[derive(Deref, DerefMut, Resource)]
// pub struct PiPostProcessMaterialMgr(pub PostProcessMaterialMgr);

// #[derive(Deref, DerefMut, Resource)]
// pub struct PiPostProcessGeometryManager(pub PostProcessGeometryManager);

#[derive(Resource)]
pub struct PostprocessResource {
    pub vballocator: VertexBufferAllocator,
    pub resources: SingleImageEffectResource,
}
impl PostprocessResource {
    pub fn new(renderdevice: &RenderDevice, queue: &RenderQueue, asset_samplers: &Share<AssetMgr<SamplerRes>>) -> Self {
        let mut vballocator = VertexBufferAllocator::new();
        let mut resources = SingleImageEffectResource::new(renderdevice, queue, &mut vballocator);
        EffectBlurBokeh::setup(&renderdevice, &mut resources, asset_samplers);
        EffectBlurDirect::setup(&renderdevice, &mut resources, asset_samplers);
        EffectBlurDual::setup(&renderdevice, &mut resources, asset_samplers);
        EffectBlurRadial::setup(&renderdevice, &mut resources, asset_samplers);
        EffectColorEffect::setup(&renderdevice, &mut resources, asset_samplers);
        EffectCopy::setup(&renderdevice, &mut resources, asset_samplers);
        EffectFilterBrightness::setup(&renderdevice, &mut resources, asset_samplers);
        EffectFilterSobel::setup(&renderdevice, &mut resources, asset_samplers);
        EffectHorizonGlitch::setup(&renderdevice, &mut resources, asset_samplers);
        EffectRadialWave::setup(&renderdevice, &mut resources, asset_samplers);

        Self {
            vballocator,
            resources,
        }
    }
}

pub struct PiPostProcessPlugin;

impl Plugin for PiPostProcessPlugin {
    fn build(&self, app: &mut App) {
        let device = app.world.get_resource::<PiRenderDevice>().unwrap();
        let queue = app.world.get_resource::<PiRenderQueue>().unwrap();
        let asset_samplers = app.world.get_resource::<ShareAssetMgr<SamplerRes>>().unwrap();
        app.insert_resource(PostprocessResource::new(device, queue, asset_samplers));
    }
}
