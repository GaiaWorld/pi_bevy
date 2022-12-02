use pi_async::{prelude::SingleTaskRunner, rt::AsyncRuntime};
use std::future::Future;

// 同步堵塞 1个 Futrue
pub(crate) fn poll_future<A, F, O: 'static + Send>(
    rt: &A,
    mut runner: Option<&mut SingleTaskRunner<()>>,
    future: F,
) -> O
where
    A: AsyncRuntime,
    F: Future<Output = O> + Send + 'static,
{
    // 目的：让目前函数 能等待 rt 任务 完成
    let (send, recv) = std::sync::mpsc::channel::<O>();

    let _ = rt.spawn(rt.alloc(), async move {
        let o = future.await;
        let _ = send.send(o);
    });

    let mut r = recv.try_recv();
    while r.is_err() {
        if let Some(r) = runner.as_mut() {
            // 只有 wasm 版本 的 单线程运行时，才会 运行到 这里
            while r.run().unwrap() > 0 {}
        }

        // 让出 时间片，让 CPU 缓一缓
        std::thread::yield_now();

        r = recv.try_recv();
    }

    r.unwrap()
}
