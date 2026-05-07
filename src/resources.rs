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
