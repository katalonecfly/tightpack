use crate::Cleanup;
use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::{AIType, BoardSize, DuelMode, DuelState, DuelTurn, GameSettings, PieceLibrary, RoundCounter};
use crate::systems::ai::{first_free_placement, greedy_block_cell, greedy_placement};
use crate::systems::draft::DraftConfirmButton;
use bevy::picking::prelude::*;
use bevy::prelude::*;
use rand::RngExt;
use std::collections::{HashMap, HashSet};

#[derive(Component)]
struct DragOffset(Vec2);

#[derive(Component)]
pub struct RoundTextDuel;

const DUEL_GAP_LARGE: f32 = 160.0;

fn duel_board_anchor(side: BoardSide, board_size: IVec2) -> Vec3 {
    let board_width = board_size.x as f32 * TILE_SIZE;
    let bottom_y = BOARD_TOP_Y - (board_size.y - 1) as f32 * TILE_SIZE;
    let x = match side {
        BoardSide::Left => -board_width - DUEL_GAP_LARGE / 2.0,
        BoardSide::Right => DUEL_GAP_LARGE / 2.0,
        _ => 0.0,
    };
    Vec3::new(x, bottom_y, 0.0)
}

fn grid_to_world_duel(grid: IVec2, side: BoardSide, board_size: IVec2) -> Vec3 {
    duel_board_anchor(side, board_size) + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

fn world_to_grid_duel(world: Vec3, side: BoardSide, board_size: IVec2) -> IVec2 {
    let local = world - duel_board_anchor(side, board_size);
    IVec2::new(
        (local.x / TILE_SIZE).round() as i32,
        (local.y / TILE_SIZE).round() as i32,
    )
}

fn score_text_pos_duel(side: BoardSide, board_size: IVec2) -> Vec3 {
    let board_anchor = duel_board_anchor(side, board_size);
    let board_left = board_anchor.x - TILE_SIZE / 2.0;
    let score_y = BOARD_TOP_Y + TILE_SIZE / 2.0 + SCORE_Y_OFFSET;
    Vec3::new(board_left, score_y, 0.0)
}

pub fn generate_duel_stash(commands: &mut Commands, library: &PieceLibrary, round_counter: &RoundCounter, board_size: IVec2) {
    if round_counter.is_game_over() {
        return;
    }
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

    let mut piece_data = Vec::new();
    for raw in &chosen {
        let (color, effects) = crate::systems::setup::randomize_piece_properties(raw, &color_map);
        piece_data.push((*raw, color, effects));
    }

    spawn_side_pieces(commands, &piece_data, BoardSide::Left, true, board_size);
    spawn_side_pieces(commands, &piece_data, BoardSide::Right, false, board_size);
}

fn spawn_side_pieces(
    commands: &mut Commands,
    piece_data: &[(&RawPieceConfig, LinearRgba, Vec<GameEffect>)],
    side: BoardSide,
    interactive: bool,
    board_size: IVec2,
) {
    let board_left = grid_to_world_duel(IVec2::ZERO, side, board_size).x;
    let mut next_left = board_left;

    for (i, (raw, color, effects)) in piece_data.iter().enumerate() {
        let type_id = i;
        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let width = (max_x - min_x + 1) as f32 * TILE_SIZE;

        let piece_left = next_left;
        let parent_x = piece_left - (min_x as f32) * TILE_SIZE;
        let parent_y = stash_y_below_board(max_y, board_size);
        let pos = Vec3::new(parent_x, parent_y, 1.0);

        let entity = crate::systems::setup::spawn_draggable_piece(
            commands, type_id, raw.shape.clone(), *color, raw.points,
            effects.clone(), pos, false, false, true, side, board_size,
        );
        if interactive {
            commands.entity(entity)
                .observe(on_drag_start_duel)
                .observe(on_drag_duel)
                .observe(on_drag_end_duel)
                .observe(crate::systems::interaction::on_right_click_unplace);
            commands.entity(entity).insert(PlayerPiece);
        } else {
            commands.entity(entity).insert(OpponentPiece);
        }

        commands.entity(entity).insert(DraftPiece);

        if interactive {
            commands.entity(entity).insert(Pickable::default());
            commands.entity(entity)
                .observe(on_drag_start_duel)
                .observe(on_drag_duel)
                .observe(on_drag_end_duel)
                .observe(on_hover_in_duel)
                .observe(on_hover_out_duel);
            commands.entity(entity).insert(PlayerPiece);
        } else {
            commands.entity(entity).insert(OpponentPiece);
        }

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

fn get_piece_entity(target: Entity, piece_query: &Query<(), With<Piece>>, child_of: &Query<&ChildOf>) -> Option<Entity> {
    if piece_query.contains(target) { Some(target) } else { child_of.get(target).ok().map(|c| c.parent()) }
}

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
        Query<(Entity, &mut Piece, &mut Transform), (With<PlayerPiece>, With<DraftPiece>, Without<LockedPiece>)>,
    )>,
    _board_size: Res<BoardSize>,
) {
    if duel_state.turn != DuelTurn::Place { return; }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return; };
    if locked_query.contains(piece_entity) { return; }

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

    if let Ok((mut transform, mut piece, _)) = param_set.p0().get_mut(piece_entity) {
        commands.entity(piece_entity).insert(Dragging);
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                duel_state.player.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
        }
        let Ok(window) = windows.single() else { return; };
        let Ok((camera, cam_transform)) = cameras.single() else { return; };
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
    board_size: Res<BoardSize>,
) {
    if duel_state.turn != DuelTurn::Place { return; }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return; };
    if locked_query.contains(piece_entity) { return; }

    if let Ok((mut transform, piece, drag_offset_opt)) = drag_piece_query.get_mut(piece_entity) {
        if let Some(drag_offset) = drag_offset_opt {
            let Ok(window) = windows.single() else { return; };
            let Ok((camera, cam_transform)) = cameras.single() else { return; };
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(cam_transform, cursor_pos) {
                    let new_pos = world_pos.origin.truncate() - drag_offset.0;
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                }
            }
        } else {
            transform.translation.x += on.delta.x;
            transform.translation.y -= on.delta.y;
        }

        for entity in &ghost_query { let _ = commands.entity(entity).try_despawn(); }
        let grid_pos = world_to_grid_duel(transform.translation, piece.board_side, board_size.0);
        let mut can_place = true;
        for offset in &piece.shape {
            let tile = grid_pos + *offset;
            if !is_cell_available(tile, &duel_state.player.board_cells, &duel_state.player.disabled_cells, board_size.0) {
                can_place = false; break;
            }
        }
        if can_place {
            let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
            for offset in &piece.shape {
                commands.spawn((
                    Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_duel(grid_pos + *offset, piece.board_side, board_size.0).with_z(1.0)),
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
    board_size: Res<BoardSize>,
) {
    for entity in &ghost_query { let _ = commands.entity(entity).try_despawn(); }
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return; };
    if locked_query.contains(piece_entity) { return; }

    commands.entity(piece_entity).remove::<Dragging>();
    commands.entity(piece_entity).remove::<DragOffset>();

    if let Ok((mut transform, mut piece, _children)) = drag_piece_query.get_mut(piece_entity) {
        if piece.placed_at.is_some() { return; }
        let grid_pos = world_to_grid_duel(transform.translation, piece.board_side, board_size.0);
        let mut can_place = true;
        for offset in &piece.shape {
            let cell = grid_pos + *offset;
            if !is_cell_available(cell, &duel_state.player.board_cells, &duel_state.player.disabled_cells, board_size.0) {
                can_place = false; break;
            }
        }
        if can_place {
            transform.translation = grid_to_world_duel(grid_pos, piece.board_side, board_size.0).with_z(1.0);
            piece.placed_at = Some(grid_pos);
            for offset in &piece.shape {
                duel_state.player.board_cells.insert(grid_pos + *offset, piece.color);
            }
            if draft_check.contains(piece_entity) {
                for other in &piece_entities {
                    if other != piece_entity && draft_check.contains(other) && drag_piece_query.get(other).map_or(false, |(_, p, _)| p.placed_at.is_some()) {
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

fn on_hover_in_duel(on: On<Pointer<Over>>, mut commands: Commands) { commands.entity(on.event_target()).insert(Hovered); }
fn on_hover_out_duel(on: On<Pointer<Out>>, mut commands: Commands) { commands.entity(on.event_target()).remove::<Hovered>(); }

pub fn handle_destroy_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut duel_state: ResMut<DuelState>,
    mut commands: Commands,
    board_size: Res<BoardSize>,
) {
    if duel_state.turn != DuelTurn::Destroy { return; }
    if !buttons.just_pressed(MouseButton::Left) { return; }
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, cam_transform)) = camera_q.single() else { return; };
    if let Some(cursor) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) {
            let world_pos = ray.origin;
            let grid = world_to_grid_duel(world_pos, BoardSide::Right, board_size.0);
            if is_in_bounds(grid, board_size.0) && !duel_state.opponent.board_cells.contains_key(&grid) && !duel_state.opponent.disabled_cells.contains(&grid) {
                if let Some((old1, old2)) = duel_state.pending_disable_preview.take() {
                    commands.entity(old1).despawn();
                    commands.entity(old2).despawn();
                }
                duel_state.pending_disable = Some(grid);
                let (line1, line2) = spawn_disabled_visual(&mut commands, grid, BoardSide::Right, board_size.0, TILE_SIZE * 0.6, 2.0);
                duel_state.pending_disable_preview = Some((line1, line2));
            }
        }
    }
}

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
    settings: Res<GameSettings>,
    mut round_counter: ResMut<RoundCounter>,
    board_size: Res<BoardSize>,
) {
    if round_counter.is_game_over() {
        return;
    }
    match duel_state.turn {
        DuelTurn::Place => {
            for entity in &player_drafts {
                if let Ok(piece) = player_pieces.get(entity) {
                    if piece.placed_at.is_some() {
                        commands.entity(entity).remove::<DraftPiece>().insert(LockedPiece);
                    } else {
                        commands.entity(entity).despawn();
                    }
                } else {
                    commands.entity(entity).despawn();
                }
            }
            for label in &player_labels { commands.entity(label).despawn(); }

            let draft_data: Vec<(Entity, Piece)> = opponent_drafts.iter().filter_map(|e| opponent_pieces.get(e).ok().map(|p| (e, p.clone()))).collect();
            let draft_refs: Vec<(Entity, &Piece)> = draft_data.iter().map(|(e, p)| (*e, p)).collect();
            let opponent_placed: Vec<&Piece> = opponent_pieces.iter().collect();
            
            let placement = if settings.ai_mode == AIType::Greedy {
                greedy_placement(&draft_refs, &duel_state.opponent, &opponent_placed, board_size.0)
            } else {
                first_free_placement(&draft_refs, &duel_state.opponent, board_size.0)
            };            
            if let Some(placement) = placement {
                let world_pos = grid_to_world_duel(placement.origin, BoardSide::Right, board_size.0);
                let entity = crate::systems::setup::spawn_draggable_piece(
                    &mut commands, 0, placement.shape.clone(), placement.color,
                    placement.raw_config.points, placement.effects.clone(),
                    world_pos.with_z(1.0), false, false, true, BoardSide::Right, board_size.0,
                );
                commands.entity(entity).insert(LockedPiece).insert(OpponentPiece);
                let placed = Piece {
                    type_id: 0, shape: placement.shape.clone(), original_shape: placement.shape.clone(),
                    color: placement.color, points: placement.raw_config.points,
                    effects: placement.effects.clone(), original_effects: placement.effects.clone(),
                    original_pos: world_pos.with_z(1.0), placed_at: Some(placement.origin), board_side: BoardSide::Right,
                };
                commands.entity(entity).insert(placed);
                for off in &placement.shape {
                    duel_state.opponent.board_cells.insert(placement.origin + *off, placement.color);
                }
            }
            for entity in &opponent_drafts { commands.entity(entity).despawn(); }
            for label in &opponent_labels { commands.entity(label).despawn(); }

            duel_state.player.score = crate::systems::scoring::recalculate_score_with_size(&duel_state.player.board_cells, &player_pieces, board_size.0);
            duel_state.opponent.score = crate::systems::scoring::recalculate_score_with_size(&duel_state.opponent.board_cells, &opponent_pieces, board_size.0);
            if duel_state.mode == DuelMode::Destroy {
                duel_state.turn = DuelTurn::Destroy;
                duel_state.pending_disable = None;
            } else {
                round_counter.advance();
                if !round_counter.is_game_over() {
                    generate_duel_stash(&mut commands, &library, &round_counter, board_size.0);
                }
            }
        }
        DuelTurn::Destroy => {
            if let Some(cell) = duel_state.pending_disable.take() {
                duel_state.opponent.disabled_cells.insert(cell);
                if let Some((preview1, preview2)) = duel_state.pending_disable_preview.take() {
                    commands.entity(preview1).despawn();
                    commands.entity(preview2).despawn();
                }
                spawn_disabled_visual(&mut commands, cell, BoardSide::Right, board_size.0, TILE_SIZE * 0.8, 4.0);
            }

            let ai_cell = if settings.ai_mode == AIType::Greedy {
                greedy_block_cell(&duel_state.player, &player_pieces, board_size.0)
            } else {
                pick_first_free(&duel_state.player.board_cells, &duel_state.player.disabled_cells, board_size.0)
            };
            if let Some(cell) = ai_cell {
                duel_state.player.disabled_cells.insert(cell);
                spawn_disabled_visual(&mut commands, cell, BoardSide::Left, board_size.0, TILE_SIZE * 0.8, 4.0);
            }

            duel_state.turn = DuelTurn::Place;
            duel_state.pending_disable_preview = None;
            round_counter.advance();
            if !round_counter.is_game_over() {
                generate_duel_stash(&mut commands, &library, &round_counter, board_size.0);
            }
        }
    }
}

pub fn update_duel_round_display(
    round_counter: Res<RoundCounter>,
    mut commands: Commands,
    button_query: Query<Entity, With<DraftConfirmButton>>,
    existing_text: Query<Entity, With<RoundTextDuel>>,
    transforms: Query<&Transform>,
    mut button_sprite: Query<&mut Sprite, With<DraftConfirmButton>>,
) {
    for entity in existing_text.iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(button_entity) = button_query.single() {
        let is_game_over = round_counter.is_game_over();
        if let Ok(mut sprite) = button_sprite.get_mut(button_entity) {
            sprite.color = if is_game_over {
                Color::srgb(0.5, 0.5, 0.5)
            } else {
                Color::srgb(0.3, 0.8, 0.3)
            };
        }
        let displayed_current = round_counter.current.min(round_counter.total);
        if let Ok(button_transform) = transforms.get(button_entity) {
            let text_pos = button_transform.translation + Vec3::new(CONFIRM_BUTTON_WIDTH / 2.0 + 80.0, 0.0, 0.0);
            let text_content = format!("Round:\n{}/{}", displayed_current, round_counter.total);
            commands.spawn((
                Text2d::new(text_content),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_translation(text_pos),
                RoundTextDuel,
                Cleanup,
            ));
        }
    }
}

fn pick_first_free(board_cells: &HashMap<IVec2, LinearRgba>, disabled: &HashSet<IVec2>, board_size: IVec2) -> Option<IVec2> {
    for y in 0..board_size.y {
        for x in 0..board_size.x {
            let grid = IVec2::new(x, y);
            if !board_cells.contains_key(&grid) && !disabled.contains(&grid) {
                return Some(grid);
            }
        }
    }
    None
}

fn spawn_disabled_visual(commands: &mut Commands, grid: IVec2, side: BoardSide, board_size: IVec2, size: f32, thickness: f32) -> (Entity, Entity) {
    let center = grid_to_world_duel(grid, side, board_size).with_z(3.0);
    let color = Color::BLACK;
    let angle1 = -std::f32::consts::FRAC_PI_4;
    let angle2 = std::f32::consts::FRAC_PI_4;
    let line_sprite = Sprite::from_color(color, Vec2::new(size, thickness));
    let e1 = commands.spawn((Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle1)), line_sprite.clone(), Cleanup)).id();
    let e2 = commands.spawn((Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle2)), line_sprite, Cleanup)).id();
    (e1, e2)
}

