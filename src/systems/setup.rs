use bevy::prelude::*;
use std::collections::HashMap;
use crate::config::*;
use crate::components::*;
use crate::helpers::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let mut color_map = HashMap::new();
    color_map.insert("RED".to_string(), LinearRgba::RED);
    color_map.insert("BLUE".to_string(), LinearRgba::BLUE);
    color_map.insert("GREEN".to_string(), LinearRgba::GREEN);
    color_map.insert("YELLOW".to_string(), LinearRgba::new(1.0, 1.0, 0.0, 1.0));

    let file_content = std::fs::read_to_string("assets/pieces.ron").expect("Missing assets/pieces.ron");
    let lib: RawPieceLibrary = ron::from_str(&file_content).expect("Failed to parse RON");

    // Board
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world(IVec2::new(x, y))),
            ));
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
            // Convert Raw Vec to Internal Option
            let offsets = if re.offsets.is_empty() { None } else { Some(re.offsets) };
            GameEffect { condition, points: re.points, offsets }
        }).collect();

        let pos = INVENTORY_OFFSET + Vec3::new(0.0, 150.0 - (type_id as f32 * 100.0), 1.0);
        let count = 10;

        commands.spawn((
            Text2d::new(format!("x{}", count)),
            TextFont { font_size: 24.0, ..default() },
            Transform::from_translation(pos + Vec3::new(-45.0, 35.0, 2.0)),
            StashLabel(type_id),
        ));

        for _ in 0..count {
            spawn_draggable_piece(&mut commands, type_id, raw.shape.clone(), piece_color, raw.points, baked_effects.clone(), pos);
        }
    }

    commands.spawn((
        Text::new("Score: 0"),
        TextFont { font_size: 40.0, ..default() },
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() },
        ScoreText,
    ));
}

fn spawn_draggable_piece(commands: &mut Commands, type_id: usize, shape: Vec<IVec2>, color: LinearRgba, points: i32, effects: Vec<GameEffect>, pos: Vec3) {
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
        }
    ))
    .observe(crate::systems::interaction::on_drag_start)
    .observe(crate::systems::interaction::on_drag)
    .observe(crate::systems::interaction::on_drag_end)
    .observe(crate::systems::interaction::on_hover_in)
    .observe(crate::systems::interaction::on_hover_out)
    .id();

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