use crate::components::*;
use crate::resources::{BoardSize, DuelState, GameState};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub fn recalculate_score_with_size<F: QueryFilter>(
    board_cells: &HashMap<IVec2, LinearRgba>,
    piece_query: &Query<&Piece, F>,
    board_size: IVec2,
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
                            if crate::helpers::is_in_bounds(target_cell, board_size) {
                                if check_condition_with_sizes(
                                    &effect.condition,
                                    Some(target_cell),
                                    board_cells,
                                    piece_query,
                                ) {
                                    total += effect.points;
                                }
                            }
                        }
                    }
                    None => match &effect.condition {
                        EffectCondition::NoColorOnBoard(c) => {
                            if no_color_on_board_excluding(board_cells, c, &exclude_cells) {
                                total += effect.points;
                            }
                        }
                        EffectCondition::MatchesSize(_) => {}
                        _ => {
                            if check_condition(&effect.condition, Some(pos), board_cells) {
                                total += effect.points;
                            }
                        }
                    },
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
        EffectCondition::NoColorOnBoard(c) => !board_cells
            .values()
            .any(|board_color| linear_rgba_near(board_color, c)),
        EffectCondition::MatchesSize(_) => false,
    }
}

pub fn check_condition_with_sizes<F: QueryFilter>(
    cond: &EffectCondition,
    target: Option<IVec2>,
    board_cells: &HashMap<IVec2, LinearRgba>,
    piece_query: &Query<&Piece, F>,
) -> bool {
    match cond {
        EffectCondition::MatchesColor(c) => target.map_or(false, |cell| {
            board_cells
                .get(&cell)
                .map_or(false, |board_color| linear_rgba_near(board_color, c))
        }),
        EffectCondition::IsEmpty => target.map_or(false, |cell| !board_cells.contains_key(&cell)),
        EffectCondition::NoColorOnBoard(c) => !board_cells
            .values()
            .any(|board_color| linear_rgba_near(board_color, c)),
        EffectCondition::MatchesSize(size) => {
            if let Some(cell) = target {
                for piece in piece_query.iter() {
                    if let Some(pos) = piece.placed_at {
                        for offset in &piece.shape {
                            if pos + *offset == cell {
                                return piece.shape.len() == *size;
                            }
                        }
                    }
                }
            }
            false
        }
    }
}

pub fn compute_piece_contribution<F: QueryFilter>(
    piece: &Piece,
    board_cells: &HashMap<IVec2, LinearRgba>,
    piece_query: &Query<&Piece, F>,
    board_size: IVec2,
) -> i32 {
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
                        if crate::helpers::is_in_bounds(target_cell, board_size) {
                            if check_condition_with_sizes(
                                &effect.condition,
                                Some(target_cell),
                                board_cells,
                                piece_query,
                            ) {
                                total += effect.points;
                            }
                        }
                    }
                }
                None => match &effect.condition {
                    EffectCondition::NoColorOnBoard(c) => {
                        if no_color_on_board_excluding(board_cells, c, &exclude_cells) {
                            total += effect.points;
                        }
                    }
                    EffectCondition::MatchesSize(_) => {}
                    _ => {
                        if check_condition(&effect.condition, Some(pos), board_cells) {
                            total += effect.points;
                        }
                    }
                },
            }
        }
    }
    total
}

pub fn linear_rgba_near(a: &LinearRgba, b: &LinearRgba) -> bool {
    let eps = 0.001;
    (a.red - b.red).abs() < eps
        && (a.green - b.green).abs() < eps
        && (a.blue - b.blue).abs() < eps
        && (a.alpha - b.alpha).abs() < eps
}

pub fn recalculate_score_system(
    mut state: ResMut<GameState>,
    piece_query: Query<&Piece>,
    board_size: Res<BoardSize>,
) {
    state.score = recalculate_score_with_size(&state.board_cells, &piece_query, board_size.0);
}

pub fn recalculate_duel_score_system(
    mut duel_state: ResMut<DuelState>,
    player_pieces: Query<&Piece, With<PlayerPiece>>,
    opponent_pieces: Query<&Piece, With<OpponentPiece>>,
    board_size: Res<BoardSize>,
) {
    duel_state.player.score =
        recalculate_score_with_size(&duel_state.player.board_cells, &player_pieces, board_size.0);
    duel_state.opponent.score = recalculate_score_with_size(
        &duel_state.opponent.board_cells,
        &opponent_pieces,
        board_size.0,
    );
}

pub fn no_color_on_board_excluding(
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

pub fn recalculate_score_from_vectors(
    board_cells: &HashMap<IVec2, LinearRgba>,
    pieces: &[Piece],
    board_size: IVec2,
) -> i32 {
    let mut total = 0;
    for piece in pieces {
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
                            if crate::helpers::is_in_bounds(target_cell, board_size) {
                                let cond_met = match &effect.condition {
                                    EffectCondition::MatchesColor(c) => board_cells
                                        .get(&target_cell)
                                        .map_or(false, |bc| linear_rgba_near(bc, c)),
                                    EffectCondition::IsEmpty => {
                                        !board_cells.contains_key(&target_cell)
                                    }
                                    EffectCondition::NoColorOnBoard(_) => false,
                                    EffectCondition::MatchesSize(size) => pieces
                                        .iter()
                                        .find_map(|p| {
                                            if let Some(p_pos) = p.placed_at {
                                                if p.shape
                                                    .iter()
                                                    .any(|off| p_pos + *off == target_cell)
                                                {
                                                    Some(p.shape.len())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        })
                                        .map_or(false, |s| s == *size),
                                };
                                if cond_met {
                                    total += effect.points;
                                }
                            }
                        }
                    }
                    None => match &effect.condition {
                        EffectCondition::NoColorOnBoard(c) => {
                            if no_color_on_board_excluding(board_cells, c, &exclude_cells) {
                                total += effect.points;
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    total
}
