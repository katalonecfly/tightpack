use crate::Cleanup;
use crate::components::ContributionDisplay;
use crate::components::*;
use crate::config::EffectDescriptions;
use crate::helpers::TILE_SIZE;
use crate::helpers::{SCORE_FONT_SIZE, score_text_world_pos, score_text_world_pos_for_side};
use crate::resources::{DuelState, GameState, TooltipState};
use crate::systems::scoring::check_condition;
use crate::systems::scoring::linear_rgba_near;
use bevy::prelude::*;
use bevy::window::Window;

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
        EffectCondition::NoColorOnBoard(c) => {
            format!("NoColorOnBoard({})", color_name_from_rgba(c))
        }
    };
    descs
        .descriptions
        .get(&key)
        .cloned()
        .unwrap_or_else(|| format!("Unknown effect: {}", key))
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
    mut player_query: Query<
        (&mut Text2d, &mut Transform),
        (With<PlayerScoreText>, Without<OpponentScoreText>),
    >,
    mut opponent_query: Query<
        (&mut Text2d, &mut Transform),
        (With<OpponentScoreText>, Without<PlayerScoreText>),
    >,
) {
    if duel_state.is_changed() {
        for (mut text, mut transform) in &mut player_query {
            let score_str = format!("Player: {}", duel_state.player.score);
            text.0 = score_str.clone();
            transform.translation =
                score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Left);
        }
        for (mut text, mut transform) in &mut opponent_query {
            let score_str = format!("Opponent: {}", duel_state.opponent.score);
            text.0 = score_str.clone();
            let mut pos = score_text_world_pos_for_side(&score_str, SCORE_FONT_SIZE, BoardSide::Right);
            pos.x += 80.0;  // shift right by 80 pixels
            transform.translation = pos;
        }
    }
}

pub fn update_duel_effect_previews(
    duel_state: Res<DuelState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>, Has<Dragging>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &mut EffectPreview)>,
) {
    for (piece, children, is_hovered, is_dragging) in &piece_query {
        let show = is_hovered || is_dragging;
        if !show {
            for &child in children {
                if let Ok((mut visibility, _, _)) = preview_query.get_mut(child) {
                    *visibility = Visibility::Hidden;
                }
            }
            continue;
        }

        let board_cells = match piece.board_side {
            BoardSide::Left => &duel_state.player.board_cells,
            BoardSide::Right => &duel_state.opponent.board_cells,
            _ => continue,
        };

        let mut offset_to_condition = std::collections::HashMap::new();
        for effect in &piece.effects {
            if let Some(offsets) = &effect.offsets {
                for &offset in offsets {
                    offset_to_condition.insert(offset, effect.condition.clone());
                }
            }
        }

        for &child in children {
            if let Ok((mut visibility, mut sprite, mut preview)) = preview_query.get_mut(child) {
                *visibility = Visibility::Visible;
                if let Some(condition) = offset_to_condition.get(&preview.offset) {
                    preview.condition = condition.clone();
                } else {
                    *visibility = Visibility::Hidden;
                    continue;
                }

                let mut active = false;
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
            }
        }
    }
}

