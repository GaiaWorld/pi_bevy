# pi_bevy_window

## 简介
`pi_bevy_window` 库是一个用于在 pi_world 框架中提供窗口支持的插件库。它定义了窗口管理、事件处理和退出条件等功能，为 pi_world 应用程序提供了完整的窗口化解决方案。

## 功能概述
### 1. 模块结构
- `cursor`：处理光标相关的事件和状态。
- `event`：定义窗口相关的事件类型。
- `raw_handle`：提供窗口的原始句柄相关功能。
- `window`：定义窗口的核心结构和功能。

### 2. 插件功能
- **窗口管理**：支持创建和管理主窗口，可自定义主窗口的设置。
- **事件处理**：定义了多种窗口事件，如窗口大小改变、创建、关闭等，并提供了事件清理机制。
- **退出条件**：支持三种退出条件，可根据需求选择应用程序在主窗口关闭、所有窗口关闭或不退出的情况下的行为。

### 3. 系统集和状态管理
- **FrameSet**：定义了一个系统集，用于组织与帧相关的系统。
- **FrameState**：定义了帧的状态，可用于控制某些系统的运行。

## 主要结构体和枚举

### 1. `WindowPlugin`
这是核心插件结构体，包含以下字段：
- `primary_window`：主窗口的设置，可自定义主窗口的属性。
- `exit_condition`：应用程序的退出条件，可选值为 `OnPrimaryClosed`、`OnAllClosed` 或 `DontExit`。
- `close_when_requested`：是否在窗口请求关闭时关闭窗口。

### 2. `ExitCondition`
枚举类型，定义了应用程序的退出条件：
- `OnPrimaryClosed`：主窗口关闭时退出应用程序。
- `OnAllClosed`：所有窗口关闭时退出应用程序。
- `DontExit`：即使所有窗口关闭，应用程序也不退出。

### 3. `FrameState`
枚举类型，定义了帧的状态：
- `Active`：帧处于活动状态。
- `UnActive`：帧处于非活动状态。

## 使用示例

### 添加依赖：
```toml
[dependencies]
pi_bevy_window = "0.1"
```

### 使用示例
```rust
use pi_world::prelude::App;
use crate::window::WindowPlugin;

fn setup(app: &mut App) {
    let window_plugin = WindowPlugin::default();
    app.add_plugin(window_plugin);
}
```
