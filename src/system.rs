use crate::{
    graph::graph::RenderGraph, util::poll_future, PiAdapterInfo, PiAsyncRuntime, PiRenderDevice,
    PiRenderGraph, PiRenderInstance, PiRenderOptions, PiRenderQueue, PiRenderWindows,
    PiScreenTexture, PiSingleTaskRunner, PiWinitWindow,
};
use bevy::prelude::{Commands, Res, ResMut};
use pi_async::rt::AsyncRuntime;
use pi_render::{
    components::view::render_window::{prepare_windows, RenderWindows},
    rhi::texture::ScreenTexture,
};

/// ================ System ================

// 初始化 渲染环境 的 System
//
// A 的 类型 见 plugin 模块
//   + wasm 环境 是 SingleTaskRuntime
//   + 否则 是 MultiTaskRuntime
//
pub(crate) fn init_render_system<A: AsyncRuntime>(
    mut commands: Commands,

    rt: Res<PiAsyncRuntime<A>>,
    mut runner: ResMut<PiSingleTaskRunner>,

    window: Res<PiWinitWindow>,
    options: Res<PiRenderOptions>,
) {
    let window = window.0.clone();
    let options = options.0.clone();

    let runner = runner.0.as_mut();
    let (instance, _, device, queue, adapter_info) = poll_future(
        &rt.0,
        runner,
        pi_render::rhi::setup_render_context(options, window),
    );

    let rg = RenderGraph::new(device.clone(), queue.clone());
    commands.insert_resource(PiRenderGraph(rg));
    commands.insert_resource(PiRenderInstance(instance));
    commands.insert_resource(PiRenderDevice(device));
    commands.insert_resource(PiRenderQueue(queue));
    commands.insert_resource(PiAdapterInfo(adapter_info));
}

// 帧推 渲染 System
//
// A 的 类型 见 plugin 模块
//   + wasm 环境 是 SingleTaskRuntime
//   + 否则 是 MultiTaskRuntime
//
pub(crate) fn run_frame_system<A: AsyncRuntime>(
    rt: Res<PiAsyncRuntime<A>>,
    mut runner: ResMut<PiSingleTaskRunner>,

    instance: Res<PiRenderInstance>,
    device: Res<PiRenderDevice>,

    mut windows: ResMut<PiRenderWindows>,
    mut view: ResMut<PiScreenTexture>,

    mut rg: ResMut<PiRenderGraph>,
) {
    // let future = async move {
    //     // ============ 1. 获取 窗口 可用纹理 ============
    //     prepare_windows(&device.0, &instance.0, &mut windows.0, &mut view.0).unwrap();

    //     // ============ 2. 执行渲染图 ============
    //     rg.0.build().unwrap();
    //     rg.0.run(&rt.0).await.unwrap();

    //     // ============ 3. 呈现，SwapBuffer ============
    //     present_windows(&windows.0, view.0.as_mut().unwrap());
    // };

    // let runner = runner.0.as_mut();
    // poll_future(&rt.0, runner, future);
}

fn present_windows(windows: &RenderWindows, screen_texture: &mut ScreenTexture) {
    for (_, _window) in windows.iter() {
        if let Some(view) = screen_texture.take_surface_texture() {
            view.present();
        }
    }

    log::trace!("render_system: after surface_texture.present");
}
