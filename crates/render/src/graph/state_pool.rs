use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use pi_share::{Share, ShareMutex};

/// 目的: 提升性能
/// 系统状态池，用于存放 渲染节点的 SystemState
/// 好处是 创建相同 struct 的 SystemState 时候，可以从缓存中获取，而不是每次都创建
/// TODO: 目前不考虑 释放问题，假设不会太多，待之后实验决定要不要管理；
#[derive(Default)]
pub struct SystemStatePool {
    data: Share<ShareMutex<SystemStatePoolImpl>>,
}

impl Clone for SystemStatePool {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl SystemStatePool {
    pub fn set<T: 'static + Send + Sync>(&mut self, state: T) {
        self.data.lock().set(state);
    }

    /// 从缓存中获取，如果没有，则返回 None
    pub fn get<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let r = self.data.lock().get();

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
    map: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
}

impl SystemStatePoolImpl {
    fn set<T: 'static + Send + Sync>(&mut self, state: T) {
        let type_id = TypeId::of::<T>();

        let vec = self.map.entry(type_id).or_insert_with(Vec::new);

        vec.push(Box::new(state));
    }

    /// 从缓存中获取，如果没有，则返回 None
    fn get<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(vec) = self.map.get_mut(&type_id) {
            if let Some(state) = vec.pop() {
                return state.downcast::<T>().ok().map(|b| *b);
            }
        }
        None
    }
}

#[test]
fn test_system_state_pool() {
    struct A {
        a: i32,
        b: f32,
    }

    let mut pool = SystemStatePool::default();

    let state = pool.get::<A>();
    assert!(state.is_none());

    pool.set(A { a: 1, b: 2.0 });

    let state = pool.get::<i32>();
    assert!(state.is_none());

    let state = pool.get::<A>();
    assert!(state.is_some());

    let a = state.unwrap();
    assert_eq!(a.a, 1);
    assert_eq!(a.b, 2.0);

    let state = pool.get::<A>();
    assert!(state.is_none());
}
