use crate::Cleanup;
use crate::components::*;
use crate::config::{RawPieceConfig, RawGameEffect, PieceType, EffectDescriptions};
use crate::helpers::*;
use crate::resources::{InventoryScroll, StashContentHeight, StashScreenRect, TooltipState};
use crate::systems::scoring::{check_condition, linear_rgba_near, compute_piece_contribution};
use crate::systems::setup::{randomize_piece_properties};
use crate::AppState;
use bevy::picking::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use bevy::prelude::Color;

const PUZZLE_STASH_GAP: f32 = 60.0;

// -----------------------------------------------------------------------------
// Puzzle data structures
// -----------------------------------------------------------------------------

#[derive(Deserialize, Clone)]
pub struct PuzzleData {
    pub board_size: IVec2,
    pub blocked_cells: Vec<IVec2>,
    pub pieces: Vec<PuzzlePieceData>,
}

#[derive(Deserialize, Clone)]
pub struct PuzzlePieceData {
    pub shape: Vec<IVec2>,
    pub color: String,
    pub points: i32,
    pub count: u32,
    #[serde(default)]
    pub effects: Vec<RawGameEffect>,
}

// Solution structures
#[derive(Deserialize, Clone)]
pub struct Solution {
    pub score: i32,
    pub placements: Vec<SolutionPlacement>,
}

#[derive(Deserialize, Clone)]
pub struct SolutionPlacement {
    pub piece: usize,  // index in PuzzleData.pieces
    pub pos: IVec2,
    pub rot: u32,      // 0,1,2,3 (0=0°, 1=90°, 2=180°, 3=270° clockwise)
}

// -----------------------------------------------------------------------------
// Resources
// -----------------------------------------------------------------------------

#[derive(Resource, Clone, Copy)]
pub struct PuzzleBoardInfo {
    pub size: IVec2,
    pub anchor: Vec3,
    pub tile_size: f32,
}

#[derive(Resource, Default)]
pub struct PuzzleGameState {
    pub board_cells: HashMap<IVec2, LinearRgba>,
    pub disabled_cells: HashSet<IVec2>,
    pub score: i32,
}

#[derive(Resource)]
pub struct CurrentPuzzle {
    pub id: String,
    pub data: PuzzleData,
}

#[derive(Resource)]
pub struct SelectedSolution {
    pub puzzle_id: String,
    pub solution: Solution,
    pub puzzle_data: PuzzleData,
}

// Helper: get list of puzzle folders
pub fn get_puzzle_list() -> Vec<String> {
    let puzzles_dir = "assets/puzzles";
    if let Ok(entries) = fs::read_dir(puzzles_dir) {
        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_dir() {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

// -----------------------------------------------------------------------------
// Puzzle list UI (with right-click to view solutions)
// -----------------------------------------------------------------------------

#[derive(Component)]
pub struct PuzzleButton {
    pub puzzle_id: String,
}

pub fn setup_puzzle_list(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let puzzles = get_puzzle_list();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Select a Puzzle"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            for id in puzzles {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        PuzzleButton { puzzle_id: id.clone() },
                        Pickable::default(),
                    ))
                    .with_child((
                        Text::new(&id),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ))
                    .observe(on_puzzle_left_click)
                    .observe(on_puzzle_right_click);
            }
        });
}

fn on_puzzle_left_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    button_query: Query<&PuzzleButton>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if trigger.event.button == PointerButton::Primary {
        if let Ok(button) = button_query.get(trigger.event_target()) {
            let puzzle_path = format!("assets/puzzles/{}/data.ron", button.puzzle_id);
            if let Ok(content) = fs::read_to_string(&puzzle_path) {
                if let Ok(data) = ron::from_str::<PuzzleData>(&content) {
                    commands.insert_resource(CurrentPuzzle {
                        id: button.puzzle_id.clone(),
                        data,
                    });
                    next_state.set(AppState::Puzzle);
                }
            }
        }
    }
}

