use bevy::prelude::{Deref, DerefMut, Component};
use pi_render::depend_graph::NodeId;

/// 渲染图节点
#[derive(Debug, Default, Deref, DerefMut, Component, Clone)]
pub struct GraphId(pub NodeId);