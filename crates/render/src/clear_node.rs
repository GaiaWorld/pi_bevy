use crate::{constant::texture_sampler::ColorFormat, node::Node, PiClearOptions, PiSafeAtlasAllocator, PiScreenTexture, RenderContext, SimpleInOut};
// use bevy_ecs::{
//     system::{Res, SystemState},
//     world::World,
// };
use pi_futures::BoxFuture;
use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, SafeTargetView, TargetDescriptor, TargetType, TextureDescriptor}};
use pi_share::{Share, ShareRefCell};
use pi_world::{single_res::{SingleRes, SingleResMut}, world::Entity};
use pi_world_macros::Resource;
use wgpu::{StoreOp, Texture, TextureView};

#[derive(Resource, Default)]
pub struct ScreenWithPostprocess(pub bool, pub Option<Share<SafeTargetView>>);
impl ScreenWithPostprocess {
    pub fn tex_and_view<'a>(&'a self, screen: &'a PiScreenTexture) -> (Option<&'a Texture>, Option<&'a TextureView>) {
        if let Some(tex) = &self.1 {
            (Some(&tex.as_ref().target().colors[0].0.texture), Some(&tex.target().colors[0].0.texture_view))
        } else {
            if let Some(screen) = &screen.0 {
                (screen.texture(), screen.view())
            } else {
                (None, None)
            }
        }
    }
}

/// 窗口清屏
/// 注：此节点 只清屏窗口
pub(crate) struct ClearNode;

pub const CLEAR_WIDNOW_NODE: &str = "clear_window";
pub const CLEAR_WIDNOW_GRAPH: &str = "clear_graph";

impl Node for ClearNode {
    type BuildParam = (SingleResMut<'static, ScreenWithPostprocess>, SingleRes<'static, PiSafeAtlasAllocator>, SingleRes<'static, PiScreenTexture>, );
	type RunParam = (SingleRes<'static, PiScreenTexture>, SingleRes<'static, PiClearOptions>, SingleRes<'static, ScreenWithPostprocess>);
    type ResetParam = ();
    
	fn build<'a>(
		&'a mut self,
		_param: &'a mut Self::BuildParam,
		_context: RenderContext,
		_id: Entity,
		_from: &'a [Entity],
		_to: &'a [Entity],
	) -> Result<(), String> {
        if _param.0.0 {
            if let Some(screen) = &_param.2.0 {
                if let Some(screen) = screen.texture() {
                    let width = screen.width();
                    let height = screen.height();
                    if let Some(rt) = &_param.0.1 {
                        let w = rt.rect().width() as u32;
                        let h = rt.rect().height() as u32;
                        if w == width && h == height {
                            return Ok(());
                        }
                    }
                    let target_type = _param.1.create_type(TargetDescriptor {
                        colors_descriptor: smallvec::SmallVec::from_slice(
                            &[TextureDescriptor {
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: screen.format(),
                                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                                base_mip_level: 0,
                                base_array_layer: 0,
                                array_layer_count: None,
                                view_dimension: Some(wgpu::TextureViewDimension::D2),
                            }]
                        ),
                        need_depth: false,
                        depth_descriptor: None,
                        default_width: width,
                        default_height: height,
                    });
                    let t: Vec<Share<SafeTargetView>> = vec![];
                    let rt = _param.1.allocate_alone_not_share(width, height, target_type, t.iter(), true);
                    _param.0.1 = Some(Share::new(rt));
                }
            }
            
		    Ok(())
        } else {
            _param.0.1 = None;
		    Ok(())
        }
	}

    fn run<'a>(
        &'a mut self,
        // world: &'a World,
        param: &'a Self::RunParam,
        _context: RenderContext,
        commands: ShareRefCell<wgpu::CommandEncoder>,
        // _input: &'a Self::Input,
        // _usage: &'a ParamUsage,
		_id: Entity,
		_from: &'a [Entity],
		_to: &'a [Entity],
    ) -> BoxFuture<'a, Result<(), String>> {
        let (view, clear, width, height) = {
            // let view = world.get_single_res::<PiScreenTexture>().unwrap().0.as_ref().unwrap().view.as_ref().unwrap().clone();
            // let clear = world.get_single_res::<PiClearOptions>().unwrap().clone();
            // let (s, clear) = param;

            let (view, width, height) = if let Some(rt) = &param.2.1 {
                (&rt.target().colors[0].0.texture_view, rt.target().colors[0].0.width, rt.target().colors[0].0.height)
            } else {
                if let Some(view) = param.0.as_ref().unwrap().view() {
                    let texture = param.0.as_ref().unwrap().texture().unwrap();
                    (view, texture.width(), texture.height())
                } else {
                    return Box::pin(async move { Ok(()) });
                }
                // param.0.as_ref().unwrap().view().as_ref().unwrap()
            };

            // let clear = clear.0.clone();

            (view, &*param.1, width, height)
        };

        Box::pin(async move {
            let mut encoder = commands.0.as_ref().borrow_mut();

            let depth_stencil_attachment = None;
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                depth_stencil_attachment,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear.color),
                        store: StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
            // 清屏
            rpass.set_scissor_rect(0, 0, width, height);
            Ok(())
        })
    }
    
    fn reset<'a>(
        &'a mut self,
        _param: &'a mut Self::ResetParam,
        _context: RenderContext,
        _id: Entity,
    ) {
        
    }

   
}
