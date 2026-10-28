use bevy::prelude::*;

use crate::assets::{play_sfx, GameAssets};
use crate::combat::PlayerProjectile;
use crate::GameEntity;

pub const PLAYER_SPEED: f32 = 340.0;
pub const PLAYER_Y: f32 = -245.0;
pub const PLAYER_X_BOUND: f32 = 365.0;
pub const MAX_PLAYER_SHOTS: usize = 3;
pub const CHARGE_DURATION_SECS: f32 = 2.0;

pub const SHIELD_DURATION_SECS: f32 = 2.5;
pub const SHIELD_COOLDOWN_SECS: f32 = 5.0;
pub const RESPAWN_DELAY_SECS: f32 = 1.1;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ChargeIndicatorOrb;

#[derive(Component)]
pub struct ShieldBubble;

#[derive(Resource)]
pub struct PlayerCharge {
    pub hold_seconds: f32,
    pub is_charging: bool,
    pub was_fire_held: bool,
}

impl Default for PlayerCharge {
    fn default() -> Self {
        Self {
            hold_seconds: 0.0,
            is_charging: false,
            was_fire_held: false,
        }
    }
}

impl PlayerCharge {
    pub fn progress(&self) -> f32 {
        (self.hold_seconds / CHARGE_DURATION_SECS).clamp(0.0, 1.0)
    }

    pub fn is_fully_charged(&self) -> bool {
        self.hold_seconds >= CHARGE_DURATION_SECS
    }
}

#[derive(Resource)]
pub struct PlayerShield {
    pub active_remaining: f32,
    pub cooldown_remaining: f32,
    pub alt_combo_used: bool,
}

impl Default for PlayerShield {
    fn default() -> Self {
        Self {
            active_remaining: 0.0,
            cooldown_remaining: 0.0,
            alt_combo_used: false,
        }
    }
}

impl PlayerShield {
    pub fn is_active(&self) -> bool {
        self.active_remaining > 0.0
    }

    pub fn is_ready(&self) -> bool {
        self.active_remaining <= 0.0 && self.cooldown_remaining <= 0.0
    }
}

#[derive(Resource, Default)]
pub struct PlayerRespawn {
    pub timer: Option<Timer>,
}

pub fn spawn_player_entities(commands: &mut Commands, assets: &GameAssets) {
    commands.insert_resource(PlayerCharge::default());

    commands
        .spawn((
            Player,
            GameEntity,
            Sprite {
                image: assets.player_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.player_layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(48.0, 48.0)),
                ..default()
            },
            Transform::from_xyz(0.0, PLAYER_Y, 1.0),
        ))
        .with_children(|parent| {
            // Visual energy orb right in front of the ship's cannon when charging
            parent.spawn((
                ChargeIndicatorOrb,
                Sprite {
                    color: Color::srgba(0.2, 0.9, 1.0, 0.0),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 26.0, 2.0),
            ));

            // Temporary energy shield aura around the player ship
            parent.spawn((
                ShieldBubble,
                Sprite {
                    color: Color::srgba(0.15, 0.85, 1.0, 0.0),
                    custom_size: Some(Vec2::new(66.0, 66.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1.5),
            ));
        });
}

pub fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.insert_resource(PlayerShield::default());
    commands.insert_resource(PlayerRespawn::default());
    spawn_player_entities(&mut commands, &assets);
}

pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };

    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Xbox / Gamepad movement (D-Pad Left/Right or Left Stick X)
    for gamepad in &gamepads {
        if gamepad.pressed(GamepadButton::DPadLeft) {
            direction -= 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadRight) {
            direction += 1.0;
        }
        if let Some(stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
            if stick_x.abs() > 0.15 {
                direction += stick_x;
            }
        }
    }

    direction = direction.clamp(-1.0, 1.0);

    transform.translation.x =
        (transform.translation.x + direction * PLAYER_SPEED * time.delta_secs())
            .clamp(-PLAYER_X_BOUND, PLAYER_X_BOUND);
}

