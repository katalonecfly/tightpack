use crate::components::*;
use crate::helpers::TILE_SIZE;
use crate::resources::{GameState, TooltipState};
use crate::systems::scoring::check_condition;
use bevy::prelude::*;
use bevy::window::Window;
use crate::helpers::{score_text_world_pos, SCORE_FONT_SIZE};

pub fn update_stash_labels(
    mut label_query: Query<(&mut Text2d, &StashLabel)>,
    piece_query: Query<(&Piece, &Transform)>,
) {
    for (mut text, label) in &mut label_query {
        let count = piece_query
            .iter()
            .filter(|(p, t)| p.type_id == label.0 && p.placed_at.is_none() && t.translation.z < 5.0)
            .count();
        text.0 = format!("x{}", count);
    }
}

pub fn update_score_ui(
    state: Res<GameState>,
    mut query: Query<(&mut Text2d, &mut Transform), With<ScoreText>>,
) {
    if state.is_changed() {
        for (mut text2d, mut transform) in &mut query {
            let score_str = format!("Score: {}", state.score);
            text2d.0 = score_str.clone();
            transform.translation = score_text_world_pos(&score_str, SCORE_FONT_SIZE);
        }
    }
}

pub fn update_effect_previews(
    state: Res<GameState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
) {
    for (piece, children, is_hovered) in &piece_query {
        for &child in children {
            if let Ok((mut visibility, mut sprite, preview)) = preview_query.get_mut(child) {
                if is_hovered {
                    *visibility = Visibility::Visible;
                    let mut active = false;

                    if let Some(grid_pos) = piece.placed_at {
                        let target_cell = grid_pos + preview.offset;
                        if crate::helpers::is_in_bounds(target_cell) {
                            active = check_condition(&preview.condition, Some(target_cell), &state);
                        }
                    }

                    if active {
                        sprite.color = Color::srgb(1.0, 1.0, 0.0).into();
                        sprite.custom_size = Some(Vec2::splat(12.0));
                    } else {
                        sprite.color = Color::srgba(1.0, 1.0, 0.0, 0.4).into();
                        sprite.custom_size = Some(Vec2::splat(8.0));
                    }
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn update_tooltip(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    piece_query: Query<(&Piece, &Transform, Has<Hovered>, Has<Dragging>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let hovered_piece = piece_query
        .iter()
        .find(|(_, _, hovered, dragging)| *hovered && !*dragging);

    match hovered_piece {
        Some((piece, transform, _, _)) => {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for offset in &piece.shape {
                let local = Vec3::new(
                    offset.x as f32 * TILE_SIZE,
                    offset.y as f32 * TILE_SIZE,
                    0.0,
                );
                let world = transform.transform_point(local);
                min_x = min_x.min(world.x);
                max_x = max_x.max(world.x);
                min_y = min_y.min(world.y);
                max_y = max_y.max(world.y);
            }

            let right_center = Vec2::new(max_x + TILE_SIZE, (min_y + max_y) / 2.0);

            if let Ok((camera, cam_transform)) = camera_query.single() {
                if let Ok(window) = windows.single() {
                    if let Some(ndc) = camera.world_to_ndc(cam_transform, right_center.extend(0.0))
                    {
                        // Convert NDC (-1..1) to screen coordinates (origin at top-left)
                        let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                        let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                        let mut text = format!("Gain {} points.", piece.points);
                        if !piece.effects.is_empty() {
                            text.push_str("\n\nEffects:");
                            for effect in &piece.effects {
                                text.push_str("\n- ");
                                text.push_str(&effect.description);
                            }
                        }

                        if let Some(entity) = tooltip_state.entity {
                            commands.entity(entity).insert((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(screen_x + 12.0),
                                    top: Val::Px(screen_y),
                                    max_width: Val::Px(250.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                Text::new(text),
                            ));
                        } else {
                            let entity = commands
                                .spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(screen_x + 12.0),
                                        top: Val::Px(screen_y),
                                        max_width: Val::Px(250.0),
                                        padding: UiRect::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                                    BorderColor::all(Color::WHITE),
                                    GlobalZIndex(20),
                                    Text::new(text),
                                ))
                                .id();
                            tooltip_state.entity = Some(entity);
                        }
                    }
                }
            }
        }
        None => {
            if let Some(entity) = tooltip_state.entity.take() {
                commands.entity(entity).despawn();
            }
        }
    }
}
