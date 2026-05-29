use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::GameState;
use crate::systems::scoring::check_condition;
use bevy::prelude::*;
//use std::collections::HashMap;

/// Data needed to place a piece chosen by the AI.
pub struct AIPlacement {
    pub raw_config: RawPieceConfig,
    pub origin: IVec2,
    pub shape: Vec<IVec2>,
    pub effects: Vec<GameEffect>,
    pub color: LinearRgba,
}

/// Try to find a placement for any draft piece (simple first free spot).
pub fn first_free_placement(
    draft_pieces: &[(Entity, &Piece)],
    opponent_state: &GameState,
) -> Option<AIPlacement> {
    for (_entity, piece) in draft_pieces.iter() {
        let mut shape = piece.shape.clone();
        let mut effects = piece.effects.clone();
        for rot in 0..4 {
            if rot > 0 {
                shape = rotate_shape(&shape);
                rotate_effects(&mut effects);
            }
            for y in 0..BOARD_SIZE.y {
                for x in 0..BOARD_SIZE.x {
                    let origin = IVec2::new(x, y);
                    if can_place(&shape, origin, opponent_state) {
                        return Some(AIPlacement {
                            raw_config: RawPieceConfig {
                                shape: shape.clone(),
                                color: "".into(),
                                points: piece.points,
                                effects: vec![],
                                piece_type: crate::config::PieceType::Static,
                            },
                            origin,
                            shape,
                            effects,
                            color: piece.color,
                        });
                    }
                }
            }
        }
    }
    None
}

/// Greedy placement that also considers other pieces' effects on the board.
/// Total score = immediate points from placed piece + immediate points from other pieces' effects
///                + potential future points from placed piece's own effects
///                - potential points lost from other pieces' effects that get blocked.
pub fn greedy_placement(
    draft_pieces: &[(Entity, &Piece)],
    opponent_state: &GameState,
    opponent_placed_pieces: &[&Piece], // all pieces already on opponent's board
) -> Option<AIPlacement> {
    let mut best_placement: Option<AIPlacement> = None;
    let mut best_score = -1;

    for (_entity, piece) in draft_pieces.iter() {
        let mut shape = piece.shape.clone();
        let mut effects = piece.effects.clone();

        for rot in 0..4 {
            if rot > 0 {
                shape = rotate_shape(&shape);
                rotate_effects(&mut effects);
            }
            for y in 0..BOARD_SIZE.y {
                for x in 0..BOARD_SIZE.x {
                    let origin = IVec2::new(x, y);
                    if !can_place(&shape, origin, opponent_state) {
                        continue;
                    }

                    // 1. Immediate points from this piece itself
                    let mut immediate_self = piece.points;
                    let mut potential_self = 0;

                    // 2. Interaction with existing pieces
                    let mut immediate_other = 0;
                    let mut penalty_other = 0;

                    let occupied_cells: Vec<IVec2> =
                        shape.iter().map(|off| origin + *off).collect();

                    // For each existing opponent piece
                    for existing in opponent_placed_pieces {
                        if let Some(ex_origin) = existing.placed_at {
                            for effect in &existing.effects {
                                if let Some(offsets) = &effect.offsets {
                                    for offset in offsets {
                                        let target = ex_origin + *offset;
                                        if occupied_cells.contains(&target) {
                                            // This placement occupies a target cell of an existing effect
                                            let condition = &effect.condition;
                                            // Check if the effect would be satisfied by this placement
                                            let satisfied = match condition {
                                                EffectCondition::MatchesColor(c) => {
                                                    // Does the new piece at that cell have the right color?
                                                    // The new piece has a uniform color.
                                                    piece.color == *c
                                                }
                                                EffectCondition::IsEmpty => false, // placing makes it non-empty
                                                EffectCondition::NoColorOnBoard(_) => {
                                                    // This effect is board-wide; not relevant per cell
                                                    false
                                                }
                                            };
                                            if satisfied {
                                                immediate_other += effect.points;
                                            } else {
                                                // Blocking: the cell could have been used later for this effect
                                                penalty_other += effect.points;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 3. Future potential from this piece's own effects (unaffected by others)
                    for effect in &effects {
                        if let Some(offsets) = &effect.offsets {
                            for offset in offsets {
                                let target = origin + *offset;
                                if !crate::helpers::is_in_bounds(target) {
                                    continue;
                                }
                                if !check_condition(
                                    &effect.condition,
                                    Some(target),
                                    &opponent_state.board_cells,
                                ) && is_cell_available(
                                    target,
                                    &opponent_state.board_cells,
                                    &opponent_state.disabled_cells,
                                ) {
                                    potential_self += effect.points;
                                }
                            }
                        }
                    }

                    let total = immediate_self + immediate_other + potential_self - penalty_other;
                    if total > best_score {
                        best_score = total;
                        best_placement = Some(AIPlacement {
                            raw_config: RawPieceConfig {
                                shape: shape.clone(),
                                color: "".into(),
                                points: piece.points,
                                effects: vec![],
                                piece_type: crate::config::PieceType::Static,
                            },
                            origin,
                            shape: shape.clone(),
                            effects: effects.clone(),
                            color: piece.color,
                        });
                    }
                }
            }
        }
    }
    best_placement
}

/// For destroy turn: choose a cell to block that would deny the most potential effect points
/// from the player's already placed pieces.
pub fn greedy_block_cell(
    player_state: &GameState,
    placed_pieces: &Query<&Piece, With<PlayerPiece>>,
) -> Option<IVec2> {
    let mut best_cell: Option<IVec2> = None;
    let mut best_value = -1;

    for y in 0..BOARD_SIZE.y {
        for x in 0..BOARD_SIZE.x {
            let cell = IVec2::new(x, y);
            if !is_cell_available(
                cell,
                &player_state.board_cells,
                &player_state.disabled_cells,
            ) {
                continue;
            }

            let mut blocked_points = 0;
            for piece in placed_pieces.iter() {
                if let Some(origin) = piece.placed_at {
                    for effect in &piece.effects {
                        if let Some(offsets) = &effect.offsets {
                            for offset in offsets {
                                let target = origin + *offset;
                                if target == cell {
                                    blocked_points += effect.points;
                                }
                            }
                        }
                    }
                }
            }
            if blocked_points > best_value {
                best_value = blocked_points;
                best_cell = Some(cell);
            }
        }
    }
    best_cell
}

// Helper functions used by both AI strategies
fn can_place(shape: &[IVec2], origin: IVec2, state: &GameState) -> bool {
    for offset in shape {
        let cell = origin + *offset;
        if !crate::helpers::is_cell_available(cell, &state.board_cells, &state.disabled_cells) {
            return false;
        }
    }
    true
}

fn rotate_shape(shape: &[IVec2]) -> Vec<IVec2> {
    shape.iter().map(|&v| IVec2::new(v.y, -v.x)).collect()
}

fn rotate_effects(effects: &mut Vec<GameEffect>) {
    for effect in effects {
        if let Some(offsets) = &mut effect.offsets {
            for offset in offsets {
                let old = *offset;
                *offset = IVec2::new(old.y, -old.x);
            }
        }
    }
}
