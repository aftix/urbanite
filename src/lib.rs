use bevy::prelude::*;

mod generate_world;
mod mainmenu;

// Plugin for the entire game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Urbanite;

// Different states of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource, States, Default)]
enum GameState {
    #[default]
    MainMenu,
    WorldGenerate,
}

// Marker component for the Ui root node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct UiRoot;

// Ui font resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
struct UiFont(Handle<Font>);

// Marker component for the player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct PlayerTag;

impl Plugin for Urbanite {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_state::<GameState>()
            .add_plugins((generate_world::WorldGenerate, mainmenu::MainMenu));
    }
}

fn teardown<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands, asst_server: Res<AssetServer>) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PlayerTag);
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(UiRoot);

    commands.insert_resource(UiFont(
        asst_server.load("fonts/mechanical-font/Mechanical-g5Y5.otf"),
    ));
}

// Run the game
pub fn run() {
    App::new().add_plugins((DefaultPlugins, Urbanite)).run();
}
