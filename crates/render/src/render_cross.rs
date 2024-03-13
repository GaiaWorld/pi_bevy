

use bevy_ecs::{prelude::Component, schedule::SystemSet};
use pi_render::depend_graph::NodeId;
use derive_deref::{Deref, DerefMut};

/***************************************************用于不同渲染系统中的渲染交叉（如在gui中渲染3d， 在3d中渲染gui）****************************************************/
/// 渲染图节点
#[derive(Debug, Default, Deref, DerefMut, Component, Clone, PartialEq, Eq)]
pub struct GraphId(pub NodeId);

/// DepthRange, 0~1
#[derive(Debug, Default, Component, Clone)]
pub struct DepthRange {pub start: f32, pub end: f32}


/// 渲染方式
#[derive(Component)]
pub struct DrawList {
	/// 要求的深度的长度， 一旦设置，表示 默认0.1, 在0~2单位内（by_draw_list为true时有效）
	pub require_depth: f32,
	/// 是否用渲染列表的方式绘制
	pub draw_list: pi_render::renderer::draw_obj_list::DrawList,
}

// impl Default for RenderWay {
//     fn default() -> Self {
//         Self {
//             require_depth: 0.1,
//             by_draw_list:  false,
//         }
//     }
// }


// /// 渲染列表， 在by_draw_list为true的时候应该提供改组件
// #[derive(Component)]
// pub struct DrawList(pub pi_render::renderer::draw_obj_list::DrawList);

/// 定义交叉渲染的阶段
/// 假如3d中的drawobj需要渲染在gui中，那么3d中设置RenderWay的系统应该在CrossRenderSet之前， 根据gui的深度信息更新3d深度信息的system应该在CrossRenderSet之后
#[derive(Debug, Clone, Hash, SystemSet, PartialEq, Eq)]
pub struct CrossRenderSet;

