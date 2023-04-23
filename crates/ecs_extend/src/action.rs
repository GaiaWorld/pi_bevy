use std::mem::replace;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct ActionList<T: Send + Sync + 'static>(Vec<T>);
impl<T: Send + Sync> Default for ActionList<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
impl<T: Send + Sync> ActionList<T> {
    pub fn push(&mut self, val: T) {
        self.0.push(val);
    }
    pub fn drain(&mut self) -> Vec<T> {
        replace(&mut self.0, vec![])
    }
}