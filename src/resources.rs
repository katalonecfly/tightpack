use crate::config::RawPieceConfig;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct PieceLibrary(pub Vec<RawPieceConfig>);

#[derive(Resource, Default)]
pub struct GameState {
    pub board_cells: HashMap<IVec2, LinearRgba>,
    pub score: i32,
}

#[derive(Resource, Default)]
pub struct TooltipState {
    pub entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct InventoryScroll {
    pub offset: f32,    // positive = scroll down (pieces move up)
}

#[derive(Resource, Default)]
pub struct StashContentHeight(pub f32);

#[derive(Resource, Default)]
pub struct StashScreenRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct DuelState {
    pub player: crate::resources::GameState,
    pub opponent: crate::resources::GameState,
}
impl Default for DuelState {
    fn default() -> Self {
        Self {
            player: GameState::default(),
            opponent: GameState::default(),
        }
    }
}