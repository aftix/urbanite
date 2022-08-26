use bevy::prelude::*;
use iyes_loopless::prelude::*;

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

// Marker component for MainMenu UI items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct MainMenuUi;

impl Plugin for Urbanite {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_loopless_state(GameState::MainMenu)
            .add_enter_system(GameState::MainMenu, menu_startup.after(setup))
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(bevy::window::close_on_esc)
                    .into(),
            );
    }
}

fn setup(mut commands: Commands, asst_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera3dBundle::default());
    commands.spawn_bundle(Camera2dBundle::default());
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

fn menu_startup(
    mut commands: Commands,
    asst_server: Res<AssetServer>,
    q: Query<(Entity, &UiRoot)>,
) {
    let (uiroot, _) = q.single();
    let text = commands
        .spawn_bundle(
            TextBundle::from_section(
                "Urbanite",
                TextStyle {
                    color: Color::WHITE,
                    font: asst_server.load("fonts/mechanical-font/Mechanical-g5Y5.otf"),
                    font_size: 100.0,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(30.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MainMenuUi)
        .id();
    commands.entity(uiroot).push_children(&[text]);
}

// Run the game
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Urbanite)
        .run();
}
