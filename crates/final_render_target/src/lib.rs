
use std::ops::Deref;

use bevy::prelude::{Res, Plugin, Resource};
use pi_bevy_render_plugin::{node::Node, PiScreenTexture, PiRenderDevice, PiRenderWindow, PiRenderGraph};
use pi_render::{rhi::{pipeline::RenderPipeline, device::RenderDevice, BufferInitDescriptor, bind_group::BindGroup, sampler::SamplerDesc, bind_group_layout::BindGroupLayout, texture::{Texture, TextureView}, buffer::Buffer}, renderer::sampler::SamplerRes};
use wgpu::Extent3d;


#[derive(Resource)]
pub struct FinalRenderTarget {
    format: wgpu::TextureFormat,
    surface_format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
    vertex: Buffer,
    vs: wgpu::ShaderModule,
    fs: wgpu::ShaderModule,
    bindgroup_layout: BindGroupLayout,
    texture: Option<Texture>,
    sampler: Option<SamplerRes>,
    bindgroup: Option<BindGroup>,
    pipeline: Option<RenderPipeline>,
    view: Option<TextureView>,
}
impl FinalRenderTarget {
    pub const KEY: &'static str = "FinalRender";
    pub fn new(
        device: &RenderDevice,
        format: wgpu::TextureFormat,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let points = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
        let vertex = device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("FinalRender"),
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Final-VS"),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(include_str!("./pass.vert")),
                stage: naga::ShaderStage::Vertex,
                defines: naga::FastHashMap::default(),
            },
        });

        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Final-FS"),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(include_str!("./pass.frag")),
                stage: naga::ShaderStage::Fragment,
                defines: naga::FastHashMap::default(),
            },
        });


        let bindgroup_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None  },
                    wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering), count: None }
                ] 
            }
        );

        Self {
            format,
            surface_format,
            size: Extent3d::default(),
            vertex,
            vs,
            fs,
            bindgroup_layout,
            texture: None,
            sampler: None,
            bindgroup: None,
            pipeline: None,
            view: None,
        }

    }
    pub fn change(
        &mut self,
        format: wgpu::TextureFormat,
        surface_size: Extent3d,
        device: &RenderDevice,
    ) {
        if self.format != format || surface_size != self.size || self.pipeline.is_none() {
            let texture = device.create_texture(
                &wgpu::TextureDescriptor {
                    label: Some("Final"),
                    size: surface_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                }
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Final"),
                format: Some(format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            self.texture = Some(texture);
            self.view = Some(view);
            let sampler = SamplerRes::new(device, &SamplerDesc::default());
            self.sampler = Some(sampler);

            let bindgroup = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.bindgroup_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&self.view.as_ref().unwrap())  },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler.as_ref().unwrap().0)  },
                    ],
                }
            );
            self.bindgroup = Some(bindgroup);
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&self.bindgroup_layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Final"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState  {
                    module: &self.vs,
                    entry_point: "main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 2 * 4,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float16x2, offset: 0, shader_location: 0 }
                            ],
                        }
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    polygon_mode: wgpu::PolygonMode::Fill,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false  },
                fragment: Some(
                    wgpu::FragmentState { module: &self.fs, entry_point: "main", targets: &[Some(wgpu::ColorTargetState { format: self.surface_format, blend: None, write_mask: wgpu::ColorWrites::ALL })]  }
                ),
                multiview: None
            });
            self.pipeline = Some(pipeline);
        }
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn view(&self) -> Option<&TextureView> {
        self.view.as_ref()
    }
}

pub struct FinalRenderTargetNode;
impl Node for FinalRenderTargetNode {
    type Input = ();

    type Output = ();

    type Param = (Res<'static, PiScreenTexture>, Res<'static, FinalRenderTarget>);

    fn run<'a>(
        &'a mut self,
        world: &'a bevy::prelude::World,
        param: &'a mut bevy::ecs::system::SystemState<Self::Param>,
        _context: pi_bevy_render_plugin::RenderContext,
        mut commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        _: &'a Self::Input,
        _usage: &'a pi_bevy_render_plugin::node::ParamUsage,
    ) -> pi_futures::BoxFuture<'a, Result<Self::Output, String>> {

        let (screen, final_render) = param.get(world);

        if final_render.pipeline.is_some() {
            let mut rpass = commands.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some(FinalRenderTarget::KEY),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: screen.0.as_ref().unwrap().view.as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        })
                    ],
                    depth_stencil_attachment: None,
                }
            );
            rpass.set_pipeline(final_render.pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, final_render.bindgroup.as_ref().unwrap(), &[]);
            rpass.set_vertex_buffer(0, final_render.vertex.slice(..).deref().clone());
            rpass.draw(0..6, 0..1);

        }

        Box::pin(async move {
            Ok(())
        })
    }
}

#[derive(Debug, Default)]
pub struct PluginFinalRender;
impl Plugin for PluginFinalRender {
    fn build(&self, app: &mut bevy::prelude::App) {
        let window = app.world.get_resource::<PiRenderWindow>().unwrap();
        let surface_size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
        let device = app.world.get_resource::<PiRenderDevice>().unwrap();

        let mut node = FinalRenderTarget::new(device, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Bgra8Unorm);
        node.change(wgpu::TextureFormat::Rgba8Unorm, surface_size, device);

        let mut rg = app.world.get_resource_mut::<PiRenderGraph>().unwrap();
        rg.add_node(FinalRenderTarget::KEY, FinalRenderTargetNode);
        rg.set_finish(FinalRenderTarget::KEY, true);

        app.insert_resource(node);
    }
}