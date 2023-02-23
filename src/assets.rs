use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_yoleck::YoleckLevelIndex;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_collection::<GameAssets>();
    }
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "sprites/Collie-run-sheet.png")]
    pub player_sheet: Handle<Image>,
    pub player_columns: usize,
    #[asset(path = "sprites/Sheep-sheet.png")]
    pub sheep_sheet: Handle<Image>,
    #[asset(path = "sprites/Collie-run-sheet.png")]
    pub grass_sheet: Handle<Image>,

    //#[asset(path = "fonts/FiraSans-Bold.ttf")]
    //pub font: Handle<Font>,
    #[asset(path = "levels/index.yoli")]
    pub level_index: Handle<YoleckLevelIndex>,
}
