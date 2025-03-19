use pi_key_alloter::KeyData;
use pi_render::components::view::target_alloc::ShareTargetView;
use pi_world::{fetch::OrDefault, insert::Component, query::Query, world::Entity};
use pi_key_alloter::Key;

use crate::render_cross::GraphId;
const TARGET_TYPE: &'static str = "asimage:://"; // 渲染目标类型的target， 其他类型TODO


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    None,
    Target
}

/// 渲染目标
#[derive(Component, Default)]
pub struct RenderTarget(pub Option<ShareTargetView>);

pub enum Data {
    Target(ShareTargetView)
}

#[derive(Debug, Clone, Copy)]
pub enum LoadError {
    EntityInvalid, // 加载失败（节点可能已经销毁）
    UrlInvalid, // url无效
    MismatchProtocol, // 协议不匹配
}

pub fn entity_to_asimage_url(entity: Entity) -> String {
    format!("{}{}v{}", TARGET_TYPE, entity.index(), entity.data().version())
}

// 从data_url中加载
pub fn load_from_asimage_url(url: &str, query: &Query<(OrDefault<RenderTarget>, OrDefault<GraphId>)>) -> Result<Option<(ShareTargetView, GraphId, Entity)>, LoadError> {
    if !url.starts_with(TARGET_TYPE) {
        return Err(LoadError::MismatchProtocol);
    }

    let r = url[TARGET_TYPE.len()..].to_string();
    let mut r = r.split("v");
    let index: u32 = match r.next() {
        Some(r) => match r.parse() {
            Ok(r) => r,
            Err(_) => return Err(LoadError::UrlInvalid),
        },
        None => return Err(LoadError::UrlInvalid),
    };
    let version: u32 = match r.next() {
        Some(r) => match r.parse() {
            Ok(r) => r,
            Err(_) => return Err(LoadError::UrlInvalid),
        },
        None => return Err(LoadError::UrlInvalid),
    };
    
    let entity = Entity::from(KeyData::from_ffi((u64::from(version) << 32) | u64::from(index)));
    // log::warn!("entity=========={:?}", (entity, index, version));
    match query.get(entity) {
        Ok((r, id)) => match &r.0 {
            Some(r) =>  return Ok(Some((r.clone(), id.clone(), entity))),
            None => return Ok(None),
        },
        Err(_r) => return Err(LoadError::EntityInvalid),
    };
}
