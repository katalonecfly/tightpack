use crate::Cleanup;
use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::{DuelMode, DuelState, DuelTurn, PieceLibrary};
use crate::systems::draft::DraftConfirmButton;
use bevy::picking::prelude::*;
use bevy::prelude::*;
use rand::RngExt;
use std::collections::{HashMap, HashSet};
use crate::resources::GameSettings;

#[derive(Component)]
struct DragOffset(Vec2);

// ── Stash generation ──────────────────────────────

pub fn generate_duel_stash(commands: &mut Commands, library: &PieceLibrary) {
    let color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
    ]
    .into();

    let all_pieces = &library.0;
    let mut rng = rand::rng();
    let mut available: Vec<&RawPieceConfig> = all_pieces.iter().collect();
    let mut chosen = Vec::with_capacity(3);
    for _ in 0..3 {
        if available.is_empty() {
            break;
        }
        let idx = rng.random_range(0..available.len());
        chosen.push(available.remove(idx));
    }

    // Pre‑randomise properties for each chosen piece (once per piece, shared between sides)
    let mut piece_data = Vec::new();
    for raw in &chosen {
        let (color, effects) = crate::systems::setup::randomize_piece_properties(raw, &color_map);
        piece_data.push((*raw, color, effects));
    }

    // Spawn for both sides using the same data
    spawn_side_pieces(commands, &piece_data, BoardSide::Left, true);
    spawn_side_pieces(commands, &piece_data, BoardSide::Right, false);
}

fn spawn_side_pieces(
    commands: &mut Commands,
    piece_data: &[(&RawPieceConfig, LinearRgba, Vec<GameEffect>)],
    side: BoardSide,
    interactive: bool,
) {
    let board_left = grid_to_world_for_side(IVec2::ZERO, side).x;
    let mut next_left = board_left;

    for (i, (raw, color, effects)) in piece_data.iter().enumerate() {
        let type_id = i;
        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let width = (max_x - min_x + 1) as f32 * TILE_SIZE;

        let piece_left = next_left;
        let parent_x = piece_left - (min_x as f32) * TILE_SIZE;
        let parent_y = stash_y_below_board(max_y);
        let pos = Vec3::new(parent_x, parent_y, 1.0);

        // Inside spawn_side_pieces
        // In systems/duel.rs, inside spawn_side_pieces

        let entity = crate::systems::setup::spawn_draggable_piece(
            commands,
            type_id,
            raw.shape.clone(),
            *color,
            raw.points,
            effects.clone(),
            pos,
            false, // draft_mode
            false, // interactive
            true,  // hoverable   ← this is the missing argument
            side,
        );
        if interactive {
            // Player side: add duel‑specific drag observers
            commands
                .entity(entity)
                .observe(on_drag_start_duel)
                .observe(on_drag_duel)
                .observe(on_drag_end_duel);
            commands.entity(entity).insert(PlayerPiece);
        } else {
            // Opponent side: no drag, only hover (already added by spawn_draggable_piece)
            commands.entity(entity).insert(OpponentPiece);
        }

        commands.entity(entity).insert(DraftPiece);

        if interactive {
            commands.entity(entity).insert(Pickable::default());
            commands
                .entity(entity)
                .observe(on_drag_start_duel)
                .observe(on_drag_duel)
                .observe(on_drag_end_duel)
                .observe(on_hover_in_duel)
                .observe(on_hover_out_duel);
            commands.entity(entity).insert(PlayerPiece);
        } else {
            commands.entity(entity).insert(OpponentPiece);
        }

        // Label
        let label_y = parent_y + max_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        let label_entity = commands
            .spawn((
                Text2d::new("x1"),
                TextFont {
                    font_size: STASH_LABEL_FONT_SIZE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(parent_x, label_y, 2.0)),
                StashLabel(type_id),
                Cleanup,
            ))
            .id();
        match side {
            BoardSide::Left => {
                commands.entity(label_entity).insert(PlayerPiece);
            }
            BoardSide::Right => {
                commands.entity(label_entity).insert(OpponentPiece);
            }
            _ => {}
        }

        next_left = piece_left + width + TILE_SIZE;
    }
}

