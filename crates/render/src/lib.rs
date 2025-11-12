#![feature(trivial_bounds)]

//!
//! pi_render 的 bevy 封装
//!

#[macro_use]
extern crate lazy_static;

use pi_render::font::FontId;
use pi_render::font::FontSheet;
use pi_render::font::Glyph;
use pi_render::font::Size;
use pi_world::world::World;
use pi_world::query::Query;
use pi_world::schedule::First;
use pi_world::world::Entity;
use pi_world::filter::With;
use serde::{Deserialize, Serialize};
use pi_bevy_ecs_extend::prelude::{EntityTag, Root, Down, Up};
use pi_null::Null;
mod async_queue;
mod clear_node;
pub mod render_cross;
pub mod constant;
pub mod graph;
mod init_render;
mod plugin;
mod render_windows;
mod resource;
pub mod system;
pub mod asimage_url;
pub mod cmd_play;
#[cfg(not(target_arch = "wasm32"))]
pub mod spector;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SpectorNode {
    pub uniqueID: f64,
    pub info: String,
    pub tag: String,
    pub childs: Vec<SpectorNode>,
}

#[derive(Serialize, Debug, Clone, Default)]
#[allow(non_snake_case)]
pub struct Cmd<T: serde::Serialize> {
    pub cmd: String,
    pub payload: T,
}


use std::mem::transmute;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;

use derive_deref::{Deref, DerefMut};
/// 渲染图
pub use graph::*;
use pi_render::components::view::target_alloc::{GetTargetView, ShareTargetView, TargetView};
use pi_world::insert::Component;
/// 渲染 插件
pub use plugin::*;
/// 单例
pub use resource::*;
/// 单例
pub use cmd_play::*;

lazy_static! {
    pub static ref IS_RESUMED: AtomicBool = AtomicBool::new(true);
}

#[derive(Default, DerefMut, Deref)]
pub struct ResStateTextureLoader(pi_render::renderer::texture_loader::loader::StateTextureLoader);
#[derive(Default, DerefMut, Deref)]
pub struct ResTextureCombineAtlas2DMgr(pi_render::renderer::texture_loader::texture_atlas::TextureCombineAtlas2DMgr);

/// 标签
pub use clear_node::CLEAR_WIDNOW_GRAPH;
pub use clear_node::ScreenWithPostprocess;

#[derive(Default, Clone, Component)]
pub struct SimpleInOut {
    pub target: Option<ShareTargetView>,
	pub valid_rect: Option<(u32, u32, u32, u32)>, // x, y, w, h
}

impl std::fmt::Debug for SimpleInOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match &self.target {
            Some(r ) => Some(r.target().colors[0].0.id),
            None => None,
        };

        f.debug_struct("SimpleInOut").field("target", &r ).field("valid_rect", &self.valid_rect).finish()
    }
}
// TODO, Send问题， 临时解决
unsafe impl Send for SimpleInOut {}
unsafe impl Sync for SimpleInOut {}

impl GetTargetView for SimpleInOut {
    fn get_target_view(&self) -> Option<&TargetView> {
        return self.target.as_ref().map(|r| &***r);
    }
}

pub mod asset_config {
    // use bevy_ecs::prelude::Resource;
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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

