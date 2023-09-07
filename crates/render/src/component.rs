use bevy_ecs::prelude::Component;
use pi_render::depend_graph::NodeId;
use derive_deref::{Deref, DerefMut};

/// 渲染图节点
#[derive(Debug, Default, Deref, DerefMut, Component, Clone, PartialEq, Eq)]
pub struct GraphId(pub NodeId);