use crate::GameState;
use bevy::prelude::*;

pub const PLAYER_ROTATION_SPEED: f32 = 2.5;
pub const PLAYER_THRUST_FORCE: f32 = 100.0;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerVelocity(pub Vec2);

pub fn spawn_player(commands: &mut Commands, player_handle: &Handle<Image>) {
    commands.spawn((
        Sprite {
            image: player_handle.clone(),
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
    gamepads: Query<&Gamepad>,
    mut player_query: Query<(&mut Transform, &mut PlayerVelocity), With<Player>>,
    time: Res<Time>,
) {
    let Some((mut player_transform, mut player_velocity)) = player_query.single_mut().ok() else {
        return;
    };

    let mut rotation_input = 0.0;
    let mut thrust_input = false;

    // Keyboard
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        rotation_input += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        rotation_input -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        thrust_input = true;
    }

    // Gamepad
    if let Some(gamepad) = gamepads.iter().next() {
        if gamepad.pressed(GamepadButton::DPadLeft) {
            rotation_input += 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadRight) {
            rotation_input -= 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadUp) {
            thrust_input = true;
        }

        if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
            if left_stick_x.abs() > 0.1 {
                rotation_input -= left_stick_x;
            }
        }
        if let Some(left_stick_y) = gamepad.get(GamepadAxis::LeftStickY) {
            if left_stick_y > 0.1 {
                thrust_input = true;
            }
        }
    }

    // Rotation
    if rotation_input != 0.0 {
        player_transform
            .rotate_z(rotation_input.clamp(-1.0, 1.0) * PLAYER_ROTATION_SPEED * time.delta_secs());
    }

    // Thrust
    if thrust_input {
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
