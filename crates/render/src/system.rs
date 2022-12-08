use crate::{
    graph::graph::RenderGraph, util::poll_future, PiAdapterInfo, PiAsyncRuntime, PiRenderDevice,
    PiRenderGraph, PiRenderInstance, PiRenderOptions, PiRenderQueue, PiRenderWindows,
    PiScreenTexture, PiSingleTaskRunner, PiWinitWindow,
};
use bevy::prelude::{Commands, Res, ResMut, World};
use pi_async::{prelude::SingleTaskRunner, rt::AsyncRuntime};
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
    let window = window.clone();
    let options = options.0.clone();
    let (instance, _, device, queue, adapter_info) = poll_future(
        &rt.0,
        runner.0.as_mut(),
        pi_render::rhi::setup_render_context(options, window),
    );

    let rg = RenderGraph::new(device.clone(), queue.clone());

    // 注：之所以写到这里，是因为 Bevy 的 内置类型 不能 传到 pi_async 的 future中。

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
pub(crate) fn run_frame_system<A: AsyncRuntime>(world: &mut World) {
    // 从 world 取 res
    let ptr_world = world as *mut World as usize;

    let world: &'static World = unsafe { std::mem::transmute(world) };
    let rt = &world.resource::<PiAsyncRuntime<A>>().0;
    let instance = &world.resource::<PiRenderInstance>().0;
    let device = &world.resource::<PiRenderDevice>().0;

    // 跨 异步运行时 的 引用 必须 声明 是 'static 的
    let view: &'static mut Option<ScreenTexture> = unsafe {
        // 同一个 world 不能 即 resource 又 resource_mut
        let w = &mut *(ptr_world as *mut World);
        let views = &mut w.resource_mut::<PiScreenTexture>().0;
        std::mem::transmute(views)
    };

    let windows: &'static mut RenderWindows = unsafe {
        let w = &mut *(ptr_world as *mut World);
        let windows = &mut w.resource_mut::<PiRenderWindows>().0;
        std::mem::transmute(windows)
    };

    let rg: &'static mut RenderGraph = unsafe {
        let w = &mut *(ptr_world as *mut World);
        let rg = &mut w.resource_mut::<PiRenderGraph>().0;
        std::mem::transmute(rg)
    };

    let rt_clone = rt.clone();
    let future = async move {
        // ============ 1. 获取 窗口 可用纹理 ============
        prepare_windows(device, instance, windows, view).unwrap();

        // ============ 2. 执行渲染图 ============
        rg.build().unwrap();
        rg.run(&rt_clone, world).await.unwrap();

        present_windows(windows, view.as_mut().unwrap());
    };

    let runner = unsafe {
        let w = &mut *(ptr_world as *mut World);
        &mut w.resource_mut::<PiSingleTaskRunner>().0
    };

    poll_future(rt, runner.as_mut(), future);
}

fn present_windows(windows: &RenderWindows, screen_texture: &mut ScreenTexture) {
    for (_, _window) in windows.iter() {
        if let Some(view) = screen_texture.take_surface_texture() {
            view.present();
        }
    }

    log::trace!("render_system: after surface_texture.present");
}
