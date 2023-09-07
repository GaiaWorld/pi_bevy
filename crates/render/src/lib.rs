#![feature(trivial_bounds)]

//!
//! pi_render 的 bevy 封装
//!

#[macro_use]
extern crate lazy_static;

mod async_queue;
mod clear_node;
pub mod component;
pub mod constant;
mod graph;
mod init_render;
mod plugin;
mod render_windows;
mod resource;
mod system;

use std::sync::atomic::AtomicBool;

/// 渲染图
pub use graph::*;
use pi_render::components::view::target_alloc::{GetTargetView, ShareTargetView, TargetView};
/// 渲染 插件
pub use plugin::*;
use render_derive::NodeParam;
/// 单例
pub use resource::*;

lazy_static! {
    pub static ref IS_RESUMED: AtomicBool = AtomicBool::new(true);
}

/// 标签
pub use clear_node::CLEAR_WIDNOW_NODE;

#[derive(Default, Clone, NodeParam)]
pub struct SimpleInOut {
    pub target: Option<ShareTargetView>,
	pub valid_rect: Option<(u32, u32, u32, u32)>, // x, y, w, h
}

impl GetTargetView for SimpleInOut {
    fn get_target_view(&self) -> Option<&TargetView> {
        return self.target.as_ref().map(|r| &***r);
    }
}

pub mod asset_config {
    use bevy_ecs::prelude::Resource;
    use pi_bevy_asset::AssetCapacity;

    /// Asset 资源管理
    pub enum EAsset {
        RenderResTextureView,
        RenderResUnuseTexture,
        TextureRes,
        ImageTexture,
        ImageTextureView,
        BindGroup,
        SamplerRes,
        VertexBuffer3D,
        ShaderMeta3D,
        Shader3D,
        RenderPipeline,
        GLTF,
        File,
    }

    #[derive(Resource)]
    pub struct AssetCfgRenderResTextureView(pub AssetCapacity);
    impl Default for AssetCfgRenderResTextureView {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 32 * 1024 * 1024,
                max: 64 * 1024 * 1024,
                timeout: 60 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgRenderResTextureView {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgRenderResUnuseTexture(pub AssetCapacity);
    impl Default for AssetCfgRenderResUnuseTexture {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 16 * 1024 * 1024,
                max: 32 * 1024 * 1024,
                timeout: 60 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgRenderResUnuseTexture {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgSamplerRes(pub AssetCapacity);
    impl Default for AssetCfgSamplerRes {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 1 * 1024 * 1024,
                max: 2 * 1024 * 1024,
                timeout: 24 * 60 * 60 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgSamplerRes {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgTextureRes(pub AssetCapacity);
    impl Default for AssetCfgTextureRes {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 10 * 1024 * 1024,
                max: 20 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgTextureRes {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgImageTexture(pub AssetCapacity);
    impl Default for AssetCfgImageTexture {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 10 * 1024 * 1024,
                max: 20 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgImageTexture {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgImageTextureView(pub AssetCapacity);
    impl Default for AssetCfgImageTextureView {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 1 * 1024 * 1024,
                max: 2 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgImageTextureView {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgBindGroup(pub AssetCapacity);
    impl Default for AssetCfgBindGroup {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 1 * 1024 * 1024,
                max: 2 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgBindGroup {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgVertexBuffer3D(pub AssetCapacity);
    impl Default for AssetCfgVertexBuffer3D {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 10 * 1024 * 1024,
                max: 20 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgVertexBuffer3D {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgShaderMeta3D(pub AssetCapacity);
    impl Default for AssetCfgShaderMeta3D {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 2 * 1024 * 1024,
                max: 4 * 1024 * 1024,
                timeout: 60 * 60 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgShaderMeta3D {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgShader3D(pub AssetCapacity);
    impl Default for AssetCfgShader3D {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 2 * 1024 * 1024,
                max: 4 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgShader3D {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }

    #[derive(Resource)]
    pub struct AssetCfgRenderPipeline(pub AssetCapacity);
    impl Default for AssetCfgRenderPipeline {
        fn default() -> Self {
            Self(AssetCapacity {
                flag: false,
                min: 2 * 1024 * 1024,
                max: 4 * 1024 * 1024,
                timeout: 10 * 1000,
            })
        }
    }
    impl AsRef<AssetCapacity> for AssetCfgRenderPipeline {
        fn as_ref(&self) -> &AssetCapacity {
            &self.0
        }
    }
}