// ── Duel-specific drag observers (using DuelState) ──

fn on_drag_start_duel(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    locked_query: Query<(), With<LockedPiece>>,
    mut duel_state: ResMut<DuelState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Piece, &Children), (Without<LockedPiece>, With<PlayerPiece>)>,
        Query<
            (Entity, &mut Piece, &mut Transform),
            (With<PlayerPiece>, With<DraftPiece>, Without<LockedPiece>),
        >,
    )>,
) {
    if duel_state.turn != DuelTurn::Place {
        return;
    }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if locked_query.contains(piece_entity) {
        return;
    }

    // Unplace any other player draft piece that is currently on board
    for (other_entity, mut other_piece, mut other_transform) in param_set.p1().iter_mut() {
        if other_entity != piece_entity && other_piece.placed_at.is_some() {
            if let Some(old_pos) = other_piece.placed_at {
                for offset in &other_piece.shape {
                    duel_state.player.board_cells.remove(&(old_pos + *offset));
                }
                other_piece.placed_at = None;
            }
            other_transform.translation = other_piece.original_pos;
            other_transform.translation.z = other_piece.original_pos.z;
            other_transform.rotation = Quat::IDENTITY;
            other_piece.shape = other_piece.original_shape.clone();
            other_piece.effects = other_piece.original_effects.clone();
        }
    }

    // Handle the dragged piece
    if let Ok((mut transform, mut piece, _)) = param_set.p0().get_mut(piece_entity) {
        commands.entity(piece_entity).insert(Dragging);
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                duel_state.player.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
        }

        // Compute drag offset: cursor world position minus piece position
        let Ok(window) = windows.single() else {
            return;
        };
        let Ok((camera, cam_transform)) = cameras.single() else {
            return;
        };
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world(cam_transform, cursor_pos) {
                let offset = world_pos.origin.truncate() - transform.translation.truncate();
                commands.entity(piece_entity).insert(DragOffset(offset));
            }
        }
    }
}

fn on_drag_duel(
    on: On<Pointer<Drag>>,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &Piece, Option<&mut DragOffset>)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut commands: Commands,
    duel_state: Res<DuelState>,
    ghost_query: Query<Entity, With<GhostTile>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if duel_state.turn != DuelTurn::Place {
        return;
    }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if locked_query.contains(piece_entity) {
        return;
    }

    if let Ok((mut transform, piece, drag_offset_opt)) = drag_piece_query.get_mut(piece_entity) {
        if let Some(drag_offset) = drag_offset_opt {
            let Ok(window) = windows.single() else {
                return;
            };
            let Ok((camera, cam_transform)) = cameras.single() else {
                return;
            };
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(cam_transform, cursor_pos) {
                    let new_pos = world_pos.origin.truncate() - drag_offset.0;
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                }
            }
        } else {
            // Fallback (should not happen)
            transform.translation.x += on.delta.x;
            transform.translation.y -= on.delta.y;
        }

        // Update ghosts
        for entity in &ghost_query {
            let _ = commands.entity(entity).try_despawn();
        }
        let grid_pos = world_to_grid_for_side(transform.translation, piece.board_side);
        let mut can_place = true;
        for offset in &piece.shape {
            let tile = grid_pos + *offset;
            if !is_cell_available(
                tile,
                &duel_state.player.board_cells,
                &duel_state.player.disabled_cells,
            ) {
                can_place = false;
                break;
            }
        }
        if can_place {
            let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
            for offset in &piece.shape {
                commands.spawn((
                    Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(
                        grid_to_world_for_side(grid_pos + *offset, piece.board_side).with_z(1.0),
                    ),
                    GhostTile,
                ));
            }
        }
    }
}

