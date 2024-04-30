// use bevy_app::{Plugin, App};
// use bevy_ecs::system::Resource;
use pi_assets::mgr::AssetMgr;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_render_plugin::{PiRenderDevice, PiRenderQueue};
use pi_postprocess::prelude::*;
use pi_render::renderer::sampler::SamplerRes;
use pi_render::renderer::vertex_buffer::VertexBufferAllocator;
use pi_render::rhi::RenderQueue;
use pi_render::rhi::device::RenderDevice;
use pi_share::Share;
use pi_world::prelude::{App, Plugin};
// use pi_postprocess::{postprocess_pipeline::PostProcessMaterialMgr, postprocess_geometry::PostProcessGeometryManager};

// #[derive(Deref, DerefMut, Resource)]
// pub struct PiPostProcessMaterialMgr(pub PostProcessMaterialMgr);

// #[derive(Deref, DerefMut, Resource)]
// pub struct PiPostProcessGeometryManager(pub PostProcessGeometryManager);


pub struct PostprocessResource {
    pub vballocator: VertexBufferAllocator,
    pub resources: SingleImageEffectResource,
}

// TODO Send问题， 临时解决
unsafe impl Send for PostprocessResource {}
unsafe impl Sync for PostprocessResource {}

impl PostprocessResource {
    pub fn new(renderdevice: &RenderDevice, queue: &RenderQueue, asset_samplers: &Share<AssetMgr<SamplerRes>>) -> Self {
        let mut vballocator = VertexBufferAllocator::new(64 * 1024, 60 * 1000);
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
        EffectBlurGauss::setup(&renderdevice, &mut resources, asset_samplers);
        EffectImageMask::setup(&renderdevice, &mut resources, asset_samplers);
        EffectClipSdf::setup(&renderdevice, &mut resources, asset_samplers);

        Self {
            vballocator,
            resources,
        }
    }
}

pub struct PiPostProcessPlugin;

impl Plugin for PiPostProcessPlugin {
    fn build(&self, app: &mut App) {
        let device = app.world.get_single_res::<PiRenderDevice>().unwrap();
        let queue = app.world.get_single_res::<PiRenderQueue>().unwrap();
        let asset_samplers = app.world.get_single_res::<ShareAssetMgr<SamplerRes>>().unwrap();
        app.world.insert_single_res(PostprocessResource::new(device, queue, asset_samplers));
    }
}
