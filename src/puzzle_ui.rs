use crate::AppState;
use crate::Cleanup;
use crate::components::*;
use crate::resources::TooltipState;
use bevy::picking::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[cfg(not(target_arch = "wasm32"))]
use chrono::Local;
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self};

// -----------------------------------------------------------------------------
// Puzzle data structures (shared)
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
    pub effects: Vec<crate::config::RawGameEffect>,
}

#[derive(Deserialize, Serialize, Clone, Hash)]
pub struct Solution {
    pub score: i32,
    pub placements: Vec<SolutionPlacement>,
    #[serde(default = "default_timestamp")]
    pub timestamp: String,
}

fn default_timestamp() -> String {
    "0000-00-00-00-00-00".to_string()
}

#[derive(Deserialize, Serialize, Clone, Hash)]
pub struct SolutionPlacement {
    pub piece: usize,
    pub pos: IVec2,
    pub rot: u32,
}

// -----------------------------------------------------------------------------
// Resource types (shared)
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
    pub solution: Solution,
    pub puzzle_data: PuzzleData,
}

#[derive(Resource)]
pub struct LastSavedSolution {
    pub hash: u64,
}

// -----------------------------------------------------------------------------
// Cross‑platform storage
// -----------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
pub mod storage {
    use super::*;

    pub fn get_puzzle_list() -> Vec<String> {
        let puzzles_dir = "assets/puzzles";
        let mut list = if let Ok(entries) = fs::read_dir(puzzles_dir) {
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
        };
        list.sort();
        list
    }

