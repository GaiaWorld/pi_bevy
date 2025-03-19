# Pi Asset Management Library

[![Crates.io](https://img.shields.io/crates/v/pi_asset)](https://crates.io/crates/pi_asset)
[![Docs.rs](https://docs.rs/pi_asset/badge.svg)](https://docs.rs/pi_asset)

资产管理核心库，提供统一的资产分配、回收和生命周期管理机制。

## 功能特性

- 🧩 插件化资产管理 (`PiAssetPlugin`)
- ⚙️ 可配置的资产容量策略 (`AssetConfig`)
- 🕒 基于时间的超时回收机制及容量整理

## 快速开始

### 添加依赖

```toml
[dependencies]
pi_asset = { version = "0.1" }
```

### 基本使用

```rust
use pi_asset::{PiAssetPlugin, AssetConfig};

fn setup_asset_system(app: &mut App) {
    // 初始化资产管理插件
    app.add_plugin(PiAssetPlugin {
        collect_interval: 1000,  // 回收间隔(ms)
        total_capacity: 64 * 1024 * 1024,  // 总容量64MB
        asset_config: AssetConfig::default(),
        allocator: None,
    });
    
    // 添加自定义资产配置
    let mut config = AssetConfig::default();
    config.insert::<TextureRes>(AssetDesc {
        min: 8 * 1024 * 1024,
        weight: 50,
        timeout: 5000,
        ref_garbage: true,
    });
    app.world.insert_single_res(config);
}
```

## 核心组件

### `PiAssetPlugin`

资产管理插件，提供：
- 资产分配器初始化
- 定时回收系统
- 配置管理系统

### `AssetConfig`

资产管理配置中心，支持：
```rust
pub struct AssetDesc {
    pub min: usize,       // 最小保留容量
    pub weight: usize,    // 在总容量中的权重
    pub timeout: usize,   // 超时时间(ms)
    pub ref_garbage: bool // 启用引用计数回收
}
```

## 性能监控

启用 `account_info` feature 可获取详细内存使用统计：
```toml
[dependencies]
pi_asset = { version = "0.1", features = ["account_info"] }
```


## 最佳实践

1. 根据资源类型合理设置`AssetDesc`
2. 高频更新资源建议设置较短timeout
3. 静态资源建议增大min值避免频繁回收
4. 使用`ref_garbage`优化引用型资源回收

## 贡献指南

欢迎提交Issue和PR，请确保：
- 通过所有cargo测试
- 更新相关文档
- 保持API兼容性
