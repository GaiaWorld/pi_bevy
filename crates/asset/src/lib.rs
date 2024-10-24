use std::any::TypeId;

use bevy_ecs::system::{Resource, ResMut, Local};
use bevy_app::prelude::{App, Plugin, Last};
use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Asset, Garbageer, Size}, homogeneous::HomogeneousMgr};
use pi_hash::XHashMap;
use pi_render::renderer::bind_group::{BindGroup, BindGroupLayout};
use pi_render::renderer::sampler::SamplerRes;
use pi_render::renderer::texture::{ImageTexture, ImageTextureView};
use pi_render::renderer::vertex_buffer::EVertexBufferRange;
use pi_render::rhi::asset::{TextureRes, RenderRes};
use pi_render::rhi::pipeline::RenderPipeline;
use pi_share::Share;
use serde::{Serialize, Deserialize};
use pi_time::now_millisecond;
use pi_null::Null;
use derive_deref::{Deref, DerefMut};

/// 资产功能插件， 负责添加容量分配器`Allocator`作为单例， 添加容量配置单例`AssetConfig`, 添加system `collect`负责定时整理资产
pub struct PiAssetPlugin {
	pub total_capacity: usize,
	pub asset_config: AssetConfig,
}
impl Default for PiAssetPlugin {
	fn default() -> Self {
		Self { total_capacity: 32 * 1024 * 1024, asset_config: AssetConfig::default() }
	}
}

impl Plugin for PiAssetPlugin {
    fn build(&self, app: &mut App) {
		let total_capacity = if self.total_capacity.is_null() || self.total_capacity == 0 {
			32 * 1024 * 1024 // 默认32M
		} else {
			self.total_capacity
		};
		app.insert_resource(Allocator(pi_assets::allocator::Allocator::new(total_capacity)));
		app.insert_resource(self.asset_config.clone());

		// 帧推结束前，整理资产（这里采用在帧推结束前整理资产， 而不是利用容量分配器自带的定时整理， 可以防止整理立即打断正在进行的其他system）
		app.add_systems(Last, collect);

		#[cfg(feature="account_info")]
		app.add_systems(Last, account);
	}
}

// 上次容量整理时间
pub struct LastCollectTime(pub u64);
impl Default for LastCollectTime {
    fn default() -> Self {
        Self(now_millisecond())
    }
}

/// 整理容量
pub fn collect(mut allocator: ResMut<Allocator>, last_collect_time: Local<LastCollectTime>) {
	// 暂时设置为每秒整理， 这里间隔配置？TODO
	if now_millisecond() - last_collect_time.0 > 1000 {
		allocator.collect(now_millisecond())
	}
}

/// 容量分配器
#[derive(Resource, Deref, DerefMut)]
pub struct Allocator(pub pi_assets::allocator::Allocator);

/// 资产配置
#[derive(Debug, Clone, Resource, Default)]
pub struct AssetConfig (XHashMap<TypeId, AssetDesc>);

impl AssetConfig {
	// 为某类型的资产管理器配置容量和超时时间
	#[inline]
    pub fn insert<T: Size>(&mut self, cfg: AssetDesc) {
        self.0.insert(std::any::TypeId::of::<T>(), cfg);
    }

	// 取到某类型的资产管理器的容量、超时配置
	#[inline]
    pub fn get<T: Size>(&self) -> Option<&AssetDesc> {
		self.0.get(&std::any::TypeId::of::<T>())
    }
}

/// 资产容量和超时描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDesc {
	pub ref_garbage: bool,
	pub min: usize,
	pub max: usize,
	pub timeout: usize,
}

/// 资源、资产管理器
#[derive(Resource, Deref, DerefMut)]
pub struct ShareAssetMgr<A: Asset, G: Garbageer<A> = GarbageEmpty>(pub Share<AssetMgr<A, G>>);

impl<A: Asset, G: Garbageer<A>> ShareAssetMgr<A, G> {
	pub fn new_with_config(garbage: G, default: &AssetDesc, asset_config: &AssetConfig, allocator: &mut Allocator) -> Self {
		let desc = asset_config.get::<A>().unwrap_or(default);
		let r = AssetMgr::new(garbage, desc.ref_garbage, desc.min, desc.timeout);
		allocator.register(r.clone(), desc.min, desc.max);
		Self(r)
	}

    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, ref_garbage: bool, capacity: usize, timeout: usize) -> Self {
		Self(AssetMgr::new(garbage, ref_garbage, capacity, timeout))
	}

    pub fn create(garbage: G, ref_garbage: bool, cfg: &AssetCapacity) -> Self {
        Self(AssetMgr::new(garbage, ref_garbage, cfg.min, cfg.timeout))
    }
}

