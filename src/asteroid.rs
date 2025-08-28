use bevy::prelude::*;
use rand::prelude::*;
use crate::GameState;

pub const ASTEROID_LARGE_SIZE: f32 = 80.0;
pub const ASTEROID_MEDIUM_SIZE: f32 = 40.0;
pub const ASTEROID_SMALL_SIZE: f32 = 20.0;

pub const ASTEROID_LARGE_SPEED: f32 = 50.0;
pub const ASTEROID_MEDIUM_SPEED: f32 = 75.0;
pub const ASTEROID_SMALL_SPEED: f32 = 100.0;

pub const INITIAL_ASTEROIDS: usize = 4;
pub const MIN_SPAWN_DISTANCE: f32 = 100.0; // Minimum distance from center for asteroid spawn

#[derive(Component)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct AsteroidVelocity(pub Vec2);

pub fn spawn_asteroid(
    commands: &mut Commands,
    size: AsteroidSize,
    position: Vec3,
    velocity: Vec2,
    asteroid_handle: &Handle<Image>,
) {
    let (asteroid_size, color) = match size {
        AsteroidSize::Large => (ASTEROID_LARGE_SIZE, Color::srgb(0.5, 0.5, 0.5)),
        AsteroidSize::Medium => (ASTEROID_MEDIUM_SIZE, Color::srgb(0.6, 0.6, 0.6)),
        AsteroidSize::Small => (ASTEROID_SMALL_SIZE, Color::srgb(0.7, 0.7, 0.7)),
    };

    commands.spawn((
        Sprite {
            image: asteroid_handle.clone(),
            color,
            custom_size: Some(Vec2::new(asteroid_size, asteroid_size)),
            ..default()
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Asteroid,
        size,
        AsteroidVelocity(velocity),
    ));
}

pub fn spawn_initial_asteroids(
    mut commands: Commands,
    windows: Query<&Window>,
    asteroid_handle: &Handle<Image>,
) {
    let window = windows.single().unwrap();
    let mut rng = rand::thread_rng();

    for _ in 0..INITIAL_ASTEROIDS {
        let mut position;
        loop {
            let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
            let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);
            position = Vec3::new(x, y, 0.0);

            // Ensure asteroid doesn't spawn too close to the center (player's initial position)
            if position.distance(Vec3::ZERO) > MIN_SPAWN_DISTANCE {
                break;
            }
        }

        let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
        let speed = ASTEROID_LARGE_SPEED;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        spawn_asteroid(&mut commands, AsteroidSize::Large, position, velocity, asteroid_handle);
    }
}

fn move_asteroids(
    mut asteroid_query: Query<(&mut Transform, &AsteroidVelocity), With<Asteroid>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in asteroid_query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_asteroids.run_if(in_state(GameState::Playing)));
    }
}
