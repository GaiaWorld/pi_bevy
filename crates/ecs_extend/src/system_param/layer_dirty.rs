//! 层脏

use super::tree::{EntityTree, RecursiveIterator};
use bevy_ecs::{
    event::ManualEventReader,
    prelude::{Component, Entity, Events},
    query::{Added, Changed, Or, WorldQuery},
    system::{Local, LocalState, Res, ResState, SystemParam, SystemParamFetch},
};
use pi_bevy_ecs_macro::all_tuples;
use pi_dirty::{
    DirtyIterator, LayerDirty as LayerDirty1, NextDirty, PreDirty, ReverseDirtyIterator,
};
use pi_map::vecmap::VecMap;
use std::ops::{Index, IndexMut};
use std::slice::Iter;
use std::{marker::PhantomData, mem::transmute};

#[inline]
pub fn marked_dirty<'w, 's, 'a, T: Eq + Clone>(
    id: Entity,
    v: T,
    dirty_mark_list: &'a mut DirtyMark,
    dirty: &'a mut LayerDirty1<T>,
    id_tree: &EntityTree,
) {
    match id_tree.get_layer(id) {
        Some(r) => marked(id, v, dirty_mark_list, dirty, r.layer()),
        _ => (),
    };
}

pub fn marked<'w, 's, 'a, T: Eq + Clone>(
    id: Entity,
    v: T,
    dirty_mark_list: &'a mut DirtyMark,
    dirty: &'a mut LayerDirty1<T>,
    layer: usize,
) {
    if layer != 0 {
        let layer1 = dirty_mark_list.get_mut_or_default(id);
        if *layer1 != layer {
            if *layer1 != 0 {
                dirty.delete(v.clone(), *layer1);
            }
            *layer1 = layer;
            dirty.mark(v, layer);
        }
    }
}

#[derive(SystemParam)]
pub struct LayerDirty<'w, 's, F: Dirty>
where
    for<'a, 'b> <<F as Dirty>::EventReaderState as SystemParamFetch<'a, 'b>>::Item: EventList,
{
    entity_tree: EntityTree<'w, 's>,
    event_reader: <<F as Dirty>::EventReaderState as SystemParamFetch<'w, 's>>::Item,

    dirty_mark: Local<'s, DirtyMark>,
    layer_list: Local<'s, LayerDirty1<Entity>>,
    #[system_param(ignore)]
    is_init: bool,
}

impl<'w, 's, F: Dirty> LayerDirty<'w, 's, F>
where
    for<'a, 'b> <<F as Dirty>::EventReaderState as SystemParamFetch<'a, 'b>>::Item: EventList,
{
    pub fn iter<'a>(&'a mut self) -> AutoLayerDirtyIter<'w, 's, 'a> {
        self.init();
        AutoLayerDirtyIter {
            matchs: true,
            iter_inner: self.layer_list.iter(),
            mark_inner: &mut self.dirty_mark,
            tree: &self.entity_tree,
            pre_iter: None,
        }
    }

    /// 返回一个手动迭代器
    pub fn iter_manual<'a>(&'a mut self) -> ManualLayerDirtyIter<'w, 's, 'a> {
        self.init();
        ManualLayerDirtyIter {
            matchs: true,
            iter_inner: self.layer_list.iter(),
            mark_inner: &mut self.dirty_mark,
            tree: &self.entity_tree,
            // archetype_id: state.archetype_id,
        }
    }

    pub fn count(&mut self) -> usize {
        self.init();
        self.layer_list.count()
    }

    pub fn start(&mut self) -> usize {
        self.init();
        self.layer_list.start()
    }

    pub fn end(&mut self) -> usize {
        self.init();
        self.layer_list.end()
    }

    pub fn split(&mut self, layer: usize) -> (RemainDirty, OutDirty) {
        self.init();
        let s = self.layer_list.split(layer);
        (RemainDirty(s.0), OutDirty(s.1, &mut self.dirty_mark))
    }

    pub fn iter_reverse<'a>(&'a mut self) -> LayerReverseDirtyIter<'w, 's, 'a> {
        self.init();
        LayerReverseDirtyIter {
            matchs: true,
            iter_inner: self.layer_list.iter_reverse(),
            mark_inner: &mut self.dirty_mark,
            tree: &self.entity_tree,
        }
    }

    pub fn init(&mut self) {
        if self.is_init {
            return;
        }
        self.dirty_mark.map.clear();
        self.layer_list.clear();
        for id in self.event_reader.iter() {
            marked_dirty(
                *id,
                *id,
                &mut self.dirty_mark,
                &mut self.layer_list,
                &self.entity_tree,
            )
        }
        self.is_init = true;
    }

    pub fn mark(&mut self, entity: Entity) {
        marked_dirty(
            entity,
            entity,
            &mut self.dirty_mark,
            &mut self.layer_list,
            &self.entity_tree,
        );
    }
}

