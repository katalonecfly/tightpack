use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::GameState;
use bevy::prelude::*;

pub struct AIPlacement {
    pub raw_config: RawPieceConfig,
    pub origin: IVec2,
    pub shape: Vec<IVec2>,
    pub effects: Vec<GameEffect>,
    pub color: LinearRgba,
}

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

pub fn greedy_placement(
    draft_pieces: &[(Entity, &Piece)],
    opponent_state: &GameState,
    opponent_placed_pieces: &[&Piece],
) -> Option<AIPlacement> {
    use crate::helpers::is_in_bounds;
    use crate::systems::scoring::recalculate_score_from_vectors;

    let current_pieces: Vec<Piece> = opponent_placed_pieces.iter().map(|&p| p.clone()).collect();
    let current_score = recalculate_score_from_vectors(&opponent_state.board_cells, &current_pieces);

    let mut best_placement: Option<AIPlacement> = None;
    let mut best_score = -1_000_000;

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

                    let mut new_board_cells = opponent_state.board_cells.clone();
                    let mut new_pieces = current_pieces.clone();
                    let new_piece = Piece {
                        type_id: piece.type_id,
                        shape: shape.clone(),
                        original_shape: piece.original_shape.clone(),
                        color: piece.color,
                        points: piece.points,
                        effects: effects.clone(),
                        original_effects: piece.original_effects.clone(),
                        original_pos: piece.original_pos,
                        placed_at: Some(origin),
                        board_side: piece.board_side,
                    };
                    for offset in &shape {
                        new_board_cells.insert(origin + *offset, piece.color);
                    }
                    new_pieces.push(new_piece);

                    let new_score = recalculate_score_from_vectors(&new_board_cells, &new_pieces);
                    let net_gain = new_score - current_score;

                    // Calculate bonus for effect offsets that are inside the board (potential future activation)
                    let mut potential_bonus = 0;
                    for effect in &effects {
                        if let Some(offsets) = &effect.offsets {
                            for offset in offsets {
                                let target = origin + *offset;
                                if is_in_bounds(target) {
                                    potential_bonus += effect.points;
                                }
                            }
                        }
                    }

                    // Weight net gain heavily, then add potential bonus as tie-breaker
                    let total_score = net_gain * 10000 + potential_bonus;

                    if total_score > best_score {
                        best_score = total_score;
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

pub fn greedy_block_cell(
    player_state: &GameState,
    placed_pieces: &Query<&Piece, With<PlayerPiece>>,
) -> Option<IVec2> {
    let mut best_cell: Option<IVec2> = None;
    let mut best_value = -1;

    for y in 0..BOARD_SIZE.y {
        for x in 0..BOARD_SIZE.x {
            let cell = IVec2::new(x, y);
            if !is_cell_available(cell, &player_state.board_cells, &player_state.disabled_cells) {
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