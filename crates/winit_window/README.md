# winit_window 库文档

## 简介
`winit_window` 库是一个用于创建和管理窗口的 Rust 库，它基于 `winit` 库，并与 `bevy` 和 `pi_world` 框架集成。该库支持不同的目标架构，包括 `wasm32` 和非 `wasm32` 平台。

## 功能特性
- **跨平台支持**：支持 `wasm32` 和非 `wasm32` 目标架构。
- **窗口创建和管理**：可以创建窗口并设置窗口大小。
- **与框架集成**：与 `bevy` 和 `pi_world` 框架集成，方便在这些框架中使用。

## 代码结构
### 主要结构体
- `WinitPlugin`：插件结构体，用于初始化和配置窗口。
- `WindowDescribe`：窗口描述结构体，用于设置窗口的大小和其他属性。
- `WindowWrapper`：窗口包装结构体，实现了 `CreateSurface`  trait。

### 主要方法
- `WinitPlugin::new`：创建一个新的 `WinitPlugin` 实例。
- `WinitPlugin::with_size`：设置窗口的大小。
- `WindowDescribe::new`：创建一个新的 `WindowDescribe` 实例。
- `WindowDescribe::with_size`：设置窗口的大小。
- `WindowDescribe::build`：构建窗口并将其插入到 `App` 中。

## 使用示例
### 非 `wasm32` 平台
```rust
use pi_world::prelude::App;
use winit::window::Window;
use std::sync::Arc;
use crates::winit_window::WinitPlugin;

pub fn setup(app: &mut App, window: Arc<Window>, width: u32, height: u32,) {
    let plugin = WinitPlugin::new(window).with_size(width, height);
    app.add_plugin(plugin);
}
```

### `wasm32` 平台
```rust
use pi_world::prelude::App;
use winit::window::Window;
use std::sync::Arc;
use crates::winit_window::WinitPlugin;

fn setup(app: &mut App, canvas: web_sys::HtmlCanvasElement, width: u32, height: u32,) {
    let window ={
		use pi_winit::platform::web::WindowBuilderExtWebSys;
		use wasm_bindgen::JsCast;
		let event_loop = pi_winit::event_loop::EventLoop::new();
		Arc::new(
			pi_winit::window::WindowBuilder::new()
				.with_canvas(Some(canvas))
				.build(&event_loop)
				.unwrap(),
		)
	};
    app.add_plugin(plugin);
}
```

## 注意事项
- 在 `wasm32` 平台下，`WinitPlugin` 结构体强行实现了 `Send` 和 `Sync` trait，因为 `wasm` 是单线程的，不会出现安全隐患。
- 在非 `wasm32` 平台下，`WinitPlugin` 结构体也实现了 `Send` 和 `Sync` trait，但需要注意线程安全问题。

