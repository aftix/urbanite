use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

// Tag for entities belonging to the game state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct GameTag;

// Tag for orbit camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Orbit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WorldGenerate;

impl Plugin for WorldGenerate {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::WorldGenerate, game_startup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::WorldGenerate)
                    .with_system(return_on_esc)
                    .with_system(movement)
                    .into(),
            )
            .add_exit_system(GameState::WorldGenerate, crate::teardown::<GameTag>)
            .add_exit_system(GameState::WorldGenerate, remove_orbit);
    }
}

fn return_on_esc(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn game_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(Entity, &mut Transform), With<crate::PlayerTag>>,
) {
    // Spawn sphere
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

    // Spawn light
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

    let (player, mut player_transform) = q.single_mut();
    commands.entity(player).insert(Orbit);

    // Set camera transform to be with Z in the up direction, looking at sphere
    *player_transform = Transform::from_xyz(10.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z);
}

// Orbit around the origin, keeping looking at the center, at constant radius
fn movement(t: Res<Time>, keys: Res<Input<KeyCode>>, mut q: Query<&mut Transform, With<Orbit>>) {
    let speed: f32 = 5f32;

    for mut transform in &mut q {
        if keys.pressed(KeyCode::W) {
            let delta = transform.up() * t.delta_seconds() * speed;
            let radius = transform.translation.length();
            let dir = (transform.translation + delta).normalize();

            let angle = delta.length() / (transform.translation + delta).length();
            let new_up = Quat::from_axis_angle(transform.left(), angle).mul_vec3(transform.up());

            *transform = Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, new_up);
        }

        if keys.pressed(KeyCode::S) {
            let delta = transform.down() * t.delta_seconds() * speed;
            let radius = transform.translation.length();
            let dir = (transform.translation + delta).normalize();

            let angle = delta.length() / (transform.translation + delta).length();
            let new_up = Quat::from_axis_angle(transform.right(), angle).mul_vec3(transform.up());

            *transform = Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, new_up);
        }

        if keys.pressed(KeyCode::A) {
            let delta = transform.left() * t.delta_seconds() * speed;
            let radius = transform.translation.length();
            let dir = (transform.translation + delta).normalize();

            *transform =
                Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, transform.up());
        }

        if keys.pressed(KeyCode::D) {
            let delta = transform.right() * t.delta_seconds() * speed;
            let radius = transform.translation.length();
            let dir = (transform.translation + delta).normalize();

            *transform =
                Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, transform.up());
        }

        if keys.pressed(KeyCode::Z) {
            let radius = transform.translation.length();

            if radius >= 6.5 {
                let new_radius = radius - t.delta_seconds() * speed * 2f32;
                *transform =
                    Transform::from_translation(transform.translation.normalize() * new_radius)
                        .looking_at(Vec3::ZERO, transform.up());
            }
        }

        if keys.pressed(KeyCode::X) {
            let radius = transform.translation.length();

            if radius <= 20.0 {
                let new_radius = radius + t.delta_seconds() * speed * 2f32;
                *transform =
                    Transform::from_translation(transform.translation.normalize() * new_radius)
                        .looking_at(Vec3::ZERO, transform.up());
            }
        }
    }
}

fn remove_orbit(mut commands: Commands, q: Query<Entity, With<Orbit>>) {
    for entity in &q {
        commands.entity(entity).remove::<Orbit>();
    }
}
