
use std::{ops::Deref, sync::Arc};

use bevy::{prelude::{Res, Plugin, Resource, ResMut, IntoSystemConfig, CoreSet, Commands, Deref, Entity}, ecs::system::CommandQueue};
use pi_bevy_render_plugin::{node::Node, PiScreenTexture, PiRenderDevice, PiRenderWindow, PiRenderGraph, SimpleInOut, ClearOptions, CLEAR_WIDNOW_NODE, component::GraphId, NodeId};
use pi_render::{rhi::{pipeline::RenderPipeline, device::RenderDevice, BufferInitDescriptor, bind_group::BindGroup, sampler::SamplerDesc, bind_group_layout::BindGroupLayout, texture::{Texture, TextureView, PiRenderDefault}, buffer::Buffer}, renderer::sampler::SamplerRes};
use wgpu::Extent3d;

#[derive(Resource)]
pub struct WindowRenderer {
    format: wgpu::TextureFormat,
    surface_format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
    vertex: Buffer,
    vs: wgpu::ShaderModule,
    fs: wgpu::ShaderModule,
    bindgroup_layout: BindGroupLayout,
    texture: Option<Arc<wgpu::Texture>>,
    view: Option<Arc<wgpu::TextureView>>,
    sampler: Option<SamplerRes>,
    bindgroup: Option<BindGroup>,
    pipeline: Option<RenderPipeline>,
    depth_texture: Option<Arc<wgpu::Texture>>,
    depth_view: Option<Arc<wgpu::TextureView>>,
    pub clearcolor: wgpu::Color,
    pub cleardepth: f32,
    pub clearstencil: u32,
    pub active: bool,
    pub clear_entity: Entity,
    pub clear_node: NodeId,
    pub render_entity: Entity,
    pub render_node: NodeId,
}
impl WindowRenderer {
    pub const CLEAR_KEY: &'static str = "FinalRenderClear";
    pub const KEY: &'static str = "FinalRender";
    pub fn new(
        device: &RenderDevice,
        format: wgpu::TextureFormat,
        surface_format: wgpu::TextureFormat,
        clear_entity: Entity,
        clear_node: NodeId,
        render_entity: Entity,
        render_node: NodeId,
    ) -> Self {
        let points: [f32; 12] = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
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
                    wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering), count: None }
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
            sampler: None,
            bindgroup: None,
            pipeline: None,
            texture: None,
            view: None,
            depth_texture: None,
            depth_view: None,
            clearcolor: wgpu::Color { r: 0., g: 0., b: 0., a: 0.  },
            cleardepth: 1.0,
            clearstencil: 0,
            active: false,
            clear_entity,
            clear_node,
            render_entity,
            render_node,
        }

    }
    pub fn change(
        &mut self,
        format: wgpu::TextureFormat,
        surface_size: Extent3d,
        device: &RenderDevice,
    ) {
        if !self.active {
            return;
        }
        if self.format != format || surface_size != self.size || self.pipeline.is_none() {
            log::warn!("FinaleRender ChangeSize {:?}", surface_size);
            self.size = surface_size;
            self.format = format;
            let texture = (**device).create_texture(
                &wgpu::TextureDescriptor {
                    label: Some("Final"),
                    size: surface_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
					view_formats: &[],
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
            self.texture = Some(Arc::new(texture));
            self.view = Some(Arc::new(view));
            
            let texture = (**device).create_texture(
                &wgpu::TextureDescriptor {
                    label: Some("Final"),
                    size: surface_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
					view_formats: &[],
                }
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Final"),
                format: Some(wgpu::TextureFormat::Depth24PlusStencil8),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            self.depth_texture = Some(Arc::new(texture));
            self.depth_view = Some(Arc::new(view));

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
                                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 }
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
            
            log::warn!("FinaleRender ChangeSize Ok!");
            self.pipeline = Some(pipeline);
        }
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn view(&self) -> Option<&Arc<wgpu::TextureView>> {
        self.view.as_ref()
    }
    pub fn depth_view(&self) -> Option<&Arc<wgpu::TextureView>> {
        self.depth_view.as_ref()
    }
    pub fn size(&self) -> Extent3d {
        self.size
    }
}

pub struct WindowRendererNode;
impl Node for WindowRendererNode {
    type Input = SimpleInOut;

    type Output = ();

    type Param = (Res<'static, PiScreenTexture>, Res<'static, WindowRenderer>);

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
                    label: Some(WindowRenderer::KEY),
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

pub struct WindowRendererClearNode;
impl Node for WindowRendererClearNode {
    type Input = ();

    type Output = ();

    type Param = Res<'static, WindowRenderer>;

    fn run<'a>(
        &'a mut self,
        world: &'a bevy::prelude::World,
        param: &'a mut bevy::ecs::system::SystemState<Self::Param>,
        context: pi_bevy_render_plugin::RenderContext,
        mut commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a pi_bevy_render_plugin::node::ParamUsage,
    ) -> pi_futures::BoxFuture<'a, Result<Self::Output, String>> {
        let final_render = param.get(world);
        if let (Some(view), Some(depth_view)) = (final_render.view(), &final_render.depth_view) {
            let mut rpass = commands.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some(WindowRenderer::CLEAR_KEY),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(final_render.clearcolor),
                                store: true,
                            },
                        })
                    ],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(final_render.cleardepth),
                                    store: true,
                                }
                            ),
                            stencil_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(final_render.clearstencil),
                                    store: true,
                                }
                            )
                        }
                    ),
                }
            );
        }

        Box::pin(async move {
            Ok(())
        })
    }
}

fn sys_changesize(
    window: Res<PiRenderWindow>,
    device: Res<PiRenderDevice>,
    mut final_render: ResMut<WindowRenderer>,
) {
    if window.width > 0 && window.height > 0 {
        let surface_size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
        final_render.change(wgpu::TextureFormat::Rgba8Unorm, surface_size, &device);
    }
}

#[derive(Debug, Default)]
pub struct PluginWindowRender;
impl Plugin for PluginWindowRender {
    fn build(&self, app: &mut bevy::prelude::App) {
        
        // #[cfg(not(target_arch="wasm32"))]
        // {
            let id_clear = app.world.spawn_empty().id();
            let id_render = app.world.spawn_empty().id();

            let device = app.world.get_resource::<PiRenderDevice>().unwrap().0.clone();


            let mut rg = app.world.get_resource_mut::<PiRenderGraph>().unwrap();
            let node_clear = rg.add_node(WindowRenderer::CLEAR_KEY, WindowRendererClearNode).unwrap();
            let node_render = rg.add_node(WindowRenderer::KEY, WindowRendererNode).unwrap();
            rg.set_finish(WindowRenderer::KEY, true).unwrap();
            rg.add_depend(CLEAR_WIDNOW_NODE, WindowRenderer::CLEAR_KEY).unwrap();

            let mut cmdqueue = CommandQueue::default();
            let mut cmds = Commands::new(&mut cmdqueue, &app.world);
            cmds.entity(id_clear).insert(GraphId(node_clear));
            cmds.entity(id_render).insert(GraphId(node_render));

            cmdqueue.apply(&mut app.world);

            let node = WindowRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::pi_render_default(), id_clear, node_clear, id_render, node_render);
            app.insert_resource(node);
            app.add_system(sys_changesize.in_base_set(CoreSet::First));
        // }
    }
}