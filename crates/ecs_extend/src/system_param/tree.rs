//! 实体树

use std::mem::transmute;

use derive_deref::{Deref, DerefMut};

use pi_null::Null;
use pi_slotmap_tree::{
    ChildrenIterator as ChildrenIterator1, Down as Down1, InsertType, Layer as Layer1, RecursiveIterator as RecursiveIterator1, Storage, StorageMut, Tree, Up as Up1
};
use serde::{Deserialize, Serialize};

use pi_world::{insert::Component, param_set::ParamSet, prelude::{Alter, Entity, Query, SystemParam, World}, system::SystemMeta, world::Tick};

// use pi_print_any::{println_any, out_any};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Component)]
pub struct Root;

pub type TreeKey = Entity;

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize, Component)]
pub struct Layer(Layer1<TreeKey>);
impl Layer  {
	#[inline]
	pub fn layer(&self) -> usize{
		self.0.layer()
	}
	#[inline]
	pub fn root(&self) -> Entity {
		self.0.root()
	}
}

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize, Component)]
pub struct Up(Up1<TreeKey>);
impl Up  {
	#[inline]
	pub fn parent(&self) -> Entity {
		self.0.parent()
	}
	#[inline]
	pub fn prev(&self) -> Entity {
		self.0.prev()
	}
	#[inline]
	pub fn next(&self) -> Entity {
		self.0.next()
	}
}

#[derive(Debug, Clone, Default, Deref, DerefMut, Serialize, Deserialize, Component)]
pub struct Down(Down1<TreeKey>);
impl Down  {
	#[inline]
	pub fn head(&self) -> Entity {
		self.0.head()
	}
	#[inline]
	pub fn tail(&self) -> Entity {
		self.0.tail()
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
		self.get_up(k).map(|r|{&**r})
	}
	fn up(&self, k: TreeKey) -> &Up1<TreeKey> {
		self.up(k)
	}

	fn get_layer(&self, k: TreeKey) -> Option<&Layer1<TreeKey>> {
		self.get_layer(k).map(|r|{&**r})
	}
	fn layer(&self, k: TreeKey) -> &Layer1<TreeKey> {
		self.layer(k)
	}

	fn get_down(&self, k: TreeKey) -> Option<&Down1<TreeKey>> {
		self.get_down(k).map(|r|{&**r})
	}

	fn down(&self, k: TreeKey) -> &Down1<TreeKey> {
		self.down(k)
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
			inner: ChildrenIterator1::new(self, node_children_head)
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<EntityTree<'w>> {
		let len = if node_children_head.is_null() {
			0
		} else {
			1
		};
		RecursiveIterator{inner:RecursiveIterator1::new(self, node_children_head, len)}
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
		self.tree.insert_child(node, parent, index);
	}

	/// 为节点添加兄弟节点
	/// 注意，调用此方法的前提条件是，node和anchor的Up组件存在
	pub fn insert_brother(&mut self, node: Entity, anchor: Entity, ty: InsertType) {
		self.tree.insert_brother(node, anchor, ty);
	}

	/// 移除节点
	pub fn remove(&mut self, node: Entity) {
		self.tree.remove(node);
	}
}

// #[derive(SystemParam)]
pub struct TreeStorageMut<'w> {
	// layer_query: Query<'w, &'static mut Layer>,
	// up_query: Query<'w, &'static mut Up>,
	// down_query: Query<'w, &'static mut Down>,
	root: ParamSet<'w, (
		Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
		Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
	)>,
}


