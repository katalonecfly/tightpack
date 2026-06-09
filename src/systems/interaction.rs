use crate::components::*;
use crate::helpers::*;
use crate::resources::GameState;
use crate::puzzle_ui::PuzzleGameState;
use crate::resources::DuelState;
use bevy::prelude::*;

fn get_piece_entity(
    target: Entity,
    piece_query: &Query<(), With<Piece>>,
    child_of_query: &Query<&ChildOf>,
) -> Option<Entity> {
    if piece_query.contains(target) {
        Some(target)
    } else if let Ok(child_of) = child_of_query.get(target) {
        Some(child_of.parent())
    } else {
        None
    }
}

pub fn on_drag_start(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    locked_query: Query<(), With<LockedPiece>>,
    mut state: ResMut<GameState>,
    opponent_query: Query<(), With<OpponentPiece>>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Piece, &Children), Without<LockedPiece>>,
        Query<(Entity, &mut Piece, &mut Transform), (With<DraftPiece>, Without<LockedPiece>)>,
    )>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if opponent_query.contains(piece_entity) {
        return;
    }
    if locked_query.contains(piece_entity) {
        return;
    }

    for (other_entity, mut other_piece, mut other_transform) in param_set.p1().iter_mut() {
        if other_entity != piece_entity && other_piece.placed_at.is_some() {
            if let Some(old_pos) = other_piece.placed_at {
                for offset in &other_piece.shape {
                    state.board_cells.remove(&(old_pos + *offset));
                }
                other_piece.placed_at = None;
            }
            other_transform.translation = other_piece.original_pos;
            other_transform.translation.z = other_piece.original_pos.z;
            other_transform.rotation = Quat::IDENTITY;
            other_piece.shape = other_piece.original_shape.clone();
            other_piece.effects = other_piece.original_effects.clone();
        }
    }

    if let Ok((mut transform, mut piece, _)) = param_set.p0().get_mut(piece_entity) {
        commands.entity(piece_entity).insert(Dragging);
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                state.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
        }
    }
}

pub fn on_drag(
    on: On<Pointer<Drag>>,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &Piece)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut commands: Commands,
    state: Res<GameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
    opponent_query: Query<(), With<OpponentPiece>>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if opponent_query.contains(piece_entity) {
        return;
    }
    if locked_query.contains(piece_entity) {
        return;
    }
    if let Ok((mut transform, piece)) = drag_piece_query.get_mut(piece_entity) {
        transform.translation.x += on.delta.x;
        transform.translation.y -= on.delta.y;

        for entity in &ghost_query {
            let _ = commands.entity(entity).try_despawn();
        }
        let grid_pos = world_to_grid_for_side(transform.translation, piece.board_side);
        let mut can_place = true;
        for offset in &piece.shape {
            let tile_pos = grid_pos + *offset;
            if !is_in_bounds(tile_pos) || state.board_cells.contains_key(&tile_pos) {
                can_place = false;
                break;
            }
        }
        if can_place {
            let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
            for offset in &piece.shape {
                commands.spawn((
                    Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(
                        grid_to_world_for_side(grid_pos + *offset, piece.board_side).with_z(1.0),
                    ),
                    GhostTile,
                ));
            }
        }
    }
}

pub fn on_drag_end(
    on: On<Pointer<DragEnd>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &mut Piece, &Children)>,
    locked_query: Query<(), With<LockedPiece>>,
    draft_check: Query<(), With<DraftPiece>>,
    piece_entities: Query<Entity, With<Piece>>,
    mut state: ResMut<GameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
    opponent_query: Query<(), With<OpponentPiece>>,
) {
    for entity in &ghost_query {
        let _ = commands.entity(entity).try_despawn();
    }

    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if opponent_query.contains(piece_entity) {
        return;
    }
    if locked_query.contains(piece_entity) {
        return;
    }

    commands.entity(piece_entity).remove::<Dragging>();
    if let Ok((mut transform, mut piece, _children)) = drag_piece_query.get_mut(piece_entity) {
        let grid_pos = world_to_grid_for_side(transform.translation, piece.board_side);
        let mut can_place = true;
        for offset in &piece.shape {
            let cell = grid_pos + *offset;
            if cell.x < 0
                || cell.x >= BOARD_SIZE.x
                || cell.y < 0
                || cell.y >= BOARD_SIZE.y
                || state.board_cells.contains_key(&cell)
            {
                can_place = false;
                break;
            }
        }

        if can_place {
            transform.translation = grid_to_world_for_side(grid_pos, piece.board_side).with_z(1.0);
            piece.placed_at = Some(grid_pos);
            for offset in &piece.shape {
                state.board_cells.insert(grid_pos + *offset, piece.color);
            }

            if draft_check.contains(piece_entity) {
                let mut to_reset = Vec::new();
                for other_entity in &piece_entities {
                    if other_entity != piece_entity
                        && draft_check.contains(other_entity)
                        && drag_piece_query
                            .get(other_entity)
                            .map_or(false, |(_, p, _)| p.placed_at.is_some())
                    {
                        to_reset.push(other_entity);
                    }
                }

                for entity in to_reset {
                    if let Ok((mut t, mut p, _)) = drag_piece_query.get_mut(entity) {
                        if let Some(old_pos) = p.placed_at {
                            for offset in &p.shape {
                                state.board_cells.remove(&(old_pos + *offset));
                            }
                            p.placed_at = None;
                        }
                        t.translation = p.original_pos;
                        t.translation.z = 1.0;
                        t.rotation = Quat::IDENTITY;
                        p.shape = p.original_shape.clone();
                        p.effects = p.original_effects.clone();
                    }
                }
            }
        } else {
            transform.translation = piece.original_pos;
            transform.translation.z = piece.original_pos.z;
            transform.rotation = Quat::IDENTITY;
            piece.shape = piece.original_shape.clone();
            piece.effects = piece.original_effects.clone();
        }
    }
}

