use crate::components::*;
use crate::helpers::TILE_SIZE;
use crate::resources::{GameState, DuelState, TooltipState};
use crate::systems::scoring::check_condition;
use bevy::prelude::*;
use bevy::window::Window;
use crate::helpers::{score_text_world_pos, score_text_world_pos_for_side, SCORE_FONT_SIZE};
use crate::config::EffectDescriptions;
use crate::systems::scoring::linear_rgba_near;

fn color_name_from_rgba(rgba: &LinearRgba) -> &'static str {
    let red = Color::srgb_u8(216, 46, 63).to_linear();
    let blue = Color::srgb_u8(53, 129, 216).to_linear();
    let green = Color::srgb_u8(40, 204, 45).to_linear();
    if linear_rgba_near(rgba, &red) {
        "RED"
    } else if linear_rgba_near(rgba, &blue) {
        "BLUE"
    } else if linear_rgba_near(rgba, &green) {
        "GREEN"
    } else {
        "UNKNOWN"
    }
}

fn get_effect_description(cond: &EffectCondition, descs: &EffectDescriptions) -> String {
    let key = match cond {
        EffectCondition::MatchesColor(c) => format!("MatchesColor({})", color_name_from_rgba(c)),
        EffectCondition::IsEmpty => "IsEmpty".to_string(),
        EffectCondition::NoColorOnBoard(c) => format!("NoColorOnBoard({})", color_name_from_rgba(c)),
    };
    descs.descriptions
        .get(&key)
        .cloned()
        .unwrap_or_else(|| format!("Unknown effect: {}", key))
}

pub fn update_stash_labels(
    mut label_query: Query<(
        &mut Text2d, &StashLabel, Option<&PlayerPiece>, Option<&OpponentPiece>,
    )>,
    piece_query_with_side: Query<(&Piece, &Transform, Option<&PlayerPiece>, Option<&OpponentPiece>)>,
) {
    for (mut text, label, label_has_player, label_has_opponent) in &mut label_query {
        let count = piece_query_with_side
            .iter()
            .filter(|(p, t, piece_player, piece_opponent)| {
                p.type_id == label.0
                    && p.placed_at.is_none()
                    && t.translation.z < 5.0
                    // If label is marked for a side, only count pieces of that side.
                    && (label_has_player.is_some() && piece_player.is_some()
                        || label_has_opponent.is_some() && piece_opponent.is_some()
                        || (label_has_player.is_none() && label_has_opponent.is_none()))
            })
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

pub fn update_duel_score_ui(
    duel_state: Res<DuelState>,
    mut player_query: Query<(&mut Text2d, &mut Transform), (With<PlayerScoreText>, Without<OpponentScoreText>)>,
    mut opponent_query: Query<(&mut Text2d, &mut Transform), (With<OpponentScoreText>, Without<PlayerScoreText>)>,
) {
    if duel_state.is_changed() {
        for (mut text, mut transform) in &mut player_query {
            let score_str = format!("Player: {}", duel_state.player.score);
            text.0 = score_str.clone();
            transform.translation = score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Left);
        }
        for (mut text, mut transform) in &mut opponent_query {
            let score_str = format!("Opponent: {}", duel_state.opponent.score);
            text.0 = score_str.clone();
            transform.translation = score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Right);
        }
    }
}

pub fn update_duel_effect_previews(
    duel_state: Res<DuelState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
) {
    for (piece, children, is_hovered) in &piece_query {
        for &child in children {
            if let Ok((mut visibility, mut sprite, preview)) = preview_query.get_mut(child) {
                if is_hovered {
                    *visibility = Visibility::Visible;
                    let mut active = false;
                    let board_cells = match piece.board_side {
                        BoardSide::Left => &duel_state.player.board_cells,
                        BoardSide::Right => &duel_state.opponent.board_cells,
                        _ => continue,
                    };
                    if let Some(grid_pos) = piece.placed_at {
                        let target_cell = grid_pos + preview.offset;
                        if crate::helpers::is_in_bounds(target_cell) {
                            active = check_condition(&preview.condition, Some(target_cell), board_cells);
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
                            active = check_condition(&preview.condition, Some(target_cell), &state.board_cells);
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
    effect_descs: Res<EffectDescriptions>,  // <-- new parameter
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
                    if let Some(ndc) = camera.world_to_ndc(cam_transform, right_center.extend(0.0)) {
                        let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                        let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                        let mut text = format!("Gain {} points.", piece.points);
                        if !piece.effects.is_empty() {
                            text.push_str("\n\nEffects:");
                            for effect in &piece.effects {
                                text.push_str("\n- ");
                                let desc_template = get_effect_description(&effect.condition, &effect_descs);
                                let desc = desc_template
                                    .replace("{points}", &effect.points.to_string())
                                    .replace("{color}", match &effect.condition {
                                        EffectCondition::MatchesColor(c) => color_name_from_rgba(c),
                                        EffectCondition::IsEmpty => "empty",
                                        EffectCondition::NoColorOnBoard(c) => color_name_from_rgba(c),
                                    });
                                text.push_str(&desc);
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