use bevy::prelude::*;
use pi_async::rt::AsyncRuntime;

use crate::{PiAsyncRuntime, PiRenderOptions};

/// ================ System ================

// 初始化
pub(crate) fn init_render_system<A: 'static + AsyncRuntime + Send>(
    commands: Commands,
    // runner: ResMut<Option<PiAsyncRunner<A>>>,
    rt: Res<PiAsyncRuntime<A>>,
    // window: Res<PiWindows>,
    options: Res<PiRenderOptions>,
) {
    // // 目的：让目前函数 能等待 rt 任务 完成
    // let is_finish = Share::new(AtomicBool::new(false));
    // let is_finish_clone = is_finish.clone();
    
    // rt.0.spawn(rt.0.alloc(), async move {
    //     let (instance, options, device, queue, adapter_info) =
    //         pi_render::rhi::setup_render_context(options.0.clone(), window.0.clone()).await;

    //     commands.insert_resource(PiRenderInstance(instance));
    //     commands.insert_resource(PiRenderDevice(device));
    //     commands.insert_resource(PiRenderQueue(queue));
    //     commands.insert_resource(PiAdapterInfo(adapter_info));
        
    //     is_finish_clone.as_ref().store(true, std::sync::atomic::Ordering::SeqCst);
    // });
    
    // while is_finish.load(std::sync::atomic::Ordering::SeqCst) {
    //     if let Some(r) = runner.get() {
    //         // 单线程运行时 才会到这里
    //         while r.run() > 0 {}
    //     }
    // }
}

// 针对 代码
pub(crate) fn run_frame_system<A: 'static + AsyncRuntime + Send>(
) {
}


// // Build RenderGraph
// // 注：System 的返回值 一定要 std::io::Result 才是 异步类型
// async fn build_graph<A>() -> std::io::Result<()>
// where
//     A: 'static + AsyncRuntime + Send,
// {
//     let rg = world.get_resource_mut::<RenderGraph>().unwrap();

//     rg.build().unwrap();

//     Ok(())
// }

// // 每帧 调用一次，用于 驱动 渲染图
// // 注：System 的返回值 一定要 std::io::Result 才是 异步类型
// async fn render_system<A>() -> std::io::Result<()>
// where
//     A: AsyncRuntime + Send + 'static,
// {
//     let graph = world.get_resource_mut::<RenderGraph>().unwrap();

//     let rt = world.get_resource::<RenderAsyncRuntime<A>>().unwrap();
//     graph.run(&rt.rt).await.unwrap();

//     let world = world.clone();

//     let screen_texture = world.get_resource_mut::<ScreenTexture>().unwrap();
//     let windows = world.get_resource::<RenderWindows>().unwrap();

//     // 呈现 所有的 窗口 -- 交换链
//     for (_, _window) in windows.iter() {
//         if let Some(view) = screen_texture.take_surface_texture() {
//             view.present();
//             trace!("render_system: after surface_texture.present");
//         }
//     }

//     Ok(())
// }

