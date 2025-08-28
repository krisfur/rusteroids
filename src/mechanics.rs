use bevy::prelude::*;
use bevy::window::Window;
use crate::player;
use crate::{GameState};
use crate::asteroid::{Asteroid, AsteroidSize, spawn_asteroid, ASTEROID_MEDIUM_SPEED, ASTEROID_SMALL_SPEED};
use rand::prelude::*;
use crate::GameAssets;


pub const BULLET_SPEED: f32 = 500.0;
pub const BULLET_LIFETIME: f32 = 2.0;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct BulletVelocity(pub Vec2);

#[derive(Component)]
pub struct BulletLifetime(pub Timer);

pub fn spawn_bullet(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<player::Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let Ok(player_transform) = player_query.single() else { return; }; // Safely get player transform
        let bullet_direction = player_transform.rotation * Vec3::Y;
        let bullet_position = player_transform.translation + bullet_direction * 20.0;

        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_translation(bullet_position),
            GlobalTransform::default(),
            Bullet,
            BulletVelocity(bullet_direction.truncate() * BULLET_SPEED),
            BulletLifetime(Timer::from_seconds(BULLET_LIFETIME, TimerMode::Once)),
        ));
    }
}

pub fn move_bullets(
    mut bullet_query: Query<(&mut Transform, &BulletVelocity), With<Bullet>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in bullet_query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

pub fn despawn_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut BulletLifetime), With<Bullet>>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in bullet_query.iter_mut() {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn wrap_around_screen(
    mut query: Query<&mut Transform, Or<(With<player::Player>, With<Asteroid>)>>,
    windows: Query<&Window>,
) {
    let Ok(_window) = windows.single() else { return; }; // Prefix with _
    let half_width = _window.width() / 2.0;
    let half_height = _window.height() / 2.0;

    for mut transform in query.iter_mut() {
        let mut translation = transform.translation;

        if translation.x > half_width {
            translation.x = -half_width;
        } else if translation.x < -half_width {
            translation.x = half_width;
        }

        if translation.y > half_height {
            translation.y = -half_height;
        } else if translation.y < -half_height {
            translation.y = half_height;
        }

        transform.translation = translation;
    }
}

pub fn despawn_out_of_bounds_bullets(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    windows: Query<&Window>,
) {
    let Ok(_window) = windows.single() else { return; }; // Prefix with _
    let half_width = _window.width() / 2.0;
    let half_height = _window.height() / 2.0;

    for (entity, transform) in bullet_query.iter() {
        let translation = transform.translation;

        if translation.x > half_width
            || translation.x < -half_width
            || translation.y > half_height
            || translation.y < -half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_asteroid_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    asteroid_query: Query<(Entity, &Transform, &AsteroidSize), With<Asteroid>>,
    windows: Query<&Window>,
    assets: Res<GameAssets>,
) {
    let Ok(_window) = windows.single() else { return; }; // Prefix with _
    let mut rng = rand::thread_rng();

    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (asteroid_entity, asteroid_transform, asteroid_size) in asteroid_query.iter() {
            // Simple AABB collision detection for now
            let bullet_size = 10.0; // Assuming bullet size is 10x10
            let asteroid_current_size = match asteroid_size {
                AsteroidSize::Large => 80.0,
                AsteroidSize::Medium => 40.0,
                AsteroidSize::Small => 20.0,
            };

            let distance = bullet_transform.translation.distance(asteroid_transform.translation);
            if distance < (bullet_size / 2.0 + asteroid_current_size / 2.0) {
                // Collision detected!
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();

                match asteroid_size {
                    AsteroidSize::Large => {
                        for _ in 0..2 {
                            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                            let speed = ASTEROID_MEDIUM_SPEED;
                            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                            spawn_asteroid(&mut commands, AsteroidSize::Medium, asteroid_transform.translation, velocity, &assets.asteroid);
                        }
                    }
                    AsteroidSize::Medium => {
                        for _ in 0..2 {
                            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
                            let speed = ASTEROID_SMALL_SPEED;
                            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                            spawn_asteroid(&mut commands, AsteroidSize::Small, asteroid_transform.translation, velocity, &assets.asteroid);
                        }
                    }
                    AsteroidSize::Small => {
                        // Small asteroids just disappear
                    }
                }
            }
        }
    }
}

fn player_asteroid_collision(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<player::Player>>,
    asteroid_query: Query<(&Transform, &AsteroidSize), With<Asteroid>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_entity, player_transform)) = player_query.single() else { return; };

    let player_size = 50.0; // Assuming player size is 50x50

    for (asteroid_transform, asteroid_size) in asteroid_query.iter() {
        let asteroid_current_size = match asteroid_size {
            AsteroidSize::Large => 80.0,
            AsteroidSize::Medium => 40.0,
            AsteroidSize::Small => 20.0,
        };

        let distance = player_transform.translation.distance(asteroid_transform.translation);
        if distance < (player_size / 2.0 + asteroid_current_size / 2.0) {
            // Collision detected! Game Over
            println!("Game Over! Player hit an asteroid.");
            commands.entity(player_entity).despawn();
            game_state.set(GameState::GameOver);
        }
    }
}

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_bullet,
            move_bullets,
            despawn_bullets,
            wrap_around_screen,
            despawn_out_of_bounds_bullets,
            bullet_asteroid_collision,
            player_asteroid_collision,
        ).run_if(in_state(GameState::Playing)));
    }
}