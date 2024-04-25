//! 实体树

// use bevy_ecs::{{prelude::{Entity, Component, EventWriter}, system::{Query, Commands, SystemParam, SystemMeta}, query::Changed, archetype::Archetype, world::unsafe_world_cell::UnsafeWorldCell, component::Tick}, prelude::World};
use pi_bevy_ecs_macro::SystemParam;
use derive_deref::{Deref, DerefMut};

use pi_null::Null;
use pi_slotmap_tree::{
    ChildrenIterator as ChildrenIterator1, Down as Down1, Layer as Layer1,
    RecursiveIterator as RecursiveIterator1, Storage, StorageMut, Tree, Up as Up1,
};
use serde::{Deserialize, Serialize};

use pi_world::{query::Query, world::{Entity, World, self}, system::SystemMeta};

// use pi_print_any::{println_any, out_any};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Root;

#[derive(Debug, Clone, Deref, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct TreeKey(pub Entity);

impl Null for TreeKey {
    fn null() -> Self {
		unsafe { transmute(u64::null())}
    }

    fn is_null(&self) -> bool {
        let r = unsafe { transmute::<_, u64>(self.0)} == u64::null();
		r
    }
}

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct Layer(Layer1<TreeKey>);
impl Layer  {
	#[inline]
	pub fn layer(&self) -> usize{
		self.0.layer()
	}
	#[inline]
	pub fn root(&self) -> Entity {
		self.0.root().0
	}
}

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct Up(Up1<TreeKey>);
impl Up  {
	#[inline]
	pub fn parent(&self) -> Entity {
		self.0.parent().0
	}
	#[inline]
	pub fn prev(&self) -> Entity {
		self.0.prev().0
	}
	#[inline]
	pub fn next(&self) -> Entity {
		self.0.next().0
	}
}

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct Down(Down1<TreeKey>);
impl Down  {
	#[inline]
	pub fn head(&self) -> Entity {
		self.0.head().0
	}
	#[inline]
	pub fn tail(&self) -> Entity {
		self.0.tail().0
	}
	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}
	#[inline]
	pub fn count(&self) -> usize {
		self.0.count()
	}
}

// 放入EntityTree， 并为其实现一个Default方法
pub struct Down2(Down1<TreeKey>);
impl Default for Down2 {
    fn default() -> Self {
        Self(
			Down1 {
				head: TreeKey::null(),
				tail: TreeKey::null(),
				len: 0,
				count: 1,
			})
    }
}


// use pi_world::prelude::
// impl_system_param_tuple!()

// impl SystemParam for EntityTree<'_> {
//     type State = (
//         <Query<'static, &'static  Layer> as SystemParam>::State,
// 		<Query<'static, &'static  Up> as SystemParam>::State,
// 		<Query<'static, &'static  Down> as SystemParam>::State,
//     );

//     type Item<'world> = EntityTree<'world>;

//     fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
//         (
// 			<Query< 'static, &'static  Layer> as SystemParam>::init_state(world, system_meta),
// 			<Query<'static, &'static  Up> as SystemParam>::init_state(world, system_meta),
// 			<Query<'static, &'static  Down> as SystemParam>::init_state(world, system_meta),
// 		)
//     }

//     fn get_param<'world>(
//         world: &'world World,
//         system_meta: &'world SystemMeta,
//         state: &'world mut Self::State,
//     ) -> Self::Item<'world> {
//         EntityTree{
//             layer_query: <Query< 'static, &'static  Layer> as SystemParam>::get_param(world, system_meta, &mut state.0),
//             up_query:  <Query< 'static, &'static  Up> as SystemParam>::get_param(world, system_meta, &mut state.1),
//             down_query:  <Query< 'static, &'static  Down> as SystemParam>::get_param(world, system_meta, &mut state.2),
//         }
//     }

//     fn get_self<'world>(
//         world: &'world pi_world::world::World,
//         system_meta: &'world pi_world::system::SystemMeta,
//         state: &'world mut Self::State,
//     ) -> Self {
//         unsafe { transmute(Self::get_param(world, system_meta, state)) }
//     }
// }

impl<'w, 's> Storage<TreeKey> for EntityTree<'w, 's> {
	fn get_up(&self, k: TreeKey) -> Option<&Up1<TreeKey>> {
		self.get_up(k.0).map(|r|{&**r})
	}
	fn up(&self, k: TreeKey) -> &Up1<TreeKey> {
		self.up(k.0)
	}

