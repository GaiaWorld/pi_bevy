use std::any::TypeId;

use bevy_ecs::system::Resource;
use bevy_derive::{DerefMut, Deref};
use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Asset, Garbageer}, homogeneous::HomogeneousMgr};
use pi_hash::XHashMap;
use pi_share::Share;
use serde::{Serialize, Deserialize};

#[derive(Resource, Deref, DerefMut)]
pub struct ShareAssetMgr<A: Asset, G: Garbageer<A> = GarbageEmpty>(pub Share<AssetMgr<A, G>>);

impl<A: Asset, G: Garbageer<A>> ShareAssetMgr<A, G> {
    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, ref_garbage: bool, capacity: usize, timeout: usize) -> Self {
		Self(AssetMgr::new(garbage, ref_garbage, capacity, timeout))
	}
}

impl<A: Asset, G: Garbageer<A>> Clone for ShareAssetMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


#[derive(Resource, Deref, DerefMut)]
pub struct ShareHomogeneousMgr<A, G: pi_assets::homogeneous::Garbageer<A> = pi_assets::homogeneous::GarbageEmpty>(pub Share<HomogeneousMgr<A, G>>);

impl<A, G: pi_assets::homogeneous::Garbageer<A>> ShareHomogeneousMgr<A, G> {
    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, capacity: usize, unit_size: usize, timeout: usize) -> Self {
		Self(HomogeneousMgr::new(garbage, capacity, unit_size, timeout))
	}
}

impl<A, G: pi_assets::homogeneous::Garbageer<A>> Clone for ShareHomogeneousMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AssertConfig (pub XHashMap<TypeId, Capacity>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capacity {
	pub min: usize,
	pub max: usize,
}