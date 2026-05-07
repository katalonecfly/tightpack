use bevy::prelude::*;
use crate::components::*;
use crate::helpers::*;
use crate::resources::GameState;
use crate::systems::scoring::recalculate_score;

// ── Helper: find the piece entity from any child or the piece itself ──
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

// ── Drag start (lock check) ──
pub fn on_drag_start(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &mut Piece, &Children)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut state: ResMut<GameState>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };

    // Locked pieces cannot be dragged
    if locked_query.contains(piece_entity) {
        return;
    }

    if let Ok((mut transform, mut piece, _)) = drag_piece_query.get_mut(piece_entity) {
        commands.entity(piece_entity).insert(Dragging);
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                state.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
            recalculate_score(&mut state, &drag_piece_query);
        }
    }
}

// ── Drag (lock check added) ──
pub fn on_drag(
    on: On<Pointer<Drag>>,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &Piece)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut commands: Commands,
    state: Res<GameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };

    // Ignore locked pieces
    if locked_query.contains(piece_entity) {
        return;
    }

    if let Ok((mut transform, piece)) = drag_piece_query.get_mut(piece_entity) {
        transform.translation.x += on.delta.x;
        transform.translation.y -= on.delta.y;

        // Ghost logic
        for entity in &ghost_query {
            commands.entity(entity).despawn();
        }
        let grid_pos = world_to_grid(transform.translation);
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
                    Transform::from_translation(grid_to_world(grid_pos + *offset).with_z(1.0)),
                    GhostTile,
                ));
            }
        }
    }
}

// ── Drag end (lock check added) ──
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
) {
    for entity in &ghost_query {
        commands.entity(entity).despawn();
    }

    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };

    if locked_query.contains(piece_entity) {
        return;
    }

    commands.entity(piece_entity).remove::<Dragging>();
    if let Ok((mut transform, mut piece, _children)) = drag_piece_query.get_mut(piece_entity) {
        let grid_pos = world_to_grid(transform.translation);
        let mut can_place = true;
        for offset in &piece.shape {
            let cell = grid_pos + *offset;
            if cell.x < 0 || cell.x >= BOARD_SIZE.x || cell.y < 0 || cell.y >= BOARD_SIZE.y
                || state.board_cells.contains_key(&cell)
            {
                can_place = false;
                break;
            }
        }

        if can_place {
            // Place the new piece
            transform.translation = grid_to_world(grid_pos).with_z(1.0);
            piece.placed_at = Some(grid_pos);
            for offset in &piece.shape {
                state.board_cells.insert(grid_pos + *offset, piece.color);
            }

            // --- Draft mode: reset all other placed draft pieces ---
            if draft_check.contains(piece_entity) {
                let mut to_reset = Vec::new();
                for other_entity in &piece_entities {
                    if other_entity != piece_entity
                        && draft_check.contains(other_entity)
                        && drag_piece_query.get(other_entity).map_or(false, |(_, p, _)| p.placed_at.is_some())
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
            // Invalid placement – return piece to stash
            transform.translation = piece.original_pos;
            transform.translation.z = 1.0;
            transform.rotation = Quat::IDENTITY;
            piece.shape = piece.original_shape.clone();
            piece.effects = piece.original_effects.clone();
        }

        recalculate_score(&mut state, &drag_piece_query);
    }
}

// ── Rotation ──
pub fn handle_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut piece_query: Query<(Entity, &mut Transform, &mut Piece, &Children), With<Dragging>>,
    mut preview_query: Query<&mut EffectPreview>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Right) {
        for (_, mut transform, mut piece, children) in &mut piece_query {
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
        }
    }
}

// ── Hover propagation from children to parent ──
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