fn on_puzzle_right_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    button_query: Query<&PuzzleButton>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if trigger.event.button == PointerButton::Secondary {
        if let Ok(button) = button_query.get(trigger.event_target()) {
            let puzzle_path = format!("assets/puzzles/{}/data.ron", button.puzzle_id);
            if let Ok(content) = fs::read_to_string(&puzzle_path) {
                if let Ok(data) = ron::from_str::<PuzzleData>(&content) {
                    commands.insert_resource(CurrentPuzzle {
                        id: button.puzzle_id.clone(),
                        data,
                    });
                    next_state.set(AppState::SolutionList);
                }
            }
        }
    }
}


// -----------------------------------------------------------------------------
// Solution list screen
// -----------------------------------------------------------------------------

#[derive(Component)]
pub struct SolutionButton {
    pub solution_name: String,
    pub score: i32,
}

fn validate_solution(solution: &Solution, puzzle_data: &PuzzleData) -> bool {
    let board_size = puzzle_data.board_size;
    let blocked_cells: HashSet<IVec2> = puzzle_data.blocked_cells.iter().copied().collect();
    let pieces = &puzzle_data.pieces;
    
    // Track used counts per piece type
    let mut used_counts = vec![0; pieces.len()];
    // Track occupied cells
    let mut occupied: HashSet<IVec2> = HashSet::new();
    // Compute actual score
    let mut actual_score = 0;
    
    for placement in &solution.placements {
        // Check piece index valid
        if placement.piece >= pieces.len() {
            return false;
        }
        let piece_def = &pieces[placement.piece];
        
        // Check count not exceeded
        used_counts[placement.piece] += 1;
        if used_counts[placement.piece] > piece_def.count {
            return false;
        }
        
        // Get shape and apply rotation
        let mut shape = piece_def.shape.clone();
        for _ in 0..placement.rot % 4 {
            shape = shape.iter().map(|&v| IVec2::new(v.y, -v.x)).collect();
        }
        
        // Check each cell of the piece
        for offset in &shape {
            let cell = placement.pos + *offset;
            // Check bounds
            if cell.x < 0 || cell.x >= board_size.x || cell.y < 0 || cell.y >= board_size.y {
                return false;
            }
            // Check blocked cell
            if blocked_cells.contains(&cell) {
                return false;
            }
            // Check overlap with other pieces
            if occupied.contains(&cell) {
                return false;
            }
            occupied.insert(cell);
        }
        
        // Add piece points to actual score
        actual_score += piece_def.points;
    }
    
    // Verify score matches
    actual_score == solution.score
}

pub fn setup_solution_list(mut commands: Commands, puzzle: Res<CurrentPuzzle>) {
    commands.spawn((Camera2d, Cleanup));

    let solutions_dir = format!("assets/puzzles/{}/solutions", puzzle.id);
    let mut valid_solutions = Vec::new();

    if let Ok(entries) = fs::read_dir(&solutions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(solution) = ron::from_str::<Solution>(&content) {
                        if validate_solution(&solution, &puzzle.data) {
                            let name = path.file_stem().unwrap().to_string_lossy().to_string();
                            valid_solutions.push((name, solution));
                        } else {
                            warn!("Invalid solution: {}", path.display());
                        }
                    }
                }
            }
        }
    }

    if valid_solutions.is_empty() {
        commands.spawn((
            Text::new("No valid solutions found for this puzzle."),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Cleanup,
        ));
        return;
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Solutions for {}", puzzle.id)),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            for (name, solution) in valid_solutions {
                let display_name = name.clone(); // clone for display
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        SolutionButton {
                            solution_name: name, // move original name
                            score: solution.score,
                        },
                        Pickable::default(),
                    ))
                    .with_child((
                        Text::new(format!("{} (Score: {})", display_name, solution.score)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            }
        });
}

