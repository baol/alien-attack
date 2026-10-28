use std::collections::HashSet;

use bevy::prelude::*;

use crate::aliens::Alien;
use crate::player::{Player, PlayerRespawn, PlayerShield, RESPAWN_DELAY_SECS};
use crate::{GameEntity, GameState, PlayerStats};

#[derive(Component)]
pub struct PlayerProjectile {
    pub piercing: bool,
    pub speed: f32,
    pub hitbox: Vec2,
}

#[derive(Component)]
pub struct AlienProjectile {
    pub speed: f32,
    pub hitbox: Vec2,
}

#[derive(Component)]
pub struct ExplosionParticle {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut player_shots: Query<(Entity, &PlayerProjectile, &mut Transform), Without<AlienProjectile>>,
    mut alien_shots: Query<(Entity, &AlienProjectile, &mut Transform), Without<PlayerProjectile>>,
) {
    for (entity, shot, mut transform) in &mut player_shots {
        transform.translation.y += shot.speed * time.delta_secs();
        if transform.translation.y > 315.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, shot, mut transform) in &mut alien_shots {
        transform.translation.y -= shot.speed * time.delta_secs();
        if transform.translation.y < -315.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn aabb_overlap(pos_a: Vec2, size_a: Vec2, pos_b: Vec2, size_b: Vec2) -> bool {
    let half_a = size_a * 0.5;
    let half_b = size_b * 0.5;
    (pos_a.x - pos_b.x).abs() < (half_a.x + half_b.x)
        && (pos_a.y - pos_b.y).abs() < (half_a.y + half_b.y)
}

pub fn spawn_explosion(commands: &mut Commands, position: Vec3, color: Color) {
    let offsets = [
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(0.0, 1.3),
        Vec2::new(0.0, -1.3),
        Vec2::new(-1.3, 0.0),
        Vec2::new(1.3, 0.0),
    ];

    for dir in offsets {
        commands.spawn((
            ExplosionParticle {
                velocity: dir.normalize_or_zero() * 120.0,
                lifetime: Timer::from_seconds(0.28, TimerMode::Once),
            },
            GameEntity,
            Sprite {
                color,
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
}

fn spawn_ship_destruction_explosion(commands: &mut Commands, position: Vec3) {
    // Multi-ring fiery explosion when the player ship is destroyed
    spawn_explosion(commands, position, Color::srgb(1.0, 0.35, 0.15));
    let outer_dirs = [
        Vec2::new(-1.5, 0.7),
        Vec2::new(1.5, 0.7),
        Vec2::new(-1.5, -0.7),
        Vec2::new(1.5, -0.7),
        Vec2::new(-0.7, 1.5),
        Vec2::new(0.7, 1.5),
        Vec2::new(-0.7, -1.5),
        Vec2::new(0.7, -1.5),
    ];

    for dir in outer_dirs {
        commands.spawn((
            ExplosionParticle {
                velocity: dir.normalize_or_zero() * 195.0,
                lifetime: Timer::from_seconds(0.45, TimerMode::Once),
            },
            GameEntity,
            Sprite {
                color: Color::srgb(1.0, 0.85, 0.2),
                custom_size: Some(Vec2::splat(8.0)),
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
}

pub fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionParticle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            transform.translation.x += particle.velocity.x * time.delta_secs();
            transform.translation.y += particle.velocity.y * time.delta_secs();
            let alpha = 1.0 - particle.lifetime.fraction();
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    mut stats: ResMut<PlayerStats>,
    shield: Res<PlayerShield>,
    mut respawn: ResMut<PlayerRespawn>,
    mut next_state: ResMut<NextState<GameState>>,
    player_shots: Query<(Entity, &PlayerProjectile, &Transform)>,
    aliens: Query<(Entity, &Alien, &Transform)>,
    alien_shots: Query<(Entity, &AlienProjectile, &Transform)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
) {
    let alien_size = Vec2::new(32.0, 28.0);
    let mut destroyed_aliens = HashSet::new();

    // 1. Player projectiles vs Aliens
    for (shot_entity, shot, shot_transform) in &player_shots {
        let shot_pos = shot_transform.translation.truncate();

        for (alien_entity, alien, alien_transform) in &aliens {
            if destroyed_aliens.contains(&alien_entity) {
                continue;
            }

            let alien_pos = alien_transform.translation.truncate();
            if aabb_overlap(shot_pos, shot.hitbox, alien_pos, alien_size) {
                destroyed_aliens.insert(alien_entity);
                commands.entity(alien_entity).despawn_recursive();
                stats.score += alien.points;

                let burst_color = if shot.piercing {
                    Color::srgb(1.0, 0.95, 0.3)
                } else {
                    Color::srgb(0.3, 1.0, 0.8)
                };
                spawn_explosion(&mut commands, alien_transform.translation, burst_color);

                // Standard shot stops after 1 ship; Charged shot pierces through multiple ships!
                if !shot.piercing {
                    commands.entity(shot_entity).despawn_recursive();
                    break;
                }
            }
        }
    }

    // 2. Alien projectiles & Roaming Aliens vs Player
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let player_size = if shield.is_active() {
        Vec2::new(56.0, 56.0)
    } else {
        Vec2::new(36.0, 28.0)
    };

    for (shot_entity, shot, shot_transform) in &alien_shots {
        let shot_pos = shot_transform.translation.truncate();
        if aabb_overlap(shot_pos, shot.hitbox, player_pos, player_size) {
            commands.entity(shot_entity).despawn_recursive();

            if shield.is_active() {
                // Shield absorbs enemy fire with a cyan spark burst!
                spawn_explosion(
                    &mut commands,
                    shot_transform.translation,
                    Color::srgb(0.2, 0.9, 1.0),
                );
                continue;
            }

            // Player hit without shield: explode and either respawn or Game Over
            spawn_ship_destruction_explosion(&mut commands, player_transform.translation);
            commands.entity(player_entity).despawn_recursive();

            if stats.lives > 1 {
                stats.lives -= 1;
                respawn.timer = Some(Timer::from_seconds(RESPAWN_DELAY_SECS, TimerMode::Once));
            } else {
                stats.lives = 0;
                next_state.set(GameState::GameOver);
            }
            return;
        }
    }

    // 3. Roaming/Alien body vs Player collision
    for (alien_entity, alien, alien_transform) in &aliens {
        if destroyed_aliens.contains(&alien_entity) {
            continue;
        }
        let alien_pos = alien_transform.translation.truncate();
        if aabb_overlap(alien_pos, alien_size, player_pos, player_size) {
            commands.entity(alien_entity).despawn_recursive();

            if shield.is_active() {
                // Shield vaporizes colliding alien and awards points!
                stats.score += alien.points;
                spawn_explosion(
                    &mut commands,
                    alien_transform.translation,
                    Color::srgb(0.2, 0.9, 1.0),
                );
                continue;
            }

            spawn_ship_destruction_explosion(&mut commands, player_transform.translation);
            commands.entity(player_entity).despawn_recursive();

            if stats.lives > 1 {
                stats.lives -= 1;
                respawn.timer = Some(Timer::from_seconds(RESPAWN_DELAY_SECS, TimerMode::Once));
            } else {
                stats.lives = 0;
                next_state.set(GameState::GameOver);
            }
            return;
        }
    }
}