fn on_drag_end_duel(
    on: On<Pointer<DragEnd>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &mut Piece, &Children)>,
    locked_query: Query<(), With<LockedPiece>>,
    draft_check: Query<(), With<DraftPiece>>,
    piece_entities: Query<Entity, With<Piece>>,
    mut duel_state: ResMut<DuelState>,
    ghost_query: Query<Entity, With<GhostTile>>,
) {
    for entity in &ghost_query {
        let _ = commands.entity(entity).try_despawn();
    }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else {
        return;
    };
    if locked_query.contains(piece_entity) {
        return;
    }

    commands.entity(piece_entity).remove::<Dragging>();
    commands.entity(piece_entity).remove::<DragOffset>(); // clean up

    if let Ok((mut transform, mut piece, _children)) = drag_piece_query.get_mut(piece_entity) {
        if piece.placed_at.is_some() {
            return;
        }

        let grid_pos = world_to_grid_for_side(transform.translation, piece.board_side);
        let mut can_place = true;
        for offset in &piece.shape {
            let cell = grid_pos + *offset;
            if !is_cell_available(
                cell,
                &duel_state.player.board_cells,
                &duel_state.player.disabled_cells,
            ) {
                can_place = false;
                break;
            }
        }
        if can_place {
            transform.translation = grid_to_world_for_side(grid_pos, piece.board_side).with_z(1.0);
            piece.placed_at = Some(grid_pos);
            // Do NOT update original_pos
            for offset in &piece.shape {
                duel_state
                    .player
                    .board_cells
                    .insert(grid_pos + *offset, piece.color);
            }
            if draft_check.contains(piece_entity) {
                for other in &piece_entities {
                    if other != piece_entity
                        && draft_check.contains(other)
                        && drag_piece_query
                            .get(other)
                            .map_or(false, |(_, p, _)| p.placed_at.is_some())
                    {
                        if let Ok((mut t, mut p, _)) = drag_piece_query.get_mut(other) {
                            if let Some(old) = p.placed_at {
                                for off in &p.shape {
                                    duel_state.player.board_cells.remove(&(old + *off));
                                }
                                p.placed_at = None;
                            }
                            t.translation = p.original_pos;
                            t.translation.z = 1.0;
                            t.rotation = Quat::IDENTITY;
                            p.shape = p.original_shape.clone();
                            p.effects = p.original_effects.clone();
                        }
                    }
                }
            }
        } else {
            transform.translation = piece.original_pos;
            transform.translation.z = piece.original_pos.z;
            transform.rotation = Quat::IDENTITY;
            piece.shape = piece.original_shape.clone();
            piece.effects = piece.original_effects.clone();
        }
    }
}

fn on_hover_in_duel(on: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(on.event_target()).insert(Hovered);
}
fn on_hover_out_duel(on: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(on.event_target()).remove::<Hovered>();
}

fn get_piece_entity(
    target: Entity,
    piece_query: &Query<(), With<Piece>>,
    child_of: &Query<&ChildOf>,
) -> Option<Entity> {
    if piece_query.contains(target) {
        Some(target)
    } else if let Ok(c) = child_of.get(target) {
        Some(c.parent())
    } else {
        None
    }
}

// ── Destroy turn input handling ──

pub fn handle_destroy_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut duel_state: ResMut<DuelState>,
    mut commands: Commands,
) {
    if duel_state.turn != DuelTurn::Destroy {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    if let Some(cursor) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) {
            let world_pos = ray.origin;
            let grid = world_to_grid_for_side(world_pos, BoardSide::Right);
            if is_in_bounds(grid)
                && !duel_state.opponent.board_cells.contains_key(&grid)
                && !duel_state.opponent.disabled_cells.contains(&grid)
            {
                // Remove old preview cross
                if let Some((old1, old2)) = duel_state.pending_disable_preview.take() {
                    commands.entity(old1).despawn();
                    commands.entity(old2).despawn();
                }
                // Set new pending disable
                duel_state.pending_disable = Some(grid);
                // Spawn small preview cross
                let (line1, line2) = spawn_disabled_visual(
                    &mut commands,
                    grid,
                    BoardSide::Right,
                    TILE_SIZE * 0.6, // small size
                    2.0,             // thin
                );
                duel_state.pending_disable_preview = Some((line1, line2));
            }
        }
    }
}