pub fn solution_list_interaction(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<(&Interaction, &SolutionButton), (Changed<Interaction>, With<Button>)>,
    puzzle: Option<Res<CurrentPuzzle>>,
) {
    let Some(puzzle) = puzzle else { return };
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed {
            let solution_path = format!("assets/puzzles/{}/solutions/{}.ron", puzzle.id, button.solution_name);
            if let Ok(content) = fs::read_to_string(&solution_path) {
                if let Ok(solution) = ron::from_str::<Solution>(&content) {
                    commands.insert_resource(SelectedSolution {
                        puzzle_id: puzzle.id.clone(),
                        solution,
                        puzzle_data: puzzle.data.clone(),
                    });
                    next_state.set(AppState::SolutionView);
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Puzzle gameplay (sandbox-like)
// -----------------------------------------------------------------------------

fn board_anchor_for_size(size: IVec2) -> Vec3 {
    let board_width = size.x as f32 * TILE_SIZE;
    let bottom_y = BOARD_TOP_Y - (size.y - 1) as f32 * TILE_SIZE;
    let x = -board_width / 2.0;
    Vec3::new(x, bottom_y, 0.0)
}

fn grid_to_world_puzzle(grid: IVec2, board: &PuzzleBoardInfo) -> Vec3 {
    board.anchor + Vec3::new(grid.x as f32 * board.tile_size, grid.y as f32 * board.tile_size, 0.0)
}

fn world_to_grid_puzzle(world: Vec3, board: &PuzzleBoardInfo) -> IVec2 {
    let local = world - board.anchor;
    IVec2::new(
        (local.x / board.tile_size).round() as i32,
        (local.y / board.tile_size).round() as i32,
    )
}

fn is_in_bounds_puzzle(grid: IVec2, board: &PuzzleBoardInfo) -> bool {
    grid.x >= 0 && grid.x < board.size.x && grid.y >= 0 && grid.y < board.size.y
}

fn is_cell_available_puzzle(
    grid: IVec2,
    board_cells: &HashMap<IVec2, LinearRgba>,
    disabled_cells: &HashSet<IVec2>,
    board_info: &PuzzleBoardInfo,
) -> bool {
    is_in_bounds_puzzle(grid, board_info) && !board_cells.contains_key(&grid) && !disabled_cells.contains(&grid)
}

fn stash_left_x_puzzle(board_info: &PuzzleBoardInfo) -> f32 {
    let board_right = board_info.anchor.x + (board_info.size.x as f32 - 0.5) * board_info.tile_size;
    board_right + PUZZLE_STASH_GAP
}

fn stash_top_y_puzzle() -> f32 {
    BOARD_TOP_Y + TILE_SIZE / 2.0
}

// Custom spawn for puzzle pieces (interactive)
fn spawn_puzzle_piece(
    commands: &mut Commands,
    type_id: usize,
    shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32,
    effects: Vec<GameEffect>,
    pos: Vec3,
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
                board_side: BoardSide::Single,
            },
            Cleanup,
            DraftPiece,
            Pickable::default(),
        ))
        .id();

    commands.entity(entity)
        .observe(on_puzzle_drag_start)
        .observe(on_puzzle_drag)
        .observe(on_puzzle_drag_end)
        .observe(crate::systems::interaction::on_hover_in)
        .observe(crate::systems::interaction::on_hover_out);

    crate::systems::visuals::refresh_piece_visuals(commands, entity, &shape, color);

    commands.entity(entity).with_children(|parent| {
        for offset in &shape {
            let mut child = parent.spawn((
                Sprite::from_color(color, Vec2::splat(TILE_SIZE - 4.0)),
                Transform::from_translation(offset.as_vec2().extend(0.0) * TILE_SIZE),
                Pickable::default(),
            ));
            child.observe(crate::systems::interaction::on_child_hover_in)
                .observe(crate::systems::interaction::on_child_hover_out)
                .observe(on_puzzle_drag_start)
                .observe(on_puzzle_drag)
                .observe(on_puzzle_drag_end);
        }
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
                    preview.insert(Pickable::default())
                        .observe(crate::systems::interaction::on_child_hover_in)
                        .observe(crate::systems::interaction::on_child_hover_out)
                        .observe(on_puzzle_drag_start)
                        .observe(on_puzzle_drag)
                        .observe(on_puzzle_drag_end);
                }
            }
        }
    });

    entity
}

