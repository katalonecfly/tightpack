use bevy::prelude::*;
use crate::components::PieceVisual;
use crate::helpers::TILE_SIZE;

pub fn refresh_piece_visuals(
    commands: &mut Commands,
    entity: Entity,
    shape: &[IVec2],
    color: LinearRgba,
) {
    let inner_size = TILE_SIZE * 1.0;
    let bridge_width = TILE_SIZE * 1.0;
    let bridge_ext = (TILE_SIZE - inner_size) / 2.0;
    let line_thickness = 3.0;
    let perimeter_color = LinearRgba::BLACK;

    for &pos in shape {
        let center = pos.as_vec2() * TILE_SIZE;

        commands.entity(entity).with_children(|parent| {
            // --- Main Square ---
            parent.spawn((
                Sprite {
                    color: color.into(),
                    custom_size: Some(Vec2::splat(inner_size)),
                    ..default()
                },
                Transform::from_translation(center.extend(0.1)),
                PieceVisual,
                Pickable::default(),
            ));

            let directions = [
                (IVec2::X, Vec2::new(bridge_ext, bridge_width), Vec2::new(inner_size / 2.0 + bridge_ext / 2.0, 0.0)),
                (IVec2::NEG_X, Vec2::new(bridge_ext, bridge_width), Vec2::new(-(inner_size / 2.0 + bridge_ext / 2.0), 0.0)),
                (IVec2::Y, Vec2::new(bridge_width, bridge_ext), Vec2::new(0.0, inner_size / 2.0 + bridge_ext / 2.0)),
                (IVec2::NEG_Y, Vec2::new(bridge_width, bridge_ext), Vec2::new(0.0, -(inner_size / 2.0 + bridge_ext / 2.0))),
            ];

            for (dir, b_size, b_offset) in directions {
                if shape.contains(&(pos + dir)) {
                    // --- Bridge ---
                    parent.spawn((
                        Sprite { color: color.into(), custom_size: Some(b_size), ..default() },
                        Transform::from_translation((center + b_offset).extend(0.1)),
                        PieceVisual,
                        Pickable::default(),
                    ));
                } else {
                    // --- Perimeter Line ---
                    let is_horizontal = dir.y != 0;
                    let line_size = if is_horizontal { Vec2::new(TILE_SIZE, line_thickness) } else { Vec2::new(line_thickness, TILE_SIZE) };
                    let line_offset = dir.as_vec2() * (TILE_SIZE / 2.0);
                    
                    parent.spawn((
                        Sprite { color: perimeter_color.into(), custom_size: Some(line_size), ..default() },
                        Transform::from_translation((center + line_offset).extend(0.5)),
                        PieceVisual,
                        Pickable::default(),
                    ));
                }
            }
        });
    }
}