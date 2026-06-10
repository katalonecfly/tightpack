use crate::components::*;
use crate::resources::{DuelState, GameState};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub fn recalculate_score<F: QueryFilter>(
    board_cells: &HashMap<IVec2, LinearRgba>,
    piece_query: &Query<&Piece, F>,
) -> i32 {
    let mut total = 0;
    for piece in piece_query.iter() {
        if let Some(pos) = piece.placed_at {
            total += piece.points;
            let mut exclude_cells = HashSet::new();
            for offset in &piece.shape {
                exclude_cells.insert(pos + *offset);
            }
            for effect in &piece.effects {
                match &effect.offsets {
                    Some(offsets) => {
                        for offset in offsets {
                            let target_cell = pos + *offset;
                            if crate::helpers::is_in_bounds(target_cell) {
                                if check_condition(&effect.condition, Some(target_cell), board_cells) {
                                    total += effect.points;
                                }
                            }
                        }
                    }
                    None => {
                        if let EffectCondition::NoColorOnBoard(c) = &effect.condition {
                            if no_color_on_board_excluding(board_cells, c, &exclude_cells) {
                                total += effect.points;
                            }
                        } else {
                            if check_condition(&effect.condition, Some(pos), board_cells) {
                                total += effect.points;
                            }
                        }
                    }
                }
            }
        }
    }
    total
}

pub fn check_condition(
    cond: &EffectCondition,
    target: Option<IVec2>,
    board_cells: &HashMap<IVec2, LinearRgba>,
) -> bool {
    match cond {
        EffectCondition::MatchesColor(c) => target.map_or(false, |cell| {
            board_cells
                .get(&cell)
                .map_or(false, |board_color| linear_rgba_near(board_color, c))
        }),
        EffectCondition::IsEmpty => target.map_or(false, |cell| !board_cells.contains_key(&cell)),
        EffectCondition::NoColorOnBoard(c) => {
            // This function is never called directly for NoColorOnBoard because we use the excluding version.
            // We keep it for completeness, but it will not be used in scoring.
            !board_cells.values().any(|board_color| linear_rgba_near(board_color, c))
        }
    }
}

pub fn compute_piece_contribution(piece: &Piece, board_cells: &HashMap<IVec2, LinearRgba>) -> i32 {
    let mut total = piece.points;
    if let Some(pos) = piece.placed_at {
        let mut exclude_cells = HashSet::new();
        for offset in &piece.shape {
            exclude_cells.insert(pos + *offset);
        }
        for effect in &piece.effects {
            match &effect.offsets {
                Some(offsets) => {
                    for offset in offsets {
                        let target_cell = pos + *offset;
                        if crate::helpers::is_in_bounds(target_cell) {
                            if check_condition(&effect.condition, Some(target_cell), board_cells) {
                                total += effect.points;
                            }
                        }
                    }
                }
                None => {
                    if let EffectCondition::NoColorOnBoard(c) = &effect.condition {
                        if no_color_on_board_excluding(board_cells, c, &exclude_cells) {
                            total += effect.points;
                        }
                    } else {
                        if check_condition(&effect.condition, Some(pos), board_cells) {
                            total += effect.points;
                        }
                    }
                }
            }
        }
    }
    total
}

fn no_color_on_board_excluding(
    board_cells: &HashMap<IVec2, LinearRgba>,
    color: &LinearRgba,
    exclude_cells: &HashSet<IVec2>,
) -> bool {
    for (cell, board_color) in board_cells.iter() {
        if exclude_cells.contains(cell) {
            continue;
        }
        if linear_rgba_near(board_color, color) {
            return false;
        }
    }
    true
}

// Helper: compare two LinearRgba colours with a small epsilon
pub fn linear_rgba_near(a: &LinearRgba, b: &LinearRgba) -> bool {
    let eps = 0.001;
    (a.red - b.red).abs() < eps
        && (a.green - b.green).abs() < eps
        && (a.blue - b.blue).abs() < eps
        && (a.alpha - b.alpha).abs() < eps
}

pub fn recalculate_score_system(mut state: ResMut<GameState>, piece_query: Query<&Piece>) {
    state.score = recalculate_score(&state.board_cells, &piece_query);
}

pub fn recalculate_duel_score_system(
    mut duel_state: ResMut<DuelState>,
    player_pieces: Query<&Piece, With<PlayerPiece>>,
    opponent_pieces: Query<&Piece, With<OpponentPiece>>,
) {
    duel_state.player.score = recalculate_score(&duel_state.player.board_cells, &player_pieces);
    duel_state.opponent.score =
        recalculate_score(&duel_state.opponent.board_cells, &opponent_pieces);
}