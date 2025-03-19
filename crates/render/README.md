# 渲染系统库

提供跨平台的图形渲染抽象层，支持构建复杂的渲染管线。

## 功能特性

### 核心架构
- **渲染图系统** (`src/graph`)
  - 基于节点的渲染管线配置
  - 自动依赖管理和执行排序
  - 状态池管理 (`state_pool`)

### 窗口集成
- 多窗口渲染支持 (`render_windows`)
- 窗口事件处理集成 (`window`)

### 渲染管线
- 可扩展的渲染节点系统 (`node`)
- 内置清屏节点 (`clear_node`)
- 跨平台渲染抽象 (`render_cross`)

### 资源管理
- 图像资源处理 (`asimage_url`)
- 渲染资源 (`resource`)

## 快速开始

1. 添加依赖：
```toml
[dependencies]
pi_bevy_render_plugin = "0.1"
```

2. 基本使用示例：
```rust
use pi_bevy_render_plugin::plugin::PiRenderPlugin;

fn setup_render(mut commands: Commands) {
    // 创建渲染图
    let mut graph = RenderGraph::default();
    
    // 添加清屏节点
    graph.add_node("clear", ClearNode::new(ClearOptions {
        color: Color::rgb(0.1, 0.1, 0.1),
        depth: 1.0,
    }));
    
    // 注册渲染图
    commands.insert_resource(graph);
}

fn setup(app: pi_world::prelude::App) {
    app.add_plugins(PiRenderPlugin);
}
```

## API 参考

### 核心类型
- `RenderGraph`: 渲染图
- `RenderNode`: 渲染节点特征
- `RenderContext`: 渲染上下文信息

### 窗口管理
- `WindowRenderer`: 窗口渲染器接口
- `WindowRenderOptions`: 窗口渲染配置

### 工具方法
- `as_image_url`: 资源路径转换方法
- `create_render_device`: 创建设备实例


## 贡献指南

1. 遵循现有代码风格和模块结构
2. 新增功能需包含对应的测试用例
3. 重大变更需更新examples中的示例应用
