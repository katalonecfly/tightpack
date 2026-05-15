use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::GameState;
use bevy::prelude::*;

/// Data needed to place a piece chosen by the AI.
pub struct AIPlacement {
    pub raw_config: RawPieceConfig,
    pub origin: IVec2,
    pub shape: Vec<IVec2>,
    pub effects: Vec<GameEffect>,
    pub color: LinearRgba,
}

/// Try to find a placement for any draft piece.
/// Returns placement data if successful, `None` otherwise.
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
                                color: "".into(),   // will be filled from piece color
                                points: piece.points,
                                effects: vec![],    // we'll pass effects directly
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