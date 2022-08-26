use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

// Tag for entities belonging to the game state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct GameTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Game, game_startup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(return_on_esc)
                    .into(),
            )
            .add_exit_system(GameState::Game, game_teardown);
    }
}

fn return_on_esc(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn game_teardown(mut commands: Commands, q: Query<Entity, With<GameTag>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}

fn game_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 3.0,
                ..default()
            })),
            material: materials.add(Color::rgb(0.4, 0.1, 0.8).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(GameTag);
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
        .insert(GameTag);
}
