use crate::components::*;
use crate::resources::{GameState, DuelState};
use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;
use std::collections::HashMap;

pub fn recalculate_score<F: QueryFilter>(
    board_cells: &HashMap<IVec2, LinearRgba>,
    piece_query: &Query<&Piece, F>,
) -> i32 {
    let mut total = 0;
    for piece in piece_query.iter() {
        if let Some(pos) = piece.placed_at {
            total += piece.points;
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

pub fn check_condition(
    cond: &EffectCondition,
    target: Option<IVec2>,
    board_cells: &HashMap<IVec2, LinearRgba>,
) -> bool {
    match cond {
        EffectCondition::MatchesColor(c) => {
            target.map_or(false, |cell| board_cells.get(&cell) == Some(c))
        }
        EffectCondition::IsEmpty => {
            target.map_or(false, |cell| !board_cells.contains_key(&cell))
        }
        EffectCondition::NoColorOnBoard(c) => !board_cells.values().any(|color| color == c),
    }
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
    duel_state.opponent.score = recalculate_score(&duel_state.opponent.board_cells, &opponent_pieces);
}