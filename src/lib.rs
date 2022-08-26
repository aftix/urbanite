use bevy::prelude::*;
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
            .add_loopless_state(GameState::MainMenu)
            .add_plugin(mainmenu::MainMenu);
    }
}

fn setup(mut commands: Commands, _asst_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera3dBundle::default());
    let mut camera2d = Camera2dBundle::default();
    camera2d.camera.priority = 2;
    commands.spawn_bundle(camera2d);
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