    // #[derive(Resource)]
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


pub fn get_document_tree(world: &mut World, root: Entity) -> SpectorNode {
    let mut query = world.query::<(&Down, &Up, Option<&EntityTag>)>();
    let query = query.get_param(world);
    let mut n = SpectorNode::default();
    init_node( root, &mut n, &query);
    return n;
}

pub fn get_roots(world: &mut World) -> Vec<Entity> {
    let mut query = world.query::<Entity, (With<Root>)>();
    return query.iter(world).collect();
}


pub fn init_node( 
    id: Entity, 
    node: &mut SpectorNode, 
    query: &Query<(&Down, &Up, Option<&EntityTag>)>,
) {
    let (down, _, tag) = query.get(id).unwrap();
    node.tag = "div".to_string();
    if let Some(tag) = tag {
        node.tag = tag.0.to_string();
    }
    node.uniqueID = unsafe {transmute(id)};
    node.info = format!("ID={:?}", id);

    let mut cur_child = down.head();
    while !cur_child.is_null() {
        let mut n = SpectorNode::default();
        init_node( cur_child, &mut n, query);
        node.childs.push(n);
        let (_c_down, c_up, tag) = query.get(cur_child).unwrap();
        cur_child = c_up.next();
    }
}

/// cmd `request-document`
pub fn _request_document(world: &mut World) -> Vec<Cmd<SpectorNode>> {
    let roots = get_roots(world);
    // log::error!("root======{:?}", &roots);
    let mut result = vec![];
    for root in roots.into_iter() {
        let msg = get_document_tree(world, root);
        let cmd = Cmd {
            cmd: "document-data".to_string(),
            payload: msg,
        };
        result.push(cmd);
    }
    result
}

#[derive(Deref)]
pub struct ShareFontSheet(pub pi_share::Share<pi_share::ShareCell<pi_render::font::FontSheet>>);

#[cfg(target_arch = "wasm32")]
unsafe impl Send for ShareFontSheet {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for ShareFontSheet {}

// impl FromWorld for ShareFontSheet {
//     fn from_world(world: &mut World) -> Self {
//         let texture_res_mgr = world.get_single_res::<ShareAssetMgr<TextureRes>>().unwrap();
//         let device = world.get_single_res::<PiRenderDevice>().unwrap();
// 		let queue = world.get_single_res::<PiRenderQueue>().unwrap();
// 		let limits = device.limits();
//         ShareFontSheet(Share::new(ShareCell::new(FontSheet::new(&device.0, &texture_res_mgr.0, &queue.0, limits.max_texture_dimension_2d, false))))
//     }
// }


impl ShareFontSheet {
    pub fn new(world: &mut World, font_type: pi_hal::font::font::FontType) -> Self {
		world.init_single_res::<crate::resource::TextureKeyAlloter>();
        let texture_res_mgr = world.get_single_res::<pi_bevy_asset::ShareAssetMgr<pi_render::rhi::asset::TextureRes>>().unwrap();
		let alloter = world.get_single_res::<crate::resource::TextureKeyAlloter>().unwrap();
		
        let device = world.get_single_res::<PiRenderDevice>().unwrap();
		let queue = world.get_single_res::<PiRenderQueue>().unwrap();
		let limits = device.limits();
        ShareFontSheet(pi_share::Share::new(pi_share::ShareCell::new(pi_render::font::FontSheet::new(&device.0, &texture_res_mgr.0, alloter.0.clone(),&queue.0, limits.max_texture_dimension_2d, font_type))))
    }
    pub fn char_calc(
        fontsheet: &mut FontSheet,
        fontid: FontId,
        char: char,
        scaleoffset: &mut [f32],
        uvtilloff: &mut [f32],
        line_height: f32,
        fontsize: f32,
        half_extend: f32, // 一半的扩展宽度（描边， 阴影， 外法光都需要额外扩展）
        global_line_height: f32,
        global_font_size: f32,
        global_ascender: f32,
    ) -> bool {
        if let Some(glyphid) = fontsheet.glyph_id(fontid, char) {
            let size = fontsheet.texture_size();
            let glyph = fontsheet.font_mgr().table.sdf2_table.glyph(glyphid);
            Self::_calc(scaleoffset, uvtilloff, line_height, fontsize, half_extend, glyph, global_line_height, global_font_size, global_ascender, size);
            true
        } else {
            false
        }
    }
    fn _calc(
        scaleoffset: &mut [f32],
        uvtilloff: &mut [f32],
        line_height: f32,
        fontsize: f32,
        half_extend: f32, // 一半的扩展宽度（描边， 阴影， 外法光都需要额外扩展）
        glyph: &Glyph,
        global_line_height: f32,
        global_font_size: f32,
        global_ascender: f32,
        size: Size<usize>,
    ) {
        let font_line_height = global_line_height * fontsize / global_font_size;
        let stroke_width = half_extend * 2.0;

        let plane_min_x = glyph.plane_min_x * fontsize - half_extend;
        let plane_min_y = ((global_line_height - font_line_height) / 2.0)/*上下一半剩余行高的空间*/ + (global_ascender - glyph.plane_max_y) * fontsize - half_extend;
        

        let scale = fontsize / global_font_size;
        let plane_width  = ((glyph.plane_max_x - glyph.plane_min_x) * fontsize + stroke_width) * 0.5;
        let plane_height = ((glyph.plane_max_y - glyph.plane_min_y) * fontsize + stroke_width) * 0.5;

        let plane_max_x = plane_min_x + plane_width;
        let plane_max_y = plane_min_y + plane_height;

        let half_stroke_uv = half_extend * scale;

        // 否则， push aabb
        scaleoffset[0] = plane_width ;
        scaleoffset[1] = plane_height;
        scaleoffset[2] = plane_min_x ;
        scaleoffset[3] = plane_min_y ;

        let u0 = glyph.x - half_stroke_uv;
        let v0 = glyph.y - half_stroke_uv;
        let us = glyph.width  + half_stroke_uv + half_stroke_uv;
        let vs = glyph.height + half_stroke_uv + half_stroke_uv;

        uvtilloff[0] = us / (size.width  as f32);
        uvtilloff[1] = vs / (size.height as f32);
        uvtilloff[2] = u0 / (size.width  as f32);
        uvtilloff[3] = v0 / (size.height as f32);
    }
}