pub fn setup_duel(mut commands: Commands, settings: Res<GameSettings>, board_size: Res<BoardSize>) {
    commands.spawn((Camera2d, Cleanup));
    let file_content = include_str!("../../assets/pieces.ron");
    let lib: crate::config::RawPieceLibrary = ron::from_str(&file_content).expect("Failed to parse RON");
    let pieces = lib.pieces.clone();
    commands.insert_resource(PieceLibrary(pieces.clone()));

    let board_size_val = board_size.0;

    spawn_board(&mut commands, BoardSide::Left, board_size_val);
    spawn_board(&mut commands, BoardSide::Right, board_size_val);

    commands.spawn((
        Text2d::new("Player: 0"),
        TextFont { font_size: SCORE_FONT_SIZE, ..default() },
        Transform::from_translation(score_text_pos_duel(BoardSide::Left, board_size_val)),
        ScoreText, PlayerScoreText, Cleanup,
    ));
    let mut opponent_transform = Transform::from_translation(score_text_pos_duel(BoardSide::Right, board_size_val));
    opponent_transform.translation.x += 580.0;
    commands.spawn((
        Text2d::new("Opponent: 0"),
        TextFont { font_size: SCORE_FONT_SIZE, ..default() },
        opponent_transform,
        ScoreText,
        OpponentScoreText,
        Cleanup,
    ));

    spawn_confirm_button(&mut commands, BoardSide::Left, board_size_val);

    let duel_mode = if settings.duel_blocking_enabled { DuelMode::Destroy } else { DuelMode::Basic };
    commands.insert_resource(DuelState { mode: duel_mode, ..default() });

    let round_counter = RoundCounter::new(settings.rounds);
    generate_duel_stash(&mut commands, &PieceLibrary(pieces), &round_counter, board_size_val);
    commands.insert_resource(round_counter);
}