pub struct OutDirty<'a>(NextDirty<'a, Entity>, &'a mut DirtyMark);
pub struct RemainDirty<'a>(PreDirty<'a, Entity>);

impl<'a> OutDirty<'a> {
    pub fn iter(&'a mut self) -> OutDirtyIter<'a> {
        let i = self.0.iter();
        OutDirtyIter(i, self.1)
    }
}

pub struct OutDirtyIter<'a>(Iter<'a, Entity>, &'a mut DirtyMark);

impl<'a> Iterator for OutDirtyIter<'a> {
    type Item = Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(r) => {
                self.1.remove(r); // 标记为不脏
                Some(*r)
            }
            None => None,
        }
    }
}

impl<'a> RemainDirty<'a> {
    pub fn mark(&mut self, id: Entity, layer: usize) {
        self.0.mark(id, layer);
    }

    pub fn delete(&mut self, id: Entity, layer: usize) {
        self.0.delete(id, layer);
    }
}

/// 手动迭代器（需要自己控制脏标记）
pub struct ManualLayerDirtyIter<'w, 's, 'a> {
    matchs: bool,
    iter_inner: DirtyIterator<'a, Entity>,

    mark_inner: &'a mut DirtyMark,

    tree: &'a EntityTree<'w, 's>,
}

impl<'w, 's, 'a> Iterator for ManualLayerDirtyIter<'w, 's, 'a> {
    type Item = (Entity, &'a mut DirtyMark, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.matchs {
            return None;
        }

