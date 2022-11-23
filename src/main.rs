use std::vec;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};
use rand::prelude::*;
use std::f32::consts::PI;

mod r120;
use r120::*;

mod reorient;
use reorient::ReorientPlugin;

// color palette:
// https://htmlcolorcodes.com/

// ----------------------------------<< Constants >>----------------------------------

const TIMESTEP: f32 = 1.0 / 60.0;

const SPEED_OF_LIGHT: f32 = 2000.0;

// Player consts
const PLAYER_SIZE: f32 = 30.0;
const PLAYER_COLOR: Color = Color::rgb(255.0 / 256.0, 195.0 / 256.0, 0.0 / 256.0 );

const PLAYER_MAX_SPEED: f32 = 1990.0;
const PLAYER_ACCELERATION_X: f32 = PLAYER_MAX_SPEED;
const PLAYER_ACCELERATION_Y: f32 = PLAYER_MAX_SPEED;
const PLAYER_FRICTION: f32 = 0.05; // Only when not accelerating

const LEFT_BOUND: f32 = -20000.0;
const UPPER_BOUND: f32 = 20000.0;
const RIGHT_BOUND: f32 = 20000.0;
const LOWER_BOUND: f32 = -20000.0;

// Angle Marker consts
const NUM_ANGLE_MARKERS: usize = 23;
const ANGLE_MARKER_SIZE: f32 = 10.0;
const ANGLE_MARKER_COLOR: Color = Color::rgb(255.0 / 256.0, 87.0 / 256.0, 51.0 / 256.0 );
const ORBIT_RADIUS: f32 = 100.0;

// Dusties
const NUM_DUSTIES: usize = 200;
const DUSTIES_SIZE_RANGE: std::ops::Range<f32> = 1.0..4.5;
const DUSTIES_COLOR: Color = Color::rgb(80.0 / 256.0, 80.0 / 256.0, 100.0 / 256.0 );

// Test points
const NUM_TEST_POINTS: usize = 300;
const TEST_POINTS_SIZE: f32 = 7.5;
const TEST_POINTS_COLOR: Color = Color::rgb( 1.0, 1.0, 1.0 );

// ----------------------------------<< Startup >>----------------------------------

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 40)))
        .insert_resource(GlobalTime(0.0))
        .insert_resource(LocalTime(0.0))
        .add_plugin(ReorientPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spacetime game".to_string(),
                ..Default::default()
            },
            ..default()
        }))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP as f64))
                .with_system(move_player)
                .with_system(debug_info.after(move_player))
                .with_system(move_dusties.after(move_player))
        )
        .add_system(bevy::window::close_on_esc)
        .run()
}

// ----------------------------------<< Resources >>----------------------------------

#[derive(Resource)]
struct GlobalTime(f32);

#[derive(Resource)]
struct LocalTime(f32);

// ----------------------------------<< Components >>----------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct AngleMarker;

#[derive(Component)]
struct Point;

// Assumption that there is always at least one element in vec
#[derive(Component, Debug)]
struct Path(Vec<(Vec2, f32)>);

#[derive(Component)]
struct Dusty;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Position(Vec2);

// ----------------------------------<< Component methods >>----------------------------------

impl Path {
    fn from_paremetric_equation(min: f32, max: f32, total_time: f32, samples: usize, eq: fn(f32) -> Vec2) -> Self {
        let mut path = Path(Vec::new());

        for i in 0..(samples + 1) {
            path.0.push((
                eq((i as f32) * (max - min) / (samples as f32) + min),
                (i as f32) * total_time / (samples as f32)
            ));
        }

        path
    }

