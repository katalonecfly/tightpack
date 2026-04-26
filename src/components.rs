use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Piece {
    pub type_id: usize,
    pub shape: Vec<IVec2>,
    pub original_shape: Vec<IVec2>,
    pub color: LinearRgba,
    pub points: i32,
    pub effects: Vec<GameEffect>,
    pub original_effects: Vec<GameEffect>,
    pub original_pos: Vec3,
    pub placed_at: Option<IVec2>,
}

#[derive(Component)]
pub struct PieceVisual;

#[derive(Clone)]
pub struct GameEffect {
    pub condition: EffectCondition,
    pub points: i32,
    pub offsets: Option<Vec<IVec2>>,
}

#[derive(Clone, PartialEq)]
pub enum EffectCondition {
    MatchesColor(LinearRgba),
    IsEmpty,
    NoColorOnBoard(LinearRgba),
}

#[derive(Component)]
pub struct EffectPreview {
    pub offset: IVec2,
    pub condition: EffectCondition,
}

#[derive(Component)] pub struct Hovered;
#[derive(Component)] pub struct StashLabel(pub usize);
#[derive(Component)] pub struct ScoreText;
#[derive(Component)] pub struct Dragging;
#[derive(Component)] pub struct GhostTile;