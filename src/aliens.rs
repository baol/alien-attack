use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::assets::{play_sfx, GameAssets};
use crate::combat::{AlienProjectile, PlayerProjectile};
use crate::player::PLAYER_Y;
use crate::{GameEntity, GameState, PlayerStats};

pub const ALIEN_ROWS: usize = 5;
pub const ALIEN_COLS: usize = 11;
pub const ALIEN_SPACING_X: f32 = 52.0;
pub const ALIEN_SPACING_Y: f32 = 42.0;
pub const ALIEN_START_Y: f32 = 210.0;
pub const ALIEN_X_LIMIT: f32 = 365.0;

#[derive(Component)]
pub struct Alien {
    pub points: u32,
    pub col: usize,
    pub row: usize,
    pub local_offset: Vec2,
    pub is_last_row: bool,
}

#[derive(Component)]
pub struct RoamingAlien {
    pub elapsed: f32,
    pub duration: f32,
    pub start_pos: Vec2,
    pub freq_x: f32,
    pub amp_x: f32,
    pub dive_depth: f32,
    pub fire_timer: Timer,
}

#[derive(Component)]
pub struct AlienAnimation {
    pub first_index: usize,
    pub last_index: usize,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct AlienFormation {
    pub origin: Vec2,
    pub direction: f32,
    pub base_speed: f32,
    pub total_spawned: usize,
    pub shots_per_volley: usize,
    pub fire_timer: Timer,
    pub roam_timer: Timer,
}

impl AlienFormation {
    pub fn for_level(level: u32) -> Self {
        if level == 1 {
            Self {
                origin: Vec2::new(0.0, ALIEN_START_Y),
                direction: 1.0,
                base_speed: 45.0,
                total_spawned: 55,
                shots_per_volley: 1,
                fire_timer: Timer::from_seconds(1.15, TimerMode::Repeating),
                roam_timer: Timer::from_seconds(4.2, TimerMode::Repeating),
            }
        } else {
            Self {
                origin: Vec2::new(0.0, ALIEN_START_Y + 12.0),
                direction: 1.0,
                base_speed: 68.0,
                total_spawned: 45,
                shots_per_volley: 2,
                fire_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
                roam_timer: Timer::from_seconds(2.3, TimerMode::Repeating),
            }
        }
    }
}

pub fn spawn_alien_entities(commands: &mut Commands, assets: &GameAssets, level: u32) {
    let mut formation = AlienFormation::for_level(level);
    let total_width = (ALIEN_COLS - 1) as f32 * ALIEN_SPACING_X;
    let start_x = -total_width / 2.0;

    // Collect valid (row, col, y_wave_offset) slots depending on the level's formation
    let mut layout_slots = Vec::new();
    let mut max_row_for_col = [0usize; ALIEN_COLS];

    for row in 0..ALIEN_ROWS {
        for col in 0..ALIEN_COLS {
            let dist_center = (col as isize - 5).unsigned_abs();
            let include = if level == 1 {
                true
            } else {
                // Level 2: Twin-Wing Chevron / Diamond Armada formation
                match row {
                    0 => dist_center <= 3,
                    1 => dist_center >= 1 && dist_center <= 4,
                    2 => col != 5,
                    3 => dist_center != 2,
                    4 => col % 2 == 0 || dist_center <= 1,
                    _ => true,
                }
            };

            if include {
                let y_arch = if level == 1 {
                    0.0
                } else {
                    // V-wing vertical stagger in Level 2
                    (dist_center as f32) * -6.5
                };
                layout_slots.push((row, col, y_arch));
                max_row_for_col[col] = max_row_for_col[col].max(row);
            }
        }
    }

    formation.total_spawned = layout_slots.len();
    let origin = formation.origin;
    commands.insert_resource(formation);

    for (row, col, y_arch) in layout_slots {
        let points = match row {
            0 => 50,
            1 => 40,
            2 => 30,
            3 => 20,
            _ => 10,
        } * level;

        let first_index = row * 4;
        let last_index = first_index + 3;

        let local_x = start_x + col as f32 * ALIEN_SPACING_X;
        let local_y = -(row as f32 * ALIEN_SPACING_Y) + y_arch;
        let local_offset = Vec2::new(local_x, local_y);
        let world_pos = origin + local_offset;

        let mut anim_timer = Timer::from_seconds(0.30, TimerMode::Repeating);
        anim_timer.tick(std::time::Duration::from_secs_f32((col as f32) * 0.04));

        // Mark bottom-most row aliens so they can detach and fly around from time to time
        let is_last_row = row == ALIEN_ROWS - 1 || row == max_row_for_col[col];

        commands.spawn((
            Alien {
                points,
                col,
                row,
                local_offset,
                is_last_row,
            },
            AlienAnimation {
                first_index,
                last_index,
                timer: anim_timer,
            },
            GameEntity,
            Sprite {
                image: assets.alien_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.alien_layout.clone(),
                    index: first_index + (col % 2),
                }),
                custom_size: Some(Vec2::new(36.0, 36.0)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        ));
    }
}

pub fn spawn_aliens(mut commands: Commands, assets: Res<GameAssets>, stats: Res<PlayerStats>) {
    spawn_alien_entities(&mut commands, &assets, stats.level);
}

pub fn animate_aliens(time: Res<Time>, mut query: Query<(&mut AlienAnimation, &mut Sprite)>) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index >= anim.last_index || atlas.index < anim.first_index {
                    atlas.index = anim.first_index;
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

pub fn move_alien_formation(
    mut commands: Commands,
    time: Res<Time>,
    mut formation: ResMut<AlienFormation>,
    mut stats: ResMut<PlayerStats>,
    assets: Res<GameAssets>,
    mut formation_aliens: Query<(&Alien, &mut Transform), Without<RoamingAlien>>,
    all_aliens: Query<Entity, With<Alien>>,
    active_projectiles: Query<Entity, Or<(With<AlienProjectile>, With<PlayerProjectile>)>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let remaining = all_aliens.iter().count();
    if remaining == 0 {
        if stats.level < 2 {
            // Advance to Level 2 automatically!
            stats.level = 2;
            play_sfx(&mut commands, assets.sfx_victory.clone());
            for proj in &active_projectiles {
                commands.entity(proj).despawn_recursive();
            }
            spawn_alien_entities(&mut commands, &assets, stats.level);
        } else {
            next_state.set(GameState::Victory);
        }
        return;
    }

    // Speed scales up dynamically as more aliens are destroyed
    let total_aliens = formation.total_spawned.max(1) as f32;
    let eliminated_ratio = 1.0 - (remaining as f32 / total_aliens);
    let current_speed = formation.base_speed + eliminated_ratio * 200.0;

    let dx = formation.direction * current_speed * time.delta_secs();
    let mut hit_edge = false;
    let mut lowest_y = f32::MAX;

    for (alien, _) in &formation_aliens {
        let next_x = formation.origin.x + dx + alien.local_offset.x;
        let current_y = formation.origin.y + alien.local_offset.y;
        if (formation.direction > 0.0 && next_x > ALIEN_X_LIMIT)
            || (formation.direction < 0.0 && next_x < -ALIEN_X_LIMIT)
        {
            hit_edge = true;
        }
        lowest_y = lowest_y.min(current_y);
    }

    if hit_edge {
        formation.direction *= -1.0;
        formation.origin.y -= 18.0;
    } else {
        formation.origin.x += dx;
    }

    for (alien, mut transform) in &mut formation_aliens {
        let target = formation.origin + alien.local_offset;
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }

    // If formation reaches the player's vertical zone, trigger Game Over
    if lowest_y <= PLAYER_Y + 32.0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn trigger_last_row_roaming(
    mut commands: Commands,
    time: Res<Time>,
    mut formation: ResMut<AlienFormation>,
    stats: Res<PlayerStats>,
    candidates: Query<(Entity, &Alien, &Transform), Without<RoamingAlien>>,
    roaming_query: Query<&RoamingAlien>,
) {
    formation.roam_timer.tick(time.delta());
    if !formation.roam_timer.just_finished() {
        return;
    }

    let max_concurrent = if stats.level == 1 { 1 } else { 3 };
    if roaming_query.iter().count() >= max_concurrent {
        return;
    }

    // Collect last-row aliens (or front-most aliens in their column) that aren't currently roaming
    let eligible: Vec<(Entity, Vec2)> = candidates
        .iter()
        .filter(|(_, alien, _)| alien.is_last_row)
        .map(|(e, _, t)| (e, t.translation.truncate()))
        .collect();

    let mut rng = rand::thread_rng();
    if let Some(&(entity, start_pos)) = eligible.choose(&mut rng) {
        let duration = rng.gen_range(3.6..5.0);
        let freq_x = if rng.gen_bool(0.5) { 1.0 } else { -1.0 } * rng.gen_range(1.5..2.4);
        let amp_x = rng.gen_range(95.0..175.0);
        let dive_depth = rng.gen_range(130.0..235.0);
        let fire_interval = if stats.level == 1 { 1.1 } else { 0.8 };

        commands.entity(entity).insert(RoamingAlien {
            elapsed: 0.0,
            duration,
            start_pos,
            freq_x,
            amp_x,
            dive_depth,
            fire_timer: Timer::from_seconds(fire_interval, TimerMode::Repeating),
        });
    }
}

pub fn update_roaming_aliens(
    mut commands: Commands,
    time: Res<Time>,
    formation: Res<AlienFormation>,
    stats: Res<PlayerStats>,
    assets: Res<GameAssets>,
    mut query: Query<(Entity, &Alien, &mut Transform, &mut RoamingAlien)>,
) {
    for (entity, alien, mut transform, mut roam) in &mut query {
        roam.elapsed += time.delta_secs();
        let t = (roam.elapsed / roam.duration).clamp(0.0, 1.0);
        let home_pos = formation.origin + alien.local_offset;

        if t >= 1.0 {
            // Rejoin formation smoothly and reset tilt
            transform.translation.x = home_pos.x;
            transform.translation.y = home_pos.y;
            transform.rotation = Quat::IDENTITY;
            commands.entity(entity).remove::<RoamingAlien>();
            continue;
        }

        // Smoothly interpolate base anchor from launch position back to moving formation slot
        let anchor = roam.start_pos.lerp(home_pos, t);
        let envelope = (t * std::f32::consts::PI).sin();

        let prev_pos = transform.translation.truncate();
        let sway_x = (t * std::f32::consts::TAU * roam.freq_x).sin() * roam.amp_x * envelope;
        let swoop_y = -roam.dive_depth * envelope;

        let new_x = (anchor.x + sway_x).clamp(-ALIEN_X_LIMIT, ALIEN_X_LIMIT);
        let new_y = (anchor.y + swoop_y).max(PLAYER_Y + 42.0);

        transform.translation.x = new_x;
        transform.translation.y = new_y;

        // Slight banking tilt in the direction of horizontal flight
        let vx = (new_x - prev_pos.x) / time.delta_secs().max(0.001);
        let bank_angle = (-vx * 0.0015).clamp(-0.45, 0.45) * envelope;
        transform.rotation = Quat::from_rotation_z(bank_angle);

        // Roaming aliens fire extra shots while swooping around!
        roam.fire_timer.tick(time.delta());
        if roam.fire_timer.just_finished() {
            play_sfx(&mut commands, assets.sfx_alien_laser.clone());
            let bullet_speed = if stats.level == 1 { 265.0 } else { 320.0 };
            commands.spawn((
                AlienProjectile {
                    speed: bullet_speed,
                    hitbox: Vec2::new(7.0, 16.0),
                },
                GameEntity,
                Sprite {
                    color: Color::srgb(1.0, 0.55, 0.15),
                    custom_size: Some(Vec2::new(7.0, 16.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(0.0, -20.0, 0.0)),
            ));
        }
    }
}

pub fn alien_shooting_system(
    mut commands: Commands,
    time: Res<Time>,
    mut formation: ResMut<AlienFormation>,
    stats: Res<PlayerStats>,
    assets: Res<GameAssets>,
    query: Query<(&Alien, &Transform)>,
) {
    formation.fire_timer.tick(time.delta());
    if !formation.fire_timer.just_finished() {
        return;
    }

    // Find bottom-most alien in each column so front-line aliens fire
    let mut bottom_aliens_by_col: [Option<(usize, Vec3)>; ALIEN_COLS] = [None; ALIEN_COLS];
    for (alien, transform) in &query {
        if alien.col < ALIEN_COLS {
            match bottom_aliens_by_col[alien.col] {
                Some((existing_row, _)) if existing_row >= alien.row => {}
                _ => {
                    bottom_aliens_by_col[alien.col] = Some((alien.row, transform.translation));
                }
            }
        }
    }

    let mut shooters: Vec<Vec3> = bottom_aliens_by_col
        .iter()
        .filter_map(|entry| entry.map(|(_, pos)| pos))
        .collect();

    let mut rng = rand::thread_rng();
    shooters.shuffle(&mut rng);

    let bullet_speed = if stats.level == 1 { 255.0 } else { 315.0 };
    if !shooters.is_empty() {
        play_sfx(&mut commands, assets.sfx_alien_laser.clone());
    }
    for &shooter_pos in shooters.iter().take(formation.shots_per_volley) {
        commands.spawn((
            AlienProjectile {
                speed: bullet_speed,
                hitbox: Vec2::new(6.0, 16.0),
            },
            GameEntity,
            Sprite {
                color: Color::srgb(1.0, 0.25, 0.45),
                custom_size: Some(Vec2::new(6.0, 16.0)),
                ..default()
            },
            Transform::from_translation(shooter_pos + Vec3::new(0.0, -20.0, 0.0)),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_formation_configs() {
        let lvl1 = AlienFormation::for_level(1);
        let lvl2 = AlienFormation::for_level(2);

        assert!(lvl2.base_speed > lvl1.base_speed);
        assert!(lvl2.shots_per_volley > lvl1.shots_per_volley);
        assert!(lvl2.fire_timer.duration() < lvl1.fire_timer.duration());
        assert!(lvl2.roam_timer.duration() < lvl1.roam_timer.duration());
    }
}

