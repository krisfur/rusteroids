use bevy::prelude::*;
use bevy::window::{PresentMode, Window};
use rand::prelude::*;

mod player;
mod mechanics;

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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Component)]
struct GameOverUi;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), Msaa::Off));
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    size: AsteroidSize,
    position: Vec3,
    velocity: Vec2,
) {
    let (asteroid_size, color) = match size {
        AsteroidSize::Large => (ASTEROID_LARGE_SIZE, Color::srgb(0.5, 0.5, 0.5)),
        AsteroidSize::Medium => (ASTEROID_MEDIUM_SIZE, Color::srgb(0.6, 0.6, 0.6)),
        AsteroidSize::Small => (ASTEROID_SMALL_SIZE, Color::srgb(0.7, 0.7, 0.7)),
    };

    commands.spawn((
        Sprite {
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

        spawn_asteroid(&mut commands, AsteroidSize::Large, position, velocity);
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

fn display_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let bold_font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Spawn the root UI entity
    commands.spawn((
        Name::new("Game Over UI"),
        GameOverUi,
        // The Node component defines the layout box.
        Node {
            // Fill the entire window
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            // Center its children
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // A transparent background for the root node.
        BackgroundColor(Color::NONE),
    ))
    // Use the new `children!` macro to spawn the text entity.
    .with_children(|parent| {
        parent.spawn((
            // The text content.
            Text::new("Game Over!\nPress fire to play again"),
            // Set the font.
            TextFont {
                font: bold_font,
                font_size: 40.0,
                ..Default::default()
            },
            // Set the color.
            TextColor(Color::WHITE),
            // Set the text alignment (replaces the old `.with_text_justify`).
            TextLayout::new(JustifyText::Center, LineBreak::NoWrap),
        ));
    });
}

fn despawn_game_over_ui(
    mut commands: Commands,
    game_over_ui_query: Query<Entity, With<GameOverUi>>,
) {
    for entity in game_over_ui_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_game_over_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<player::Player>>,
    bullet_query: Query<Entity, With<mechanics::Bullet>>,
    asteroid_query: Query<Entity, With<Asteroid>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Despawn all game entities
        for entity in player_query.iter().chain(bullet_query.iter()).chain(asteroid_query.iter()) {
            commands.entity(entity).despawn();
        }

        // Transition to Playing state
        game_state.set(GameState::Playing);

        // The OnEnter(GameState::Playing) system will handle re-spawning entities
    }
}

fn spawn_game_entities(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    player::spawn_player(&mut commands);
    spawn_initial_asteroids(commands, windows);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800., 600.).into(),
                title: "Rusteroids".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_systems(Update, move_asteroids.run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), display_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), despawn_game_over_ui)
        .add_systems(Update, handle_game_over_input.run_if(in_state(GameState::GameOver)))
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mechanics::MechanicsPlugin)
        .run();
}