pub fn player_shield_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut shield: ResMut<PlayerShield>,
    assets: Res<GameAssets>,
    player_query: Query<Entity, With<Player>>,
    mut bubble_query: Query<(&mut Sprite, &mut Transform), With<ShieldBubble>>,
) {
    let dt = time.delta_secs();
    if shield.active_remaining > 0.0 {
        shield.active_remaining = (shield.active_remaining - dt).max(0.0);
    } else if shield.cooldown_remaining > 0.0 {
        shield.cooldown_remaining = (shield.cooldown_remaining - dt).max(0.0);
    }

    // Only allow activating shield while player is alive on screen
    if player_query.get_single().is_ok() {
        let alt_held =
            keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
        if alt_held && keyboard.just_pressed(KeyCode::Enter) {
            shield.alt_combo_used = true;
        }

        let alt_just_released =
            keyboard.just_released(KeyCode::AltLeft) || keyboard.just_released(KeyCode::AltRight);
        let alt_shield_trigger = if alt_just_released {
            let trigger = !shield.alt_combo_used;
            shield.alt_combo_used = false;
            trigger
        } else {
            false
        };

        let b_just_pressed = gamepads
            .iter()
            .any(|gp| gp.just_pressed(GamepadButton::East));

        if (alt_shield_trigger || b_just_pressed) && shield.is_ready() {
            shield.active_remaining = SHIELD_DURATION_SECS;
            shield.cooldown_remaining = SHIELD_COOLDOWN_SECS;
            play_sfx(&mut commands, assets.sfx_shield_up.clone());
        }
    }

    if let Ok((mut sprite, mut transform)) = bubble_query.get_single_mut() {
        if shield.is_active() {
            let pulse = (time.elapsed_secs() * 14.0).sin() * 0.5 + 0.5;
            let size = 62.0 + pulse * 6.0;
            sprite.custom_size = Some(Vec2::splat(size));
            sprite.color = Color::srgba(0.15, 0.85, 1.0, 0.32 + pulse * 0.18);
            transform.rotation = Quat::from_rotation_z(time.elapsed_secs() * 2.5);
        } else {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }
}

pub fn player_respawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut respawn: ResMut<PlayerRespawn>,
    mut shield: ResMut<PlayerShield>,
    assets: Res<GameAssets>,
) {
    if let Some(ref mut timer) = respawn.timer {
        timer.tick(time.delta());
        if timer.finished() {
            respawn.timer = None;
            spawn_player_entities(&mut commands, &assets);
            // Grant a brief spawn-protection shield upon respawning
            shield.active_remaining = 1.5;
        }
    }
}

