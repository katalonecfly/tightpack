use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct RawPieceLibrary {
    pub pieces: Vec<RawPieceConfig>,
}

#[derive(Deserialize, Clone)]
pub struct RawPieceConfig {
    pub shape: Vec<IVec2>,
    pub color: String,
    pub points: i32,
    #[serde(default)]
    pub effects: Vec<RawGameEffect>,
    #[serde(default = "default_piece_type")]
    pub piece_type: PieceType,
}

fn default_piece_type() -> PieceType {
    PieceType::Static
}

#[derive(Deserialize, Clone, PartialEq)]
pub enum PieceType {
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "dynamic")]
    Dynamic,
}

impl Default for PieceType {
    fn default() -> Self {
        PieceType::Static
    }
}

#[derive(Deserialize, Clone)]
pub struct RawGameEffect {
    pub condition: RawEffectCondition,
    pub points: i32,
    #[serde(default)]
    pub offsets: Vec<IVec2>,
}

#[derive(Deserialize, Clone)]
pub enum RawEffectCondition {
    MatchesColor(String),
    IsEmpty,
    NoColorOnBoard(String),
    MatchesSize(u32),
}

#[derive(Resource, Deserialize, Clone, Default)]
pub struct EffectDescriptions {
    pub descriptions: HashMap<String, String>,
}