	fn get_layer(&self, k: TreeKey) -> Option<&Layer1<TreeKey>> {
		self.get_layer(k.0).map(|r|{&**r})
	}
	fn layer(&self, k: TreeKey) -> &Layer1<TreeKey> {
		self.layer(k.0)
	}

	fn get_down(&self, k: TreeKey) -> Option<&Down1<TreeKey>> {
		self.get_down(k.0).map(|r|{&**r})
	}

	fn down(&self, k: TreeKey) -> &Down1<TreeKey> {
		self.down(k.0)
	}
}

impl<'w, 's> EntityTree<'w, 's> {
	pub fn get_up(&self, k: Entity) -> Option<&Up> {
		match self.up_query.get(k) {
			Ok(r) => Some(r),
			_ => None,
		}
	}
	pub fn up(&self, k: Entity) -> &Up {
		self.up_query.get(k).unwrap()
	}

	pub fn get_layer(&self, k: Entity) -> Option<&Layer> {
		match self.layer_query.get( k) {
			Ok(r) => Some(r),
			_ => None,
		}
	}
	pub fn layer(&self, k: Entity) -> &Layer{
		self.layer_query.get(k).unwrap()
	}

	pub fn get_down(&self, k: Entity) -> Option<&Down> {
		match self.down_query.get(k) {
			Ok(r) => Some(r),
			_ => None,
		}
	}

	pub fn down(&self, k: Entity) -> &Down {
		self.down_query.get(k).unwrap()
	}

	pub fn iter(&self, node_children_head: Entity) -> ChildrenIterator<EntityTree<'w, 's>> {
		ChildrenIterator {
			inner: ChildrenIterator1::new(self, TreeKey(node_children_head))
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<EntityTree<'w, 's>> {
		let head = TreeKey(node_children_head);
		let len = if head.is_null() {
			0
		} else {
			1
		};
		RecursiveIterator{inner:RecursiveIterator1::new(self, head, len)}
	}
}

pub struct ChildrenIterator<'a, S: Storage<TreeKey>> {
	inner: ChildrenIterator1<'a, TreeKey, S>
}

impl<'a, S: Storage<TreeKey>> Iterator for ChildrenIterator<'a, S> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
		unsafe{transmute(self.inner.next())}
    }
}

pub struct RecursiveIterator<'a, S: Storage<TreeKey>> {
	inner: RecursiveIterator1<'a, TreeKey, S>
}

impl<'a, S: Storage<TreeKey>> Iterator for RecursiveIterator<'a, S> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
		unsafe{transmute(self.inner.next())}
    }
}

pub struct EntityTreeMut<'w, 's> {
	tree: Tree<TreeKey, TreeStorageMut<'w, 's>>,
}

impl<'w, 's> EntityTreeMut<'w, 's> {
	/// 为节点插入子节点
	/// 注意，调用此方法的前提条件是，parent的Down组件存在，node的Up组件存在
	pub fn insert_child(&mut self, node: Entity, parent: Entity, index: usize) {
		self.tree.insert_child(TreeKey(node), TreeKey(parent), index);
	}

	/// 为节点添加兄弟节点
	/// 注意，调用此方法的前提条件是，node和anchor的Up组件存在
	pub fn insert_brother(&mut self, node: Entity, anchor: Entity, ty: InsertType) {
		self.tree.insert_brother(TreeKey(node), TreeKey(anchor), ty);
	}

	/// 移除节点
	pub fn remove(&mut self, node: Entity) {
		self.tree.remove(TreeKey(node));
	}
}

#[derive(SystemParam)]
pub struct TreeStorageMut<'w, 's> {
	layer_query: Query<'w, 's, &'static mut Layer>,
	up_query: Query<'w, 's, &'static mut Up>,
	down_query: Query<'w, 's, &'static mut Down>,
	command: Commands<'w, 's>, // 用于插入Root组件
	layer_notify: EventWriter<'w, ComponentEvent<Changed<Layer>>>, // 用于通知Layer修改
}