fn spawn_board(commands: &mut Commands, side: BoardSide, board_size: IVec2) {
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..board_size.x {
        for y in 0..board_size.y {
            let tile = commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world_duel(IVec2::new(x, y), side, board_size)),
            )).id();
            commands.entity(board_root).add_child(tile);
        }
    }
}

fn spawn_confirm_button(commands: &mut Commands, side: BoardSide, board_size: IVec2) {
    let board_right = duel_board_anchor(side, board_size).x + (board_size.x as f32 - 0.5) * TILE_SIZE;
    let board_top = BOARD_TOP_Y + TILE_SIZE / 2.0;
    let score_y = board_top + SCORE_Y_OFFSET;
    let button_pos = Vec3::new(board_right - CONFIRM_BUTTON_WIDTH / 2.0, score_y, 0.0);
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.3, 0.8, 0.3), Vec2::new(CONFIRM_BUTTON_WIDTH, CONFIRM_BUTTON_HEIGHT)),
            Transform::from_translation(button_pos),
            Pickable::default(),
            DraftConfirmButton,
            Cleanup,
        ))
        .with_child((
            Text2d::new("Confirm"),
            TextFont { font_size: CONFIRM_BUTTON_FONT_SIZE, ..default() },
            TextColor(Color::WHITE),
            Transform::default(),
        ))
        .observe(on_confirm_click_duel);
}