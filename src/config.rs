use serde::Deserialize;
use bevy::prelude::*;

#[derive(Deserialize, Clone)]      // <-- added Clone
pub struct RawPieceLibrary {
    pub pieces: Vec<RawPieceConfig>,
}

#[derive(Deserialize, Clone)]      // <-- added Clone
pub struct RawPieceConfig {
    pub shape: Vec<IVec2>,
    pub color: String,
    pub points: i32,
    pub effects: Vec<RawGameEffect>,
}

#[derive(Deserialize, Clone)]      // <-- added Clone
pub struct RawGameEffect {
    pub condition: RawEffectCondition,
    pub points: i32,
    #[serde(default)] 
    pub offsets: Vec<IVec2>,
    pub description: String,
}

#[derive(Deserialize, Clone)]      // <-- added Clone
pub enum RawEffectCondition {
    MatchesColor(String),
    IsEmpty,
    NoColorOnBoard(String),
}