impl pi_world::system_params::SystemParam for EntityTreeMut<'_> {
	type State = (
        // <Query<'static, &'static mut  Layer> as SystemParam>::State,
		// <Query<'static, &'static mut  Up> as SystemParam>::State,
		// <Query<'static, &'static mut Down> as SystemParam>::State,
        <ParamSet<'static, (
			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
		)> as SystemParam>::State,
    );

    type Item<'world> = EntityTreeMut<'world>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        (
			// <Query< 'static, &'static mut  Layer> as SystemParam>::init_state(world, system_meta),
			// <Query<'static, &'static mut  Up> as SystemParam>::init_state(world, system_meta),
			// <Query<'static, &'static mut  Down> as SystemParam>::init_state(world, system_meta),
            <ParamSet<'static, (
				Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
				Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
			)> as SystemParam>::init_state(world, system_meta),
		)
    }

    fn get_param<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
		tick: Tick,
    ) -> Self::Item<'world> {
        EntityTreeMut{
            tree: Tree::new(
                TreeStorageMut {
                    // layer_query: <Query< 'static, &'static mut Layer> as SystemParam>::get_param(world, system_meta, &mut state.0, tick),
                    // up_query:  <Query< 'static, &'static mut Up> as SystemParam>::get_param(world, system_meta, &mut state.1, tick),
                    // down_query:  <Query< 'static, &'static mut Down> as SystemParam>::get_param(world, system_meta, &mut state.2, tick),
					root: <ParamSet<'static, (
						Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
						Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
					)> as SystemParam>::get_param(world, system_meta, &mut state.0, tick)
                }
            )
        }
    }

	// #[inline]
    // fn archetype_depend(
    //     world: &World,
    //     system_meta: &SystemMeta,
    //     state: &Self::State,
    //     archetype: &Archetype,
    //     result: &mut ArchetypeDependResult,
    // ) {
    //     // <Query< 'static, &'static mut  Layer> as SystemParam>::archetype_depend(world, system_meta, &state.0, archetype, result);
    //     // <Query<'static, &'static mut  Up> as SystemParam>::archetype_depend(world, system_meta, &state.1, archetype, result);
    //     // <Query<'static, &'static mut  Down> as SystemParam>::archetype_depend(world, system_meta, &state.2, archetype, result);
    //     <ParamSet<'static, (
	// 		Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
	// 		Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
	// 	)> as SystemParam>::archetype_depend(world, system_meta, &state.0, archetype, result);
    // }

	#[inline]
    #[allow(unused_variables)]
    fn align(world: &World, system_meta: &SystemMeta, state: &mut Self::State) {
		// <Query< 'static, &'static mut  Layer> as SystemParam>::align(world, system_meta, &mut state.0);
        // <Query<'static, &'static mut  Up> as SystemParam>::align(world, system_meta, &mut state.1);
        // <Query<'static, &'static mut  Down> as SystemParam>::align(world, system_meta, &mut state.2);
        <ParamSet<'static, (
			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
		)> as SystemParam>::align(world, system_meta, &mut state.0);
	}


    fn get_self<'world>(
        world: &'world pi_world::world::World,
        system_meta: &'world pi_world::system::SystemMeta,
        state: &'world mut Self::State,
		tick: Tick,
    ) -> Self {
        unsafe { transmute(Self::get_param(world, system_meta, state, tick)) }
    }
}

// impl ParamSetElement for EntityTreeMut<'_> {
//     fn init_set_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
// 		let r = <(
// 			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (Root,), ()>, // 用于插入Root组件
// 			Alter<'static, (&'static mut Layer, &'static mut Up, &'static mut Down), (), (), (Root,)> // 用于删除Root组件
// 		) as ParamSetElement>::init_set_state(world, system_meta);
//         (
// 			// <Query< 'static, &'static mut  Layer> as ParamSetElement>::init_set_state(world, system_meta),
// 			// <Query<'static, &'static mut  Up> as ParamSetElement>::init_set_state(world, system_meta),
// 			// <Query<'static, &'static mut  Down> as ParamSetElement>::init_set_state(world, system_meta),
// 			unsafe {
// 				transmute(r)
// 			}
// 		)
//     }
// }

impl<'w> EntityTreeMut<'w> {
	pub fn get_up(&self, k: Entity) -> Option<&Up> {
		unsafe{transmute(self.tree.get_up(k))}
	}
	pub fn up(&self, k: Entity) -> &Up {
		unsafe{transmute(self.tree.up(k))}
	}

	pub fn get_layer(&self, k: Entity) -> Option<&Layer> {
		unsafe{transmute(self.tree.get_layer(k))}
	}
	pub fn layer(&self, k: Entity) -> &Layer{
		unsafe{transmute(self.tree.layer(k))}
	}