pub fn player_weapon_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    assets: Res<GameAssets>,
    mut charge: ResMut<PlayerCharge>,
    mut player_query: Query<(&Transform, &mut Sprite), With<Player>>,
    mut orb_query: Query<(&mut Sprite, &mut Transform), (With<ChargeIndicatorOrb>, Without<Player>)>,
    active_shots_query: Query<&PlayerProjectile>,
) {
    let Ok((player_transform, mut player_sprite)) = player_query.get_single_mut() else {
        // Reset charge state if player is currently destroyed and waiting to respawn
        charge.hold_seconds = 0.0;
        charge.is_charging = false;
        charge.was_fire_held = false;
        return;
    };

    let active_shots = active_shots_query.iter().count();
    let can_fire = active_shots < MAX_PLAYER_SHOTS;

    // Fire input: Space on keyboard OR 'A' button (GamepadButton::South) on Xbox controller
    let gamepad_fire_pressed = gamepads
        .iter()
        .any(|gp| gp.pressed(GamepadButton::South));
    let fire_pressed = keyboard.pressed(KeyCode::Space) || gamepad_fire_pressed;
    let fire_just_released = !fire_pressed && charge.was_fire_held;
    charge.was_fire_held = fire_pressed;

    let was_charged = charge.is_fully_charged();

    // Begin or continue charging while Space or Xbox 'A' is held
    if fire_pressed {
        if can_fire {
            charge.is_charging = true;
            charge.hold_seconds += time.delta_secs();
            if !was_charged && charge.is_fully_charged() {
                play_sfx(&mut commands, assets.sfx_charge_ready.clone());
            }
        } else {
            // Cannot start/accumulate charge while 3 shots are already on screen
            charge.is_charging = false;
            charge.hold_seconds = 0.0;
        }
    }

    // Update ship visual state & charging orb while charging
    let progress = charge.progress();
    let fully_charged = charge.is_fully_charged();

    if let Some(atlas) = player_sprite.texture_atlas.as_mut() {
        atlas.index = if fully_charged { 1 } else { 0 };
    }

    if let Ok((mut orb_sprite, mut orb_transform)) = orb_query.get_single_mut() {
        if charge.is_charging && charge.hold_seconds > 0.1 {
            let pulse = (time.elapsed_secs() * 18.0).sin() * 0.5 + 0.5;
            let size = 8.0 + progress * 18.0 + pulse * 4.0;
            orb_sprite.custom_size = Some(Vec2::splat(size));
            orb_transform.rotation = Quat::from_rotation_z(time.elapsed_secs() * 6.0);

            if fully_charged {
                orb_sprite.color = Color::srgba(1.0, 0.95, 0.25, 0.95);
            } else {
                orb_sprite.color = Color::srgba(0.1, 0.85, 1.0, 0.35 + 0.55 * progress);
            }
        } else {
            orb_sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }

    // Fire when Space or Xbox 'A' is released
    if fire_just_released {
        if can_fire {
            let spawn_pos = player_transform.translation + Vec3::new(0.0, 28.0, 0.0);
            if fully_charged {
                play_sfx(&mut commands, assets.sfx_mega_beam.clone());
                // Spawn Piercing Charged Mega-Beam (destroys multiple ships!)
                commands
                    .spawn((
                        PlayerProjectile {
                            piercing: true,
                            speed: 620.0,
                            hitbox: Vec2::new(26.0, 56.0),
                        },
                        GameEntity,
                        Sprite {
                            color: Color::srgb(1.0, 0.92, 0.2),
                            custom_size: Some(Vec2::new(18.0, 54.0)),
                            ..default()
                        },
                        Transform::from_translation(spawn_pos),
                    ))
                    .with_children(|parent| {
                        // Outer plasma aura
                        parent.spawn(Sprite {
                            color: Color::srgba(0.1, 0.95, 1.0, 0.55),
                            custom_size: Some(Vec2::new(30.0, 62.0)),
                            ..default()
                        });
                    });
            } else {
                play_sfx(&mut commands, assets.sfx_laser.clone());
                // Spawn Standard Laser Shot
                commands.spawn((
                    PlayerProjectile {
                        piercing: false,
                        speed: 500.0,
                        hitbox: Vec2::new(6.0, 20.0),
                    },
                    GameEntity,
                    Sprite {
                        color: Color::srgb(0.2, 1.0, 0.9),
                        custom_size: Some(Vec2::new(5.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(spawn_pos),
                ));
            }
        }

        charge.hold_seconds = 0.0;
        charge.is_charging = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_progress_and_threshold() {
        let mut charge = PlayerCharge::default();
        assert!(!charge.is_fully_charged());
        assert_eq!(charge.progress(), 0.0);

        charge.hold_seconds = 1.0;
        assert!(!charge.is_fully_charged());
        assert!((charge.progress() - 0.5).abs() < f32::EPSILON);

        charge.hold_seconds = 2.0;
        assert!(charge.is_fully_charged());
        assert_eq!(charge.progress(), 1.0);

        charge.hold_seconds = 3.2;
        assert!(charge.is_fully_charged());
        assert_eq!(charge.progress(), 1.0);
    }

    #[test]
    fn test_shield_state() {
        let mut shield = PlayerShield::default();
        assert!(shield.is_ready());
        assert!(!shield.is_active());

        shield.active_remaining = SHIELD_DURATION_SECS;
        shield.cooldown_remaining = SHIELD_COOLDOWN_SECS;
        assert!(shield.is_active());
        assert!(!shield.is_ready());
    }
}
