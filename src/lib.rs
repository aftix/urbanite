use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

mod mainmenu;

// Plugin for the entire game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Urbanite;

// Different states of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
}

// Marker component for the Ui root node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct UiRoot;

impl Plugin for Urbanite {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_plugin(PhysicsPlugin::default())
            .add_loopless_state(GameState::MainMenu)
            .add_plugin(mainmenu::MainMenu);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asst_server: Res<AssetServer>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
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
