use serde::Deserialize;
use bevy::prelude::*;

#[derive(Deserialize)]
pub struct RawPieceLibrary {
    pub pieces: Vec<RawPieceConfig>,
}

#[derive(Deserialize)]
pub struct RawPieceConfig {
    pub shape: Vec<IVec2>,
    pub color: String,
    pub points: i32,
    pub effects: Vec<RawGameEffect>,
}

#[derive(Deserialize)]
pub struct RawGameEffect {
    pub condition: RawEffectCondition,
    pub points: i32,
    #[serde(default)] 
    pub offsets: Vec<IVec2>, 
}

#[derive(Deserialize)]
pub enum RawEffectCondition {
    MatchesColor(String),
    IsEmpty,
    NoColorOnBoard(String),
}