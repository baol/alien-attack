use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub alien_texture: Handle<Image>,
    pub alien_layout: Handle<TextureAtlasLayout>,
    pub player_texture: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
    pub sfx_laser: Handle<AudioSource>,
    pub sfx_mega_beam: Handle<AudioSource>,
    pub sfx_charge_ready: Handle<AudioSource>,
    pub sfx_alien_laser: Handle<AudioSource>,
    pub sfx_alien_explode: Handle<AudioSource>,
    pub sfx_player_explode: Handle<AudioSource>,
    pub sfx_shield_up: Handle<AudioSource>,
    pub sfx_shield_hit: Handle<AudioSource>,
    pub sfx_victory: Handle<AudioSource>,
    pub sfx_game_over: Handle<AudioSource>,
}

pub fn play_sfx(commands: &mut Commands, source: Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(source), PlaybackSettings::DESPAWN));
}

pub fn load_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let alien_texture = asset_server.load("sprites/aliens.png");
    let alien_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        5,
        None,
        None,
    ));

    let player_texture = asset_server.load("sprites/player.png");
    let player_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        2,
        1,
        None,
        None,
    ));

    commands.insert_resource(GameAssets {
        alien_texture,
        alien_layout,
        player_texture,
        player_layout,
        sfx_laser: asset_server.load("sounds/laser.wav"),
        sfx_mega_beam: asset_server.load("sounds/mega_beam.wav"),
        sfx_charge_ready: asset_server.load("sounds/charge_ready.wav"),
        sfx_alien_laser: asset_server.load("sounds/alien_laser.wav"),
        sfx_alien_explode: asset_server.load("sounds/alien_explode.wav"),
        sfx_player_explode: asset_server.load("sounds/player_explode.wav"),
        sfx_shield_up: asset_server.load("sounds/shield_up.wav"),
        sfx_shield_hit: asset_server.load("sounds/shield_hit.wav"),
        sfx_victory: asset_server.load("sounds/victory.wav"),
        sfx_game_over: asset_server.load("sounds/game_over.wav"),
    });
}
