use bevy::prelude::*;
use pi_bevy_plugin::PiRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PiRenderPlugin)
        .run();
}
