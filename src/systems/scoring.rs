use bevy::prelude::*;
use crate::components::*;
use crate::resources::GameState;

pub fn recalculate_score(state: &mut GameState, query: &Query<(&mut Transform, &mut Piece, &Children)>) {
    let mut total = 0;
    for (_, piece, _) in query.iter() {
        if let Some(pos) = piece.placed_at {
            total += piece.points;
            for effect in &piece.effects {
                match &effect.offsets {
                    Some(offsets) => {
                        for offset in offsets {
                            if check_condition(&effect.condition, Some(pos + *offset), state) {
                                total += effect.points;
                            }
                        }
                    }
                    None => {
                        if check_condition(&effect.condition, Some(pos), state) {
                            total += effect.points;
                        }
                    }
                }
            }
        }
    }
    state.score = total;
}

pub fn check_condition(cond: &EffectCondition, target: Option<IVec2>, state: &GameState) -> bool {
    match cond {
        EffectCondition::MatchesColor(c) => target.map_or(false, |cell| state.board_cells.get(&cell) == Some(c)),
        EffectCondition::IsEmpty => target.map_or(false, |cell| !state.board_cells.contains_key(&cell)),
        EffectCondition::NoColorOnBoard(c) => !state.board_cells.values().any(|color| color == c),
    }
}