// Drag & drop for puzzles (sandbox style - no unplacing other pieces)
fn get_piece_entity(
    target: Entity,
    piece_query: &Query<(), With<Piece>>,
    child_of_query: &Query<&ChildOf>,
) -> Option<Entity> {
    if piece_query.contains(target) {
        Some(target)
    } else if let Ok(child_of) = child_of_query.get(target) {
        Some(child_of.parent())
    } else {
        None
    }
}

pub fn on_puzzle_drag_start(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    locked_query: Query<(), With<LockedPiece>>,
    mut puzzle_state: ResMut<PuzzleGameState>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Piece, &Children), Without<LockedPiece>>,
        Query<(Entity, &mut Piece, &mut Transform), (With<DraftPiece>, Without<LockedPiece>)>,
    )>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return };
    if locked_query.contains(piece_entity) { return; }

    // Unplace any other draft piece that is on board (only for puzzle mode)
    for (other_entity, mut other_piece, mut other_transform) in param_set.p1().iter_mut() {
        if other_entity != piece_entity && other_piece.placed_at.is_some() {
            if let Some(old_pos) = other_piece.placed_at {
                for offset in &other_piece.shape {
                    puzzle_state.board_cells.remove(&(old_pos + *offset));
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
                puzzle_state.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
        }
    }
}

pub fn on_puzzle_drag(
    on: On<Pointer<Drag>>,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &Piece)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut commands: Commands,
    puzzle_state: Res<PuzzleGameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
    board_info: Res<PuzzleBoardInfo>,
) {
    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return };
    if locked_query.contains(piece_entity) { return; }
    if let Ok((mut transform, piece)) = drag_piece_query.get_mut(piece_entity) {
        transform.translation.x += on.delta.x;
        transform.translation.y -= on.delta.y;

        for entity in &ghost_query {
            let _ = commands.entity(entity).try_despawn();
        }
        let grid_pos = world_to_grid_puzzle(transform.translation, &board_info);
        let mut can_place = true;
        for offset in &piece.shape {
            let tile = grid_pos + *offset;
            if !is_cell_available_puzzle(tile, &puzzle_state.board_cells, &puzzle_state.disabled_cells, &board_info) {
                can_place = false;
                break;
            }
        }
        if can_place {
            let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
            for offset in &piece.shape {
                commands.spawn((
                    Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_puzzle(grid_pos + *offset, &board_info).with_z(1.0)),
                    GhostTile,
                ));
            }
        }
    }
}

pub fn on_puzzle_drag_end(
    on: On<Pointer<DragEnd>>,
    mut commands: Commands,
    piece_query: Query<(), With<Piece>>,
    child_of_query: Query<&ChildOf>,
    mut drag_piece_query: Query<(&mut Transform, &mut Piece, &Children)>,
    locked_query: Query<(), With<LockedPiece>>,
    mut puzzle_state: ResMut<PuzzleGameState>,
    ghost_query: Query<Entity, With<GhostTile>>,
    board_info: Res<PuzzleBoardInfo>,
) {
    for entity in &ghost_query {
        let _ = commands.entity(entity).try_despawn();
    }

    let target = on.event_target();
    let Some(piece_entity) = get_piece_entity(target, &piece_query, &child_of_query) else { return };
    if locked_query.contains(piece_entity) { return; }
    commands.entity(piece_entity).remove::<Dragging>();

    if let Ok((mut transform, mut piece, _children)) = drag_piece_query.get_mut(piece_entity) {
        let grid_pos = world_to_grid_puzzle(transform.translation, &board_info);
        let mut can_place = true;
        for offset in &piece.shape {
            let cell = grid_pos + *offset;
            if !is_cell_available_puzzle(cell, &puzzle_state.board_cells, &puzzle_state.disabled_cells, &board_info) {
                can_place = false;
                break;
            }
        }

        if can_place {
            transform.translation = grid_to_world_puzzle(grid_pos, &board_info).with_z(1.0);
            piece.placed_at = Some(grid_pos);
            for offset in &piece.shape {
                puzzle_state.board_cells.insert(grid_pos + *offset, piece.color);
            }
            // Do NOT unplace other pieces (sandbox behavior)
        } else {
            transform.translation = piece.original_pos;
            transform.translation.z = piece.original_pos.z;
            transform.rotation = Quat::IDENTITY;
            piece.shape = piece.original_shape.clone();
            piece.effects = piece.original_effects.clone();
        }
    }
}

