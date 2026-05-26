use crate::config::RawPieceConfig;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct PieceLibrary(pub Vec<RawPieceConfig>);

#[derive(Resource, Default)]
pub struct GameState {
    pub board_cells: HashMap<IVec2, LinearRgba>,
    pub disabled_cells: HashSet<IVec2>,
    pub score: i32,
}

#[derive(Resource, Default)]
pub struct TooltipState {
    pub entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct InventoryScroll {
    pub offset: f32,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DuelMode {
    Basic,
    Destroy,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DuelTurn {
    #[default]
    Place,
    Destroy,
}

#[derive(Resource)]
pub struct DuelState {
    pub player: GameState,
    pub opponent: GameState,
    pub mode: DuelMode,
    pub turn: DuelTurn,
    pub pending_disable: Option<IVec2>,
    pub pending_disable_preview: Option<(Entity, Entity)>,
}

impl Default for DuelState {
    fn default() -> Self {
        Self {
            player: GameState::default(),
            opponent: GameState::default(),
            mode: DuelMode::Basic,
            turn: DuelTurn::default(),
            pending_disable: None,
            pending_disable_preview: None,
        }
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub duel_blocking_enabled: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            duel_blocking_enabled: true,
        }
    }
}
