use bevy::prelude::*;
use bevy::window::{PresentMode, Window};


mod player;
mod mechanics;

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

#[derive(Resource)]
pub struct GameAssets {
    player: Handle<Image>,
    asteroid: Handle<Image>,
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
    asteroid_query: Query<Entity, With<asteroid::Asteroid>>,
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
    assets: Res<GameAssets>,
) {
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
        .add_systems(Startup, (setup_camera, load_assets))

        // This system now runs every frame ONLY when in the Loading state
        .add_systems(Update, check_assets_loaded.run_if(in_state(GameState::Loading)))

        // This will now run correctly AFTER check_assets_loaded switches the state
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)

        .add_plugins(asteroid::AsteroidPlugin)
        .add_systems(OnEnter(GameState::GameOver), display_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), despawn_game_over_ui)
        .add_systems(Update, handle_game_over_input.run_if(in_state(GameState::GameOver)))
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mechanics::MechanicsPlugin)
        .run();
}
