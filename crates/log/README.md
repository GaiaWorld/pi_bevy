# pi_bevy_log

`pi_bevy_log` 是一个为 [pi_world](https://bevyengine.org) 应用程序提供日志功能和配置的 Rust 库。它会自动配置特定平台的日志处理程序，如 WASM 或 Android 平台。

## 功能概述
- **日志宏**：该库重新导出了 [`tracing`](https://docs.rs/tracing) 中的日志宏，使用方式与 `tracing` 相同。
- **平台适配**：根据不同的目标平台，自动选择合适的日志收集器：
  - 在默认情况下，使用 [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) 并将日志输出到 `stdout`。
  - 在 Android 平台上，使用 [`android_log-sys`](https://crates.io/crates/android_log-sys) 将日志输出到 Android 日志系统。
  - 在 WASM 环境中，使用 [`tracing-wasm`](https://crates.io/crates/tracing-wasm) 将日志输出到浏览器控制台。

## 安装
在 `Cargo.toml` 中添加以下依赖：
```toml
[dependencies]
pi_bevy_log = "0.1"
```

## 使用方法
### 默认配置
```rust
use pi_world::prelude::App;

fn setup(app: &mut App) {
    let mut log = pi_bevy_log::LogPlugin::<Vec<u8>>::default();
    app.add_plugins(log);
    app.run();

    log::info!("This is an info log!");
}
```

### 自定义配置
可以在应用程序初始化时自定义 `LogPlugin` 的配置：
```rust
use pi_world::prelude::App;
use pi_bevy_log::LogPlugin;
use bevy::utils::tracing::Level;

fn setup(app: &mut App) {
    App::new()
       .add_plugin(LogPlugin {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
        })
       .run();
}
```

### 环境变量配置
也可以使用 `RUST_LOG` 环境变量来更改日志级别和过滤条件：
```sh
RUST_LOG=wgpu=error,bevy_render=info,bevy_ecs=trace
```

## 贡献
欢迎对本项目进行贡献！如果你发现了问题或有改进建议，请提交 issue 或 pull request。

## 许可证
本项目采用 [许可证名称] 许可证，详情请参阅 `LICENSE` 文件。