        // 上一个子树迭代完成，继续迭代下一个脏
        loop {
            let item = self.iter_inner.next();
            if let Some((local, layer)) = item {
                if let Some(layer1) = self.mark_inner.get(local) {
                    let layer1 = *layer1;

                    // 记录的层次和实际层次相等，并且在idtree中的层次也相等，则返回该值
                    if layer == layer1 {
                        if let Some(r) = self.tree.get_layer(local.clone()) {
                            if r.layer() == layer {
                                return Some((
                                    local.clone(),
                                    unsafe {
                                        transmute(
                                            self.mark_inner as *mut DirtyMark as usize
                                                as *mut DirtyMark,
                                        )
                                    },
                                    r.layer(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return None;
            }
        }
    }
}

/// 逆序迭代，从叶子节点向父迭代
pub struct LayerReverseDirtyIter<'w, 's, 'a> {
    matchs: bool,
    iter_inner: ReverseDirtyIterator<'a, Entity>,

    mark_inner: &'a mut DirtyMark,

    tree: &'a EntityTree<'w, 's>,
}

impl<'w, 's, 'a> Iterator for LayerReverseDirtyIter<'w, 's, 'a> {
    type Item = Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.matchs {
            return None;
        }

        let item = self.iter_inner.next();
        if let Some((local, layer)) = item {
            if let Some(layer1) = self.mark_inner.get(local) {
                let layer1 = *layer1;

                // 记录的层次和实际层次相等，并且在idtree中的层次也相等，则返回该值
                if layer == layer1 {
                    if let Some(r) = self.tree.get_layer(local.clone()) {
                        if r.layer() == layer {
                            // 是否判断changed？TODO
                            // 记录上次迭代出的实体id，下次将对该节点在itree进行先序迭代
                            return Some(local.clone());
                        }
                    }
                }
            }
        }
        return None;
    }
}

impl<T: Component> Dirty for Changed<T> {
    type EventReaderState = ComponentEventFetchState<Changed<T>>;
}
impl<T: Component> Dirty for Added<T> {
    type EventReaderState = ComponentEventFetchState<Added<T>>;
}

macro_rules! impl_dirty_tuple {
	() => {
	};
	($filter: ident) => {
	};
    ($($filter: ident),*) => {
		// Or TODO
		impl<$($filter: Dirty),*> Dirty for Or<($($filter,)*)> {
			type EventReaderState = ($($filter::EventReaderState,)*);
		}

		// impl<$($filter: Dirty),*> Dirty for Or<($($filter,)*)> {
		// 	type EventReaderState = Or<($($filter::EventReaderState,)*)>;
		// }

		#[allow(non_snake_case)]
		impl<'w, 's, $($filter: Dirty),*> EventList for ($(ComponentEventReader<'w, 's, $filter>,)*) {
			fn iter<'a>(&'a mut self) -> impl Iterator<Item = &'a Entity> {
				let ($($filter),*) = self;
				EmptyIterator(PhantomData)$(.chain($filter.iter()))*
			}
		}
	}
}

all_tuples!(impl_dirty_tuple, 2, 3, F);

pub struct ComponentEvent<T: Dirty> {
    pub id: Entity,
    mark: PhantomData<T>,
}

impl<T: Dirty> ComponentEvent<T> {
    pub fn new(id: Entity) -> Self {
        Self {
            id,
            mark: PhantomData,
        }
    }
}

// 这里的实现必然是安全的，因为ComponentEvent中的唯一字段"id"实现了Send和Sync
unsafe impl<T: Dirty> Send for ComponentEvent<T> {}
unsafe impl<T: Dirty> Sync for ComponentEvent<T> {}

pub trait Dirty: WorldQuery + 'static {
    type EventReaderState: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub struct AutoLayerDirtyIter<'w, 's, 'a> {
    // mark: PhantomData<&'a F>,
    matchs: bool,
    iter_inner: DirtyIterator<'a, Entity>,

    mark_inner: &'a mut DirtyMark,

    tree: &'a EntityTree<'w, 's>,
    // archetype_id: Local,
    pre_iter: Option<RecursiveIterator<'a, EntityTree<'w, 's>>>,
    // layers: &'a mut  ReadFetch<C>,
}

struct EmptyIterator<'a>(PhantomData<&'a ()>);
impl<'a> Iterator for EmptyIterator<'a> {
    type Item = &'a Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'w, 's, 'a> Iterator for AutoLayerDirtyIter<'w, 's, 'a> {
    type Item = Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.matchs {
            return None;
        }
        if let Some(r) = &mut self.pre_iter {
            // 上次迭代的脏还没完成，继续迭代
            match r.next() {
                Some(next) => {
                    self.mark_inner.remove(&next); // 标记为不脏
                    return Some(next);
                }
                None => self.pre_iter = None,
            };
        }

        // 上一个子树迭代完成，继续迭代下一个脏
        let item = self.iter_inner.next();
        if let Some((local, layer)) = item {
            if let Some(layer1) = self.mark_inner.get(local) {
                let layer1 = *layer1;
                self.mark_inner.remove(local); // 标记为不脏

                // 记录的层次和实际层次相等，并且在idtree中的层次也相等，则返回该值
                if layer == layer1 {
                    if let Some(r) = self.tree.get_layer(*local) {
                        if r.layer() == layer {
                            // 是否判断changed？TODO
                            // 记录上次迭代出的实体id，下次将对该节点在itree进行先序迭代
                            if let Some(down) = self.tree.get_down(*local) {
                                let head = down.head();
                                self.pre_iter = Some(self.tree.recursive_iter(head));
                            }
                            return Some(*local);
                        }
                    }
                }
            }
        }
        return None;
    }
}

