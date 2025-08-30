use bevy::prelude::*;
use bevy::window::{PresentMode, Window};
use rand::Rng;

mod mechanics;
mod player;

mod asteroid;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default] // <-- This is now the starting state
    Loading,
    Playing,
    GameOver,
}

#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
pub struct GameAssets {
    player: Handle<Image>,
    asteroid: Handle<Image>,
}

#[derive(Resource)]
pub struct Score(pub u32);

#[derive(Resource)]
pub struct AsteroidSpawnTimer(pub Timer);

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: Query<&mut Window>,
) {
    let window = windows.single_mut().unwrap();
    let background_handle: Handle<Image> = asset_server.load("background.png");

    commands.spawn((
        // The image handle is a component
        // The sprite component defines how to render the image
        Sprite {
            image: background_handle.clone(),

            // Scale the sprite to fill the entire window
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        // The transform component defines the position
        // Set Z to a negative value to ensure it's drawn behind other sprites
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        player: asset_server.load("sprites/player.png"),
        asteroid: asset_server.load("sprites/asteroid.png"),
    });
}

fn check_assets_loaded(
    mut next_state: ResMut<NextState<GameState>>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    let player_loaded = asset_server.is_loaded_with_dependencies(&game_assets.player);
    let asteroid_loaded = asset_server.is_loaded_with_dependencies(&game_assets.asteroid);

    if player_loaded && asteroid_loaded {
        // All assets are now loaded, we can transition to the Playing state
        next_state.set(GameState::Playing);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), Msaa::Off));
}

fn setup_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Spawn the root node for positioning
    commands
        .spawn((
            // This Node component positions the score in the top-left corner
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            // It's good practice to give root UI nodes a transparent background
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // Spawn the text entity as a child
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font: font_handle,
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreText, // The marker component to find and update this text
            ));
        });
}

fn update_score_display(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            // CORRECTED: Access the sections Vec with .0
            text.0 = format!("Score: {}", score.0);
        }
    }
}

fn spawn_asteroids_over_time(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<AsteroidSpawnTimer>,
    windows: Query<&Window>,
    assets: Res<GameAssets>,
) {
    // Tick the timer
    timer.0.tick(time.delta());

    // If the timer just finished, spawn a new asteroid
    if timer.0.just_finished() {
        let window = windows.single().unwrap();
        let mut rng = rand::thread_rng();

        // Choose a random edge of the screen to spawn from
        let edge = rng.gen_range(0..4);
        let (x, y) = match edge {
            0 => (
                rng.gen_range(-window.width() / 2.0..window.width() / 2.0),
                window.height() / 2.0 + 50.0,
            ), // Top
            1 => (
                rng.gen_range(-window.width() / 2.0..window.width() / 2.0),
                -window.height() / 2.0 - 50.0,
            ), // Bottom
            2 => (
                -window.width() / 2.0 - 50.0,
                rng.gen_range(-window.height() / 2.0..window.height() / 2.0),
            ), // Left
            _ => (
                window.width() / 2.0 + 50.0,
                rng.gen_range(-window.height() / 2.0..window.height() / 2.0),
            ), // Right
        };
        let position = Vec3::new(x, y, 0.0);

        // Aim the asteroid towards the center with some randomness
        let direction_to_center = (Vec3::ZERO - position).normalize_or_zero();
        let angle_offset = rng.gen_range(-0.5..0.5); // Approx +/- 28 degrees
        let final_direction = Quat::from_rotation_z(angle_offset) * direction_to_center;

        let speed = asteroid::ASTEROID_LARGE_SPEED;
        let velocity = final_direction.truncate() * speed;

        // Use your existing helper function to spawn the asteroid
        asteroid::spawn_asteroid(
            &mut commands,
            asteroid::AsteroidSize::Large,
            position,
            velocity,
            &assets.asteroid,
        );
    }
}

fn display_game_over_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bold_font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Spawn the root UI entity
    commands
        .spawn((
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
    gamepads: Query<&Gamepad>,
    mut game_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<player::Player>>,
    bullet_query: Query<Entity, With<mechanics::Bullet>>,
    asteroid_query: Query<Entity, With<asteroid::Asteroid>>,
    mut score: ResMut<Score>,
) {
    let mut restart = keyboard_input.just_pressed(KeyCode::Space);

    if !restart {
        if let Some(gamepad) = gamepads.iter().next() {
            if gamepad.just_pressed(GamepadButton::South) {
                restart = true;
            }
        }
    }

    if restart {
        // Despawn all game entities
        for entity in player_query
            .iter()
            .chain(bullet_query.iter())
            .chain(asteroid_query.iter())
        {
            commands.entity(entity).despawn();
        }

        // Reset score
        score.0 = 0;

        // Transition to Playing state
        game_state.set(GameState::Playing);

        // The OnEnter(GameState::Playing) system will handle re-spawning entities
    }
}

fn spawn_game_entities(mut commands: Commands, windows: Query<&Window>, assets: Res<GameAssets>) {
    player::spawn_player(&mut commands, &assets.player);
    asteroid::spawn_initial_asteroids(commands, windows, &assets.asteroid);
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
        .init_state::<GameState>() // Starts in GameState::Loading
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_background,
                load_assets,
                setup_score_display,
            ),
        )
        .insert_resource(Score(0))
        .insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        // This system now runs every frame ONLY when in the Loading state
        .add_systems(
            Update,
            check_assets_loaded.run_if(in_state(GameState::Loading)),
        )
        // This will now run correctly AFTER check_assets_loaded switches the state
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_plugins(asteroid::AsteroidPlugin)
        .add_systems(OnEnter(GameState::GameOver), display_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), despawn_game_over_ui)
        .add_systems(
            Update,
            handle_game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            update_score_display.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            spawn_asteroids_over_time.run_if(in_state(GameState::Playing)),
        )
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mechanics::MechanicsPlugin)
        .run();
}
