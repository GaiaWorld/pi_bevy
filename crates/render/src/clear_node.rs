use crate::{node::Node, PiClearOptions, PiScreenTexture, RenderContext};
// use bevy_ecs::{
//     system::{Res, SystemState},
//     world::World,
// };
use pi_futures::BoxFuture;
use pi_share::ShareRefCell;
use pi_world::{single_res::SingleRes, world::Entity};
use wgpu::StoreOp;

/// 窗口清屏
/// 注：此节点 只清屏窗口
pub(crate) struct ClearNode;

pub const CLEAR_WIDNOW_NODE: &str = "clear_window";
pub const CLEAR_WIDNOW_GRAPH: &str = "clear_graph";

impl Node for ClearNode {
    type BuildParam = ();
	type RunParam = (SingleRes<'static, PiScreenTexture>, SingleRes<'static, PiClearOptions>);
    type ResetParam = ();
    
	fn build<'a>(
		&'a mut self,
		// _world: &'a  World,
		_param: &'a mut Self::BuildParam,
		_context: RenderContext,
		_id: Entity,
		_from: &'a [Entity],
		_to: &'a [Entity],
	) -> Result<(), String> {
		Ok(())
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
        let (view, clear) = {
            // let view = world.get_single_res::<PiScreenTexture>().unwrap().0.as_ref().unwrap().view.as_ref().unwrap().clone();
            // let clear = world.get_single_res::<PiClearOptions>().unwrap().clone();
            // let (s, clear) = param;

            let view = param.0.as_ref().unwrap().view().unwrap().clone();

            // let clear = clear.0.clone();

            (view, &*param.1)
        };

        Box::pin(async move {
            let mut encoder = commands.0.as_ref().borrow_mut();

            let depth_stencil_attachment = None;
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                depth_stencil_attachment,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear.color),
                        store: StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            Ok(())
        })
    }
    
    fn reset<'a>(
        &'a mut self,
        // world: &'a mut World,
        _param: &'a mut Self::ResetParam,
        _context: RenderContext,
            // input: &'a Self::Input,
        // usage: &'a ParamUsage,
        _id: Entity,
            // from: &'a [Entity],
            // to: &'a [Entity],
    ) {
        
    }

   
}
