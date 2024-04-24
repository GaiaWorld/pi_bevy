use std::{any::TypeId, borrow::Cow, mem::transmute};

use pi_world::{archetype::Flags, prelude::{SingleRes, SingleResMut}, system::SystemMeta, system_params::SystemParam, world::{FromWorld, SingleResource, World}};
use derive_deref::{DerefMut, Deref};

#[derive(Debug, Deref)]
pub struct OrInitSingleRes<'w, T: FromWorld + 'static + Sync + Send>(SingleRes<'w, T>);

impl<T: FromWorld + 'static + Sync + Send> SystemParam for OrInitSingleRes<'_, T> {
    type State = SingleResource;
	type Item<'world> = OrInitSingleRes<'world, T>;

	#[inline(never)]
	fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        if world.get_single_res::<T>().is_none() {
            let v = T::from_world(world);
            world.register_single_res(v);
        }
        SingleRes::<T>::init_state(world, system_meta)
    }
    
    #[inline]
    fn res_depend(
        world: &World,
        system_meta: &SystemMeta,
        state: &Self::State,
        res_tid: &TypeId,
        res_name: &Cow<'static, str>,
        single: bool,
        result: &mut Flags,
    ) {
        SingleRes::<T>::res_depend(world, system_meta, state, res_tid, res_name, single, result);
    }

    #[inline]
    fn get_param<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
    ) -> Self::Item<'world> {
        OrInitSingleRes(SingleRes::get_param(world, system_meta, state))
    }
    #[inline]
    fn get_self<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
    ) -> Self {
        unsafe { transmute(Self::get_param(world, system_meta, state)) }
    }
}


#[derive(Debug, Deref, DerefMut)]
pub struct OrInitSingleResMut<'w, T: FromWorld + 'static + Sync + Send>(SingleResMut<'w, T>);

impl<T: FromWorld + 'static + Sync + Send> SystemParam for OrInitSingleResMut<'_, T> {
    type State = SingleResource;
	type Item<'world> = OrInitSingleResMut<'world, T>;

	#[inline(never)]
	fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        if world.get_single_res::<T>().is_none() {
            let v = T::from_world(world);
            world.register_single_res(v);
        }
        SingleResMut::<T>::init_state(world, system_meta)
    }
    
    #[inline]
    fn res_depend(
        world: &World,
        system_meta: &SystemMeta,
        state: &Self::State,
        res_tid: &TypeId,
        res_name: &Cow<'static, str>,
        single: bool,
        result: &mut Flags,
    ) {
        SingleResMut::<T>::res_depend(world, system_meta, state, res_tid, res_name, single, result);
    }

    #[inline]
    fn get_param<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
    ) -> Self::Item<'world> {
        OrInitSingleResMut(SingleResMut::<T>::get_param(world, system_meta, state))
    }
    #[inline]
    fn get_self<'world>(
        world: &'world World,
        system_meta: &'world SystemMeta,
        state: &'world mut Self::State,
    ) -> Self {
        unsafe { transmute(Self::get_param(world, system_meta, state)) }
    }
}

