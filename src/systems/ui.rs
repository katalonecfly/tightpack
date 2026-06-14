use crate::Cleanup;
use crate::components::*;
use crate::config::EffectDescriptions;
use crate::helpers::*;
use crate::resources::{BoardSize, DuelState, GameState, TooltipState};
use crate::systems::scoring::{check_condition_with_sizes, linear_rgba_near, compute_piece_contribution};
use bevy::prelude::*;
use bevy::window::Window;
use crate::helpers::grid_to_world_for_side;

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
    match cond {
        EffectCondition::MatchesColor(c) => {
            let key = "MatchesColor(X)";
            let template = descs.descriptions.get(key).cloned().unwrap_or_else(|| format!("Unknown effect: {}", key));
            template.replace("{X}", color_name_from_rgba(c))
        }
        EffectCondition::IsEmpty => {
            descs.descriptions.get("IsEmpty").cloned().unwrap_or_else(|| "Unknown effect: IsEmpty".to_string())
        }
        EffectCondition::NoColorOnBoard(c) => {
            let key = "NoColorOnBoard(X)";
            let template = descs.descriptions.get(key).cloned().unwrap_or_else(|| format!("Unknown effect: {}", key));
            template.replace("{X}", color_name_from_rgba(c))
        }
        EffectCondition::MatchesSize(size) => {
            let key = "MatchesSize(X)";
            let template = descs.descriptions.get(key).cloned().unwrap_or_else(|| {
                "Unknown effect: MatchesSize(X)".to_string()
            });
            template.replace("{X}", &size.to_string())
        }
    }
}

