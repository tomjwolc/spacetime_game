use bevy::prelude::*;

use super::*;

pub struct ReorientPlugin;

impl Plugin for ReorientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(reorient_angle_markers)
            .add_system(reorient_points)
            .add_system(reorient_paths);
    }
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
    query_global_time: Res<GlobalTime>
) {
    let (player_position, player_velocity) = query_player.single();
    let rotor = velocity_to_rotor(player_velocity);
    let global_time = query_global_time.0 * SPEED_OF_LIGHT;

    for (mut transform, path) in points_transforms.iter_mut() {
        let bounds = path.get_bounds_at_time(player_position, query_global_time.0);
        
        let a: f32 = (bounds.1.0.x - bounds.0.0.x).powi(2) + (bounds.1.0.y - bounds.0.0.y).powi(2) - (bounds.1.1 - bounds.0.1).powi(2);
        let b: f32 = 2.0 * (bounds.0.0.x * (bounds.1.0.x - bounds.0.0.x) + bounds.0.0.y * (bounds.1.0.y - bounds.0.0.y) + (global_time - bounds.0.1) * (bounds.1.1 - bounds.0.1));
        let c: f32 = bounds.0.0.x.powi(2) + bounds.0.0.y.powi(2) - bounds.0.1.powi(2) + 2.0 * bounds.0.1 * global_time - global_time.powi(2);
        let mut p: f32 = (-b - (b.powi(2) - 4.0 * a * c).abs().powf(0.5)) / (2.0 * a);

        if p < 0.0 || p > 1.0 {
            p = (-b + (b.powi(2) - 4.0 * a * c).abs().powf(0.5)) / (2.0 * a);
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