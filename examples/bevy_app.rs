use bevy::prelude::*;
use pi_render_bevy::PiRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PiRenderPlugin)
        .run();
}