pub fn handle_puzzle_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut piece_query: Query<
        (&mut Transform, &mut Piece, &Children),
        (With<Dragging>, Without<OpponentPiece>),
    >,
    mut preview_query: Query<&mut EffectPreview>,
    mut commands: Commands,
    ghost_query: Query<Entity, With<GhostTile>>,
    puzzle_state: Res<PuzzleGameState>,
    board_info: Res<PuzzleBoardInfo>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Right) {
        for (mut transform, mut piece, children) in piece_query.iter_mut() {
            transform.rotate_z(-std::f32::consts::FRAC_PI_2);
            for offset in &mut piece.shape {
                let old = *offset;
                *offset = IVec2::new(old.y, -old.x);
            }
            for effect in &mut piece.effects {
                if let Some(offsets) = &mut effect.offsets {
                    for offset in offsets {
                        let old = *offset;
                        *offset = IVec2::new(old.y, -old.x);
                    }
                }
            }
            for &child in children {
                if let Ok(mut preview) = preview_query.get_mut(child) {
                    let old = preview.offset;
                    preview.offset = IVec2::new(old.y, -old.x);
                }
            }
            for entity in ghost_query.iter() {
                let _ = commands.entity(entity).try_despawn();
            }
            let grid_pos = world_to_grid_puzzle(transform.translation, &board_info);
            let mut can_place = true;
            for offset in &piece.shape {
                let tile = grid_pos + *offset;
                if !is_cell_available_puzzle(tile, &puzzle_state.board_cells, &puzzle_state.disabled_cells, &board_info) {
                    can_place = false;
                    break;
                }
            }
            if can_place {
                let ghost_color = LinearRgba::WHITE.with_alpha(0.3);
                for offset in &piece.shape {
                    commands.spawn((
                        Sprite::from_color(ghost_color, Vec2::splat(TILE_SIZE - 2.0)),
                        Transform::from_translation(grid_to_world_puzzle(grid_pos + *offset, &board_info).with_z(1.0)),
                        GhostTile,
                    ));
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// UI systems for puzzle gameplay
// -----------------------------------------------------------------------------

pub fn update_puzzle_score_ui(
    puzzle_state: Res<PuzzleGameState>,
    mut query: Query<(&mut Text2d, &mut Transform), With<ScoreText>>,
) {
    if puzzle_state.is_changed() {
        for (mut text, mut transform) in &mut query {
            let score_str = format!("Score: {}", puzzle_state.score);
            text.0 = score_str.clone();
            transform.translation = score_text_world_pos(&score_str, SCORE_FONT_SIZE);
        }
    }
}

pub fn update_puzzle_stash_labels(
    mut label_query: Query<(&mut Text2d, &StashLabel)>,
    piece_query: Query<&Piece>,
) {
    for (mut text, label) in &mut label_query {
        let count = piece_query
            .iter()
            .filter(|p| p.type_id == label.0 && p.placed_at.is_none() && p.board_side == BoardSide::Single)
            .count();
        text.0 = format!("x{}", count);
    }
}

pub fn update_puzzle_effect_previews(
    puzzle_state: Res<PuzzleGameState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>, Has<Dragging>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
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
                        if crate::helpers::is_in_bounds(target_cell) {
                            active = check_condition(&preview.condition, Some(target_cell), &puzzle_state.board_cells);
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

fn color_name_from_rgba(rgba: &LinearRgba) -> &'static str {
    let red = Color::srgb_u8(216, 46, 63).to_linear();
    let blue = Color::srgb_u8(53, 129, 216).to_linear();
    let green = Color::srgb_u8(40, 204, 45).to_linear();
    if linear_rgba_near(rgba, &red) { "RED" }
    else if linear_rgba_near(rgba, &blue) { "BLUE" }
    else if linear_rgba_near(rgba, &green) { "GREEN" }
    else { "UNKNOWN" }
}

fn get_effect_description(cond: &EffectCondition, descs: &EffectDescriptions) -> String {
    let key = match cond {
        EffectCondition::MatchesColor(c) => format!("MatchesColor({})", color_name_from_rgba(c)),
        EffectCondition::IsEmpty => "IsEmpty".to_string(),
        EffectCondition::NoColorOnBoard(c) => format!("NoColorOnBoard({})", color_name_from_rgba(c)),
    };
    descs.descriptions.get(&key).cloned().unwrap_or_else(|| format!("Unknown effect: {}", key))
}

pub fn update_puzzle_tooltip(
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
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            for offset in &piece.shape {
                let local = Vec3::new(offset.x as f32 * TILE_SIZE, offset.y as f32 * TILE_SIZE, 0.0);
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
                                let color_name = match &effect.condition {
                                    EffectCondition::MatchesColor(c) => color_name_from_rgba(c),
                                    EffectCondition::IsEmpty => "empty",
                                    EffectCondition::NoColorOnBoard(c) => color_name_from_rgba(c),
                                };
                                let desc = desc_template
                                    .replace("{points}", &effect.points.to_string())
                                    .replace("{color}", color_name);
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
                            let entity = commands.spawn((
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
                            )).id();
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

pub fn recalculate_puzzle_score_system(
    mut puzzle_state: ResMut<PuzzleGameState>,
    piece_query: Query<&Piece>,
) {
    puzzle_state.score = crate::systems::scoring::recalculate_score(&puzzle_state.board_cells, &piece_query);
}

pub fn update_puzzle_contributions_system(
    mut commands: Commands,
    puzzle_state: Res<PuzzleGameState>,
    board_info: Res<PuzzleBoardInfo>,
    mut piece_query: Query<(Entity, &Piece, Option<&mut ContributionDisplay>)>,
) {
    for (piece_entity, piece, display_opt) in piece_query.iter_mut() {
        if let Some(pos) = piece.placed_at {
            let contribution = compute_piece_contribution(piece, &puzzle_state.board_cells);
            let sign = if contribution >= 0 { "+" } else { "" };
            let text_str = format!("{}{}", sign, contribution);
            // Compute world position at the center of the piece's bounding box
            let mut min_x = i32::MAX;
            let mut max_x = i32::MIN;
            let mut min_y = i32::MAX;
            let mut max_y = i32::MIN;
            for offset in &piece.shape {
                let cell = pos + *offset;
                min_x = min_x.min(cell.x);
                max_x = max_x.max(cell.x);
                min_y = min_y.min(cell.y);
                max_y = max_y.max(cell.y);
            }
            let center_x = (min_x + max_x) as f32 / 2.0;
            let center_y = (min_y + max_y) as f32 / 2.0;
            let world_pos = grid_to_world_puzzle(IVec2::new(center_x as i32, center_y as i32), &board_info).with_z(5.0);
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
            let text_entity = commands.spawn((
                Text2d::new(text_str),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_translation(world_pos),
                Cleanup,
            )).id();
            commands.entity(piece_entity).insert(ContributionDisplay(text_entity));
        } else {
            if let Some(display) = display_opt {
                commands.entity(display.0).despawn();
                commands.entity(piece_entity).remove::<ContributionDisplay>();
            }
        }
    }
}

// Setup puzzle gameplay
pub fn setup_puzzle(
    mut commands: Commands,
    puzzle: Res<CurrentPuzzle>,
    windows: Query<&Window>,
) {
    let data = &puzzle.data;
    let board_size = data.board_size;
    let anchor = board_anchor_for_size(board_size);
    let board_info = PuzzleBoardInfo {
        size: board_size,
        anchor,
        tile_size: TILE_SIZE,
    };
    commands.insert_resource(board_info.clone());
    commands.insert_resource(PuzzleGameState::default());
    commands.insert_resource(InventoryScroll::default());

    commands.spawn((Camera2d, Cleanup));

    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..board_size.x {
        for y in 0..board_size.y {
            let tile = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_puzzle(IVec2::new(x, y), &board_info)),
                ))
                .id();
            commands.entity(board_root).add_child(tile);
        }
    }

    let mut disabled = HashSet::new();
    for &cell in &data.blocked_cells {
        if is_in_bounds_puzzle(cell, &board_info) {
            disabled.insert(cell);
            spawn_disabled_visual_puzzle(&mut commands, cell, &board_info);
        }
    }
    commands.insert_resource(PuzzleGameState {
        disabled_cells: disabled,
        ..default()
    });

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

    let color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let stash_left = stash_left_x_puzzle(&board_info);
    let stash_width = STASH_WIDTH;
    let stash_visible_height = STASH_VISIBLE_HEIGHT;
    let stash_top = stash_top_y_puzzle();
    let stash_bottom = stash_top - stash_visible_height;

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
                Transform::from_xyz(stash_left + stash_width, (stash_top + stash_bottom) / 2.0, 0.5),
            ));
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(stash_width, thickness)),
                Transform::from_xyz(stash_left + stash_width / 2.0, stash_top, 0.5),
            ));
            parent.spawn((
                Sprite::from_color(outline_color, Vec2::new(stash_width, thickness)),
                Transform::from_xyz(stash_left + stash_width / 2.0, stash_bottom, 0.5),
            ));
        });

    let window = windows.single().expect("Primary window missing");
    let screen_x = (window.width() / 2.0) + stash_left;
    let screen_y = (window.height() / 2.0) - stash_top;
    commands.insert_resource(StashScreenRect {
        x: screen_x,
        y: screen_y,
        width: stash_width,
        height: stash_visible_height,
    });

    let mut current_y_offset = 0.0f32;
    let mut total_height = 0.0f32;
    let stash_center_x = stash_left + stash_width / 2.0;

    for (type_id, piece_data) in data.pieces.iter().enumerate() {
        let raw = RawPieceConfig {
            shape: piece_data.shape.clone(),
            color: piece_data.color.clone(),
            points: piece_data.points,
            effects: piece_data.effects.clone(),
            piece_type: PieceType::Static,
        };
        let (color, effects) = randomize_piece_properties(&raw, &color_map);
        let min_x = piece_data.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = piece_data.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let min_y = piece_data.shape.iter().map(|o| o.y).min().unwrap_or(0);
        let max_y = piece_data.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let piece_height = (max_y - min_y + 1) as f32 * TILE_SIZE;

        let piece_x = stash_center_x - ((min_x + max_x) as f32) / 2.0 * TILE_SIZE;
        let base_y = stash_top - current_y_offset - (max_y as f32 * TILE_SIZE + TILE_SIZE / 2.0);

        for copy_idx in 0..piece_data.count {
            let pos = Vec3::new(piece_x, base_y, 1.0 + copy_idx as f32 * 0.001);
            let entity = spawn_puzzle_piece(
                &mut commands,
                type_id,
                piece_data.shape.clone(),
                color,
                piece_data.points,
                effects.clone(),
                pos,
            );
            commands.entity(entity).insert(StashPosition {
                desired_world_y: base_y,
            });
        }

        let label_y = base_y + max_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        commands.spawn((
            Text2d::new(format!("x{}", piece_data.count)),
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
        total_height = current_y_offset;
    }

    commands.insert_resource(StashContentHeight(total_height));
    commands.insert_resource(InventoryScroll::default());
}

fn spawn_disabled_visual_puzzle(commands: &mut Commands, grid: IVec2, board: &PuzzleBoardInfo) {
    let center = grid_to_world_puzzle(grid, board).with_z(3.0);
    let color = Color::BLACK;
    let size = TILE_SIZE * 0.8;
    let thickness = 4.0;
    let angle1 = -std::f32::consts::FRAC_PI_4;
    let angle2 = std::f32::consts::FRAC_PI_4;
    let line_sprite = Sprite::from_color(color, Vec2::new(size, thickness));
    commands.spawn((
        Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle1)),
        line_sprite.clone(),
        Cleanup,
    ));
    commands.spawn((
        Transform::from_translation(center).with_rotation(Quat::from_rotation_z(angle2)),
        line_sprite,
        Cleanup,
    ));
}

