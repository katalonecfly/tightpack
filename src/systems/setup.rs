use bevy::prelude::*;
use std::collections::HashMap;
use crate::config::*;
use crate::components::*;
use crate::helpers::*;
use crate::resources::PieceLibrary;
use crate::Cleanup;
use crate::systems::draft::DraftConfirmButton;   // <-- import
use bevy::sprite::Anchor;

// ─── S A N D B O X ────────────────────────────────────
pub fn setup_sandbox(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let mut color_map = HashMap::new();         // <-- put it back
    color_map.insert("RED".into(), Color::srgb_u8(216, 46, 63).to_linear());
    color_map.insert("BLUE".into(), Color::srgb_u8(53, 129, 216).to_linear());
    color_map.insert("GREEN".into(), Color::srgb_u8(40, 204, 45).to_linear());
    color_map.insert("YELLOW".into(), Color::srgb_u8(255, 225, 53).to_linear());

    let file_content = std::fs::read_to_string("assets/pieces.ron")
        .expect("Missing pieces.ron");
    let lib: RawPieceLibrary = ron::from_str(&file_content)
        .expect("Failed to parse RON");
    commands.insert_resource(PieceLibrary(lib.pieces.clone()));

    // Board
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2),
                    Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world(IVec2::new(x, y))),
            )).id();
            commands.entity(board_root).add_child(tile);
        }
    }

    // Pieces (vertical stash)
    for (type_id, raw) in lib.pieces.iter().enumerate() {
        let piece_color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        let baked = bake_effects(raw, &color_map);
        let top_y = (BOARD_SIZE.y - 1) as f32 * TILE_SIZE;
        let pos = INVENTORY_OFFSET
            + Vec3::new(0.0, top_y - (type_id as f32 * 100.0), 1.0);
        let count = 10;

        commands.spawn((
            Text2d::new(format!("x{}", count)),
            TextFont { font_size: 24.0, ..default() },
            Transform::from_translation(pos + Vec3::new(-45.0, 35.0, 2.0)),
            StashLabel(type_id),
            Cleanup,
        ));
        for _ in 0..count {
            spawn_draggable_piece(
                &mut commands,
                type_id,
                raw.shape.clone(),
                piece_color,
                raw.points,
                baked.clone(),
                pos,
                false,
            );
        }
    }

    // ─── Score (2D world text) ──────────────────
    let board_left = grid_to_world(IVec2::ZERO).x - TILE_SIZE / 2.0;
    let board_top = grid_to_world(IVec2::new(0, BOARD_SIZE.y - 1)).y + TILE_SIZE / 2.0;
    let score_y = board_top + 30.0;

    let score_font_size = 30.0;
    let initial_text = "Score: 0";
    // approximate half-width: each character ≈ 0.25 * font_size
    let initial_half_width = initial_text.len() as f32 * score_font_size * 0.25;

    commands.spawn((
        Text2d::new(initial_text),
        TextFont { font_size: score_font_size, ..default() },
        Transform::from_translation(Vec3::new(board_left + initial_half_width, score_y, 0.0)),
        ScoreText,
        Cleanup,
    ));
}