// #[derive(SystemParam)]
// pub struct OrSystemParam<T> {

// }

pub struct ComponentEventReader<'w, 's, F: Dirty> {
    reader: Local<'s, ManualEventReader<ComponentEvent<F>>>,
    events: Res<'w, Events<ComponentEvent<F>>>,
}

impl<'w, 's, F: Dirty> bevy_ecs::system::SystemParam for ComponentEventReader<'w, 's, F> {
    type Fetch = ComponentEventFetchState<F>;
}

#[doc(hidden)]
pub struct ComponentEventFetchState<F: Dirty> {
    state: (
        LocalState<ManualEventReader<ComponentEvent<F>>>,
        ResState<Events<ComponentEvent<F>>>,
    ),
}
unsafe impl<F: Dirty> bevy_ecs::system::SystemParamState for ComponentEventFetchState<F> {
    fn init(
        world: &mut bevy_ecs::world::World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) -> Self {
        Self {
            state: (
                LocalState::<ManualEventReader<ComponentEvent<F>>>::init(world, system_meta),
                ResState::<Events<ComponentEvent<F>>>::init(world, system_meta),
            ),
        }
    }
    fn new_archetype(
        &mut self,
        archetype: &bevy_ecs::archetype::Archetype,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) {
        self.state.new_archetype(archetype, system_meta)
    }
    fn apply(&mut self, world: &mut bevy_ecs::world::World) {
        self.state.apply(world)
    }
}
impl<'w, 's, F: Dirty> bevy_ecs::system::SystemParamFetch<'w, 's> for ComponentEventFetchState<F> {
    type Item = ComponentEventReader<'w, 's, F>;
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: &'w bevy_ecs::world::World,
        change_tick: u32,
    ) -> Self::Item {
        ComponentEventReader { reader : < < Local < 's , ManualEventReader < ComponentEvent<F> > > as bevy_ecs :: system :: SystemParam > :: Fetch as bevy_ecs :: system :: SystemParamFetch > :: get_param (& mut state . state . 0 , system_meta , world , change_tick) , events : < < Res < 'w , Events < ComponentEvent<F> > > as bevy_ecs :: system :: SystemParam > :: Fetch as bevy_ecs :: system :: SystemParamFetch > :: get_param (& mut state . state . 1 , system_meta , world , change_tick) , }
    }
}
unsafe impl<F: Dirty> bevy_ecs::system::ReadOnlySystemParamFetch for ComponentEventFetchState<F> {}

pub trait EventList: SystemParam {
    fn iter<'a>(&'a mut self) -> impl Iterator<Item = &'a Entity>;
}

impl<'w, 's, F: Dirty> EventList for ComponentEventReader<'w, 's, F> {
    fn iter(&mut self) -> impl Iterator<Item = &Entity> {
        self.reader.iter_with_id(&self.events).map(|r @ (_, _id)| {
            // trace!("EventReader::iter() -> {}", id);
            &r.0.id
        })
    }
}

#[derive(Debug, Default)]
pub struct DirtyMark {
    map: VecMap<usize>,
}

impl DirtyMark {
    pub fn get(&self, id: &Entity) -> Option<&usize> {
        self.map.get(id.index() as usize)
    }

    pub fn get_mut_or_default(&mut self, id: Entity) -> &mut usize {
        if let Some(r) = self.map.get_mut(id.index() as usize) {
            return unsafe { transmute(r) }; // 语法不让过，实际写法没问题，这里非安全写法是安全的
        }

        self.map.insert(id.index() as usize, 0);
        &mut self.map[id.index() as usize]
    }

    pub fn remove(&mut self, id: &Entity) -> Option<usize> {
        self.map.remove(id.index() as usize)
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}

impl Index<Entity> for DirtyMark {
    type Output = usize;

    #[inline]
    fn index(&self, index: Entity) -> &Self::Output {
        &self.map[index.index() as usize]
    }
}

impl IndexMut<Entity> for DirtyMark {
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        &mut self.map[index.index() as usize]
    }
}
