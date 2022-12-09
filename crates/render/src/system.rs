use crate::{
    graph::graph::RenderGraph,
    render_windows::{prepare_window, RenderWindow},
    PiAsyncRuntime, PiRenderDevice, PiRenderGraph, PiRenderInstance, PiRenderWindow,
    PiScreenTexture,
};
use bevy_ecs::world::World;
use pi_async::prelude::*;
use pi_render::rhi::texture::ScreenTexture;

/// ================ System ================

// 帧推 渲染 System
//
// A 的 类型 见 plugin 模块
//   + wasm 环境 是 SingleTaskRuntime
//   + 否则 是 MultiTaskRuntime
//
pub(crate) fn run_frame_system<A: AsyncRuntime + AsyncRuntimeExt>(world: &mut World) {
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

    let (width, height) = world
        .resource::<bevy_window::Windows>()
        .get_primary().map(|window| (window.physical_width(), window.physical_height()))
        .unwrap();

    let window: &'static mut RenderWindow = unsafe {
        let w = &mut *(ptr_world as *mut World);
        let window = &mut w.resource_mut::<PiRenderWindow>().0;
        std::mem::transmute(window)
    };

    let rg: &'static mut RenderGraph = unsafe {
        let w = &mut *(ptr_world as *mut World);
        let rg = &mut w.resource_mut::<PiRenderGraph>().0;
        std::mem::transmute(rg)
    };

    let rt_clone = rt.clone();
    rt.block_on(async move {
        // ============ 1. 获取 窗口 可用纹理 ============

        prepare_window(window, view, device, instance, width, height).unwrap();

        // ============ 2. 执行渲染图 ============
        rg.build().unwrap();
        rg.run(&rt_clone, world).await.unwrap();

        present_window(view.as_mut().unwrap());
    }).unwrap();
}

fn present_window(screen_texture: &mut ScreenTexture) {
    if let Some(view) = screen_texture.take_surface_texture() {
        view.present();
    }
    
    log::trace!("render_system: after surface_texture.present");
}
