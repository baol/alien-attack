use bevy::prelude::*;

use crate::assets::{play_sfx, GameAssets};
use crate::combat::PlayerProjectile;
use crate::player::{PlayerCharge, PlayerShield, MAX_PLAYER_SHOTS};
use crate::{GameState, PlayerStats};

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct ShieldText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct ShotsText;

#[derive(Component)]
pub struct ChargeBarFill;

#[derive(Component)]
pub struct ChargeStatusText;

#[derive(Component)]
pub struct OverlayRoot;

pub fn setup_ui(mut commands: Commands) {
    // Top HUD bar
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(44.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.12, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("SCORE: 0000"),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.95, 1.0)),
            ));

            parent.spawn((
                LevelText,
                Text::new("LEVEL: 1 / 2"),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.25)),
            ));

            parent.spawn((
                ShieldText,
                Text::new("SHIELD: READY"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.95, 0.7)),
            ));

            parent.spawn((
                ShotsText,
                Text::new("SHOTS: 0 / 3"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
            ));

            parent.spawn((
                LivesText,
                Text::new("LIVES: 3"),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.65)),
            ));
        });

    // Bottom Charge Bar & Controls HUD
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            height: Val::Px(28.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "Arrows/Stick/Swipe: Move | Space/A: Fire (2s Beam) | Alt/B: Shield | F: Full",
                ),
                TextFont {
                    font_size: 11.5,
                    ..default()
                },
                TextColor(Color::srgba(0.75, 0.8, 0.95, 0.85)),
            ));

            // Charge container
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ChargeStatusText,
                        Text::new("CHARGE (2s):"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.85, 1.0)),
                    ));

                    row.spawn((
                        Node {
                            width: Val::Px(130.0),
                            height: Val::Px(14.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.35, 0.6, 0.9)),
                        BackgroundColor(Color::srgb(0.08, 0.1, 0.18)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            ChargeBarFill,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.85, 1.0)),
                        ));
                    });
                });
        });
}

pub fn update_hud(
    stats: Res<PlayerStats>,
    charge: Res<PlayerCharge>,
    shield: Res<PlayerShield>,
    active_shots: Query<&PlayerProjectile>,
    mut score_q: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<LevelText>,
            Without<ShieldText>,
            Without<LivesText>,
            Without<ShotsText>,
            Without<ChargeStatusText>,
        ),
    >,
    mut level_q: Query<
        &mut Text,
        (
            With<LevelText>,
            Without<ScoreText>,
            Without<ShieldText>,
            Without<LivesText>,
            Without<ShotsText>,
            Without<ChargeStatusText>,
        ),
    >,
    mut shield_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ShieldText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<LivesText>,
            Without<ShotsText>,
            Without<ChargeStatusText>,
        ),
    >,
    mut lives_q: Query<
        &mut Text,
        (
            With<LivesText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ShieldText>,
            Without<ShotsText>,
            Without<ChargeStatusText>,
        ),
    >,
    mut shots_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ShotsText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ShieldText>,
            Without<LivesText>,
            Without<ChargeStatusText>,
        ),
    >,
    mut status_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ChargeStatusText>,
            Without<ScoreText>,
            Without<LevelText>,
            Without<ShieldText>,
            Without<LivesText>,
            Without<ShotsText>,
        ),
    >,
    mut bar_q: Query<(&mut Node, &mut BackgroundColor), With<ChargeBarFill>>,
) {
    if let Ok(mut text) = score_q.get_single_mut() {
        let new_val = format!("SCORE: {:04}", stats.score);
        if **text != new_val {
            **text = new_val;
        }
    }

    if let Ok(mut text) = level_q.get_single_mut() {
        let new_val = format!("LEVEL: {} / 2", stats.level);
        if **text != new_val {
            **text = new_val;
        }
    }

    if let Ok((mut text, mut color)) = shield_q.get_single_mut() {
        let (new_val, new_col) = if shield.is_active() {
            (
                format!("SHIELD: {:.1}s", shield.active_remaining),
                Color::srgb(0.2, 0.95, 1.0),
            )
        } else if shield.cooldown_remaining > 0.0 {
            (
                format!("SHIELD CD: {:.1}s", shield.cooldown_remaining),
                Color::srgb(0.65, 0.65, 0.75),
            )
        } else {
            ("SHIELD: READY".to_string(), Color::srgb(0.2, 0.95, 0.7))
        };
        if **text != new_val {
            **text = new_val;
            *color = TextColor(new_col);
        }
    }

    if let Ok(mut text) = lives_q.get_single_mut() {
        let new_val = format!("LIVES: {}", stats.lives);
        if **text != new_val {
            **text = new_val;
        }
    }

    let shot_count = active_shots.iter().count();
    if let Ok((mut text, mut color)) = shots_q.get_single_mut() {
        let new_val = format!("SHOTS: {} / {}", shot_count, MAX_PLAYER_SHOTS);
        if **text != new_val {
            **text = new_val;
            if shot_count >= MAX_PLAYER_SHOTS {
                *color = TextColor(Color::srgb(1.0, 0.45, 0.35));
            } else {
                *color = TextColor(Color::srgb(0.85, 0.9, 1.0));
            }
        }
    }

    let progress = charge.progress();
    let ready = charge.is_fully_charged();

    if let Ok((mut node, mut bg)) = bar_q.get_single_mut() {
        let target_width = Val::Percent((progress * 100.0).round());
        if node.width != target_width {
            node.width = target_width;
        }
        let target_bg = if ready {
            Color::srgb(1.0, 0.92, 0.2)
        } else {
            Color::srgb(0.1, 0.85, 1.0)
        };
        if bg.0 != target_bg {
            *bg = BackgroundColor(target_bg);
        }
    }

    if let Ok((mut text, mut color)) = status_q.get_single_mut() {
        let (new_val, new_col) = if ready {
            ("MEGA-BEAM READY!".to_string(), Color::srgb(1.0, 0.92, 0.2))
        } else if charge.is_charging {
            (
                format!("CHARGING {:.1}s", charge.hold_seconds),
                Color::srgb(0.3, 0.9, 1.0),
            )
        } else {
            ("CHARGE (2s):".to_string(), Color::srgb(0.6, 0.85, 1.0))
        };
        if **text != new_val {
            **text = new_val;
            *color = TextColor(new_col);
        }
    }
}

pub fn spawn_end_overlay(
    mut commands: Commands,
    state: Res<State<GameState>>,
    stats: Res<PlayerStats>,
    assets: Res<GameAssets>,
) {
    let (title, title_color) = match state.get() {
        GameState::Victory => {
            play_sfx(&mut commands, assets.sfx_victory.clone());
            ("VICTORY! SECTOR CLEARED", Color::srgb(0.3, 1.0, 0.55))
        }
        GameState::GameOver => {
            play_sfx(&mut commands, assets.sfx_game_over.clone());
            ("GAME OVER", Color::srgb(1.0, 0.3, 0.45))
        }
        GameState::Playing => return,
    };

    commands
        .spawn((
            OverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.82)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(title_color),
            ));

            parent.spawn((
                Text::new(format!("FINAL SCORE: {:04}", stats.score)),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Press R, ENTER, or Controller START / A to Play Again"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.9, 0.35)),
            ));
        });
}

pub fn despawn_end_overlay(mut commands: Commands, query: Query<Entity, With<OverlayRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
