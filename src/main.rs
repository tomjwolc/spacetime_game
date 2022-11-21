use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};
use rand::prelude::*;

mod r120;
use r120::*;

// color palette:
// https://htmlcolorcodes.com/

const TIMESTEP: f32 = 1.0 / 60.0;

const SPEED_OF_LIGHT: f32 = 2000.0;

// Player consts
const PLAYER_SIZE: f32 = 30.0;
const PLAYER_COLOR: Color = Color::rgb(255.0 / 256.0, 195.0 / 256.0, 0.0 / 256.0 );

const PLAYER_MAX_SPEED: f32 = 1000.0;
const PLAYER_ACCELERATION_X: f32 = 1000.0;
const PLAYER_ACCELERATION_Y: f32 = 1000.0;
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
const NUM_DUSTIES: usize = 200;
const DUSTIES_SIZE_RANGE: std::ops::Range<f32> = 1.0..4.5;
const DUSTIES_COLOR: Color = Color::rgb(80.0 / 256.0, 80.0 / 256.0, 100.0 / 256.0 );

// Test points
const NUM_TEST_POINTS: usize = 200;
const TEST_POINTS_SIZE: f32 = 5.0;
const TEST_POINTS_COLOR: Color = Color::rgb( 1.0, 1.0, 1.0 );

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 40)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP as f64))
                .with_system(player_accelerate)
                .with_system(reorient_angle_markers.after(player_accelerate))
                .with_system(move_points.after(player_accelerate))
                .with_system(reorient_points.after(move_points))
                .with_system(move_dusties.after(player_accelerate))
        )
        .add_system(bevy::window::close_on_esc)
        .run()
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct AngleMarker;

#[derive(Component)]
struct CenterPoint;

#[derive(Component)]
struct Point;

#[derive(Component)]
struct Dusty;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Position(Vec2);

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
    }, Player, Velocity(Vec2::new(0.0, 0.0))));

    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::default().into()).into(),
        material: materials.add(ColorMaterial::from(Color::NONE)),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    }, CenterPoint, Point, Position(Vec2::new(0.0, 0.0))));

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
}

fn player_accelerate(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_velocity: Query<&mut Velocity, With<Player>>,
    query_center: Query<&Position, With<CenterPoint>>
) {
    let mut player_velocity = query_velocity.single_mut();
    let center_pos = query_center.single();
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

    if dx == 0.0 && dy == 0.0 {
        dx -= PLAYER_FRICTION * player_velocity.0.x;
        dy -= PLAYER_FRICTION * player_velocity.0.y;
    }

    // Boundry detection
    if center_pos.0.x < LEFT_BOUND { 
        dx += center_pos.0.x - LEFT_BOUND;
    }

    if center_pos.0.x > RIGHT_BOUND {
        dx += center_pos.0.x - RIGHT_BOUND;
    }

    if center_pos.0.y < LOWER_BOUND { 
        dy += center_pos.0.y - LOWER_BOUND;
    }

    if center_pos.0.y > UPPER_BOUND {
        dy += center_pos.0.y - UPPER_BOUND;
    }

    player_velocity.0.x += dx;
    player_velocity.0.y += dy;

    // caps the speed
    if player_velocity.0.length() > PLAYER_MAX_SPEED {
        player_velocity.0 = PLAYER_MAX_SPEED * player_velocity.0.normalize();
    }
}

fn move_points(
    query_velocity: Query<&Velocity, With<Player>>,
    mut points_transforms: Query<&mut Position, With<Point>>
) {
    let player_velocity = query_velocity.single();

    for mut pos in points_transforms.iter_mut() {
        pos.0.x -= player_velocity.0.x * TIMESTEP;
        pos.0.y -= player_velocity.0.y * TIMESTEP;
    }
}

fn reorient_angle_markers(
    query_velocity: Query<&Velocity, With<Player>>,
    mut angle_marker_transforms: Query<&mut Transform, With<AngleMarker>>
) {
    let player_velocity = query_velocity.single();

    // if player_velocity.0.length() < 0.01 { return; }

    let mut velocity_vector = R120::zero();
    velocity_vector[2] = -player_velocity.0.x / SPEED_OF_LIGHT;
    velocity_vector[3] = -player_velocity.0.y / SPEED_OF_LIGHT;

    let bivector = velocity_vector ^ R120::new(1.0, 1);
    let rotor = bivector.norm().cosh() + (bivector.normalized() * bivector.norm().sinh());

    for (i, mut transform) in angle_marker_transforms.iter_mut().enumerate() {
        let mut vector = R120::new(1.0, 1);
        vector[2] = (i as f32 * 2.0 * std::f32::consts::PI / (NUM_ANGLE_MARKERS as f32)).cos();
        vector[3] = (i as f32 * 2.0 * std::f32::consts::PI / (NUM_ANGLE_MARKERS as f32)).sin();

        let prev_vector = vector;
        vector = rotor * (vector * rotor.Reverse());
        if i == 3 { println!("\nrotor:    {}\nbivector: {}\nvector:   {}\nprev:     {}\n", rotor, bivector, vector, prev_vector) }

        // This turns it back into a circles
        vector[1] = 0.0;
        vector = vector.normalized();

        transform.translation.x = ORBIT_RADIUS * vector[2];
        transform.translation.y = ORBIT_RADIUS * vector[3];
    }
}

fn reorient_points(
    query_velocity: Query<&Velocity, With<Player>>,
    mut points_transforms: Query<(&mut Transform, &Position), With<Point>>
) {
    let player_velocity = query_velocity.single();

    if player_velocity.0.length() < 0.01 { return; }

    let mut velocity_vector = R120::zero();
    velocity_vector[2] = -player_velocity.0.x / SPEED_OF_LIGHT;
    velocity_vector[3] = -player_velocity.0.y / SPEED_OF_LIGHT;

    let bivector = velocity_vector ^ R120::new(1.0, 1);
    let rotor = bivector.norm().cosh() + bivector.normalized() * bivector.norm().sinh();

    for (mut transform, Position(pos)) in points_transforms.iter_mut() {
        // if !isOnscreen { continue; }

        let mut vector = R120::new(1.0, 1);
        vector[2] = pos.x;
        vector[3] = pos.y;

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