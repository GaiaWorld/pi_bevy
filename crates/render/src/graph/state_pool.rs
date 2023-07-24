use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use pi_share::{Share, ShareCell};

/// 目的: 提升性能
/// 系统状态池，用于存放 渲染节点的 SystemState
/// 好处是 创建相同 struct 的 SystemState 时候，可以从缓存中获取，而不是每次都创建
/// TODO: 目前不考虑 释放问题，假设不会太多，待之后实验决定要不要管理；
#[derive(Default)]
pub(crate) struct SystemStatePool(Share<ShareCell<SystemStatePoolImpl>>);

impl Clone for SystemStatePool {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SystemStatePool {
    pub(crate) fn set<T: 'static + Send + Sync>(&mut self, state: T) {
        self.0.borrow_mut().set(state);
    }

    /// 从缓存中获取，如果没有，则返回 None
    pub(crate) fn get<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let r = self.0.borrow_mut().get();

        log::debug!(
            "SystemStatePool::get, type: {:?}, is_some: {}",
            std::any::type_name::<T>(),
            r.is_some()
        );

        r
    }
}

#[derive(Default)]
struct SystemStatePoolImpl {
    map: HashMap<TypeId, Vec<Arc<dyn Any + Send + Sync>>>,
}

impl SystemStatePoolImpl {
    fn set<T: 'static + Send + Sync>(&mut self, state: T) {
        let type_id = TypeId::of::<T>();
        let vec = self.map.entry(type_id).or_insert_with(Vec::new);
        vec.push(Arc::new(state));
    }

    /// 从缓存中获取，如果没有，则返回 None
    fn get<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(vec) = self.map.get_mut(&type_id) {
            if let Some(state) = vec.pop() {
                return downcast_and_unwrap::<T>(state);
            }
        }
        None
    }
}

fn downcast_and_unwrap<T: 'static + Send + Sync>(arc_any: Arc<dyn Any>) -> Option<T> {
    let raw = Arc::into_raw(arc_any);
    let unique: Box<dyn Any> = unsafe { Box::from_raw(raw as *mut _) };
    let downcasted = unique.downcast::<T>();
    match downcasted {
        Ok(t) => Some(*t),
        Err(_) => None,
    }
}
