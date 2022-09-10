use crate::GameState;
use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use iyes_loopless::prelude::*;

use self::{
    generation::{SimplexGenerator, WorldGenerator},
    shader::GenerationMaterial,
};

const RADIUS: f32 = 3.0;
const SIZE: u32 = 6000;

mod generation;
mod shader;

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
        app.add_plugin(MaterialPlugin::<shader::GenerationMaterial>::default())
            .add_enter_system(GameState::WorldGenerate, game_startup)
            .add_enter_system(GameState::WorldGenerate, grab_cursor)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::WorldGenerate)
                    .with_system(return_on_esc)
                    .with_system(movement)
                    .with_system(interpolate)
                    .into(),
            )
            .add_exit_system(GameState::WorldGenerate, crate::teardown::<GameTag>)
            .add_exit_system(GameState::WorldGenerate, release_cursor)
            .add_exit_system(GameState::WorldGenerate, remove_orbit);
    }
}

fn return_on_esc(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn interpolate(
    mut materials: ResMut<Assets<GenerationMaterial>>,
    q: Query<&Handle<GenerationMaterial>>,
    mut dir: Local<f32>,
    t: Res<Time>,
) {
    let handle = q.single();

    if *dir == 0.0 {
        *dir = 1.0;
    }

    if let Some(mat) = materials.get_mut(handle) {
        mat.interp += *dir * t.delta().as_secs_f32();
        if mat.interp > 1.0 {
            mat.interp = 1.0;
            *dir = -1.0;
        } else if mat.interp < 0.0 {
            mat.interp = 0.0;
            *dir = 1.0;
        }
    }
}

fn game_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<shader::GenerationMaterial>>,
    imgs: ResMut<Assets<Image>>,
    mut q: Query<(Entity, &mut Transform), With<crate::PlayerTag>>,
) {
    // Spawn sphere
    let mut material: GenerationMaterial = Color::rgb(0.4, 0.1, 0.8).into();
    let gen = SimplexGenerator::new(SIZE, 2 * SIZE);
    material.elevation_other = Some(gen.get_elevation_map(imgs));

    commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 3.0,
                subdivisions: 32,
            })),
            material: materials.add(material.clone()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(GameTag);

    commands.insert_resource(material);

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
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.9, 4.9).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        })
        .insert(GameTag);

    let (player, mut player_transform) = q.single_mut();
    commands.entity(player).insert(Orbit);

    // Set camera transform to be with Z in the up direction, looking at sphere
    *player_transform = Transform::from_xyz(10.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z);
}

fn grab_cursor(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().expect("No primary window");

    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
}

fn release_cursor(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().expect("No primary window");

    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
}

// Orbit around the origin, keeping looking at the center, at constant radius
fn movement(
    t: Res<Time>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut q: Query<&mut Transform, With<Orbit>>,
) {
    let speed = 5f32;
    let rough_sensitity = 0.01f32;
    let smooth_sensitivity = 0.1f32;

    for mut transform in &mut q {
        for ev in motion_evr.iter() {
            let delta_vertical = transform.up() * t.delta_seconds() * speed * ev.delta.y;
            let delta_horizontal = transform.right() * t.delta_seconds() * speed * ev.delta.x;

            let radius = transform.translation.length();

            let angle_vertical =
                delta_vertical.length() / (transform.translation + delta_vertical).length();

            let new_up =
                Quat::from_axis_angle(transform.left(), angle_vertical).mul_vec3(transform.up());

            let dir = (transform.translation + delta_horizontal + delta_vertical).normalize();

            *transform = Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, new_up);
        }

        for ev in scroll_evr.iter() {
            let radius = transform.translation.length();

            let new_radius = match ev.unit {
                MouseScrollUnit::Line => {
                    radius - t.delta_seconds() * speed * rough_sensitity * ev.y
                }
                MouseScrollUnit::Pixel => {
                    radius - t.delta_seconds() * speed * smooth_sensitivity * ev.y
                }
            };

            *transform = Transform::from_translation(
                transform.translation.normalize() * new_radius.clamp(6.5, 20.0),
            )
            .looking_at(Vec3::ZERO, transform.up());
        }
    }
}

fn remove_orbit(mut commands: Commands, q: Query<Entity, With<Orbit>>) {
    for entity in &q {
        commands.entity(entity).remove::<Orbit>();
    }
}
