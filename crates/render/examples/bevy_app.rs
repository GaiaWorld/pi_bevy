use bevy_app::{App, Plugin};
use bevy_ecs::system::{Res, ResMut};
use bevy_log::LogPlugin;
use bevy_window::WindowPlugin;
use bevy_winit::WinitPlugin;
use pi_bevy_render_plugin::{node::Node, PiRenderGraph, PiRenderPlugin, PiScreenTexture};

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_plugin(PiRenderPlugin::default())
        .add_plugin(DemoPlugin::default())
        .run();
}

#[derive(Default)]
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup_demo).run();
    }
}

fn startup_demo(mut rg: ResMut<PiRenderGraph>) {
    rg.add_node("clear", ClearNode).unwrap();
    rg.set_finish("clear", true).unwrap();
}

// =================

struct ClearNode;

impl Node for ClearNode {
    type Input = ();
    type Output = ();
    type Param = Res<'static, PiScreenTexture>;

    fn run<'a>(
        &'a mut self,
        world: &'a bevy_ecs::world::World,
        param: &'a mut bevy_ecs::system::SystemState<Self::Param>,
        context: pi_bevy_render_plugin::RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a pi_bevy_render_plugin::node::ParamUsage,
    ) -> pi_futures::BoxFuture<'a, Result<Self::Output, String>> {
        let view = {
            param
                .get(world)
                .0
                .as_ref()
                .unwrap()
                .view
                .as_ref()
                .unwrap()
                .clone()
        };

        Box::pin(async move {
            let mut encoder = commands.0.as_ref().borrow_mut();

            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view.as_ref(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            Ok(())
        })
    }
}