pub fn update_effect_previews(
    state: Res<GameState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>, Has<Dragging>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &mut EffectPreview)>,
) {
    for (piece, children, is_hovered, is_dragging) in &piece_query {
        let show = is_hovered || is_dragging;
        if !show {
            // Hide all previews for this piece
            for &child in children {
                if let Ok((mut visibility, _, _)) = preview_query.get_mut(child) {
                    *visibility = Visibility::Hidden;
                }
            }
            continue;
        }

        // Build a map from offset to the corresponding effect condition (from piece.effects)
        let mut offset_to_condition = std::collections::HashMap::new();
        for effect in &piece.effects {
            if let Some(offsets) = &effect.offsets {
                for &offset in offsets {
                    offset_to_condition.insert(offset, effect.condition.clone());
                }
            }
        }

        // Update each child preview
        for &child in children {
            if let Ok((mut visibility, mut sprite, mut preview)) = preview_query.get_mut(child) {
                *visibility = Visibility::Visible;

                // Sync preview with the piece's current effect data
                if let Some(condition) = offset_to_condition.get(&preview.offset) {
                    preview.condition = condition.clone();
                } else {
                    // Offset no longer exists in current effects – hide or skip
                    *visibility = Visibility::Hidden;
                    continue;
                }

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
    effect_descs: Res<EffectDescriptions>, // <-- new parameter
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
                        let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                        let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                        let mut text = format!("Gain {} points.", piece.points);
                        if !piece.effects.is_empty() {
                            text.push_str("\n\nEffects:");
                            for effect in &piece.effects {
                                text.push_str("\n- ");
                                let desc_template =
                                    get_effect_description(&effect.condition, &effect_descs);
                                let desc = desc_template
                                    .replace("{points}", &effect.points.to_string())
                                    .replace(
                                        "{color}",
                                        match &effect.condition {
                                            EffectCondition::MatchesColor(c) => {
                                                color_name_from_rgba(c)
                                            }
                                            EffectCondition::IsEmpty => "empty",
                                            EffectCondition::NoColorOnBoard(c) => {
                                                color_name_from_rgba(c)
                                            }
                                        },
                                    );
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
                                    Cleanup,
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

pub fn update_contributions_system(
    mut commands: Commands,
    state: Res<GameState>,
    mut piece_query: Query<(Entity, &Piece, &Transform, Option<&mut ContributionDisplay>), Without<OpponentPiece>>,
) {
    for (piece_entity, piece, _transform, display_opt) in piece_query.iter_mut() {
        if let Some(pos) = piece.placed_at {
            let contribution =
                crate::systems::scoring::compute_piece_contribution(piece, &state.board_cells);
            let sign = if contribution >= 0 { "+" } else { "" };
            let text_str = format!("{}{}", sign, contribution);

            // Compute centroid of the piece's cells
            let mut centroid_grid = IVec2::ZERO;
            for offset in &piece.shape {
                centroid_grid += pos + *offset;
            }
            centroid_grid.x = (centroid_grid.x as f32 / piece.shape.len() as f32).round() as i32;
            centroid_grid.y = (centroid_grid.y as f32 / piece.shape.len() as f32).round() as i32;
            let world_pos = crate::helpers::grid_to_world_for_side(centroid_grid, piece.board_side).with_z(5.0);

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
) {
    for (piece_entity, piece, _transform, display_opt) in piece_query.iter_mut() {
        let board_cells = match piece.board_side {
            BoardSide::Left => &duel_state.player.board_cells,
            BoardSide::Right => &duel_state.opponent.board_cells,
            _ => continue,
        };
        if let Some(pos) = piece.placed_at {
            let contribution =
                crate::systems::scoring::compute_piece_contribution(piece, board_cells);
            let sign = if contribution >= 0 { "+" } else { "" };
            let text_str = format!("{}{}", sign, contribution);

            // Compute centroid of the piece's cells in grid coordinates, then convert to world
            let mut centroid_grid = IVec2::ZERO;
            for offset in &piece.shape {
                centroid_grid += pos + *offset;
            }
            centroid_grid.x = (centroid_grid.x as f32 / piece.shape.len() as f32).round() as i32;
            centroid_grid.y = (centroid_grid.y as f32 / piece.shape.len() as f32).round() as i32;
            let world_pos = crate::helpers::grid_to_world_for_side(centroid_grid, piece.board_side).with_z(5.0);

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

// In systems/ui.rs

pub fn update_duel_tooltip(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    piece_query: Query<(&Piece, &Transform, Has<Hovered>, Has<Dragging>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    effect_descs: Res<EffectDescriptions>,
) {
    let hovered_piece = piece_query
        .iter()
        .find(|(_, _, hovered, dragging)| *hovered && !*dragging);

    match hovered_piece {
        Some((piece, transform, _, _)) => {
            // Compute bounding box for tooltip positioning (same as update_tooltip)
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
                        let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                        let screen_y = (1.0 - ndc.y) * 0.5 * window.height();

                        let mut text = format!("Gain {} points.", piece.points);
                        if !piece.effects.is_empty() {
                            text.push_str("\n\nEffects:");
                            for effect in &piece.effects {
                                text.push_str("\n- ");
                                let desc_template =
                                    get_effect_description(&effect.condition, &effect_descs);
                                let desc = desc_template
                                    .replace("{points}", &effect.points.to_string())
                                    .replace(
                                        "{color}",
                                        match &effect.condition {
                                            EffectCondition::MatchesColor(c) => {
                                                color_name_from_rgba(c)
                                            }
                                            EffectCondition::IsEmpty => "empty",
                                            EffectCondition::NoColorOnBoard(c) => {
                                                color_name_from_rgba(c)
                                            }
                                        },
                                    );
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
                                    Cleanup,
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
