use std::vec;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};
use rand::prelude::*;

mod r120;
use r120::*;

// color palette:
// https://htmlcolorcodes.com/

// ----------------------------------<< Constants >>----------------------------------

const TIMESTEP: f32 = 1.0 / 30.0;

const SPEED_OF_LIGHT: f32 = 2000.0;

// Player consts
const PLAYER_SIZE: f32 = 30.0;
const PLAYER_COLOR: Color = Color::rgb(255.0 / 256.0, 195.0 / 256.0, 0.0 / 256.0 );

const PLAYER_MAX_SPEED: f32 = 1000.0;
const PLAYER_ACCELERATION_X: f32 = PLAYER_MAX_SPEED;
const PLAYER_ACCELERATION_Y: f32 = PLAYER_MAX_SPEED;
const PLAYER_FRICTION: f32 = 0.05; // Only when not accelerating

const LEFT_BOUND: f32 = -2000.0;
const UPPER_BOUND: f32 = 2000.0;
const RIGHT_BOUND: f32 = 2000.0;
const LOWER_BOUND: f32 = -2000.0;

// Angle Marker consts
const NUM_ANGLE_MARKERS: usize = 23;
const ANGLE_MARKER_SIZE: f32 = 10.0;
const ANGLE_MARKER_COLOR: Color = Color::rgb(255.0 / 256.0, 87.0 / 256.0, 51.0 / 256.0 );
const ORBIT_RADIUS: f32 = 100.0;

// Dusties
const NUM_DUSTIES: usize = 0;
const DUSTIES_SIZE_RANGE: std::ops::Range<f32> = 1.0..4.5;
const DUSTIES_COLOR: Color = Color::rgb(80.0 / 256.0, 80.0 / 256.0, 100.0 / 256.0 );

// Test points
const NUM_TEST_POINTS: usize = 1000;
const TEST_POINTS_SIZE: f32 = 2.5;
const TEST_POINTS_COLOR: Color = Color::rgb( 1.0, 1.0, 1.0 );

// ----------------------------------<< Startup >>----------------------------------

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 40)))
        .insert_resource(GlobalTime(0.0))
        .insert_resource(LocalTime(0.0))
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
                // .with_system(debug_info.after(move_player))
                .with_system(reorient_angle_markers.after(move_player))
                .with_system(reorient_points.after(move_player))
                .with_system(reorient_paths.after(move_player))
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
#[derive(Component)]
struct Path(Vec<(Vec2, f32)>);

#[derive(Component)]
struct Dusty;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Position(Vec2);

// ----------------------------------<< Component methods >>----------------------------------

