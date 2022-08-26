use bevy::prelude::*;

fn hello_world() {
    println!("Hello World!");
}

fn main() {
    App::new()
        .add_startup_system(hello_world)
        .add_plugins(DefaultPlugins)
        .run();
}
