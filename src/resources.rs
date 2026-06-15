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

#[derive(Resource)]
pub struct BoardSize(pub IVec2);

impl Default for BoardSize {
    fn default() -> Self {
        Self(IVec2::new(10, 10))
    }
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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AIType {
    #[default]
    Dummy,
    Greedy,
    Random,
}

#[derive(Resource)]
pub struct GameSettings {
    pub duel_blocking_enabled: bool,
    pub ai_mode: AIType,
    pub rounds: u32,
    pub board_width: u32,
    pub board_height: u32,
    pub same_piece_set: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            duel_blocking_enabled: true,
            ai_mode: AIType::default(),
            rounds: 20,
            board_width: 10,
            board_height: 10,
            same_piece_set: true,
        }
    }
}

#[derive(Resource, Clone)]
pub struct TempSettings {
    pub duel_blocking_enabled: bool,
    pub ai_mode: AIType,
    pub rounds: u32,
    pub board_width: u32,
    pub board_height: u32,
    pub same_piece_set: bool,
}

#[derive(Resource)]
pub struct RoundCounter {
    pub current: u32,
    pub total: u32,
}

impl RoundCounter {
    pub fn new(total: u32) -> Self {
        Self { current: 0, total }
    }
    pub fn is_game_over(&self) -> bool {
        self.current >= self.total
    }
    pub fn advance(&mut self) {
        self.current += 1;
    }
}