unsafe impl bevy_ecs::system::SystemParam for EntityTreeMut<'_, '_> {
	type State = (
		<Query<'static, 'static, &'static mut Layer> as bevy_ecs::system::SystemParam>::State,
		<Query<'static, 'static, &'static mut Up> as bevy_ecs::system::SystemParam>::State,
		<Query<'static, 'static, &'static mut Down> as bevy_ecs::system::SystemParam>::State,
		<Commands<'static, 'static> as bevy_ecs::system::SystemParam>::State,
		<EventWriter <'static, ComponentEvent<Changed<Layer>> > as bevy_ecs::system::SystemParam>::State,
	);
	type Item<'world, 'state> = EntityTreeMut<'world, 'state>;
	// type Fetch = FetchState<(
	//     <Query<'w, 's, &'static mut Layer> as bevy_ecs::system::SystemParam>::Fetch,
	//     <Query<'w, 's, &'static mut Up> as bevy_ecs::system::SystemParam>::Fetch,
	//     <Query<'w, 's, &'static mut Down> as bevy_ecs::system::SystemParam>::Fetch,
	//     <Commands<'w, 's> as bevy_ecs::system::SystemParam>::Fetch,
	// 	<EventWriter <'w, ComponentEvent<Changed<Layer>> > as bevy_ecs::system::SystemParam>::Fetch,
	// )>;

	fn init_state(world: &mut bevy_ecs::prelude::World, system_meta: &mut bevy_ecs::system::SystemMeta) -> Self::State {
		(
			<Query<'static, 'static, &'static mut Layer> as bevy_ecs::system::SystemParam>::init_state(world, system_meta),
			<Query<'static, 'static, &'static mut Up> as bevy_ecs::system::SystemParam>::init_state(world, system_meta),
			<Query<'static, 'static, &'static mut Down> as bevy_ecs::system::SystemParam>::init_state(world, system_meta),
			<Commands<'static, 'static> as bevy_ecs::system::SystemParam>::init_state(world, system_meta),
			<EventWriter <'static, ComponentEvent<Changed<Layer>> > as bevy_ecs::system::SystemParam>::init_state(world, system_meta),
		)
	}
	fn new_archetype(state: &mut Self::State, archetype: &Archetype, _system_meta: &mut SystemMeta) {
		state.0.new_archetype(archetype);
		state.1.new_archetype(archetype);
		state.2.new_archetype(archetype);
	}

	fn apply(state: &mut Self::State, system_meta: &SystemMeta, world: &mut World) {
		<Query<'static, 'static, &'static mut Layer> as bevy_ecs::system::SystemParam>::apply(&mut state.0, system_meta, world);
		<Query<'static, 'static, &'static mut Up> as bevy_ecs::system::SystemParam>::apply(&mut state.1, system_meta, world);
		<Query<'static, 'static, &'static mut Down> as bevy_ecs::system::SystemParam>::apply(&mut state.2, system_meta, world);
		<Commands<'static, 'static> as bevy_ecs::system::SystemParam>::apply(&mut state.3, system_meta, world);
		<EventWriter <'static, ComponentEvent<Changed<Layer>> > as bevy_ecs::system::SystemParam>::apply(&mut state.4, system_meta, world);
	}

	unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        change_tick: Tick,
    ) -> Self::Item<'w, 's> {
		EntityTreeMut{
			tree: Tree::new(
				TreeStorageMut { 
					layer_query : <Query < 'w, 's , &'static mut Layer > as SystemParam>::get_param (&mut state.0, system_meta, world, change_tick), 
					up_query : <Query<'w, 's, &'static mut Up > as SystemParam> :: get_param (&mut state.1, system_meta, world, change_tick),
					down_query : <Query<'w, 's ,& 'static mut Down> as SystemParam >:: get_param (& mut state.2 , system_meta, world, change_tick), 
					command : <Commands<'w, 's> as SystemParam>::get_param (& mut state. 3 , system_meta, world, change_tick) , 
					layer_notify: <EventWriter <'w, ComponentEvent<Changed<Layer>>> as SystemParam>::get_param(&mut state.4, system_meta, world, change_tick)}
				)
		}
	}
}

impl<'w, 's> EntityTreeMut<'w, 's> {
	pub fn get_up(&self, k: Entity) -> Option<&Up> {
		unsafe{transmute(self.tree.get_up(TreeKey(k)))}
	}
	pub fn up(&self, k: Entity) -> &Up {
		unsafe{transmute(self.tree.up(TreeKey(k)))}
	}

	pub fn get_layer(&self, k: Entity) -> Option<&Layer> {
		unsafe{transmute(self.tree.get_layer(TreeKey(k)))}
	}
	pub fn layer(&self, k: Entity) -> &Layer{
		unsafe{transmute(self.tree.layer(TreeKey(k)))}
	}

	pub fn get_down(&self, k: Entity) -> Option<&Down> {
		unsafe{transmute(self.tree.get_down(TreeKey(k)))}
	}

