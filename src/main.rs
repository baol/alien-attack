mod aliens;
mod assets;
mod combat;
mod player;
mod ui;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::{MonitorSelection, WindowMode};
use rand::Rng;

use aliens::{
    alien_shooting_system, animate_aliens, move_alien_formation, spawn_alien_entities, spawn_aliens,
    trigger_last_row_roaming, update_roaming_aliens,
};
use assets::load_game_assets;
use combat::{handle_collisions, move_projectiles, update_explosions};
use player::{
    player_movement, player_respawn_system, player_shield_system, player_weapon_system,
    spawn_player, spawn_player_entities, PlayerCharge, PlayerRespawn, PlayerShield,
};
use ui::{despawn_end_overlay, setup_ui, spawn_end_overlay, update_hud};

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Victory,
    GameOver,
}

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct Star {
    pub speed: f32,
}

#[derive(Resource)]
pub struct PlayerStats {
    pub score: u32,
    pub lives: u32,
    pub level: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            score: 0,
            lives: 3,
            level: 1,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Alien Attack - Space Invaders".to_string(),
                        resolution: (800.0_f32, 600.0_f32).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.07)))
        .init_resource::<PlayerStats>()
        .init_resource::<PlayerCharge>()
        .init_resource::<PlayerShield>()
        .init_resource::<PlayerRespawn>()
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (
                setup_camera_and_starfield,
                load_game_assets,
                setup_ui,
                (spawn_player, spawn_aliens).after(load_game_assets),
            ),
        )
        .add_systems(Update, (animate_starfield, toggle_fullscreen_system))
        .add_systems(
            Update,
            (
                player_respawn_system,
                player_movement,
                player_shield_system,
                player_weapon_system,
                animate_aliens,
                move_alien_formation,
                trigger_last_row_roaming,
                update_roaming_aliens,
                alien_shooting_system,
                move_projectiles,
                handle_collisions,
                update_explosions,
                update_hud,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Victory), spawn_end_overlay)
        .add_systems(OnExit(GameState::Victory), despawn_end_overlay)
        .add_systems(OnEnter(GameState::GameOver), spawn_end_overlay)
        .add_systems(OnExit(GameState::GameOver), despawn_end_overlay)
        .add_systems(
            Update,
            restart_game_system
                .run_if(in_state(GameState::Victory).or(in_state(GameState::GameOver))),
        )
        .run();
}

fn setup_camera_and_starfield(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 800.0,
                min_height: 600.0,
            },
            ..OrthographicProjection::default_2d()
        },
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..85 {
        let x = rng.gen_range(-400.0..400.0);
        let y = rng.gen_range(-300.0..300.0);
        let size = rng.gen_range(1.5..3.2);
        let speed = rng.gen_range(12.0..45.0);
        let brightness = rng.gen_range(0.45..0.95);

        commands.spawn((
            Star { speed },
            Sprite {
                color: Color::srgba(brightness, brightness, 1.0, brightness),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, -5.0),
        ));
    }
}

fn animate_starfield(time: Res<Time>, mut query: Query<(&Star, &mut Transform)>) {
    for (star, mut transform) in &mut query {
        transform.translation.y -= star.speed * time.delta_secs();
        if transform.translation.y < -305.0 {
            transform.translation.y = 305.0;
        }
    }
}

fn toggle_fullscreen_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut windows: Query<&mut Window>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let alt_enter = alt_held && keyboard.just_pressed(KeyCode::Enter);
    let key_toggle = keyboard.just_pressed(KeyCode::KeyF)
        || keyboard.just_pressed(KeyCode::F11)
        || alt_enter;

    // Xbox controller: Y button (GamepadButton::North) or View/Select button (GamepadButton::Select)
    let gamepad_toggle = gamepads.iter().any(|gp| {
        gp.just_pressed(GamepadButton::North) || gp.just_pressed(GamepadButton::Select)
    });

    if key_toggle || gamepad_toggle {
        if let Ok(mut window) = windows.get_single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                }
                _ => WindowMode::Windowed,
            };
        }
    }
}

fn restart_game_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    game_entities: Query<Entity, With<GameEntity>>,
    assets: Res<assets::GameAssets>,
    mut stats: ResMut<PlayerStats>,
    mut shield: ResMut<PlayerShield>,
    mut respawn: ResMut<PlayerRespawn>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let enter_restart = keyboard.just_pressed(KeyCode::Enter) && !alt_held;
    let gamepad_restart = gamepads
        .iter()
        .any(|gp| gp.just_pressed(GamepadButton::Start) || gp.just_pressed(GamepadButton::South));

    if keyboard.just_pressed(KeyCode::KeyR) || enter_restart || gamepad_restart {
        for entity in &game_entities {
            commands.entity(entity).despawn_recursive();
        }

        *stats = PlayerStats::default();
        *shield = PlayerShield::default();
        *respawn = PlayerRespawn::default();
        spawn_player_entities(&mut commands, &assets);
        spawn_alien_entities(&mut commands, &assets, stats.level);
        next_state.set(GameState::Playing);
    }
}
