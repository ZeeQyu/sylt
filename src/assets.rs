// use crate::imports::*;
// use bevy_asset_loader::prelude::*;
//
// #[derive(Default)]
// pub struct GameAssetPlugin;
// impl Plugin for GameAssetPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_loading_state(
//             LoadingState::new(GameState::Loading)
//                 .continue_to_state(GameState::Editor)
//                 .with_collection::<GameAssets>()
//         );
//     }
// }
//
// #[derive(AssetCollection, Resource)]
// pub struct GameAssets {
//     #[asset(path = "fonts/eight-bit-dragon.font/EightBitDragon-anqx.ttf")]
//     pub font: Handle<Font>,
// }