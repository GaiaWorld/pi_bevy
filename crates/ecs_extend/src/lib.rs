#![feature(prelude_import)]
#![feature(min_specialization)]
#![feature(associated_type_bounds)]
#![feature(associated_type_defaults)]
#![feature(return_position_impl_trait_in_trait)]

// #[macro_use]
// extern crate serde;
// #[macro_use]
// extern crate derive_deref;
// #[macro_use]
// extern crate pi_ecs_macros;

// use bevy_ecs::prelude::{World, Resource};
// use bevy_app::prelude::App;
use derive_deref::Deref;
use pi_world::{world::World, prelude::App};

pub mod system_param;
// pub mod query;
// pub mod async_system;
// pub mod dispatch;
pub mod action;


pub mod prelude {
    pub use crate::{
        system_param::{
			tree::{Layer, Down, Up, EntityTreeMut, EntityTree, Root},
			// layer_dirty::LayerDirty
		},
		// query::or_default::{OrDefault, DefaultComponent},
		
    };
}

pub trait TShell {
	fn world(&self) -> &World;
	fn world_mut(&mut self) -> &mut World;
	fn app(&self) -> &App;
	fn app_mut(&mut self) -> &mut App;
}

#[derive(Debug, Deref, Default)]
pub struct IsNotRun(pub bool);