impl<A: Asset, G: Garbageer<A>> Clone for ShareAssetMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// 资源， 同质资产管理器
#[derive(Resource, Deref, DerefMut)]
pub struct ShareHomogeneousMgr<A: Size, G: pi_assets::homogeneous::Garbageer<A> = pi_assets::homogeneous::GarbageEmpty>(pub Share<HomogeneousMgr<A, G>>);

impl<A: Asset + Size, G: pi_assets::homogeneous::Garbageer<A>> ShareHomogeneousMgr<A, G> {
    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, capacity: usize, timeout: usize) -> Self {
		Self(HomogeneousMgr::new(garbage, capacity, timeout))
	}

	pub fn new_with_config(garbage: G, default: &AssetDesc, asset_config: &AssetConfig, allocator: &mut Allocator) -> Self {
		let desc = asset_config.get::<A>().unwrap_or(default);
		let r = HomogeneousMgr::new(garbage, desc.min, desc.timeout);
		allocator.register(r.clone(), desc.min, desc.max);
		Self(r)
	}
}

impl<A: Size, G: pi_assets::homogeneous::Garbageer<A>> Clone for ShareHomogeneousMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


pub trait TAssetCapacity {
	const ASSET_TYPE: &'static str;
	fn capacity() -> AssetCapacity;
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssetCapacity {
	pub flag: bool,
	pub min: usize,
	pub max: usize,
	pub timeout: usize,
}
impl Default for AssetCapacity {
    fn default() -> Self {
        Self { flag: false, min: 1024, max: 10 * 1024, timeout: 10 * 1000 }
    }
}
#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct AssetMgrConfigs (pub XHashMap<String, AssetCapacity>);
impl AssetMgrConfigs {
	#[inline]
    pub fn insert(&mut self, key: String, cfg: AssetCapacity) {
        self.0.insert(key, cfg);
    }
    pub fn query<T: TAssetCapacity>(&mut self) -> AssetCapacity  {
		if let Some(cfg) = self.0.get(T::ASSET_TYPE) {
			cfg.clone()
		} else {
			self.0.insert(String::from(T::ASSET_TYPE), T::capacity());
			T::capacity()
		}
    }
}

impl TAssetCapacity for ImageTexture {
	const ASSET_TYPE: &'static str = "IMAGE_TEXTURE";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 1024 * 1024, max: 10 * 1024 * 1024, timeout: 10 * 1000 }
	}
}
impl TAssetCapacity for ImageTextureView {
	const ASSET_TYPE: &'static str = "IMAGE_TEXTURE_VIEW";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 1024 * 1024, max: 10 * 1024 * 1024, timeout: 10 * 1000 }
	}
}
impl TAssetCapacity for SamplerRes {
	const ASSET_TYPE: &'static str = "SAMPLER_RES";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 10 * 1024, max: 100 * 1024, timeout: 1000 * 1000 }
	}
}
impl TAssetCapacity for TextureRes {
	const ASSET_TYPE: &'static str = "TEXTURE_RES";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 1024 * 1024, max: 10 * 1024 * 1024, timeout: 10 * 1000 }
	}
}
impl TAssetCapacity for EVertexBufferRange {
	const ASSET_TYPE: &'static str = "VERTEX_BUFFER_RANGE";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 256, max: 1024, timeout: 1000 }
	}
}
impl TAssetCapacity for BindGroup {
	const ASSET_TYPE: &'static str = "BIND_GROUP";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 32 * 1024, max: 64 * 1024, timeout: 1000 }
	}
}
impl TAssetCapacity for BindGroupLayout {
	const ASSET_TYPE: &'static str = "BIND_GROUP_LAYOUT";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 32 * 1024, max: 64 * 1024, timeout: 1000 }
	}
}
impl TAssetCapacity for RenderRes<RenderPipeline> {
	const ASSET_TYPE: &'static str = "RENDER_PIPELINE";
	fn capacity() -> AssetCapacity {
        AssetCapacity { flag: false, min: 1024 * 1024, max: 2 * 1024 * 1024, timeout: 10 * 1000 }
	}
}

#[cfg(feature="account_info")]
fn account(allotor: ResMut<Allocator>, mut pre_time: Local<u64>) {
	if *pre_time == 0 {
		*pre_time = now_millisecond();
		return;
	}

	if now_millisecond() - *pre_time > 10000 {
		log::warn!("asset account: {:?}", allotor.account());
		*pre_time = now_millisecond();
	}
}