	pub fn get_down(&self, k: Entity) -> Option<&Down> {
		unsafe{transmute(self.tree.get_down(k))}
	}

	pub fn down(&self, k: Entity) -> &Down {
		unsafe{transmute(self.tree.down(k))}
	}

	pub fn iter(&self, node_children_head: Entity) -> ChildrenIterator<TreeStorageMut<'w>> {
		ChildrenIterator {
			inner: self.tree.iter(node_children_head)
		}
	}

	/// 迭代指定节点的所有递归子元素
	pub fn recursive_iter(&self, node_children_head: Entity) -> RecursiveIterator<TreeStorageMut<'w>> {
		RecursiveIterator{inner:self.tree.recursive_iter(node_children_head)}
	}
}

impl<'w> Storage<TreeKey> for TreeStorageMut<'w> {
	fn get_up(&self, k: TreeKey) -> Option<&Up1<TreeKey>> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		match s.root.p0().get(k) {
			Ok(r) => Some(r.1),
			_ => None,
		}
	}
	fn up(&self, k: TreeKey) -> &Up1<TreeKey> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		&s.root.p0().get(k).unwrap().1
	}

	fn get_layer(&self, k: TreeKey) -> Option<&Layer1<TreeKey>> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		match s.root.p0().get(k) {
			Ok(r) => Some(r.0),
			_ => None,
		}
		// unsafe{transmute(match self.layer_query.get( k) {
		// 	Ok(r) => Some(r),
		// 	_ => None,
		// })}
	}
	fn layer(&self, k: TreeKey) -> &Layer1<TreeKey> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		&s.root.p0().get(k).unwrap().0
		// unsafe{transmute(self.layer_query.get(k).unwrap())}
	}

	fn get_down(&self, k: TreeKey) -> Option<&Down1<TreeKey>> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		match s.root.p0().get(k) {
			Ok(r) => Some(r.2),
			_ => None,
		}
		// unsafe{transmute(match self.down_query.get(k) {
		// 	Ok(r) => Some(r),
		// 	_ => None,
		// })}
	}
	fn down(&self, k: TreeKey) -> &Down1<TreeKey> {
		let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
		&s.root.p0().get(k).unwrap().2
		// unsafe{transmute(self.down_query.get(k).unwrap())}
	}
}

impl<'w> StorageMut<TreeKey> for TreeStorageMut<'w> {
	fn get_up_mut(&mut self, k: TreeKey) -> Option<&mut Up1<TreeKey>> {
		match self.root.p0().get_mut(k) {
			Ok(r) => Some(r.1.into_inner()),
			_ => None,
		}
	}
	fn up_mut(&mut self, k: TreeKey) -> &mut Up1<TreeKey> {
		self.root.p0().get_mut(k).unwrap().1.into_inner()
	}

	fn set_up(&mut self, k: TreeKey, up: Up1<TreeKey>) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.1 = Up(up);
		}
	}

	fn remove_up(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.1 = Up(Up1::default());
		}
	}

	fn set_layer(&mut self, k: TreeKey, layer: Layer1<TreeKey>) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.0 = Layer(layer);
		}
	}
	
	fn remove_layer(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.0 = Layer(Layer1::default());
		}
	}

	fn get_down_mut(&mut self, k: TreeKey) -> Option<&mut Down1<TreeKey>> {
		match self.root.p0().get_mut(k) {
			Ok(r) => Some(r.2.into_inner()),
			_ => None,
		}
	}

	fn down_mut(&mut self, k: TreeKey) -> &mut Down1<TreeKey> {
		self.root.p0().get_mut(k).unwrap().2.into_inner()
	}

	fn set_down(&mut self, k: TreeKey, down: Down1<TreeKey>) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.2 = Down(down);
		}
	}

	fn remove_down(&mut self, k: TreeKey) {
		if let Ok(mut write) = self.root.p0().get_mut(k) {
			*write.2 = Down(Down1::default());
		}
	}

	// 通知， TODO
	fn set_root(&mut self, k: TreeKey) {
        let _ = self.root.p0().alter(k, (Root, ));
	}

	fn remove_root(&mut self, k: TreeKey) {
        let _ = self.root.p1().alter(k, ());
	}
}




