use bevy::prelude::*;
use urbanite::Urbanite;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Urbanite)
        .run();
}