    pub fn load_puzzle_data(id: &str) -> Option<PuzzleData> {
        let path = format!("assets/puzzles/{}/data.ron", id);
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| ron::from_str(&content).ok())
    }

    pub fn get_solutions(puzzle_id: &str) -> Vec<(String, Solution)> {
        let solutions_dir = format!("assets/puzzles/{}/solutions", puzzle_id);
        let mut solutions = Vec::new();
        if let Ok(entries) = fs::read_dir(solutions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(solution) = ron::from_str::<Solution>(&content) {
                            if validate_solution(&solution, puzzle_id) {
                                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                                solutions.push((name, solution));
                            }
                        }
                    }
                }
            }
        }
        solutions.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));
        solutions
    }

    pub fn save_solution(puzzle_id: &str, solution: &Solution) -> bool {
        let solutions_dir = format!("assets/puzzles/{}/solutions", puzzle_id);
        let _ = fs::create_dir_all(&solutions_dir);
        let filename = format!("{}.ron", solution.timestamp);
        let path = format!("{}/{}", solutions_dir, filename);
        let content = match ron::ser::to_string_pretty(solution, ron::ser::PrettyConfig::default())
        {
            Ok(c) => c,
            Err(_) => return false,
        };
        fs::write(&path, content).is_ok()
    }

    pub fn delete_all_user_solutions() -> Result<u32, String> {
        let puzzles_dir = "assets/puzzles";
        let mut deleted_count = 0;
        if let Ok(entries) = fs::read_dir(puzzles_dir) {
            for puzzle_entry in entries.flatten() {
                let puzzle_path = puzzle_entry.path();
                if puzzle_path.is_dir() {
                    let solutions_dir = puzzle_path.join("solutions");
                    if solutions_dir.exists() && solutions_dir.is_dir() {
                        if let Ok(solution_files) = fs::read_dir(&solutions_dir) {
                            for file_entry in solution_files.flatten() {
                                let path = file_entry.path();
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    let pattern = regex::Regex::new(
                                        r"^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}\.ron$",
                                    )
                                    .unwrap();
                                    if pattern.is_match(name) && fs::remove_file(&path).is_ok() {
                                        deleted_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(deleted_count)
    }
}

#[cfg(target_arch = "wasm32")]
pub mod storage {
    use super::*;
    use web_sys::{Storage, window};

    fn local_storage() -> Option<Storage> {
        window()?.local_storage().ok()?
    }

    pub fn get_puzzle_list() -> Vec<String> {
        let mut list = vec![
            "001".to_string(),
            "002".to_string(),
            "003".to_string(),
            "004".to_string(),
            "005".to_string(),
            "006".to_string(),
        ];
        list.sort();
        list
    }

    pub fn load_puzzle_data(id: &str) -> Option<PuzzleData> {
        match id {
            "001" => {
                let content = include_str!("../assets/puzzles/001/data.ron");
                ron::from_str(content).ok()
            }
            "002" => {
                let content = include_str!("../assets/puzzles/002/data.ron");
                ron::from_str(content).ok()
            }
            "003" => {
                let content = include_str!("../assets/puzzles/003/data.ron");
                ron::from_str(content).ok()
            }
            "004" => {
                let content = include_str!("../assets/puzzles/004/data.ron");
                ron::from_str(content).ok()
            }
            "005" => {
                let content = include_str!("../assets/puzzles/005/data.ron");
                ron::from_str(content).ok()
            }
            "006" => {
                let content = include_str!("../assets/puzzles/006/data.ron");
                ron::from_str(content).ok()
            }
            _ => None,
        }
    }

    fn storage_key(puzzle_id: &str, timestamp: &str) -> String {
        format!("puzzle_{}_solution_{}", puzzle_id, timestamp)
    }

    pub fn get_solutions(puzzle_id: &str) -> Vec<(String, Solution)> {
        let mut solutions = Vec::new();

        let base_solution_opt = match puzzle_id {
            "001" => {
                let content = include_str!("../assets/puzzles/001/solutions/base.ron");
                ron::from_str(content).ok()
            }
            "002" => {
                let content = include_str!("../assets/puzzles/002/solutions/base.ron");
                ron::from_str(content).ok()
            }
            "003" => {
                let content = include_str!("../assets/puzzles/003/solutions/base.ron");
                ron::from_str(content).ok()
            }
            "004" => {
                let content = include_str!("../assets/puzzles/004/solutions/base.ron");
                ron::from_str(content).ok()
            }
            "005" => {
                let content = include_str!("../assets/puzzles/005/solutions/base.ron");
                ron::from_str(content).ok()
            }
            "006" => {
                let content = include_str!("../assets/puzzles/006/solutions/base.ron");
                ron::from_str(content).ok()
            }
            _ => None,
        };
        if let Some(solution) = base_solution_opt {
            if validate_solution(&solution, puzzle_id) {
                solutions.push(("base".to_string(), solution));
            }
        }

        let storage = match local_storage() {
            Some(s) => s,
            None => return solutions,
        };
        let prefix = format!("puzzle_{}_solution_", puzzle_id);
        let len = storage.length().unwrap_or(0);
        for i in 0..len {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with(&prefix) {
                    if let Ok(Some(value)) = storage.get_item(&key) {
                        if let Ok(solution) = ron::from_str::<Solution>(&value) {
                            if validate_solution(&solution, puzzle_id) {
                                let timestamp = key.trim_start_matches(&prefix).to_string();
                                if timestamp != "base" {
                                    solutions.push((timestamp, solution));
                                }
                            }
                        }
                    }
                }
            }
        }
        solutions.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));
        solutions
    }

    pub fn save_solution(puzzle_id: &str, solution: &Solution) -> bool {
        let storage = match local_storage() {
            Some(s) => s,
            None => return false,
        };
        let key = storage_key(puzzle_id, &solution.timestamp);
        let value = match ron::ser::to_string_pretty(solution, ron::ser::PrettyConfig::default()) {
            Ok(v) => v,
            Err(_) => return false,
        };
        storage.set_item(&key, &value).is_ok()
    }

    pub fn delete_all_user_solutions() -> Result<u32, String> {
        let storage = match local_storage() {
            Some(s) => s,
            None => return Ok(0),
        };
        let mut deleted = 0;
        let len = storage
            .length()
            .map_err(|_| "failed to get length".to_string())?;
        let mut to_remove = Vec::new();
        for i in 0..len {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with("puzzle_") && key.contains("_solution_") {
                    to_remove.push(key);
                }
            }
        }
        for key in to_remove {
            if storage.remove_item(&key).is_ok() {
                deleted += 1;
            }
        }
        Ok(deleted)
    }
}

// -----------------------------------------------------------------------------
// Shared validation and helpers
// -----------------------------------------------------------------------------

pub fn validate_solution(solution: &Solution, puzzle_id: &str) -> bool {
    use crate::components::{EffectCondition, GameEffect};
    use crate::config::RawEffectCondition;
    use std::collections::{HashMap, HashSet};

    let puzzle_data = match storage::load_puzzle_data(puzzle_id) {
        Some(d) => d,
        None => return false,
    };
    let board_size = puzzle_data.board_size;
    let blocked_cells: HashSet<IVec2> = puzzle_data.blocked_cells.iter().copied().collect();
    let pieces = &puzzle_data.pieces;
    let mut used_counts = vec![0; pieces.len()];
    let mut board_cells: HashMap<IVec2, LinearRgba> = HashMap::new();
    let mut placed = Vec::new(); // stores (origin, shape, color, points, effects, occupied_cells)

    let color_map = crate::colors::get_color_map();

    // First pass: validate placements and build board
    for placement in &solution.placements {
        if placement.piece >= pieces.len() {
            return false;
        }
        let piece_def = &pieces[placement.piece];
        used_counts[placement.piece] += 1;
        if used_counts[placement.piece] > piece_def.count {
            return false;
        }

        let mut shape = piece_def.shape.clone();
        let rot = placement.rot % 4;
        for _ in 0..rot {
            shape = shape.iter().map(|&v| IVec2::new(v.y, -v.x)).collect();
        }

        // Build rotated effects
        let raw_effects = piece_def.effects.clone();
        let mut rotated_effects = Vec::new();
        for re in raw_effects {
            let mut offsets = re.offsets.clone();
            for _ in 0..rot {
                offsets = offsets.iter().map(|&v| IVec2::new(v.y, -v.x)).collect();
            }
            let condition = match &re.condition {
                RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
                RawEffectCondition::MatchesColor(c) => {
                    EffectCondition::MatchesColor(*color_map.get(c).unwrap())
                }
                RawEffectCondition::NoColorOnBoard(c) => {
                    EffectCondition::NoColorOnBoard(*color_map.get(c).unwrap())
                }
                RawEffectCondition::MatchesSize(size) => {
                    EffectCondition::MatchesSize(*size as usize)
                }
            };
            rotated_effects.push(GameEffect {
                condition,
                points: re.points,
                offsets: if offsets.is_empty() {
                    None
                } else {
                    Some(offsets)
                },
            });
        }

        let color = *color_map.get(&piece_def.color).unwrap();

        // Check cells and insert into board
        let mut occupied = Vec::new();
        for offset in &shape {
            let cell = placement.pos + *offset;
            if cell.x < 0 || cell.x >= board_size.x || cell.y < 0 || cell.y >= board_size.y {
                return false;
            }
            if blocked_cells.contains(&cell) || board_cells.contains_key(&cell) {
                return false;
            }
            board_cells.insert(cell, color);
            occupied.push(cell);
        }
        placed.push((
            placement.pos,
            shape,
            color,
            piece_def.points,
            rotated_effects,
            occupied,
        ));
    }

    // Second pass: compute total score including effects, excluding own cells for NoColorOnBoard
    // We need piece shapes for size checks. Build a list of (cell, size)
    let mut piece_cells: HashMap<IVec2, usize> = HashMap::new();
    for (origin, shape, _color, _, _, _) in &placed {
        for offset in shape {
            piece_cells.insert(origin + *offset, shape.len());
        }
    }

    let mut total_score = 0;
    for (origin, _shape, _color, raw_points, effects, occupied) in &placed {
        total_score += raw_points;
        let exclude_set: HashSet<IVec2> = occupied.iter().copied().collect();
        for effect in effects {
            if let Some(offsets) = &effect.offsets {
                for offset in offsets {
                    let target_cell = *origin + *offset;
                    if target_cell.x >= 0
                        && target_cell.x < board_size.x
                        && target_cell.y >= 0
                        && target_cell.y < board_size.y
                    {
                        let condition_met = match &effect.condition {
                            EffectCondition::MatchesColor(c) => board_cells
                                .get(&target_cell)
                                .map_or(false, |&col| linear_rgba_near(&col, c)),
                            EffectCondition::IsEmpty => !board_cells.contains_key(&target_cell),
                            EffectCondition::NoColorOnBoard(_) => false,
                            EffectCondition::MatchesSize(size) => {
                                piece_cells.get(&target_cell).map_or(false, |&s| s == *size)
                            }
                        };
                        if condition_met {
                            total_score += effect.points;
                        }
                    }
                }
            } else {
                // Global effect (NoColorOnBoard)
                if let EffectCondition::NoColorOnBoard(c) = &effect.condition {
                    let mut found_other = false;
                    for (cell, board_color) in board_cells.iter() {
                        if exclude_set.contains(cell) {
                            continue;
                        }
                        if linear_rgba_near(board_color, c) {
                            found_other = true;
                            break;
                        }
                    }
                    if !found_other {
                        total_score += effect.points;
                    }
                }
            }
        }
    }

    total_score == solution.score
}

fn linear_rgba_near(a: &LinearRgba, b: &LinearRgba) -> bool {
    let eps = 0.001;
    (a.red - b.red).abs() < eps
        && (a.green - b.green).abs() < eps
        && (a.blue - b.blue).abs() < eps
        && (a.alpha - b.alpha).abs() < eps
}

pub fn get_current_solution(
    pieces: &Query<&Piece>,
    puzzle_state: &PuzzleGameState,
    puzzle_data: &PuzzleData,
) -> Option<(Solution, u64)> {
    let mut placements = Vec::new();
    for piece in pieces.iter() {
        if let Some(pos) = piece.placed_at {
            let piece_idx = piece.type_id;
            if piece_idx >= puzzle_data.pieces.len() {
                return None;
            }
            let original = &puzzle_data.pieces[piece_idx].shape;
            let current = &piece.shape;
            let rot = compute_rotation(original, current);
            placements.push(SolutionPlacement {
                piece: piece_idx,
                pos,
                rot,
            });
        }
    }
    if placements.is_empty() {
        return None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    #[cfg(target_arch = "wasm32")]
    let timestamp = js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_else(|| "unknown".to_string());

    let solution = Solution {
        score: puzzle_state.score,
        placements,
        timestamp,
    };
    let mut hasher = DefaultHasher::new();
    solution.hash(&mut hasher);
    let hash = hasher.finish();
    Some((solution, hash))
}

fn compute_rotation(original: &[IVec2], current: &[IVec2]) -> u32 {
    for rot in 0..4 {
        let mut rotated: Vec<IVec2> = original
            .iter()
            .map(|&v| {
                let mut r = v;
                for _ in 0..rot {
                    r = IVec2::new(r.y, -r.x);
                }
                r
            })
            .collect();
        rotated.sort_by_key(|v| (v.x, v.y));
        let mut curr = current.to_vec();
        curr.sort_by_key(|v| (v.x, v.y));
        if rotated == curr {
            return rot;
        }
    }
    0
}

// -----------------------------------------------------------------------------
// Puzzle list UI
// -----------------------------------------------------------------------------

#[derive(Component)]
pub struct PuzzleButton {
    pub puzzle_id: String,
}

#[derive(Component)]
pub struct HelpButton;

pub fn setup_puzzle_list(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let puzzles = storage::get_puzzle_list();
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
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Select a Puzzle"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    header
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            HelpButton,
                            Pickable::default(),
                        ))
                        .with_child((
                            Text::new("?"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });

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
                        PuzzleButton {
                            puzzle_id: id.clone(),
                        },
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

pub fn update_help_tooltip(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    help_query: Query<&Interaction>,
    windows: Query<&Window>,
) {
    let hovering = help_query
        .iter()
        .any(|interaction| *interaction == Interaction::Hovered);
    if hovering {
        if let Ok(window) = windows.single() {
            let tooltip_x = window.width() - 230.0;
            let tooltip_y = 70.0;
            let text = "Left-click to solve puzzle\nRight-click to view solutions";
            if let Some(entity) = tooltip_state.entity {
                commands.entity(entity).insert((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(tooltip_x),
                        top: Val::Px(tooltip_y),
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
                            left: Val::Px(tooltip_x),
                            top: Val::Px(tooltip_y),
                            max_width: Val::Px(250.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                        BorderColor::all(Color::WHITE),
                        GlobalZIndex(100),
                        Text::new(text),
                        Cleanup,
                    ))
                    .id();
                tooltip_state.entity = Some(entity);
            }
        }
    } else if let Some(entity) = tooltip_state.entity.take() {
        commands.entity(entity).despawn();
    }
}

fn on_puzzle_left_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    button_query: Query<&PuzzleButton>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if trigger.event.button == PointerButton::Primary {
        if let Ok(button) = button_query.get(trigger.event_target()) {
            if let Some(data) = storage::load_puzzle_data(&button.puzzle_id) {
                commands.insert_resource(CurrentPuzzle {
                    id: button.puzzle_id.clone(),
                    data,
                });
                next_state.set(AppState::Puzzle);
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
            if let Some(data) = storage::load_puzzle_data(&button.puzzle_id) {
                commands.insert_resource(CurrentPuzzle {
                    id: button.puzzle_id.clone(),
                    data,
                });
                next_state.set(AppState::SolutionList);
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
}

pub fn setup_solution_list(mut commands: Commands, puzzle: Res<CurrentPuzzle>) {
    commands.spawn((Camera2d, Cleanup));

    let valid_solutions = storage::get_solutions(&puzzle.id);

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
                let display_name = name.clone();
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(375.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        SolutionButton {
                            solution_name: name,
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
            let solutions = storage::get_solutions(&puzzle.id);
            for (name, solution) in solutions {
                if name == button.solution_name {
                    commands.insert_resource(SelectedSolution {
                        solution,
                        puzzle_data: puzzle.data.clone(),
                    });
                    next_state.set(AppState::SolutionView);
                    break;
                }
            }
        }
    }
}
