use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub alien_texture: Handle<Image>,
    pub alien_layout: Handle<TextureAtlasLayout>,
    pub player_texture: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
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
    });
}
