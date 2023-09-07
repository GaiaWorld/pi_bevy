use crate::{node::Node, PiClearOptions, PiScreenTexture, RenderContext};
use bevy_ecs::{
    system::{Res, SystemState},
    world::World,
};
use pi_futures::BoxFuture;
use pi_render::depend_graph::node::ParamUsage;
use pi_share::ShareRefCell;
use pi_render::depend_graph::NodeId;

/// 窗口清屏
/// 注：此节点 只清屏窗口
pub(crate) struct ClearNode;

pub const CLEAR_WIDNOW_NODE: &str = "clear_window";

impl Node for ClearNode {
    type Input = ();
    type Output = ();
    type Param = (Res<'static, PiScreenTexture>, Res<'static, PiClearOptions>);

    fn run<'a>(
        &'a mut self,
        world: &'a World,
        param: &'a mut SystemState<Self::Param>,
        _context: RenderContext,
        commands: ShareRefCell<wgpu::CommandEncoder>,
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId,
		_from: &'a [NodeId],
		_to: &'a [NodeId],
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        let (view, clear) = {
            let (s, clear) = param.get(world);

            let view = s.0.as_ref().unwrap().view.as_ref().unwrap().clone();

            let clear = clear.0.clone();

            (view, clear)
        };

        Box::pin(async move {
            let mut encoder = commands.0.as_ref().borrow_mut();

            let depth_stencil_attachment = None;
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                depth_stencil_attachment,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view.as_ref(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear.color),
                        store: true,
                    },
                })],
            });

            Ok(())
        })
    }
}
