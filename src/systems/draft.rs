use bevy::prelude::*;
use std::collections::HashMap;
use rand::RngExt;
use crate::config::RawPieceConfig;
use crate::components::*;
use crate::helpers::*;
use crate::resources::PieceLibrary;
use crate::Cleanup;
use crate::components::LockedPiece;
use bevy::picking::prelude::{Pointer, Click};

#[derive(Component)]
pub struct DraftConfirmButton;

#[allow(unused_mut)]
pub fn refresh_draft_stash(commands: &mut Commands, library: &PieceLibrary) {
    let mut color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let mut rng = rand::rng();
    let all_pieces = &library.0;

    // Pick 3 distinct pieces
    let mut available: Vec<&RawPieceConfig> = all_pieces.iter().collect();
    let mut chosen = Vec::with_capacity(3);
    for _ in 0..3 {
        let idx = rng.random_range(0..available.len());
        chosen.push(available.remove(idx));
    }

    let board_left = grid_to_world(IVec2::ZERO).x;
    let mut next_left = board_left;

    for (i, raw) in chosen.iter().enumerate() {
        // Calculate shape bounds
        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let width = (max_x - min_x + 1) as f32 * TILE_SIZE;

        // World position of the piece's leftmost tile
        let piece_left = next_left;
        // Parent's world position: shift left by min_x tiles so that the leftmost tile lands at piece_left
        let parent_x = piece_left - (min_x as f32) * TILE_SIZE;
        // Parent's world y: top of piece = -TILE_SIZE (board bottom is y=0, gap of TILE_SIZE)
        let parent_y = -TILE_SIZE - (max_y as f32 + 1.0) * TILE_SIZE;
        let pos = Vec3::new(parent_x, parent_y, 1.0);
        // Label position: above the piece's top
        let label_y = parent_y + (max_y as f32) * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        let label_pos = Vec3::new(parent_x, label_y, 2.0);

        let type_id = all_pieces
            .iter()
            .position(|p| std::ptr::eq(p, *raw))
            .unwrap_or(i);
        let color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        let effects = crate::systems::setup::bake_effects(raw, &color_map);

        // Label (world space, unparented)
        commands.spawn((
            Text2d::new("x1"),
            TextFont { font_size: 20.0, ..default() },
            Transform::from_translation(label_pos),
            StashLabel(type_id),
            DraftPiece,
            Cleanup,
        ));

        // Spawn draggable piece at calculated position
        crate::systems::setup::spawn_draggable_piece(
            commands,
            type_id,
            raw.shape.clone(),
            color,
            raw.points,
            effects,
            pos,
            true,
        );

        // Advance next_left: piece width + gap
        next_left = piece_left + width + TILE_SIZE;
    }
}

pub fn generate_draft_stash(mut commands: Commands, library: Res<PieceLibrary>) {
    refresh_draft_stash(&mut commands, &library);
}

pub fn on_confirm_click(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    draft_entities: Query<Entity, With<DraftPiece>>,
    piece_query: Query<&Piece>,
    library: Res<PieceLibrary>,
) {
    for entity in &draft_entities {
        if let Ok(piece) = piece_query.get(entity) {
            if piece.placed_at.is_some() {
                commands.entity(entity)
                    .remove::<DraftPiece>()
                    .insert(LockedPiece);
                info!("Piece {:?} locked", entity);
            } else {
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
    refresh_draft_stash(&mut commands, &library);
}