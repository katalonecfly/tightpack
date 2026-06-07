use crate::Cleanup;
use crate::components::StashPosition;
use crate::components::*;
use crate::config::*;
use crate::helpers::BOARD_TOP_Y;
use crate::helpers::*;
use crate::resources::PieceLibrary;
use crate::resources::{InventoryScroll, StashContentHeight, StashScreenRect};
use crate::systems::draft::DraftConfirmButton; // <-- added import
use bevy::prelude::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::collections::HashMap;

const AVAILABLE_COLORS: &[&str] = &["RED", "BLUE", "GREEN"];

fn spawn_common(commands: &mut Commands) -> Vec<RawPieceConfig> {
    commands.spawn((Camera2d, Cleanup));

    let file_content = include_str!("../../assets/pieces.ron");

    let lib: RawPieceLibrary = ron::from_str(&file_content).expect("Failed to parse RON");
    let pieces = lib.pieces.clone();
    commands.insert_resource(PieceLibrary(lib.pieces));

    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world(IVec2::new(x, y))),
                ))
                .id();
            commands.entity(board_root).add_child(tile);
        }
    }

    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos("Score: 0", SCORE_FONT_SIZE)),
        ScoreText,
        Cleanup,
    ));

    pieces
}

pub fn setup_sandbox(mut commands: Commands, windows: Query<&Window>) {
    let pieces = spawn_common(&mut commands);
    let window = windows.single().expect("Primary window missing");

    let stash_left = stash_left_x();
    let stash_width = STASH_WIDTH;
    let stash_visible_height = STASH_VISIBLE_HEIGHT;

    let board_top_y = BOARD_TOP_Y + TILE_SIZE / 2.0;
    let stash_top = board_top_y;
    let stash_bottom = stash_top - stash_visible_height;
    let stash_right = stash_left + stash_width;

    // Screen‑space rectangle for mouse‑wheel detection
    let screen_x = (window.width() / 2.0) + stash_left;
    let screen_y = (window.height() / 2.0) - stash_top;
    commands.insert_resource(StashScreenRect {
        x: screen_x,
        y: screen_y,
        width: stash_width,
        height: stash_visible_height,
    });

    // Stash outline (perimeter)
    let outline_color = Color::srgba(0.4, 0.4, 0.4, 0.6);
    let thickness = 2.0;
    commands
        .spawn((Transform::default(), Visibility::default(), Cleanup))
        .with_children(|parent| {
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(thickness, stash_visible_height)),
                Transform::from_xyz(stash_left, (stash_top + stash_bottom) / 2.0, 0.5),
            ));
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(thickness, stash_visible_height)),
                Transform::from_xyz(stash_right, (stash_top + stash_bottom) / 2.0, 0.5),
            ));
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(stash_width, thickness)),
                Transform::from_xyz((stash_left + stash_right) / 2.0, stash_top, 0.5),
            ));
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(stash_width, thickness)),
                Transform::from_xyz((stash_left + stash_right) / 2.0, stash_bottom, 0.5),
            ));
        });

    // Pieces and labels (unchanged except using stash_left)
    let color_map: HashMap<String, LinearRgba> = [
        ("RED".into(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".into(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".into(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".into(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let mut current_y_offset = 0.0f32;
    for (type_id, raw) in pieces.iter().enumerate() {
        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let min_y = raw.shape.iter().map(|o| o.y).min().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let piece_height = (max_y - min_y + 1) as f32 * TILE_SIZE;

        let stash_center_x = stash_left + stash_width / 2.0;
        let piece_x = stash_center_x - ((min_x + max_x) as f32) / 2.0 * TILE_SIZE;

        let top_offset = max_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let base_y = stash_top - current_y_offset - top_offset;

        let copy_count = 10;
        for copy_idx in 0..copy_count {
            let (color, effects) = randomize_piece_properties(raw, &color_map);
            let pos = Vec3::new(piece_x, base_y, 1.0 + copy_idx as f32 * 0.001);
            // Inside setup_sandbox, around line 100
            // In setup_sandbox (around line 100)
            // Inside setup_sandbox, around line 100
            let entity = spawn_draggable_piece(
                &mut commands,
                type_id,
                raw.shape.clone(),
                color,
                raw.points,
                effects,
                pos,
                false, // draft_mode
                true,  // interactive
                true,  // hoverable   <-- added
                BoardSide::Single,
            );
            commands.entity(entity).insert(StashPosition {
                desired_world_y: base_y,
            });
        }

        let label_y = base_y + max_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        commands.spawn((
            Text2d::new(format!("x{}", copy_count)),
            TextFont {
                font_size: STASH_LABEL_FONT_SIZE,
                ..default()
            },
            Transform::from_translation(Vec3::new(piece_x, label_y, 2.0)),
            StashLabel(type_id),
            StashPosition {
                desired_world_y: label_y,
            },
            Cleanup,
        ));

        current_y_offset += piece_height + TILE_SIZE;
    }

    commands.insert_resource(InventoryScroll::default());
    commands.insert_resource(StashContentHeight(current_y_offset));
}

pub fn setup_draft(mut commands: Commands) {
    let _pieces = spawn_common(&mut commands);

    let board_right = board_right_edge(BoardSide::Single);
    let board_top = board_top_edge();
    let score_y = board_top + SCORE_Y_OFFSET;
    let button_pos = Vec3::new(board_right - CONFIRM_BUTTON_WIDTH / 2.0, score_y, 0.0);
    commands
        .spawn((
            Sprite::from_color(
                Color::srgb(0.3, 0.8, 0.3),
                Vec2::new(CONFIRM_BUTTON_WIDTH, CONFIRM_BUTTON_HEIGHT),
            ),
            Transform::from_translation(button_pos),
            Pickable::default(),
            DraftConfirmButton,
            Cleanup,
        ))
        .with_child((
            Text2d::new("Confirm"),
            TextFont {
                font_size: CONFIRM_BUTTON_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::default(),
        ))
        .observe(crate::systems::draft::on_confirm_click);
}

pub fn bake_effects(
    raw: &RawPieceConfig,
    color_map: &HashMap<String, LinearRgba>,
) -> Vec<GameEffect> {
    raw.effects
        .iter()
        .map(|re| {
            let condition = match &re.condition {
                RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
                RawEffectCondition::MatchesColor(name) => EffectCondition::MatchesColor(
                    *color_map.get(name).unwrap_or(&LinearRgba::WHITE),
                ),
                RawEffectCondition::NoColorOnBoard(name) => EffectCondition::NoColorOnBoard(
                    *color_map.get(name).unwrap_or(&LinearRgba::WHITE),
                ),
            };
            GameEffect {
                condition,
                points: re.points,
                offsets: if re.offsets.is_empty() {
                    None
                } else {
                    Some(re.offsets.clone())
                },
            }
        })
        .collect()
}

/// For a dynamic piece: randomly choose a color and one effect + non‑empty offset subset.
/// For a static piece: use the color from the RON and all effects/offsets.
pub fn randomize_piece_properties(
    raw: &RawPieceConfig,
    color_map: &HashMap<String, LinearRgba>,
) -> (LinearRgba, Vec<GameEffect>) {
    if raw.piece_type == PieceType::Static {
        let color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        let effects = bake_effects(raw, color_map);
        return (color, effects);
    }

    // Dynamic piece: must have at least one effect, otherwise fallback
    if raw.effects.is_empty() {
        let color = random_color(color_map);
        return (color, Vec::new());
    }

    let mut rng = rand::rng();
    let chosen_raw = raw.effects.choose(&mut rng).unwrap();

    let condition = match &chosen_raw.condition {
        RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
        RawEffectCondition::MatchesColor(name) => {
            EffectCondition::MatchesColor(*color_map.get(name).unwrap_or(&LinearRgba::WHITE))
        }
        RawEffectCondition::NoColorOnBoard(name) => {
            EffectCondition::NoColorOnBoard(*color_map.get(name).unwrap_or(&LinearRgba::WHITE))
        }
    };

    let chosen_offsets = if chosen_raw.offsets.is_empty() {
        None
    } else {
        let mut offsets = chosen_raw.offsets.clone();
        let k = rng.random_range(1..=offsets.len());
        offsets.shuffle(&mut rng);
        offsets.truncate(k);
        Some(offsets)
    };

    let effects = vec![GameEffect {
        condition,
        points: chosen_raw.points,
        offsets: chosen_offsets,
    }];

    let color = random_color(color_map);
    (color, effects)
}

/// Pick a random color from the available list.
pub fn random_color(color_map: &HashMap<String, LinearRgba>) -> LinearRgba {
    let mut rng = rand::rng();
    let color_name = AVAILABLE_COLORS.choose(&mut rng).unwrap();
    *color_map.get(*color_name).unwrap_or(&LinearRgba::WHITE)
}

// In systems/setup.rs

// In systems/setup.rs

// In systems/setup.rs

// In systems/setup.rs

pub fn spawn_draggable_piece(
    commands: &mut Commands,
    type_id: usize,
    shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32,
    effects: Vec<GameEffect>,
    pos: Vec3,
    draft_mode: bool,
    interactive: bool,
    hoverable: bool,
    board_side: BoardSide,
) -> Entity {
    let entity = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            Piece {
                type_id,
                shape: shape.clone(),
                original_shape: shape.clone(),
                color,
                points,
                effects: effects.clone(),
                original_effects: effects.clone(),
                original_pos: pos,
                placed_at: None,
                board_side,
            },
            Cleanup,
        ))
        .id();

    if draft_mode {
        commands.entity(entity).insert(DraftPiece);
    }

    // Attach observers to parent
    if hoverable {
        commands
            .entity(entity)
            .insert(Pickable::default())
            .observe(crate::systems::interaction::on_hover_in)
            .observe(crate::systems::interaction::on_hover_out);
    }
    if interactive {
        commands
            .entity(entity)
            .observe(crate::systems::interaction::on_drag_start)
            .observe(crate::systems::interaction::on_drag)
            .observe(crate::systems::interaction::on_drag_end);
    }

    // Draw the full visual appearance (bridges, perimeters)
    crate::systems::visuals::refresh_piece_visuals(commands, entity, &shape, color);

    // Add interactive children (small tile sprites and effect previews)
    commands.entity(entity).with_children(|parent| {
        // Small tile sprites (these are the hitboxes for dragging/hovering)
        for offset in &shape {
            let mut child = parent.spawn((
                Sprite::from_color(color, Vec2::splat(TILE_SIZE - 4.0)),
                Transform::from_translation(offset.as_vec2().extend(0.0) * TILE_SIZE),
            ));
            if hoverable {
                child.insert(Pickable::default());
                child
                    .observe(crate::systems::interaction::on_child_hover_in)
                    .observe(crate::systems::interaction::on_child_hover_out);
            }
            if interactive {
                child
                    .observe(crate::systems::interaction::on_drag_start)
                    .observe(crate::systems::interaction::on_drag)
                    .observe(crate::systems::interaction::on_drag_end);
            }
        }

        // Effect preview sprites
        for effect in &effects {
            if let Some(offsets) = &effect.offsets {
                for offset in offsets {
                    let mut preview = parent.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0).into(),
                            custom_size: Some(Vec2::splat(12.0)),
                            ..default()
                        },
                        Transform::from_translation(offset.as_vec2().extend(5.0) * TILE_SIZE),
                        Visibility::Hidden,
                        EffectPreview {
                            offset: *offset,
                            condition: effect.condition.clone(),
                        },
                    ));
                    if hoverable {
                        preview.insert(Pickable::default());
                        preview
                            .observe(crate::systems::interaction::on_child_hover_in)
                            .observe(crate::systems::interaction::on_child_hover_out);
                    }
                    if interactive {
                        preview
                            .observe(crate::systems::interaction::on_drag_start)
                            .observe(crate::systems::interaction::on_drag)
                            .observe(crate::systems::interaction::on_drag_end);
                    }
                }
            }
        }
    });

    entity
}