// ─── D R A F T   S E T U P ─────────────────────────────
pub fn setup_draft(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let file_content = std::fs::read_to_string("assets/pieces.ron")
        .expect("Missing pieces.ron");
    let lib: RawPieceLibrary = ron::from_str(&file_content)
        .expect("Failed to parse RON");
    commands.insert_resource(PieceLibrary(lib.pieces.clone()));

    // Board
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2),
                    Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world(IVec2::new(x, y))),
            )).id();
            commands.entity(board_root).add_child(tile);
        }
    }

    // Position calculations
    // ... after board generation
    let board_left = grid_to_world(IVec2::ZERO).x - TILE_SIZE / 2.0;
    let board_top = grid_to_world(IVec2::new(0, BOARD_SIZE.y - 1)).y + TILE_SIZE / 2.0;
    let score_y = board_top + 30.0;

    let score_font_size = 30.0;
    let initial_text = "Score: 0";
    // approximate half-width: each character ≈ 0.25 * font_size
    let initial_half_width = initial_text.len() as f32 * score_font_size * 0.25;

    commands.spawn((
        Text2d::new(initial_text),
        TextFont { font_size: score_font_size, ..default() },
        Transform::from_translation(Vec3::new(board_left + initial_half_width, score_y, 0.0)),
        ScoreText,
        Cleanup,
    ));
    // Confirm button remains unchanged (already aligned to the right).

    // Confirm button (world-space sprite)
    let board_right = grid_to_world(IVec2::new(BOARD_SIZE.x - 1, BOARD_SIZE.y - 1)).x
        + TILE_SIZE / 2.0;
    let button_width = 120.0;
    let button_height = 50.0;
    let button_x = board_right - button_width / 2.0;
    let button_y = score_y;   // same height as score
    let button_pos = Vec3::new(button_x, button_y, 0.0);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.8, 0.3), Vec2::new(button_width, button_height)),
        Transform::from_translation(button_pos),
        Pickable::default(),
        DraftConfirmButton,
        Cleanup,
    ))
    .with_child((
        Text2d::new("Confirm"),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::WHITE),
        Transform::default(),
    ))
    .observe(super::draft::on_confirm_click);
}

// ─── H E L P E R S ─────────────────────────────────────
pub fn bake_effects(
    raw: &RawPieceConfig,
    color_map: &HashMap<String, LinearRgba>,
) -> Vec<GameEffect> {
    raw.effects
        .iter()
        .map(|re| {
            let condition = match &re.condition {
                RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
                RawEffectCondition::MatchesColor(name) => {
                    EffectCondition::MatchesColor(
                        *color_map.get(name).unwrap_or(&LinearRgba::WHITE),
                    )
                }
                RawEffectCondition::NoColorOnBoard(name) => {
                    EffectCondition::NoColorOnBoard(
                        *color_map.get(name).unwrap_or(&LinearRgba::WHITE),
                    )
                }
            };
            GameEffect {
                condition,
                points: re.points,
                offsets: if re.offsets.is_empty() {
                    None
                } else {
                    Some(re.offsets.clone())
                },
                description: re.description.clone(),
            }
        })
        .collect()
}

pub fn spawn_draggable_piece(
    commands: &mut Commands,
    type_id: usize,
    shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32,
    effects: Vec<GameEffect>,
    pos: Vec3,
    draft_mode: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Transform::from_translation(pos),
        Visibility::default(),
        Pickable::default(),
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
        },
        Cleanup,
    ));
    if draft_mode {
        entity.insert(DraftPiece);
    }
    let parent = entity
        .observe(crate::systems::interaction::on_drag_start)
        .observe(crate::systems::interaction::on_drag)
        .observe(crate::systems::interaction::on_drag_end)
        .observe(crate::systems::interaction::on_hover_in)
        .observe(crate::systems::interaction::on_hover_out)
        .id();

    crate::systems::visuals::refresh_piece_visuals(commands, parent, &shape, color);

    use crate::systems::interaction;

    for offset in shape {
        let child = commands
            .spawn((
                Sprite::from_color(color, Vec2::splat(TILE_SIZE - 4.0)),
                Transform::from_translation(offset.as_vec2().extend(0.0) * TILE_SIZE),
                Pickable::default(),
            ))
            .observe(interaction::on_child_hover_in)
            .observe(interaction::on_child_hover_out)
            .observe(interaction::on_drag_start)
            .observe(interaction::on_drag)
            .observe(interaction::on_drag_end)
            .id();
        commands.entity(parent).add_child(child);
    }

    for effect in effects {
        if let Some(offsets) = effect.offsets {
            for offset in offsets {
                let preview = commands
                    .spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0).into(),
                            custom_size: Some(Vec2::splat(12.0)),
                            ..default()
                        },
                        Transform::from_translation(
                            offset.as_vec2().extend(5.0) * TILE_SIZE,
                        ),
                        Visibility::Hidden,
                        EffectPreview {
                            offset,
                            condition: effect.condition.clone(),
                        },
                    ))
                    .observe(interaction::on_child_hover_in)
                    .observe(interaction::on_child_hover_out)
                    .observe(interaction::on_drag_start)
                    .observe(interaction::on_drag)
                    .observe(interaction::on_drag_end)
                    .id();
                commands.entity(parent).add_child(preview);
            }
        }
    }
    parent
}

