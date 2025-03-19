# ECS 扩展库

提供基于 `pi_world` ECS 系统的扩展功能，增强实体组件系统的开发体验。

## 功能特性

### 系统参数扩展
- **实体树管理** (`system_param/tree`)
  - 提供层级实体结构管理
  - 支持父子关系遍历操作
  - 包含 `Layer`、`Down`、`Up` 等遍历工具

- **脏标记系统** (`system_param/layer_dirty`)
  - 自动追踪组件变更
  - 通过 `DirtyMark` 标记需要更新的实体
  - 优化系统执行效率

- **资源初始化** (`system_param/res`)
  - `OrInitSingleRes`：按需初始化单例资源
  - `OrInitSingleResMut`：可变资源引用封装

### 实用工具
- `IsNotRun` 标记：控制系统执行流程
- `TShell` trait：统一访问 World 和 App

## 快速开始

1. 添加依赖：
```toml
[dependencies]
pi_ecs_extend = "0.1"
```

2. 基本使用示例：
```rust
use pi_ecs_extend::prelude::*;

fn update_system(
    mut tree: EntityTreeMut,
    mut dirty: DirtyMark,
    res: OrInitSingleRes<MyResource>
) {
}
```

## API 参考

### 实体树操作
- `EntityTree`：只读实体树访问
- `EntityTreeMut`：可变实体树访问
- `Root`：根实体标记

### 资源管理
- `OrInitSingleRes<T>`：获取或初始化资源
- `OrInitSingleResMut<T>`：获取或初始化可变资源


## 贡献

欢迎通过 Issue 和 PR 参与贡献，请遵循现有代码风格。