pub fn update_stash_labels(
    mut label_query: Query<(
        &mut Text2d,
        &StashLabel,
        Option<&PlayerPiece>,
        Option<&OpponentPiece>,
    )>,
    piece_query_with_side: Query<(
        &Piece,
        &Transform,
        Option<&PlayerPiece>,
        Option<&OpponentPiece>,
    )>,
) {
    for (mut text, label, label_has_player, label_has_opponent) in &mut label_query {
        let count = piece_query_with_side
            .iter()
            .filter(|(p, t, piece_player, piece_opponent)| {
                p.type_id == label.0
                    && p.placed_at.is_none()
                    && t.translation.z < 5.0
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
    board_size: Res<BoardSize>,
) {
    if state.is_changed() {
        for (mut text2d, mut transform) in &mut query {
            let score_str = format!("Score: {}", state.score);
            text2d.0 = score_str.clone();
            transform.translation = score_text_world_pos(&score_str, SCORE_FONT_SIZE, board_size.0);
        }
    }
}

pub fn update_duel_score_ui(
    duel_state: Res<DuelState>,
    mut player_query: Query<
        (&mut Text2d, &mut Transform),
        (With<PlayerScoreText>, Without<OpponentScoreText>),
    >,
    mut opponent_query: Query<
        (&mut Text2d, &mut Transform),
        (With<OpponentScoreText>, Without<PlayerScoreText>),
    >,
    board_size: Res<BoardSize>,
) {
    if duel_state.is_changed() {
        for (mut text, mut transform) in &mut player_query {
            let score_str = format!("Player: {}", duel_state.player.score);
            text.0 = score_str.clone();
            transform.translation =
                score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Left, board_size.0);
        }
        for (mut text, mut transform) in &mut opponent_query {
            let score_str = format!("Opponent: {}", duel_state.opponent.score);
            text.0 = score_str.clone();
            let mut pos = score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Right, board_size.0);
            pos.x += 80.0;
            transform.translation = pos;
        }
    }
}

pub fn update_effect_previews(
    state: Res<GameState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>, Has<Dragging>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
    all_pieces: Query<&Piece>,
    board_size: Res<BoardSize>,
) {
    for (piece, children, is_hovered, is_dragging) in &piece_query {
        let show = is_hovered || is_dragging;
        for &child in children {
            if let Ok((mut visibility, mut sprite, preview)) = preview_query.get_mut(child) {
                if show {
                    *visibility = Visibility::Visible;
                    let mut active = false;
                    if let Some(grid_pos) = piece.placed_at {
                        let target_cell = grid_pos + preview.offset;
                        if is_in_bounds(target_cell, board_size.0) {
                            active = check_condition_with_sizes(&preview.condition, Some(target_cell), &state.board_cells, &all_pieces);
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

pub fn update_duel_effect_previews(
    duel_state: Res<DuelState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>, Has<Dragging>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
    all_pieces: Query<&Piece>,
    board_size: Res<BoardSize>,
) {
    for (piece, children, is_hovered, is_dragging) in &piece_query {
        let show = is_hovered || is_dragging;
        let board_cells = match piece.board_side {
            BoardSide::Left => &duel_state.player.board_cells,
            BoardSide::Right => &duel_state.opponent.board_cells,
            _ => continue,
        };
        for &child in children {
            if let Ok((mut visibility, mut sprite, preview)) = preview_query.get_mut(child) {
                if show {
                    *visibility = Visibility::Visible;
                    let mut active = false;
                    if let Some(grid_pos) = piece.placed_at {
                        let target_cell = grid_pos + preview.offset;
                        if is_in_bounds(target_cell, board_size.0) {
                            active = check_condition_with_sizes(&preview.condition, Some(target_cell), board_cells, &all_pieces);
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
    effect_descs: Res<EffectDescriptions>,
) {
    // Despawn existing tooltip
    if let Some(entity) = tooltip_state.entity.take() {
        commands.entity(entity).despawn();
    }

    let hovered_piece = piece_query
        .iter()
        .find(|(_, _, hovered, dragging)| *hovered && !*dragging);

    if let Some((piece, transform, _, _)) = hovered_piece {
        // Find leftmost and bottommost cell centers
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;

        for offset in &piece.shape {
            let world = Vec3::new(
                transform.translation.x + offset.x as f32 * TILE_SIZE,
                transform.translation.y + offset.y as f32 * TILE_SIZE,
                0.0,
            );
            min_x = min_x.min(world.x);
            min_y = min_y.min(world.y);
        }

        // Shift by 1.5 * TILE_SIZE left and down from the leftmost cell center
        let shift = 1.5 * TILE_SIZE;
        let anchor = Vec2::new(min_x - shift, min_y - shift);

        if let Ok((camera, cam_transform)) = camera_query.single() {
            if let Ok(window) = windows.single() {
                if let Some(ndc) = camera.world_to_ndc(cam_transform, anchor.extend(0.0)) {
                    let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                    let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                    let mut text = format!("Gain {} points.", piece.points);
                    if !piece.effects.is_empty() {
                        text.push_str("\nEffects:");
                        for effect in &piece.effects {
                            text.push_str("\n- ");
                            let desc = get_effect_description(&effect.condition, &effect_descs)
                                .replace("{points}", &effect.points.to_string());
                            text.push_str(&desc);
                        }
                    }

                    let entity = commands
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(screen_x),
                                top: Val::Px(screen_y),
                                max_width: Val::Px(300.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::WHITE),
                            GlobalZIndex(20),
                            Text::new(text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            Cleanup,
                        ))
                        .id();
                    tooltip_state.entity = Some(entity);
                }
            }
        }
    }
}

pub fn update_duel_tooltip(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    piece_query: Query<(&Piece, &Transform, Has<Hovered>, Has<Dragging>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    effect_descs: Res<EffectDescriptions>,
) {
    // Despawn existing tooltip
    if let Some(entity) = tooltip_state.entity.take() {
        commands.entity(entity).despawn();
    }

    let hovered_piece = piece_query
        .iter()
        .find(|(_, _, hovered, dragging)| *hovered && !*dragging);

    if let Some((piece, transform, _, _)) = hovered_piece {
        // Find leftmost and bottommost cell centers
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;

        for offset in &piece.shape {
            let world = Vec3::new(
                transform.translation.x + offset.x as f32 * TILE_SIZE,
                transform.translation.y + offset.y as f32 * TILE_SIZE,
                0.0,
            );
            min_x = min_x.min(world.x);
            min_y = min_y.min(world.y);
        }

        // Shift by 1.5 * TILE_SIZE left and down from the leftmost cell center
        let shift = 1.5 * TILE_SIZE;
        let anchor = Vec2::new(min_x - shift, min_y - shift);

        if let Ok((camera, cam_transform)) = camera_query.single() {
            if let Ok(window) = windows.single() {
                if let Some(ndc) = camera.world_to_ndc(cam_transform, anchor.extend(0.0)) {
                    let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                    let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                    let mut text = format!("Gain {} points.", piece.points);
                    if !piece.effects.is_empty() {
                        text.push_str("\nEffects:");
                        for effect in &piece.effects {
                            text.push_str("\n- ");
                            let desc = get_effect_description(&effect.condition, &effect_descs)
                                .replace("{points}", &effect.points.to_string());
                            text.push_str(&desc);
                        }
                    }

                    let entity = commands
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(screen_x),
                                top: Val::Px(screen_y),
                                max_width: Val::Px(300.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::WHITE),
                            GlobalZIndex(20),
                            Text::new(text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            Cleanup,
                        ))
                        .id();
                    tooltip_state.entity = Some(entity);
                }
            }
        }
    }
}

pub fn update_contributions_system(
    mut commands: Commands,
    state: Res<GameState>,
    mut piece_query: Query<(Entity, &Piece, &Transform, Option<&mut ContributionDisplay>), Without<OpponentPiece>>,
    all_pieces: Query<&Piece>,
    board_size: Res<BoardSize>,
) {
    for (piece_entity, piece, _transform, display_opt) in piece_query.iter_mut() {
        if let Some(pos) = piece.placed_at {
            let contribution = compute_piece_contribution(piece, &state.board_cells, &all_pieces, board_size.0);            let sign = if contribution >= 0 { "+" } else { "" };
            let text_str = format!("{}{}", sign, contribution);

            let first_offset = piece.shape.first().unwrap_or(&IVec2::ZERO);
            let cell_pos = pos + *first_offset;
            let world_pos = grid_to_world_for_side(cell_pos, piece.board_side, board_size.0).with_z(5.0);
            
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
            let text_entity = commands
                .spawn((
                    Text2d::new(text_str),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_translation(world_pos),
                    Cleanup,
                ))
                .id();
            commands.entity(piece_entity).insert(ContributionDisplay(text_entity));
        } else {
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
        }
    }
}

pub fn update_duel_contributions_system(
    mut commands: Commands,
    duel_state: Res<DuelState>,
    mut piece_query: Query<(Entity, &Piece, &Transform, Option<&mut ContributionDisplay>)>,
    all_pieces: Query<&Piece>,
    board_size: Res<BoardSize>,
) {
    for (piece_entity, piece, _transform, display_opt) in piece_query.iter_mut() {
        let board_cells = match piece.board_side {
            BoardSide::Left => &duel_state.player.board_cells,
            BoardSide::Right => &duel_state.opponent.board_cells,
            _ => continue,
        };
        if let Some(pos) = piece.placed_at {
            let contribution = compute_piece_contribution(piece, board_cells, &all_pieces, board_size.0);            let sign = if contribution >= 0 { "+" } else { "" };
            let text_str = format!("{}{}", sign, contribution);

            let first_offset = piece.shape.first().unwrap_or(&IVec2::ZERO);
            let cell_pos = pos + *first_offset;
            let world_pos = grid_to_world_for_side(cell_pos, piece.board_side, board_size.0).with_z(5.0);
            
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
            let text_entity = commands
                .spawn((
                    Text2d::new(text_str),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_translation(world_pos),
                    Cleanup,
                ))
                .id();
            commands.entity(piece_entity).insert(ContributionDisplay(text_entity));
        } else {
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
        }
    }
}