/*
use bevy::prelude::*;
use std::collections::HashMap;
use crate::config::*;
use crate::components::*;
use crate::helpers::*;
use crate::Cleanup;

pub fn setup_game(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let mut color_map = HashMap::new();
    color_map.insert("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear());
    color_map.insert("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear());
    color_map.insert("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear());
    color_map.insert("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear());

    let file_content = std::fs::read_to_string("assets/pieces.ron").expect("Missing assets/pieces.ron");
    let lib: RawPieceLibrary = ron::from_str(&file_content).expect("Failed to parse RON");

    // Board
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world(IVec2::new(x, y))),
            )).id();
            commands.entity(board_root).add_child(tile);
        }
    }

    // Pieces
    for (type_id, raw) in lib.pieces.into_iter().enumerate() {
        let piece_color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);

        let baked_effects: Vec<GameEffect> = raw.effects.into_iter().map(|re| {
            let condition = match re.condition {
                RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
                RawEffectCondition::MatchesColor(name) =>
                    EffectCondition::MatchesColor(*color_map.get(&name).unwrap_or(&LinearRgba::WHITE)),
                RawEffectCondition::NoColorOnBoard(name) =>
                    EffectCondition::NoColorOnBoard(*color_map.get(&name).unwrap_or(&LinearRgba::WHITE)),
            };
            let offsets = if re.offsets.is_empty() { None } else { Some(re.offsets) };
            GameEffect {
                condition,
                points: re.points,
                offsets,
                description: re.description,
            }
        }).collect();

        let top_y = (BOARD_SIZE.y - 1) as f32 * TILE_SIZE;
        let pos = INVENTORY_OFFSET + Vec3::new(0.0, top_y - (type_id as f32 * 100.0), 1.0);
        let count = 10;

        commands.spawn((
            Text2d::new(format!("x{}", count)),
            TextFont { font_size: 24.0, ..default() },
            Transform::from_translation(pos + Vec3::new(-45.0, 35.0, 2.0)),
            StashLabel(type_id),
            Cleanup,
        ));

        for _ in 0..count {
            spawn_draggable_piece(&mut commands, type_id, raw.shape.clone(), piece_color, raw.points, baked_effects.clone(), pos);
        }
    }

    // Score UI
    commands.spawn((
        Text::new("Score: 0"),
        TextFont { font_size: 40.0, ..default() },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
        Cleanup,
    ));
}

fn spawn_draggable_piece(
    commands: &mut Commands,
    type_id: usize,
    shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32,
    effects: Vec<GameEffect>,
    pos: Vec3,
) {
    let parent = commands.spawn((
        Transform::from_translation(pos),
        Visibility::default(),
        Pickable::default(),
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
        },
        Cleanup,
    ))
    .observe(crate::systems::interaction::on_drag_start)
    .observe(crate::systems::interaction::on_drag)
    .observe(crate::systems::interaction::on_drag_end)
    .observe(crate::systems::interaction::on_hover_in)
    .observe(crate::systems::interaction::on_hover_out)
    .id();

    crate::systems::visuals::refresh_piece_visuals(commands, parent, &shape, color);

    for offset in shape {
        let child = commands.spawn((
            Sprite::from_color(color, Vec2::splat(TILE_SIZE - 4.0)),
            Transform::from_translation(offset.as_vec2().extend(0.0) * TILE_SIZE),
            Pickable::default(),
        )).id();
        commands.entity(parent).add_child(child);
    }

    for effect in effects {
        if let Some(offsets) = effect.offsets {
            for offset in offsets {
                let preview = commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 1.0, 0.0).into(),
                        custom_size: Some(Vec2::splat(12.0)),
                        ..default()
                    },
                    Transform::from_translation(offset.as_vec2().extend(5.0) * TILE_SIZE),
                    Visibility::Hidden,
                    EffectPreview { offset, condition: effect.condition.clone() },
                )).id();
                commands.entity(parent).add_child(preview);
            }
        }
    }
}
*/