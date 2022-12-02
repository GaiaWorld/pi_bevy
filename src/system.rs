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
use pi_share::{Share, ShareCell};

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
    let options = options.0.as_ref().clone();

    let (instance, _, device, queue, adapter_info) = poll_future(
        &rt.0,
        runner.0.as_mut(),
        pi_render::rhi::setup_render_context(options, window),
    );

    let rg = RenderGraph::new(device.clone(), queue.clone());

    // 注：之所以写到这里，是因为 Bevy 的 内置类型 不能 传到 pi_async 的 future中。

    commands.insert_resource(PiRenderGraph(Share::new(ShareCell::new(rg))));
    commands.insert_resource(PiRenderInstance(Share::new(instance)));
    commands.insert_resource(PiRenderDevice(device));
    commands.insert_resource(PiRenderQueue(queue));
    commands.insert_resource(PiAdapterInfo(Share::new(adapter_info)));
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

    windows: Res<PiRenderWindows>,
    view: Res<PiScreenTexture>,

    rg: Res<PiRenderGraph>,
) {
    let rt_clone = rt.0.clone();

    let instance = instance.0.clone();
    let device = device.0.clone();
    let rg = rg.0.clone();

    let windows = windows.0.clone();
    let view = view.0.clone();

    let future = async move {
        // ============ 1. 获取 窗口 可用纹理 ============
        prepare_windows(
            &device,
            instance.as_ref(),
            &mut windows.as_ref().borrow_mut(),
            &mut view.as_ref().borrow_mut(),
        )
        .unwrap();

        // ============ 2. 执行渲染图 ============
        let mut rg = rg.as_ref().borrow_mut();
        rg.build().unwrap();
        rg.run(&rt_clone).await.unwrap();

        // ============ 3. 呈现，SwapBuffer ============
        present_windows(
            &windows.as_ref().borrow(),
            view.as_ref().borrow_mut().as_mut().unwrap(),
        );
    };

    poll_future(&rt.0, runner.0.as_mut(), future);
}

fn present_windows(windows: &RenderWindows, screen_texture: &mut ScreenTexture) {
    for (_, _window) in windows.iter() {
        if let Some(view) = screen_texture.take_surface_texture() {
            view.present();
        }
    }

    log::trace!("render_system: after surface_texture.present");
}
