use crate::{GameState, UiRoot};
use bevy::{app::AppExit, prelude::*};
use iyes_loopless::prelude::*;

// Marker component for MainMenu UI items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct MainMenuUi;

// Marker component for MainMenu selected item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Selected;

// Components for navigating the menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Next(Entity);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Previous(Entity);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChangeEvent(Entity);

// Enable a menu entity to quit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct QuitTag;

// Enable a menu entity to enter game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct PlayTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::MainMenu, menu_startup)
            .add_event::<ChangeEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(bevy::window::close_on_esc)
                    .with_system(change_color)
                    .with_system(change_color_removed)
                    .with_system(move_selection)
                    .with_system(selection_quit)
                    .with_system(selection_play)
                    .into(),
            )
            .add_exit_system(GameState::MainMenu, menu_teardown);
    }
}

fn move_selection(
    mut commands: Commands,
    mut keys: ResMut<Input<KeyCode>>,
    q: Query<(Entity, &Next, &Previous), With<Selected>>,
    mut ev_writer: EventWriter<ChangeEvent>,
) {
    let (entity, next, prev) = q.single();
    if keys.just_pressed(KeyCode::Up) {
        commands.entity(entity).remove::<Selected>();
        commands.entity(prev.0).insert(Selected);
        ev_writer.send(ChangeEvent(entity));
        keys.clear();
    } else if keys.just_pressed(KeyCode::Down) {
        commands.entity(entity).remove::<Selected>();
        commands.entity(next.0).insert(Selected);
        ev_writer.send(ChangeEvent(entity));
        keys.clear();
    }
}

fn selection_quit(
    mut exit: EventWriter<AppExit>,
    keys: Res<Input<KeyCode>>,
    q: Query<(), (With<Selected>, With<QuitTag>)>,
) {
    q.for_each(move |_| {
        if keys.just_pressed(KeyCode::Return) {
            exit.send(AppExit);
        }
    });
}

fn selection_play(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    q: Query<(), (With<Selected>, With<PlayTag>)>,
) {
    q.for_each(move |_| {
        if keys.just_pressed(KeyCode::Return) {
            commands.insert_resource(NextState(GameState::Game));
        }
    });
}

fn change_color(mut q: Query<(&mut Text, Added<Selected>)>) {
    let (mut text, _) = q.single_mut();
    for section in &mut text.sections {
        section.style.color = Color::RED;
    }
}

fn change_color_removed(
    mut ev_moved: EventReader<ChangeEvent>,
    mut q: Query<(Entity, &mut Text), Without<Selected>>,
) {
    for changed in ev_moved.iter() {
        for (entity, mut text) in &mut q {
            if changed.0 == entity {
                for section in &mut text.sections {
                    section.style.color = Color::WHITE;
                }
            }
        }
    }
}

fn menu_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asst_server: Res<AssetServer>,
    q: Query<(Entity, &UiRoot)>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(MainMenuUi);
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(MainMenuUi);
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

    let start = commands
        .spawn_bundle(
            TextBundle::from_section(
                "Start",
                TextStyle {
                    color: Color::WHITE,
                    font: asst_server.load("fonts/mechanical-font/Mechanical-g5Y5.otf"),
                    font_size: 24.0,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(30.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MainMenuUi)
        .insert(Selected)
        .insert(PlayTag)
        .id();

    let quit = commands
        .spawn_bundle(
            TextBundle::from_section(
                "Quit",
                TextStyle {
                    color: Color::WHITE,
                    font: asst_server.load("fonts/mechanical-font/Mechanical-g5Y5.otf"),
                    font_size: 24.0,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                position: UiRect {
                    top: Val::Px(35.0),
                    left: Val::Px(30.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(QuitTag)
        .insert(MainMenuUi)
        .insert(Next(start))
        .insert(Previous(start))
        .id();

    commands
        .entity(start)
        .insert(Next(quit))
        .insert(Previous(quit));

    commands.entity(uiroot).push_children(&[text, start, quit]);
}

fn menu_teardown(mut commands: Commands, q: Query<Entity, With<MainMenuUi>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}
