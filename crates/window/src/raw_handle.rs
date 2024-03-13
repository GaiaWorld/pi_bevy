
use std::sync::Arc;

use bevy_ecs::prelude::Component;

pub trait CreateSurface: 'static + Send + Sync{
	fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface<'static>;
}


#[derive(Clone, Component)]
pub struct HandleWrapper {
	pub handle: Arc<dyn CreateSurface>,
    // pub window_handle: WindowHandle,
    // pub display_handle: RawDisplayHandle,
}

