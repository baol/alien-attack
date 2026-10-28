use bevy::prelude::*;

use crate::assets::{play_sfx, GameAssets};
use crate::player::PlayerCharge;
use crate::{GameState, HighScore, PlayerStats};
use crate::save_highscore;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct LifeIcon {
    pub slot: u32,
}

#[derive(Component)]
pub struct ChargeBarFill;

#[derive(Component)]
pub struct OverlayRoot;

#[derive(Component)]
pub struct HelpOverlayRoot;

pub fn setup_ui(mut commands: Commands, assets: Res<GameAssets>) {
    // Top HUD: score, highscore, lives icons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("SCORE 0000"),
                TextFont {
                    font: assets.font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.95, 1.0)),
            ));

            parent.spawn((
                HighScoreText,
                Text::new("HI 0000"),
                TextFont {
                    font: assets.font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.25)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    for slot in 0..3u32 {
                        row.spawn((
                            LifeIcon { slot },
                            ImageNode::from_atlas_image(
                                assets.player_texture.clone(),
                                TextureAtlas {
                                    layout: assets.player_layout.clone(),
                                    index: 0,
                                },
                            ),
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                        ));
                    }
                });
        });

    // Bottom: charge bar only
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(8.0),
                        border: UiRect::all(Val::Px(1.0)),
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
}

pub fn update_hud(
    stats: Res<PlayerStats>,
    mut highscore: ResMut<HighScore>,
    charge: Res<PlayerCharge>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>)>,
    mut highscore_q: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>)>,
    mut lives_q: Query<(&mut Visibility, &LifeIcon)>,
    mut bar_q: Query<(&mut Node, &mut BackgroundColor), With<ChargeBarFill>>,
) {
    if stats.score > highscore.value {
        highscore.value = stats.score;
        save_highscore(highscore.value);
    }

    if let Ok(mut text) = score_q.get_single_mut() {
        let new_val = format!("SCORE {:04}", stats.score);
        if **text != new_val {
            **text = new_val;
        }
    }

    if let Ok(mut text) = highscore_q.get_single_mut() {
        let new_val = format!("HI {:04}", highscore.value);
        if **text != new_val {
            **text = new_val;
        }
    }

    for (mut vis, icon) in &mut lives_q {
        *vis = if icon.slot < stats.lives as u32 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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
}

pub fn toggle_help_overlay(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    existing: Query<Entity, With<HelpOverlayRoot>>,
    assets: Res<GameAssets>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) || keyboard.just_pressed(KeyCode::F1) {
        if let Some(entity) = existing.iter().next() {
            commands.entity(entity).despawn_recursive();
        } else {
            spawn_help_overlay(&mut commands, &assets);
        }
    }
}

fn spawn_help_overlay(commands: &mut Commands, assets: &GameAssets) {
    commands
        .spawn((
            HelpOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.88)),
        ))
        .with_children(|parent| {
            let lines = [
                "ALIEN ATTACK",
                "Arrows / Stick / Swipe: Move",
                "Space / A: Fire (hold 2s = Mega-Beam)",
                "Alt / B: Shield",
                "F / F11: Fullscreen",
                "R / Enter: Restart",
                "H / F1: Toggle Help",
            ];
            for (i, line) in lines.iter().enumerate() {
                parent.spawn((
                    Text::new(*line),
                    TextFont {
                        font: assets.font.clone(),
                        font_size: if i == 0 { 20.0 } else { 12.0 },
                        ..default()
                    },
                    TextColor(if i == 0 {
                        Color::srgb(0.3, 0.95, 1.0)
                    } else {
                        Color::srgb(0.85, 0.9, 1.0)
                    }),
                ));
            }
        });
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
                    font: assets.font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(title_color),
            ));

            parent.spawn((
                Text::new(format!("FINAL SCORE: {:04}", stats.score)),
                TextFont {
                    font: assets.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Press R, ENTER, or Controller START / A to Play Again"),
                TextFont {
                    font_size: 14.0,
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