pub fn handle_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut piece_query: Query<
        (&mut Transform, &mut Piece, &Children),
        (With<Dragging>, Without<OpponentPiece>),
    >,
    mut preview_query: Query<&mut EffectPreview>,
    mut commands: Commands,
    ghost_query: Query<Entity, With<GhostTile>>,
    state: Res<GameState>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Right) {
        for (mut transform, mut piece, children) in piece_query.iter_mut() {
            transform.rotate_z(-std::f32::consts::FRAC_PI_2);
            for offset in &mut piece.shape {
                let old = *offset;
                *offset = IVec2::new(old.y, -old.x);
            }
            for effect in &mut piece.effects {
                if let Some(offsets) = &mut effect.offsets {
                    for offset in offsets {
                        let old = *offset;
                        *offset = IVec2::new(old.y, -old.x);
                    }
                }
            }
            for &child in children {
                if let Ok(mut preview) = preview_query.get_mut(child) {
                    let old = preview.offset;
                    preview.offset = IVec2::new(old.y, -old.x);
                }
            }
            for entity in ghost_query.iter() {
                let _ = commands.entity(entity).try_despawn();
            }
            let grid_pos = world_to_grid_for_side(transform.translation, piece.board_side);
            let mut can_place = true;
            for offset in &piece.shape {
                let tile = grid_pos + *offset;
                if !is_cell_available(tile, &state.board_cells, &state.disabled_cells) {
                    can_place = false;
                    break;
                }
            }
            if can_place {
                let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
                for offset in &piece.shape {
                    commands.spawn((
                        Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                        Transform::from_translation(
                            grid_to_world_for_side(grid_pos + *offset, piece.board_side)
                                .with_z(1.0),
                        ),
                        GhostTile,
                    ));
                }
            }
        }
    }
}

pub fn on_child_hover_in(
    trigger: On<Pointer<Over>>,
    mut commands: Commands,
    child_of_query: Query<&ChildOf>,
) {
    let entity = trigger.event_target();
    if let Ok(child_of) = child_of_query.get(entity) {
        commands.entity(child_of.parent()).insert(Hovered);
    }
}

pub fn on_child_hover_out(
    trigger: On<Pointer<Out>>,
    mut commands: Commands,
    child_of_query: Query<&ChildOf>,
) {
    let entity = trigger.event_target();
    if let Ok(child_of) = child_of_query.get(entity) {
        commands.entity(child_of.parent()).remove::<Hovered>();
    }
}

pub fn on_hover_in(on: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(on.event_target()).insert(Hovered);
}

pub fn on_hover_out(on: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(on.event_target()).remove::<Hovered>();
}

pub fn on_right_click_unplace(
    trigger: On<Pointer<Click>>,
    mut piece_query: Query<(&mut Piece, &mut Transform, &Children)>,
    locked_query: Query<(), With<LockedPiece>>,
    opponent_query: Query<(), With<OpponentPiece>>,
    dragging_query: Query<(), With<Dragging>>,
    child_of_query: Query<&ChildOf>,
    mut game_state: ResMut<GameState>,
    mut puzzle_state: Option<ResMut<PuzzleGameState>>,
    mut duel_state: Option<ResMut<DuelState>>,
) {
    if trigger.event.button != PointerButton::Secondary {
        return;
    }
    let target = trigger.event_target();
    let piece_entity = if piece_query.contains(target) {
        target
    } else if let Ok(child_of) = child_of_query.get(target) {
        child_of.parent()
    } else {
        return;
    };

    if dragging_query.contains(piece_entity)
        || locked_query.contains(piece_entity)
        || opponent_query.contains(piece_entity)
    {
        return;
    }

    // In Duel mode, only allow unplacing during the Place turn
    if let Some(duel) = duel_state.as_ref() {
        if duel.turn != crate::resources::DuelTurn::Place {
            return;
        }
    }

    let Ok((mut piece, mut transform, _)) = piece_query.get_mut(piece_entity) else { return };

    if let Some(old_pos) = piece.placed_at {
        for offset in &piece.shape {
            let cell = old_pos + *offset;
            if let Some(ref mut puzzle) = puzzle_state {
                puzzle.board_cells.remove(&cell);
            } else if let Some(ref mut duel) = duel_state {
                match piece.board_side {
                    BoardSide::Left => duel.player.board_cells.remove(&cell),
                    BoardSide::Right => duel.opponent.board_cells.remove(&cell),
                    _ => continue,
                };
            } else {
                game_state.board_cells.remove(&cell);
            }
        }
        piece.placed_at = None;
        transform.translation = piece.original_pos;
        transform.translation.z = piece.original_pos.z;
        transform.rotation = Quat::IDENTITY;
        piece.shape = piece.original_shape.clone();
        piece.effects = piece.original_effects.clone();
    }
}