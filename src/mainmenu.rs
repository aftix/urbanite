use crate::{GameState, UiRoot};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

// Marker component for MainMenu UI items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct MainMenuUi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::MainMenu, menu_startup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(bevy::window::close_on_esc)
                    .into(),
            );
    }
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
