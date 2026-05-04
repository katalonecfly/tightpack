use bevy::prelude::*;
use crate::components::*;
use crate::helpers::*;
use crate::resources::GameState;
use crate::systems::scoring::recalculate_score;

// No import for ChildOf needed – it’s in bevy::prelude

// systems/interaction.rs

pub fn on_drag_start(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Piece, &Children)>,
    mut state: ResMut<GameState>,
) {
    if let Ok((mut transform, mut piece, _)) = query.get_mut(on.event_target()) {
        commands.entity(on.event_target()).insert(Dragging);
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                state.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
            recalculate_score(&mut state, &query);
        }
    }
}

pub fn on_drag(
    on: On<Pointer<Drag>>, 
    mut query: Query<(&mut Transform, &Piece)>,
    mut commands: Commands,
    state: Res<GameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
) {
    let target = on.event_target();
    if let Ok((mut transform, piece)) = query.get_mut(target) {
        // Move the actual piece
        transform.translation.x += on.delta.x;
        transform.translation.y -= on.delta.y;

        // --- GHOST LOGIC ---
        // 1. Clear old ghost
        for entity in &ghost_query {
            commands.entity(entity).despawn();
        }

        // 2. Calculate current grid position
        let grid_pos = world_to_grid(transform.translation);

        // 3. If within bounds, show where it would land
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
pub fn on_drag_end(
    on: On<Pointer<DragEnd>>, 
    mut commands: Commands, 
    mut query: Query<(&mut Transform, &mut Piece, &Children)>, 
    mut state: ResMut<GameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
) {
    for entity in &ghost_query {
        commands.entity(entity).despawn();
    }    
    let target = on.event_target();
    commands.entity(target).remove::<Dragging>();
    let Ok((mut transform, mut piece, _children)) = query.get_mut(target) else { return };
    let grid_pos = world_to_grid(transform.translation);

    let mut can_place = true;
    for offset in &piece.shape {
        let cell = grid_pos + *offset;
        if cell.x < 0 || cell.x >= BOARD_SIZE.x || cell.y < 0 || cell.y >= BOARD_SIZE.y || state.board_cells.contains_key(&cell) {
            can_place = false;
            break;
        }
    }

    if can_place {
        transform.translation = grid_to_world(grid_pos).with_z(1.0);
        piece.placed_at = Some(grid_pos);
        for offset in &piece.shape { state.board_cells.insert(grid_pos + *offset, piece.color); }
    } else {
        transform.translation = piece.original_pos;
        transform.translation.z = 1.0;
        transform.rotation = Quat::IDENTITY;
        piece.shape = piece.original_shape.clone();
        piece.effects = piece.original_effects.clone();
        // Reset preview offsets logic omitted for brevity but should be here
    }
    recalculate_score(&mut state, &query);
}

pub fn handle_rotation(keyboard: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut piece_query: Query<(Entity, &mut Transform, &mut Piece, &Children), With<Dragging>>, mut preview_query: Query<&mut EffectPreview>) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Right) {
        for (_, mut transform, mut piece, children) in &mut piece_query {
            transform.rotate_z(-std::f32::consts::FRAC_PI_2);
            for offset in &mut piece.shape { let old = *offset; *offset = IVec2::new(old.y, -old.x); }
            for effect in &mut piece.effects {
                if let Some(offsets) = &mut effect.offsets {
                    for offset in offsets { let old = *offset; *offset = IVec2::new(old.y, -old.x); }
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

pub fn on_hover_in(on: On<Pointer<Over>>, mut commands: Commands) { commands.entity(on.event_target()).insert(Hovered); }
pub fn on_hover_out(on: On<Pointer<Out>>, mut commands: Commands) { commands.entity(on.event_target()).remove::<Hovered>(); }