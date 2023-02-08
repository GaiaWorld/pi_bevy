use bevy::ecs::{world::{FromWorld, World}, system::{Resource, ResState, ResMutState, SystemParamState, SystemMeta, ReadOnlySystemParamFetch, SystemParam, Res, SystemParamFetch, ResMut}};
use derive_deref::{DerefMut, Deref};

#[derive(Debug, Deref, DerefMut)]
pub struct OrInitRes<'w, T: FromWorld + Resource>(pub Res<'w, T>);

impl<'w, T: Resource + FromWorld> SystemParam for OrInitRes<'w, T> {
    type Fetch = OrInitResState<T>;
}

#[derive(Debug, Deref, DerefMut)]
pub struct OrInitResMut<'w, T: FromWorld + Resource>(pub ResMut<'w, T>);

impl<'w, T: Resource + FromWorld> SystemParam for OrInitResMut<'w, T> {
    type Fetch = OrInitResMutState<T>;
}

pub struct OrInitResState<T: FromWorld + Resource>(ResState<T>);

unsafe impl<T: FromWorld + Resource> SystemParamState for OrInitResState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		if world.get_resource::<T>().is_none() {
			let value = T::from_world(world);
			world.insert_resource(value);
		}
		Self(ResState::init(world, system_meta))
	}
}

unsafe impl<T: Resource + FromWorld> ReadOnlySystemParamFetch for OrInitResState<T> {}

impl<'w, 's, T: Resource + FromWorld> SystemParamFetch<'w, 's> for OrInitResState<T> {
    type Item = OrInitRes<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        OrInitRes(ResState::<T>::get_param(&mut state.0, system_meta, world, change_tick))
    }
}

pub struct OrInitResMutState<T: FromWorld + Resource>(ResMutState<T>);

unsafe impl<T: FromWorld + Resource> SystemParamState for OrInitResMutState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		if world.get_resource::<T>().is_none() {
			let value = T::from_world(world);
			world.insert_resource(value);
		}
		Self(ResMutState::init(world, system_meta))
	}
}

impl<'w, 's, T: Resource + FromWorld> SystemParamFetch<'w, 's> for OrInitResMutState<T> {
    type Item = OrInitResMut<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        OrInitResMut(ResMutState::<T>::get_param(&mut state.0, system_meta, world, change_tick))
    }
}