// ── Confirm button logic ──

pub fn on_confirm_click_duel(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    player_drafts: Query<Entity, (With<DraftPiece>, With<PlayerPiece>)>,
    opponent_drafts: Query<Entity, (With<DraftPiece>, With<OpponentPiece>)>,
    library: Res<PieceLibrary>,
    mut duel_state: ResMut<DuelState>,
    player_pieces: Query<&Piece, With<PlayerPiece>>,
    opponent_pieces: Query<&Piece, With<OpponentPiece>>,
    player_labels: Query<Entity, (With<StashLabel>, With<PlayerPiece>)>,
    opponent_labels: Query<Entity, (With<StashLabel>, With<OpponentPiece>)>,
) {
    match duel_state.turn {
        DuelTurn::Place => {
            // Existing placement logic
            for entity in &player_drafts {
                if let Ok(piece) = player_pieces.get(entity) {
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
            for label in &player_labels {
                commands.entity(label).despawn();
            }

            // AI placement
            let draft_data: Vec<(Entity, Piece)> = opponent_drafts
                .iter()
                .filter_map(|e| opponent_pieces.get(e).ok().map(|p| (e, p.clone())))
                .collect();
            if let Some(placement) = crate::systems::ai::first_free_placement(
                &draft_data.iter().map(|(e, p)| (*e, p)).collect::<Vec<_>>(),
                &duel_state.opponent,
            ) {
                let world_pos = grid_to_world_for_side(placement.origin, BoardSide::Right);
                let entity = crate::systems::setup::spawn_draggable_piece(
                    &mut commands,
                    0,
                    placement.shape.clone(),
                    placement.color,
                    placement.raw_config.points,
                    placement.effects.clone(),
                    world_pos.with_z(1.0),
                    false, // draft_mode
                    false, // interactive
                    true,  // hoverable   ← added
                    BoardSide::Right,
                );
                commands
                    .entity(entity)
                    .insert(LockedPiece)
                    .insert(OpponentPiece);
                let placed = Piece {
                    type_id: 0,
                    shape: placement.shape.clone(),
                    original_shape: placement.shape.clone(),
                    color: placement.color,
                    points: placement.raw_config.points,
                    effects: placement.effects.clone(),
                    original_effects: placement.effects.clone(),
                    original_pos: world_pos.with_z(1.0),
                    placed_at: Some(placement.origin),
                    board_side: BoardSide::Right,
                };
                commands.entity(entity).insert(placed);
                for off in &placement.shape {
                    duel_state
                        .opponent
                        .board_cells
                        .insert(placement.origin + *off, placement.color);
                }
            }
            for entity in &opponent_drafts {
                commands.entity(entity).despawn();
            }
            for label in &opponent_labels {
                commands.entity(label).despawn();
            }

            // Update scores
            duel_state.player.score = crate::systems::scoring::recalculate_score(
                &duel_state.player.board_cells,
                &player_pieces,
            );
            duel_state.opponent.score = crate::systems::scoring::recalculate_score(
                &duel_state.opponent.board_cells,
                &opponent_pieces,
            );

            if duel_state.mode == DuelMode::Destroy {
                // Switch to destroy turn, do not generate new stash yet
                duel_state.turn = DuelTurn::Destroy;
                duel_state.pending_disable = None;
            } else {
                generate_duel_stash(&mut commands, &library);
            }
        }
        DuelTurn::Destroy => {
            // Apply player's disable if any
            if let Some(cell) = duel_state.pending_disable.take() {
                duel_state.opponent.disabled_cells.insert(cell);
                // Remove preview cross
                if let Some((preview1, preview2)) = duel_state.pending_disable_preview.take() {
                    commands.entity(preview1).despawn();
                    commands.entity(preview2).despawn();
                }
                // Spawn final large cross
                spawn_disabled_visual(&mut commands, cell, BoardSide::Right, TILE_SIZE * 0.8, 4.0);
            }
            // AI disables one cell on player's board
            let ai_cell = pick_first_free(
                &duel_state.player.board_cells,
                &duel_state.player.disabled_cells,
            );
            if let Some(cell) = ai_cell {
                duel_state.player.disabled_cells.insert(cell);
                spawn_disabled_visual(&mut commands, cell, BoardSide::Left, TILE_SIZE * 0.8, 4.0);
            }
            // Switch back to place and generate new stash
            duel_state.turn = DuelTurn::Place;
            duel_state.pending_disable_preview = None;
            generate_duel_stash(&mut commands, &library);
        }
    }
}

fn pick_first_free(
    board_cells: &HashMap<IVec2, LinearRgba>,
    disabled: &HashSet<IVec2>,
) -> Option<IVec2> {
    for y in 0..BOARD_SIZE.y {
        for x in 0..BOARD_SIZE.x {
            let grid = IVec2::new(x, y);
            if !board_cells.contains_key(&grid) && !disabled.contains(&grid) {
                return Some(grid);
            }
        }
    }
    None
}

fn spawn_disabled_visual(
    commands: &mut Commands,
    grid: IVec2,
    side: BoardSide,
    size: f32,
    thickness: f32,
) -> (Entity, Entity) {
    let center = grid_to_world_for_side(grid, side).with_z(3.0);
    let color = Color::BLACK;
    let angle1 = -std::f32::consts::FRAC_PI_4;
    let angle2 = std::f32::consts::FRAC_PI_4;
    let line_sprite = Sprite::from_color(color, Vec2::new(size, thickness));
    let e1 = commands
        .spawn((
            Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle1)),
            line_sprite.clone(),
            Cleanup,
        ))
        .id();
    let e2 = commands
        .spawn((
            Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle2)),
            line_sprite,
            Cleanup,
        ))
        .id();
    (e1, e2)
}

