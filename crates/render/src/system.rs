use std::sync::atomic::Ordering;

use crate::{
    graph::graph::RenderGraph,
    render_windows::{prepare_window, RenderWindow},
    PiAsyncRuntime, PiRenderDevice, PiRenderGraph, PiRenderInstance,
    PiRenderWindow, IS_RESUMED, PiScreenTexture,
};
use bevy_ecs::prelude::{World, With};
use bevy_window::{PrimaryWindow, Window};
use pi_async_rt::prelude::*;
use pi_render::rhi::texture::ScreenTexture;
#[cfg(feature = "trace")]
use tracing::Instrument;

/// ================ System ================

// 帧推 渲染 System
//
// A 的 类型 见 plugin 模块
//   + wasm 环境 是 SingleTaskRuntime
//   + 否则 是 MultiTaskRuntime
//
pub(crate) fn run_frame_system<A: AsyncRuntime + AsyncRuntimeExt>(world: &mut World) {
    if !IS_RESUMED.load(Ordering::Relaxed){
        return;
    }
    let mut primary_window = world.query_filtered::<&Window, With<PrimaryWindow>>();

    let (width, height) = match primary_window.get_single(world) {
        Ok(primary_window) => (
            primary_window.physical_width(),
            primary_window.physical_height(),
        ),
        _ => return,
    };

    // 从 world 取 res
    let ptr_world = world as *mut World as usize;

    let first_surface = {
        let w = unsafe { &mut *(ptr_world as *mut World) };
        w.resource_mut::<crate::PiFirstSurface>().0.take()
    };

    let world_ref: &'static World = unsafe { std::mem::transmute(world) };
	let world_mut: &'static mut World = unsafe { &mut *(world_ref as *const World as usize as *mut World) };
    let rt = &world_ref.resource::<PiAsyncRuntime<A>>().0;
    let instance = &world_ref.resource::<PiRenderInstance>().0;
    let device = &world_ref.resource::<PiRenderDevice>().0;

    // 跨 异步运行时 的 引用 必须 声明 是 'static 的
    let view: &'static mut Option<ScreenTexture> = unsafe {
        // 同一个 world 不能 即 resource 又 resource_mut
        let w = &mut *(ptr_world as *mut World);
        let views = &mut w.resource_mut::<PiScreenTexture>().0;
        std::mem::transmute(views)
    };

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

    #[cfg(not(feature = "trace"))]
    let task = async move {
        // ============ 1. 获取 窗口 可用纹理 ============
        prepare_window(window, first_surface, view, device, instance, width, height).unwrap();
        // ============ 2. 执行渲染图 ============
        // rg.build().unwrap();
		// log::warn!("run before====================");
		// pi_hal::runtime::LOGS.lock().0.push("system run before".to_string());
        rg.run(&rt_clone, world_mut).await.unwrap();
		// pi_hal::runtime::LOGS.lock().0.push("system run after".to_string());
        
        // ============ 3. 呈现 ============
		// log::warn!("take_surface_texture before====================");
        if let Some(view) = view.as_mut().unwrap().take_surface_texture() {
            view.present();
        }
    };
    #[cfg(feature = "trace")]
    let task = async move {
   		let prepare_window_span = tracing::warn_span!("prepare_window");
        // ============ 1. 获取 窗口 可用纹理 ============
        async {
            prepare_window(window, first_surface, view, device, instance, width, height).unwrap();
        }
        .instrument(prepare_window_span)
        .await;

        // ============ 2. 执行渲染图 ============
        // async {
        //     rg.build().unwrap();
        // }
        // .instrument(rg_build_span)
        // .await;
        rg.run(&rt_clone, world_mut).await.unwrap();
    };

    rt.block_on(task).unwrap();
	#[cfg(feature = "trace")]
	{
		let view: &'static mut Option<ScreenTexture> = unsafe {
			// 同一个 world 不能 即 resource 又 resource_mut
			let w = &mut *(ptr_world as *mut World);
			let views = &mut w.resource_mut::<PiScreenTexture>().0;
			std::mem::transmute(views)
		};
		let present = async move {
			// ============ 3. 呈现 ============
			let take_texture_span = tracing::info_span!("take_texture");
			let view = async move { view.as_mut().unwrap().take_surface_texture() }
			.instrument(take_texture_span)
			.await;
			if let Some(view) = view {
				let system_present_span = tracing::warn_span!("present");
				let r = async move {
					view.present();
				}.instrument(system_present_span)
				.await;
				r
			}
		};
		rt.block_on(present).unwrap();
	}
	

}


pub(crate) fn build_graph<A: AsyncRuntime + AsyncRuntimeExt>(world: &mut World) {
    if !IS_RESUMED.load(Ordering::Relaxed){
        return;
    }
	// 从 world 取 res
	let ptr_world = world as *mut World as usize;
	let world_ref: &'static World = unsafe { std::mem::transmute(world) };
	let world_mut: &'static mut World = unsafe { &mut *(world_ref as *const World as usize as *mut World) };
	let rt: &A = &world_ref.resource::<PiAsyncRuntime<A>>().0;
	let rg: &'static mut RenderGraph = unsafe {
        let w = &mut *(ptr_world as *mut World);
        let rg = &mut w.resource_mut::<PiRenderGraph>().0;
        std::mem::transmute(rg)
    };

	rg.build(rt, world_mut).unwrap();
}
// fn present_window(screen_texture: &mut ScreenTexture) {
//     if let Some(view) = screen_texture.take_surface_texture() {
//         view.present();
//     }

//     log::trace!("render_system: after surface_texture.present");
// }
