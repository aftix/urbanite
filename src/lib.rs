use bevy::prelude::*;

pub struct Urbanite;

impl Plugin for Urbanite {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

pub fn setup(mut commands: Commands, asst_server: Res<AssetServer>) {}