    fn get_bounds_at_time(&self, player_position: &Position, mut global_time: f32) -> ((Vec2, f32), (Vec2, f32)) {
        global_time *= SPEED_OF_LIGHT; // turn time units to ct

        let mut i = 0;
        let last = self.0.last().expect("Path was empty in Path::get_bounds_at_time");
        let offset = last.1 * SPEED_OF_LIGHT * (((last.0 - player_position.0).length() + global_time) / (last.1 * SPEED_OF_LIGHT)).floor();

        // get the index of the first rest stop that is above or on the light cone
        while (self.0[i].0 - player_position.0).length() + global_time >= self.0[i].1 * SPEED_OF_LIGHT + offset {
            i += 1;
        };

        let prev_index = if i == 0 { self.0.len() - 1 } else { i - 1 };
        
        (
            (self.0[prev_index].0 - player_position.0, if i == 0 { 0.0 } else { self.0[prev_index].1 * SPEED_OF_LIGHT } + offset),
            (self.0[i].0 - player_position.0, self.0[i].1 * SPEED_OF_LIGHT + offset)
        )
    }
}

// ----------------------------------<< Systems >>----------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>
) {
    commands.spawn(Camera2dBundle::default());

    // Spawns player
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0))
            .with_scale(Vec3::new(PLAYER_SIZE, PLAYER_SIZE, 0.0)),
        ..default()
    }, Player, Position(Vec2::new(0.0, 0.0)), Velocity(Vec2::new(0.0, 0.0))));

    // spawns all of the angle markers
    for i in 0..NUM_ANGLE_MARKERS {
        let dist_x = ORBIT_RADIUS * ((i as f32) * (2.0 * std::f32::consts::PI) / (NUM_ANGLE_MARKERS as f32)).cos();
        let dist_y = ORBIT_RADIUS * ((i as f32) * (2.0 * std::f32::consts::PI) / (NUM_ANGLE_MARKERS as f32)).sin();

        commands.spawn((MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(ANGLE_MARKER_COLOR)),
            transform: Transform::from_translation(Vec3::new(dist_x, dist_y, 1.0))
                .with_scale(Vec3::new(ANGLE_MARKER_SIZE, ANGLE_MARKER_SIZE, 0.0)),
            ..default()
        }, AngleMarker));
    }

    let mut rng = thread_rng();
    let window = windows.get_primary().expect("No primary window during setup");
    let width = window.width();
    let height = window.height();

    // spawns all of the dusties
    for _ in 0..NUM_DUSTIES {
        let dist_x = width / 2.0 * rng.gen_range(-1.0..1.0);
        let dist_y = height / 2.0 * rng.gen_range(-1.0..1.0);
        let size = rng.gen_range(DUSTIES_SIZE_RANGE);

        commands.spawn((MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(DUSTIES_COLOR)),
            transform: Transform::from_translation(Vec3::new(dist_x, dist_y, 0.0))
                .with_scale(Vec3::new(size, size, 0.0)),
            ..default()
        }, Dusty));
    }

    // spawns all of the test points
    for _ in 0..NUM_TEST_POINTS {
        let dist_x = rng.gen_range(LEFT_BOUND..RIGHT_BOUND);
        let dist_y = rng.gen_range(LOWER_BOUND..UPPER_BOUND);

        commands.spawn((MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(TEST_POINTS_COLOR)),
            transform: Transform::from_translation(Vec3::new(dist_x, dist_y, 0.0))
                .with_scale(Vec3::new(TEST_POINTS_SIZE, TEST_POINTS_SIZE, 0.0)),
            ..default()
        }, Point, Position(Vec2::new(dist_x, dist_y))));
    }

    // spawns the path
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(Color::CYAN)),
        transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
        ..default()
    }, Path::from_paremetric_equation(
        0.0, 
        2.0 * PI, 
        10.0, 
        40, 
        |t| Vec2::new(200.0 * t.cos(), 50.0 * (2.0 * t).sin() + 100.0)
    )));

    // spawns the path
    // commands.spawn((MaterialMesh2dBundle {
    //     mesh: meshes.add(shape::Circle::default().into()).into(),
    //     material: materials.add(ColorMaterial::from(Color::CYAN)),
    //     transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
    //     ..default()
    // }, Path(vec![
    //     (Vec2::new(-1000.0, 0.0), 1.01),
    //     (Vec2::new(1000.0, 0.0), 2.02)
    // ])));

    // spawns the clock
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(Color::RED)),
        transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
        ..default()
    }, Path::from_paremetric_equation(
        0.0, 
        2.0 * PI, 
        60.0, 
        40, 
        |t| Vec2::new(100.0 * t.cos(), 100.0 * t.sin())
    )));
    
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(Color::ORANGE)),
        transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
        ..default()
    }, Path::from_paremetric_equation(
        0.0, 
        2.0 * PI, 
        60.0 * 60.0, 
        40, 
        |t| Vec2::new(100.0 * t.cos(), 100.0 * t.sin())
    )));
    
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        transform: Transform::from_scale(Vec3::new(20.0, 20.0, 0.0)),
        ..default()
    }, Point, Position(Vec2::new(0.0, 0.0))));
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_player: Query<(&mut Position, &mut Velocity), With<Player>>,
    mut global_time: ResMut<GlobalTime>,
    mut local_time: ResMut<LocalTime>,
    time: Res<Time>
) {
    // Acceleration changing velocity
    let (mut player_position, mut player_velocity) = query_player.single_mut();
    let mut dx = 0.0;
    let mut dy = 0.0;

    if keyboard_input.pressed(KeyCode::D) {
        dx += PLAYER_ACCELERATION_X * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::A) {
        dx -= PLAYER_ACCELERATION_X * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::W) {
        dy += PLAYER_ACCELERATION_Y * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::S) {
        dy -= PLAYER_ACCELERATION_Y * time.delta_seconds();
    }

    // Adds the friction if the player is pressing space
    if keyboard_input.pressed(KeyCode::Space) {
        if player_velocity.0.length() < PLAYER_MAX_SPEED / 100.0 {
            player_velocity.0.x = 0.0;
            player_velocity.0.y = 0.0;
        }

        dx -= PLAYER_FRICTION * player_velocity.0.x;
        dy -= PLAYER_FRICTION * player_velocity.0.y;
    }

    // Boundry detection
    if player_position.0.x < LEFT_BOUND { 
        dx -= player_position.0.x - LEFT_BOUND;
    }

    if player_position.0.x > RIGHT_BOUND {
        dx -= player_position.0.x - RIGHT_BOUND;
    }

    if player_position.0.y < LOWER_BOUND { 
        dy -= player_position.0.y - LOWER_BOUND;
    }

    if player_position.0.y > UPPER_BOUND {
        dy -= player_position.0.y - UPPER_BOUND;
    }

    player_velocity.0.x += dx;
    player_velocity.0.y += dy;

    // caps the speed
    if player_velocity.0.length() > PLAYER_MAX_SPEED {
        player_velocity.0 = PLAYER_MAX_SPEED * player_velocity.0.normalize();
    }

    // Velocity change position
    player_position.0.x += player_velocity.0.x * time.delta_seconds();
    player_position.0.y += player_velocity.0.y * time.delta_seconds();

    // Update global time
    global_time.0 += time.delta_seconds() / (1.0 - player_velocity.0.length().powi(2) / SPEED_OF_LIGHT.powi(2)).sqrt();
    local_time.0 += time.delta_seconds();
}

fn debug_info(
    global_time: Res<GlobalTime>,
    local_time: Res<LocalTime>
) {
    println!("\n\n  Global time: {:.4}\n   Local time: {:.4}\n\n", global_time.0, local_time.0);
}

// Stops working if the circle can move a window width/height in a 1/60th of a second
fn move_dusties(
    mut query_velocity: Query<&Velocity, With<Player>>,
    mut dusties_transforms: Query<&mut Transform, With<Dusty>>,
    windows: Res<Windows>,
    time: Res<Time>
) {
    let player_velocity = query_velocity.single_mut();
    let window = windows.get_primary().expect("No primary window during move_dusties");
    let width = window.width();
    let height = window.height();

    for mut transform in dusties_transforms.iter_mut() {
        transform.translation.x -= player_velocity.0.x * time.delta_seconds();
        transform.translation.y -= player_velocity.0.y * time.delta_seconds();

        
        if transform.translation.x < width / -2.0 {
            transform.translation.x += width;
        } else if transform.translation.x > width / 2.0 {
            transform.translation.x -= width;
        }

        if transform.translation.y < height / -2.0 {
            transform.translation.y += height;
        } else if transform.translation.y > height / 2.0 {
            transform.translation.y -= height;
        }
    }
}