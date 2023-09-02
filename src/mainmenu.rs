use crate::{GameState, UiRoot};
use bevy::{app::AppExit, prelude::*};

// Marker component for MainMenu UI items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct MainMenuTag;

// Marker component for MainMenu selected item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Selected;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Cube;

// Components for navigating the menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Next(Entity);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Previous(Entity);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
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
        app.add_systems(OnEnter(GameState::MainMenu), menu_startup)
            .add_systems(OnExit(GameState::MainMenu), crate::teardown::<MainMenuTag>)
            .add_event::<ChangeEvent>()
            .add_systems(
                Update,
                (
                    bevy::window::close_on_esc,
                    change_color,
                    change_color_removed,
                    move_selection,
                    selection_quit,
                    selection_play,
                    rotate_cube,
                )
                    .run_if(in_state(GameState::MainMenu)),
            );
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
            commands.insert_resource(NextState(Some(GameState::WorldGenerate)));
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

fn rotate_cube(t: Res<Time>, mut q: Query<&mut Transform, With<Cube>>) {
    let speed = 0.1;
    let mut transform = q.single_mut();
    let axis = transform.up();
    transform.rotate_axis(axis, speed * std::f32::consts::TAU * t.delta_seconds());
}

fn menu_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    uifont: Res<crate::UiFont>,
    q: Query<(Entity, &UiRoot)>,
    mut camera_q: Query<&mut Transform, With<crate::PlayerTag>>,
) {
    let mut transform = camera_q.single_mut();
    *transform = Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Z);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(MainMenuTag)
        .insert(Cube);

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(MainMenuTag);
    let (uiroot, _) = q.single();
    let text = commands
        .spawn(
            TextBundle::from_section(
                "Urbanite",
                TextStyle {
                    color: Color::WHITE,
                    font: uifont.0.clone(),
                    font_size: 100.0,
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                bottom: Val::Px(5.0),
                right: Val::Px(30.0),
                ..default()
            }),
        )
        .insert(MainMenuTag)
        .id();

    let start = commands
        .spawn(
            TextBundle::from_section(
                "Generate World",
                TextStyle {
                    color: Color::WHITE,
                    font: uifont.0.clone(),
                    font_size: 24.0,
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                top: Val::Px(5.0),
                left: Val::Px(30.0),
                ..default()
            }),
        )
        .insert(MainMenuTag)
        .insert(Selected)
        .insert(PlayTag)
        .id();

    let quit = commands
        .spawn(
            TextBundle::from_section(
                "Quit",
                TextStyle {
                    color: Color::WHITE,
                    font: uifont.0.clone(),
                    font_size: 24.0,
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                top: Val::Px(35.0),
                left: Val::Px(30.0),
                ..default()
            }),
        )
        .insert(QuitTag)
        .insert(MainMenuTag)
        .insert(Next(start))
        .insert(Previous(start))
        .id();

    commands
        .entity(start)
        .insert(Next(quit))
        .insert(Previous(quit));

    commands.entity(uiroot).push_children(&[text, start, quit]);
}
