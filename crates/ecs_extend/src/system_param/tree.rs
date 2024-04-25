//! 实体树

use std::{any::TypeId, borrow::Cow, mem::transmute};

use derive_deref::{Deref, DerefMut};

use pi_null::Null;
use pi_slotmap_tree::{
    ChildrenIterator as ChildrenIterator1, Down as Down1, InsertType, Layer as Layer1, RecursiveIterator as RecursiveIterator1, Storage, StorageMut, Tree, Up as Up1
};
use serde::{Deserialize, Serialize};

use pi_world::{archetype::Flags, prelude::{Entity, Query, SystemParam, World, Alter}, system::SystemMeta};

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

#[derive(SystemParam)]
pub struct EntityTree<'w> {
	layer_query: Query<'w, &'static Layer>,
	up_query: Query<'w, &'static Up>,
	down_query: Query<'w, &'static Down>,
}

impl<'w> Storage<TreeKey> for EntityTree<'w> {
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

impl<'w> EntityTree<'w> {
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

	pub fn iter(&self, node_children_head: Entity) -> ChildrenIterator<EntityTree<'w>> {
		ChildrenIterator {
			inner: ChildrenIterator1::new(self, TreeKey(node_children_head))
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<EntityTree<'w>> {
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

pub struct EntityTreeMut<'w> {
	tree: Tree<TreeKey, TreeStorageMut<'w>>,
}

impl<'w> EntityTreeMut<'w> {
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

// #[derive(SystemParam)]
pub struct TreeStorageMut<'w> {
	layer_query: Query<'w, &'static mut Layer>,
	up_query: Query<'w, &'static mut Up>,
	down_query: Query<'w, &'static mut Down>,
	root_alter: Alter<'w, (), (), (Root,), (Root,)>, // 用于插入Root组件
}

impl pi_world::system_params::SystemParam for EntityTreeMut<'_> {
	type State = (
        <Query<'static, &'static mut  Layer> as SystemParam>::State,
		<Query<'static, &'static mut  Up> as SystemParam>::State,
		<Query<'static, &'static mut Down> as SystemParam>::State,
        <Alter<'static, (), (), (Root,), (Root,)> as SystemParam>::State,
    );

    type Item<'world> = EntityTreeMut<'world>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        (
			<Query< 'static, &'static mut  Layer> as SystemParam>::init_state(world, system_meta),
			<Query<'static, &'static mut  Up> as SystemParam>::init_state(world, system_meta),
			<Query<'static, &'static mut  Down> as SystemParam>::init_state(world, system_meta),
            <Alter<'static, (), (), (Root,), (Root,)> as SystemParam>::init_state(world, system_meta),
		)
    }

    fn get_param<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
    ) -> Self::Item<'world> {
        EntityTreeMut{
            tree: Tree::new(
                TreeStorageMut {
                    layer_query: <Query< 'static, &'static mut Layer> as SystemParam>::get_param(world, system_meta, &mut state.0),
                    up_query:  <Query< 'static, &'static mut Up> as SystemParam>::get_param(world, system_meta, &mut state.1),
                    down_query:  <Query< 'static, &'static mut Down> as SystemParam>::get_param(world, system_meta, &mut state.2),
                    root_alter: <Alter<'static, (), (), (Root,), (Root,)> as SystemParam>::get_param(world, system_meta, &mut state.3), // 用于插入Root组件
                }
            )
        }
    }

    #[inline]
    #[allow(unused_variables)]
    fn res_depend(
        world: &World,
        system_meta: &SystemMeta,
        state: &Self::State,
        res_tid: &TypeId,
        res_name: &Cow<'static, str>,
        single: bool,
        result: &mut Flags,
    ) {
        <Query< 'static, &'static mut  Layer> as SystemParam>::res_depend(world, system_meta, &state.0, res_tid, res_name, single, result);
        <Query<'static, &'static mut  Up> as SystemParam>::res_depend(world, system_meta, &state.1, res_tid, res_name, single, result);
        <Query<'static, &'static mut  Down> as SystemParam>::res_depend(world, system_meta, &state.2, res_tid, res_name, single, result);
        <Alter<'static, (), (), (Root,)> as SystemParam>::res_depend(world, system_meta, &state.3, res_tid, res_name, single, result);
    }


    fn get_self<'world>(
        world: &'world pi_world::world::World,
        system_meta: &'world pi_world::system::SystemMeta,
        state: &'world mut Self::State,
    ) -> Self {
        unsafe { transmute(Self::get_param(world, system_meta, state)) }
    }
}

impl<'w> EntityTreeMut<'w> {
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

	pub fn iter(&self, node_children_head: Entity) -> ChildrenIterator<TreeStorageMut<'w>> {
		ChildrenIterator {
			inner: self.tree.iter(TreeKey(node_children_head))
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<TreeStorageMut<'w>> {
		RecursiveIterator{inner:self.tree.recursive_iter(TreeKey(node_children_head))}
	}
}

impl<'w> Storage<TreeKey> for TreeStorageMut<'w> {
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

impl<'w> StorageMut<TreeKey> for TreeStorageMut<'w> {
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
        let _ = self.root_alter.alter(k.0, (Root, ));
	}

	fn remove_root(&mut self, k: TreeKey) {
        let _ = self.root_alter.delete(k.0);
	}
}




