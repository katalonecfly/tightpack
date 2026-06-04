use crate::Cleanup;
use crate::components::LockedPiece;
use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::{GameSettings, PieceLibrary, RoundCounter};
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use rand::RngExt;
use std::collections::HashMap;

#[derive(Component)]
pub struct DraftConfirmButton;

#[derive(Component)]
pub struct RoundText;

pub fn refresh_draft_stash(commands: &mut Commands, library: &PieceLibrary, round_counter: &RoundCounter) {
    if round_counter.is_game_over() {
        return;
    }
    let color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let mut rng = rand::rng();
    let all_pieces = &library.0;

    let mut available: Vec<&RawPieceConfig> = all_pieces.iter().collect();
    let mut chosen = Vec::with_capacity(3);
    for _ in 0..3 {
        if available.is_empty() {
            break;
        }
        let idx = rng.random_range(0..available.len());
        chosen.push(available.remove(idx));
    }

    let board_left = grid_to_world(IVec2::ZERO).x;
    let mut next_left = board_left;

    for (i, raw) in chosen.iter().enumerate() {
        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let width = (max_x - min_x + 1) as f32 * TILE_SIZE;

        let piece_left = next_left;
        let parent_x = piece_left - (min_x as f32) * TILE_SIZE;
        let parent_y = crate::helpers::stash_y_below_board(max_y);
        let pos = Vec3::new(parent_x, parent_y, 1.0);

        let label_y = parent_y + (max_y as f32) * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        let label_pos = Vec3::new(parent_x, label_y, 2.0);

        let type_id = all_pieces
            .iter()
            .position(|p| std::ptr::eq(p, *raw))
            .unwrap_or(i);
        let (color, effects) = crate::systems::setup::randomize_piece_properties(raw, &color_map);
        commands.spawn((
            Text2d::new("x1"),
            TextFont {
                font_size: STASH_LABEL_FONT_SIZE,
                ..default()
            },
            Transform::from_translation(label_pos),
            StashLabel(type_id),
            DraftPiece,
            Cleanup,
        ));

        crate::systems::setup::spawn_draggable_piece(
            commands,
            type_id,
            raw.shape.clone(),
            color,
            raw.points,
            effects,
            pos,
            true,
            true,
            true,
            BoardSide::Single,
        );

        next_left = piece_left + width + TILE_SIZE;
    }
}

pub fn generate_draft_stash(mut commands: Commands, library: Res<PieceLibrary>, settings: Res<GameSettings>) {
    let round_counter = RoundCounter::new(settings.rounds);
    commands.insert_resource(round_counter);
    refresh_draft_stash(&mut commands, &library, &RoundCounter::new(settings.rounds));
}

pub fn on_confirm_click(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    draft_entities: Query<Entity, With<DraftPiece>>,
    piece_query: Query<&Piece>,
    library: Res<PieceLibrary>,
    mut round_counter: ResMut<RoundCounter>,
    _settings: Res<GameSettings>,
) {
    // Disable if game over
    if round_counter.is_game_over() {
        return;
    }

    for entity in &draft_entities {
        if let Ok(piece) = piece_query.get(entity) {
            if piece.placed_at.is_some() {
                commands
                    .entity(entity)
                    .remove::<DraftPiece>()
                    .insert(LockedPiece);
            } else {
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }

    round_counter.advance();
    if !round_counter.is_game_over() {
        refresh_draft_stash(&mut commands, &library, &round_counter);
    }
}

pub fn update_draft_round_display(
    round_counter: Res<RoundCounter>,
    mut commands: Commands,
    button_query: Query<Entity, With<DraftConfirmButton>>,
    existing_text: Query<Entity, With<RoundText>>,
    transforms: Query<&Transform>,
    mut button_sprite: Query<&mut Sprite, With<DraftConfirmButton>>,
) {
    // Remove old round text
    for entity in existing_text.iter() {
        commands.entity(entity).despawn();
    }

    if let Ok(button_entity) = button_query.single() {
        let is_game_over = round_counter.is_game_over();
        // Update button sprite color
        if let Ok(mut sprite) = button_sprite.get_mut(button_entity) {
            sprite.color = if is_game_over {
                Color::srgb(0.5, 0.5, 0.5)
            } else {
                Color::srgb(0.3, 0.8, 0.3)
            };
        }

        // Always show round text, cap current at total for display
        let displayed_current = round_counter.current.min(round_counter.total);
        if let Ok(button_transform) = transforms.get(button_entity) {
            let text_pos = button_transform.translation + Vec3::new(CONFIRM_BUTTON_WIDTH / 2.0 + 60.0, 0.0, 0.0);
            let text_content = format!("Round:\n{}/{}", displayed_current, round_counter.total);
            commands.spawn((
                Text2d::new(text_content),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_translation(text_pos),
                RoundText,
                Cleanup,
            ));
        }
    }
}