impl Path {
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
    windows: Res<Windows>,
    asset_server: Res<AssetServer>
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
    for i in 0..NUM_TEST_POINTS {
        let dist_x = 3.0 * 33.0 * (((i as f32) / 33.0).floor() - 16.6);
        let dist_y = 3.0 * 33.0 * (((i as f32) % 33.0) - 16.6);

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
        transform: Transform::from_translation(Vec3::new(180.0, 50.0, 0.5)).with_scale(Vec3::new(10.0, 10.0, 0.0)),
        ..default()
    }, Path(vec![
        (Vec2::new(0.0, 0.0), 0.3),
        (Vec2::new(100.0, 75.0), 0.6),
        (Vec2::new(150.0, 0.0), 0.8),
        (Vec2::new(100.0, -75.0), 1.0),
        (Vec2::new(-100.0, 75.0), 1.5),
        (Vec2::new(-150.0, 0.0), 1.7),
        (Vec2::new(-100.0, -75.0), 1.9)
    ])));
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_player: Query<(&mut Position, &mut Velocity), With<Player>>,
    mut global_time: ResMut<GlobalTime>,
    mut local_time: ResMut<LocalTime>
) {
    // Acceleration changing velocity
    let (mut player_position, mut player_velocity) = query_player.single_mut();
    let mut dx = 0.0;
    let mut dy = 0.0;

    if keyboard_input.pressed(KeyCode::D) {
        dx += PLAYER_ACCELERATION_X * TIMESTEP;
    }

    if keyboard_input.pressed(KeyCode::A) {
        dx -= PLAYER_ACCELERATION_X * TIMESTEP;
    }

    if keyboard_input.pressed(KeyCode::W) {
        dy += PLAYER_ACCELERATION_Y * TIMESTEP;
    }

    if keyboard_input.pressed(KeyCode::S) {
        dy -= PLAYER_ACCELERATION_Y * TIMESTEP;
    }

    // Adds the friction if the player is pressing space
    if keyboard_input.pressed(KeyCode::Space) {
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
    player_position.0.x += player_velocity.0.x * TIMESTEP;
    player_position.0.y += player_velocity.0.y * TIMESTEP;

    // Update global time
    global_time.0 += TIMESTEP / (1.0 - player_velocity.0.length().powi(2) / SPEED_OF_LIGHT.powi(2)).sqrt();
    local_time.0 += TIMESTEP;
}

fn debug_info(
    global_time: Res<GlobalTime>,
    local_time: Res<LocalTime>
) {
    println!("\n\n  Global time: {:.4}\n   Local time: {:.4}\n\n", global_time.0, local_time.0);
}

fn reorient_angle_markers(
    query_velocity: Query<&Velocity, With<Player>>,
    mut angle_marker_transforms: Query<&mut Transform, With<AngleMarker>>
) {
    let player_velocity = query_velocity.single();
    let rotor = velocity_to_rotor(player_velocity);

    for (i, mut transform) in angle_marker_transforms.iter_mut().enumerate() {
        let mut vector = R120::new(1.0, 1);
        vector[2] = (i as f32 * 2.0 * std::f32::consts::PI / (NUM_ANGLE_MARKERS as f32)).cos();
        vector[3] = (i as f32 * 2.0 * std::f32::consts::PI / (NUM_ANGLE_MARKERS as f32)).sin();

        vector = rotor * (vector * rotor.Reverse());

        // This turns it back into a circles
        vector[1] = 0.0;
        vector = vector.normalized();

        transform.translation.x = ORBIT_RADIUS * vector[2];
        transform.translation.y = ORBIT_RADIUS * vector[3];
    }
}

fn reorient_points(
    query_player: Query<(&Position, &Velocity), With<Player>>,
    mut points_transforms: Query<(&mut Transform, &Position), With<Point>>
) {
    let (player_position, player_velocity) = query_player.single();
    let rotor = velocity_to_rotor(player_velocity);

    for (mut transform, Position(pos)) in points_transforms.iter_mut() {
        let mut vector = R120::new(((pos.x - player_position.0.x).powi(2) + (pos.y - player_position.0.y).powi(2)).powf(0.5), 1);
        vector[2] = pos.x - player_position.0.x;
        vector[3] = pos.y - player_position.0.y;

        vector = rotor * (vector * rotor.Reverse());

        transform.translation.x = vector[2];
        transform.translation.y = vector[3];
    }
}

fn reorient_paths(
    query_player: Query<(&Position, &Velocity), With<Player>>,
    mut points_transforms: Query<(&mut Transform, &Path), With<Path>>,
    global_time: Res<GlobalTime>
) {
    let (player_position, player_velocity) = query_player.single();
    let rotor = velocity_to_rotor(player_velocity);
    let time = global_time.0 * SPEED_OF_LIGHT;

    for (mut transform, path) in points_transforms.iter_mut() {
        let bounds = path.get_bounds_at_time(player_position, global_time.0);
        
        let a: f32 = (bounds.1.0.x - bounds.0.0.x).powi(2) + (bounds.1.0.y - bounds.0.0.y).powi(2) - (bounds.1.1 - bounds.0.1).powi(2);
        let b: f32 = 2.0 * (bounds.0.0.x * (bounds.1.0.x - bounds.0.0.x) + bounds.0.0.y * (bounds.1.0.y - bounds.0.0.y) + (time - bounds.0.1) * (bounds.1.1 - bounds.0.1));
        let c: f32 = bounds.0.0.x.powi(2) + bounds.0.0.y.powi(2) - bounds.0.1.powi(2) + 2.0 * bounds.0.1 * time - time.powi(2);
        let mut p: f32 = (-b - (b.powi(2) - 4.0 * a * c).powf(0.5)) / (2.0 * a);

        if p < 0.0 || p > 1.0 {
            p = (-b + (b.powi(2) - 4.0 * a * c).powf(0.5)) / (2.0 * a);
        }

        let point: Vec2 = bounds.0.0 + p * (bounds.1.0 - bounds.0.0);

        let mut vector = R120::new(point.length(), 1);
        vector[2] = point.x;
        vector[3] = point.y;

        vector = rotor * (vector * rotor.Reverse());

        transform.translation.x = vector[2];
        transform.translation.y = vector[3];
    }
}

// Stops working if the circle can move a window width/height in a 1/60th of a second
fn move_dusties(
    mut query_velocity: Query<&Velocity, With<Player>>,
    mut dusties_transforms: Query<&mut Transform, With<Dusty>>,
    windows: Res<Windows>
) {
    let player_velocity = query_velocity.single_mut();
    let window = windows.get_primary().expect("No primary window during move_dusties");
    let width = window.width();
    let height = window.height();

    for mut transform in dusties_transforms.iter_mut() {
        transform.translation.x -= player_velocity.0.x * TIMESTEP;
        transform.translation.y -= player_velocity.0.y * TIMESTEP;

        
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

// ----------------------------------<< Helper Functions >>----------------------------------

fn velocity_to_rotor(velocity: &Velocity) -> R120 {
    let mut velocity_vector = R120::new(1.0, 1);
    velocity_vector[2] = velocity.0.x / SPEED_OF_LIGHT;
    velocity_vector[3] = velocity.0.y / SPEED_OF_LIGHT;

    velocity_vector = velocity_vector.normalized();

    let mut product = R120::new(1.0, 1) * velocity_vector;
    let angle = product[0].acosh() / 2.0;
    product[0] = 0.0;
    let rotor = angle.cosh() + angle.sinh() * product.normalized();

    // println!("Bivector: {} \nRotor: {}\nVelocity vector: {}\nRv(~R): {}", product, rotor, velocity_vector, rotor * (velocity_vector * rotor.Reverse()));

    rotor
}