// Reset puzzle state
pub fn reset_puzzle_state(mut commands: Commands) {
    commands.remove_resource::<PuzzleGameState>();
    commands.remove_resource::<PuzzleBoardInfo>();
    commands.remove_resource::<CurrentPuzzle>();
    commands.remove_resource::<InventoryScroll>();
    commands.remove_resource::<StashContentHeight>();
    commands.remove_resource::<StashScreenRect>();
}

// -----------------------------------------------------------------------------
// Solution view (read-only)
// -----------------------------------------------------------------------------

pub fn setup_solution_view(mut commands: Commands, selected: Res<SelectedSolution>) {
    let data = &selected.puzzle_data;
    let board_size = data.board_size;
    let anchor = board_anchor_for_size(board_size);
    let board_info = PuzzleBoardInfo {
        size: board_size,
        anchor,
        tile_size: TILE_SIZE,
    };
    commands.insert_resource(board_info);

    commands.spawn((Camera2d, Cleanup));

    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..board_size.x {
        for y in 0..board_size.y {
            let tile = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_puzzle(IVec2::new(x, y), &board_info)),
                ))
                .id();
            commands.entity(board_root).add_child(tile);
        }
    }

    for &cell in &data.blocked_cells {
        if is_in_bounds_puzzle(cell, &board_info) {
            spawn_disabled_visual_puzzle(&mut commands, cell, &board_info);
        }
    }

    commands.spawn((
        Text2d::new(format!("Solution Score: {}", selected.solution.score)),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos(&format!("Solution Score: {}", selected.solution.score), SCORE_FONT_SIZE)),
        ScoreText,
        Cleanup,
    ));

    let color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let mut board_cells = HashMap::new();
    for placement in &selected.solution.placements {
        let piece_index = placement.piece;
        if piece_index >= data.pieces.len() {
            continue;
        }
        let piece_data = &data.pieces[piece_index];
        let color = *color_map.get(&piece_data.color).unwrap_or(&LinearRgba::WHITE);
        let mut shape = piece_data.shape.clone();
        for _ in 0..placement.rot % 4 {
            shape = shape.iter().map(|&v| IVec2::new(v.y, -v.x)).collect();
        }
        let world_pos = grid_to_world_puzzle(placement.pos, &board_info);
        // Convert effects (if any) - for now empty
        let effects = vec![]; // piece_data.effects could be converted if needed
        spawn_solution_piece(
            &mut commands,
            shape.clone(),
            color,
            world_pos,
            placement.pos,
            piece_data.points,
            effects,
        );
        for offset in &shape {
            board_cells.insert(placement.pos + *offset, color);
        }
    }

    commands.insert_resource(PuzzleGameState {
        board_cells,
        disabled_cells: data.blocked_cells.iter().copied().collect(),
        score: selected.solution.score,
    });
}

fn spawn_solution_piece(
    commands: &mut Commands,
    shape: Vec<IVec2>,
    color: LinearRgba,
    pos: Vec3,
    origin: IVec2,
    points: i32,
    effects: Vec<GameEffect>,
) -> Entity {
    let entity = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            Piece {
                type_id: 0,
                shape: shape.clone(),
                original_shape: shape.clone(),
                color,
                points,
                effects: effects.clone(),
                original_effects: effects,
                original_pos: pos,
                placed_at: Some(origin),
                board_side: BoardSide::Single,
            },
            LockedPiece,
            Cleanup,
            Pickable::default(),
        ))
        .observe(crate::systems::interaction::on_hover_in)
        .observe(crate::systems::interaction::on_hover_out)
        .id();

    crate::systems::visuals::refresh_piece_visuals(commands, entity, &shape, color);
    entity
}

pub fn reset_solution_view(mut commands: Commands) {
    commands.remove_resource::<PuzzleBoardInfo>();
    commands.remove_resource::<PuzzleGameState>();
    commands.remove_resource::<SelectedSolution>();
    // Keep CurrentPuzzle - it's needed when returning to the solution list
}