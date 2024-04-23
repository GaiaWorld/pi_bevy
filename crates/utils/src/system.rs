use std::ops::{Deref, DerefMut};

use pi_world::{system::SystemMeta, world::World};

pub trait SystemBuffer:  Send + 'static {
    /// Applies any deferred mutations to the [`World`].
    fn apply(&mut self, system_meta: &SystemMeta, world: &mut World);
}

pub struct Deferred<'a, T: SystemBuffer>(pub &'a mut T);

impl<'a, T: SystemBuffer> Deref for Deferred<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: SystemBuffer> DerefMut for Deferred<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}