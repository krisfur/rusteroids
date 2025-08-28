use bevy::{asset, prelude::*};
use crate::GameState;

pub const PLAYER_ROTATION_SPEED: f32 = 2.5;
pub const PLAYER_THRUST_FORCE: f32 = 100.0;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerVelocity(pub Vec2);

pub fn spawn_player(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite {
            image: asset_server.load("sprites/player.png"),
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::new(75.0, 75.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Player,
        PlayerVelocity::default(),
    ));
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut PlayerVelocity), With<Player>>,
    time: Res<Time>,
) {
    let Some((mut player_transform, mut player_velocity)) = player_query.single_mut().ok() else { return; };

    // Rotation
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        player_transform.rotate_z(PLAYER_ROTATION_SPEED * time.delta_secs());
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        player_transform.rotate_z(-PLAYER_ROTATION_SPEED * time.delta_secs());
    }

    // Thrust
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        let forward = player_transform.rotation * Vec3::Y;
        player_velocity.0 += forward.truncate() * PLAYER_THRUST_FORCE * time.delta_secs();
    }

    // Apply velocity
    player_transform.translation.x += player_velocity.0.x * time.delta_secs();
    player_transform.translation.y += player_velocity.0.y * time.delta_secs();
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}