	pub fn down(&self, k: Entity) -> &Down {
		unsafe{transmute(self.tree.down(TreeKey(k)))}
	}

	pub fn iter(&self, node_children_head: Entity) -> ChildrenIterator<TreeStorageMut<'w, 's>> {
		ChildrenIterator {
			inner: self.tree.iter(TreeKey(node_children_head))
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<TreeStorageMut<'w, 's>> {
		RecursiveIterator{inner:self.tree.recursive_iter(TreeKey(node_children_head))}
	}
}

impl<'w, 's> Storage<TreeKey> for TreeStorageMut<'w, 's> {
	fn get_up(&self, k: TreeKey) -> Option<&Up1<TreeKey>> {
		unsafe{transmute(match self.up_query.get(k.0) {
			Ok(r) => Some(r),
			_ => None,
		})}
	}
	fn up(&self, k: TreeKey) -> &Up1<TreeKey> {
		unsafe{transmute(self.up_query.get(k.0).unwrap())}
	}

	fn get_layer(&self, k: TreeKey) -> Option<&Layer1<TreeKey>> {
		unsafe{transmute(match self.layer_query.get( k.0) {
			Ok(r) => Some(r),
			_ => None,
		})}
	}
	fn layer(&self, k: TreeKey) -> &Layer1<TreeKey> {
		unsafe{transmute(self.layer_query.get(k.0).unwrap())}
	}

	fn get_down(&self, k: TreeKey) -> Option<&Down1<TreeKey>> {
		unsafe{transmute(match self.down_query.get(k.0) {
			Ok(r) => Some(r),
			_ => None,
		})}
	}
	fn down(&self, k: TreeKey) -> &Down1<TreeKey> {
		unsafe{transmute(self.down_query.get(k.0).unwrap())}
	}
}

impl<'w, 's> StorageMut<TreeKey> for TreeStorageMut<'w, 's> {
	fn get_up_mut(&mut self, k: TreeKey) -> Option<&mut Up1<TreeKey>> {
		match self.up_query.get_mut(k.0) {
			Ok(r) => Some(r.into_inner()),
			_ => None,
		}
	}
	fn up_mut(&mut self, k: TreeKey) -> &mut Up1<TreeKey> {
		self.up_query.get_mut(k.0).unwrap().into_inner()
	}

	fn set_up(&mut self, k: TreeKey, up: Up1<TreeKey>) {
		if let Ok(mut write) = self.up_query.get_mut(k.0) {
			*write = Up(up);
		}
	}

	fn remove_up(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.up_query.get_mut(k.0) {
			*write = Up(Up1::default());
		}
	}

	fn set_layer(&mut self, k: TreeKey, layer: Layer1<TreeKey>) {
		if let Ok(mut write) = self.layer_query.get_mut(k.0) {
			*write = Layer(layer);
			self.layer_notify.send(ComponentEvent::new(k.0));
		}
	}
	
	fn remove_layer(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.layer_query.get_mut(k.0) {
			*write = Layer(Layer1::default());
		}
	}

	fn get_down_mut(&mut self, k: TreeKey) -> Option<&mut Down1<TreeKey>> {
		match self.down_query.get_mut(k.0) {
			Ok(r) => Some(r.into_inner()),
			_ => None,
		}
	}

	fn down_mut(&mut self, k: TreeKey) -> &mut Down1<TreeKey> {
		self.down_query.get_mut(k.0).unwrap().into_inner()
	}

	fn set_down(&mut self, k: TreeKey, down: Down1<TreeKey>) {
		if let Ok(mut write) = self.down_query.get_mut(k.0) {
			*write = Down(down);
		}
	}

	fn remove_down(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.down_query.get_mut(k.0) {
			*write = Down(Down1::default());
		}
	}

	// 通知， TODO
	fn set_root(&mut self, k: TreeKey) {
		self.command.entity(k.0).insert(Root);
	}

	fn remove_root(&mut self, k: TreeKey) {
		self.command.entity(k.0).remove::<Root>();
	}
}

// // #[derive(Deref)]
// // pub struct EntityTree<'s, A: ArchetypeIdent>(Tree<Id, &'s IdtreeState>);

// // // impl<A: ArchetypeIdent> Clone for EntityTree {
// // // 	fn clone(&self) -> Self {
// // // 		Self(Tree::new(IdtreeState(self.0.get_storage().0.clone())))
// // // 	}
// // // }

