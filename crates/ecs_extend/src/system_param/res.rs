use bevy::ecs::{world::{FromWorld, World}, system::{Resource, SystemMeta, ReadOnlySystemParam, SystemParam, Res, ResMut}, component::ComponentId};
use derive_deref::{DerefMut, Deref};

#[derive(Debug, Deref, DerefMut)]
pub struct OrInitRes<'w, T: FromWorld + Resource>(pub Res<'w, T>);

unsafe impl<T: Resource + FromWorld> SystemParam for OrInitRes<'_, T> {
    type State = ComponentId;
	type Item<'world, 'state> = OrInitRes<'world, T>;

	fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
		world.init_resource::<T>();
        Res::<T>::init_state(world, system_meta)
    }

	#[inline]
    unsafe fn get_param<'w, 's>(
        component_id: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item<'w, 's> {
		OrInitRes(Res::<T>::get_param(component_id, system_meta, world, change_tick))
    }
}

unsafe impl<T: Resource + FromWorld> ReadOnlySystemParam for OrInitRes<'_, T> {}

#[derive(Debug, Deref, DerefMut)]
pub struct OrInitResMut<'w, T: FromWorld + Resource>(pub ResMut<'w, T>);

unsafe impl<T: Resource + FromWorld> SystemParam for OrInitResMut<'_, T> {
    type State = ComponentId;
	type Item<'world, 'state> = OrInitResMut<'world, T>;

	fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
		world.init_resource::<T>();
        ResMut::<T>::init_state(world, system_meta)
    }

	#[inline]
    unsafe fn get_param<'w, 's>(
        component_id: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item<'w, 's> {
		OrInitResMut(ResMut::<T>::get_param(component_id, system_meta, world, change_tick))
    }
}

