use crate::GameState;
use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    window::CursorGrabMode,
};
use futures_lite::future;

use self::{
    generation::{GenerationTask, SimplexGenerator, WorldGenerator},
    shader::GenerationMaterial,
};

const SIZE: u32 = 6000;

mod generation;
mod shader;

// Tag for entities belonging to the game state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct GameTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct WorldTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct InterpTag;

#[derive(Component)]
struct GenerateTask(Task<Option<Image>>);

// Tag for orbit camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct Orbit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WorldGenerate;

impl Plugin for WorldGenerate {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<shader::GenerationMaterial>::default())
            .add_systems(
                OnEnter(GameState::WorldGenerate),
                (game_startup, grab_cursor),
            )
            .add_systems(
                OnExit(GameState::WorldGenerate),
                (
                    release_cursor,
                    remove_ambient,
                    remove_orbit,
                    crate::teardown::<GameTag>,
                ),
            )
            .add_systems(
                Update,
                (return_on_esc, movement, interpolate, poll_task)
                    .run_if(in_state(GameState::WorldGenerate)),
            );
    }
}

fn remove_ambient(mut commands: Commands) {
    commands.insert_resource(AmbientLight { ..default() });
}

fn return_on_esc(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(Some(GameState::MainMenu)));
    }
}

fn interpolate(
    mut materials: ResMut<Assets<GenerationMaterial>>,
    q: Query<&Handle<GenerationMaterial>, With<InterpTag>>,
    mut dir: Local<f32>,
    t: Res<Time>,
) {
    for handle in &q {
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
}

fn poll_task(
    mut commands: Commands,
    mut q: Query<(Entity, &mut GenerateTask)>,
    mut imgs: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<shader::GenerationMaterial>>,
    m: Query<&Handle<GenerationMaterial>>,
    world: Query<Entity, With<WorldTag>>,
) {
    let genmat = m.single();
    let world = world.single();
    for (entity, mut task) in &mut q {
        if let Some(Some(img)) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(mat) = mats.get_mut(genmat) {
                mat.elevation_other = Some(imgs.add(img));
            }
            commands.entity(world).insert(InterpTag);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn game_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<shader::GenerationMaterial>>,
    mut q: Query<(Entity, &mut Transform), With<crate::PlayerTag>>,
) {
    // Spawn sphere
    let material: GenerationMaterial = Color::rgb(0.4, 0.1, 0.8).into();
    let gen = SimplexGenerator::new(SIZE, 2 * SIZE);

    let task = AsyncComputeTaskPool::get().spawn(GenerationTask::new(gen));
    commands.spawn_empty().insert(GenerateTask(task));

    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 3.0,
                    subdivisions: 32,
                })
                .expect("Could not load mesh from icosphere"),
            ),
            material: materials.add(material.clone()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(GameTag)
        .insert(WorldTag);

    commands.insert_resource(material);

    // Spawn light
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
        .insert(GameTag);

    commands.insert_resource(AmbientLight {
        brightness: 10.0,
        ..default()
    });

    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 1000.0,
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

fn grab_cursor(mut q: Query<&mut Window>) {
    let mut window = q.iter_mut().next().expect("No primary window");
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

fn release_cursor(mut q: Query<&mut Window>) {
    let mut window = q.iter_mut().next().expect("No primary window");
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
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

#[cfg(test)]
mod test {
    use bevy::prelude::*;

    fn generate_app() -> App {
        use super::GenerationMaterial;
        use crate::PlayerTag;

        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default());
        app.add_asset::<Mesh>().add_asset::<GenerationMaterial>();
        app.world.spawn(Camera3dBundle::default()).insert(PlayerTag);

        app
    }

    #[test]
    fn setup_adds_orbit_to_player() {
        use super::{game_startup, Orbit};
        use crate::PlayerTag;

        let mut app = generate_app();
        app.add_systems(Startup, game_startup);

        app.update();

        //let q: Query<Entity, With<Camera3d>> = app.world.query_filtered();
        assert_eq!(
            app.world
                .query_filtered::<Entity, (With<Orbit>, With<PlayerTag>)>()
                .iter(&app.world)
                .count(),
            1
        );
    }

    #[test]
    fn setup_adds_point_light() {
        use super::game_startup;

        let mut app = generate_app();
        app.add_systems(Startup, game_startup);

        app.update();

        assert_eq!(
            app.world
                .query_filtered::<Entity, With<PointLight>>()
                .iter(&app.world)
                .count(),
            1
        );
    }

    #[test]
    fn setup_adds_mesh_and_material() {
        use super::{game_startup, GenerationMaterial};

        let mut app = generate_app();
        app.add_systems(Startup, game_startup);

        app.update();

        assert_eq!(
            app.world.query::<&Handle<Mesh>>().iter(&app.world).count(),
            1
        );

        assert_eq!(
            app.world
                .query::<&Handle<GenerationMaterial>>()
                .iter(&app.world)
                .count(),
            1
        );
    }

    #[test]
    fn setup_adds_ambient_light() {
        use super::game_startup;

        let mut app = generate_app();
        app.add_systems(Startup, game_startup);

        app.update();

        let light = app.world.get_resource::<AmbientLight>();
        assert!(light.is_some());
        let light = light.unwrap();
        assert_eq!(light.color, Color::WHITE);
        assert!(light.brightness > 1.0);
    }

    #[test]
    fn setup_starts_task() {
        use super::{game_startup, GenerateTask};

        let mut app = generate_app();
        app.add_systems(Startup, game_startup);

        app.update();

        let mut task: Vec<_> = app
            .world
            .query::<&mut GenerateTask>()
            .iter_mut(&mut app.world)
            .collect();

        assert_eq!(task.len(), 1);

        let result = futures_lite::future::block_on(&mut task[0].0);
        assert!(result.is_some());
    }

    // TODO: Test orbit controls
}
