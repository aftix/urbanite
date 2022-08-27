use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

mod game;
mod mainmenu;

// Plugin for the entire game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Urbanite;

// Different states of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    Game,
}

// Marker component for the Ui root node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct UiRoot;

// Marker component for the player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct PlayerTag;

impl Plugin for Urbanite {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_plugin(PhysicsPlugin::default())
            .add_loopless_state(GameState::MainMenu)
            .add_plugin(mainmenu::MainMenu)
            .add_plugin(game::Game);
    }
}

fn teardown<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands, _asst_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PlayerTag);
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(UiRoot);
}

// Run the game
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Urbanite)
        .run();
}