// ── Setup ──

pub fn setup_duel(mut commands: Commands, settings: Res<GameSettings>) {
    commands.spawn((Camera2d, Cleanup));

    let file_content = std::fs::read_to_string("assets/pieces.ron").expect("Missing pieces.ron");
    let lib: crate::config::RawPieceLibrary =
        ron::from_str(&file_content).expect("Failed to parse RON");
    let pieces = lib.pieces.clone();
    commands.insert_resource(PieceLibrary(pieces.clone()));

    spawn_board(&mut commands, BoardSide::Left);
    spawn_board(&mut commands, BoardSide::Right);

    commands.spawn((
        Text2d::new("Player: 0"),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos_for_side(
            "Player: 0",
            SCORE_FONT_SIZE,
            BoardSide::Left,
        )),
        ScoreText,
        PlayerScoreText,
        Cleanup,
    ));
    commands.spawn((
        Text2d::new("Opponent: 0"),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos_for_side(
            "Opponent: 0",
            SCORE_FONT_SIZE,
            BoardSide::Right,
        )),
        ScoreText,
        OpponentScoreText,
        Cleanup,
    ));

    spawn_confirm_button(&mut commands, BoardSide::Left);

    let duel_mode = if settings.duel_blocking_enabled {
        DuelMode::Destroy
    } else {
        DuelMode::Basic
    };
    commands.insert_resource(DuelState {
        mode: duel_mode,
        ..default()
    });
    generate_duel_stash(&mut commands, &PieceLibrary(pieces));

}

fn spawn_board(commands: &mut Commands, side: BoardSide) {
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_for_side(IVec2::new(x, y), side)),
                ))
                .id();
            commands.entity(board_root).add_child(tile);
        }
    }
}

fn spawn_confirm_button(commands: &mut Commands, side: BoardSide) {
    let board_right = board_right_edge(side);
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
        .observe(on_confirm_click_duel);
}
