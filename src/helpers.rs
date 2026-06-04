use crate::components::BoardSide;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub const TILE_SIZE: f32 = 40.0;
pub const BOARD_SIZE: IVec2 = IVec2::new(10, 10); // change freely

// Layout constants
const SINGLE_BOARD_LEFT_X: f32 = -250.0; // center of cell (0,0) for single board
const DUEL_GAP: f32 = 160.0; // horizontal gap between duel boards
pub const BOARD_TOP_Y: f32 = 280.0; // world Y of the **top row cell center**
const STASH_GAP: f32 = 60.0; // gap between board and stash in Sandbox

pub const SCORE_FONT_SIZE: f32 = 30.0;
pub const SCORE_Y_OFFSET: f32 = 30.0;
pub const STASH_LABEL_FONT_SIZE: f32 = 24.0;
pub const CONFIRM_BUTTON_WIDTH: f32 = 120.0;
pub const CONFIRM_BUTTON_HEIGHT: f32 = 50.0;
pub const CONFIRM_BUTTON_FONT_SIZE: f32 = 28.0;
pub const STASH_VISIBLE_HEIGHT: f32 = 360.0;
pub const STASH_SCROLL_SPEED: f32 = 1.0;
pub const STASH_WIDTH: f32 = 200.0;

// ── Dynamic helpers ──────────────────────────────

/// World position of the center of the bottom‑left cell (0,0) for a given board side.
pub fn board_anchor(side: BoardSide) -> Vec3 {
    let board_width = BOARD_SIZE.x as f32 * TILE_SIZE;
    let bottom_y = BOARD_TOP_Y - (BOARD_SIZE.y - 1) as f32 * TILE_SIZE;
    let x = match side {
        BoardSide::Single => SINGLE_BOARD_LEFT_X,
        BoardSide::Left => -board_width - DUEL_GAP / 2.0,
        BoardSide::Right => DUEL_GAP / 2.0,
    };
    Vec3::new(x, bottom_y, 0.0)
}

/// World x of the right edge of the board (rightmost tile's right side).
pub fn board_right_edge(side: BoardSide) -> f32 {
    board_anchor(side).x + (BOARD_SIZE.x as f32 - 0.5) * TILE_SIZE
}

/// World y of the top edge of the board (topmost tile's top side).
pub fn board_top_edge() -> f32 {
    BOARD_TOP_Y + TILE_SIZE / 2.0
}

/// Convert grid coordinates to world position, using the side‑aware anchor.
pub fn grid_to_world_for_side(grid: IVec2, side: BoardSide) -> Vec3 {
    board_anchor(side) + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

/// Single‑board version (kept for backward compatibility).
pub fn grid_to_world(grid: IVec2) -> Vec3 {
    grid_to_world_for_side(grid, BoardSide::Single)
}

/// Convert a world position back to grid coordinates for a given side.
pub fn world_to_grid_for_side(world: Vec3, side: BoardSide) -> IVec2 {
    let local = world - board_anchor(side);
    IVec2::new(
        (local.x / TILE_SIZE).round() as i32,
        (local.y / TILE_SIZE).round() as i32,
    )
}

/// Single‑board version.
pub fn world_to_grid(world: Vec3) -> IVec2 {
    world_to_grid_for_side(world, BoardSide::Single)
}

pub fn is_in_bounds(grid: IVec2) -> bool {
    grid.x >= 0 && grid.x < BOARD_SIZE.x && grid.y >= 0 && grid.y < BOARD_SIZE.y
}

pub fn is_cell_available(
    grid: IVec2,
    board_cells: &HashMap<IVec2, LinearRgba>,
    disabled_cells: &HashSet<IVec2>,
) -> bool {
    is_in_bounds(grid) && !board_cells.contains_key(&grid) && !disabled_cells.contains(&grid)
}

/// World position of the score text, based on the board's top edge.
pub fn score_text_world_pos_for_side(text: &str, font_size: f32, side: BoardSide) -> Vec3 {
    let board_left = board_anchor(side).x - TILE_SIZE / 2.0; // left edge of board
    let score_y = board_top_edge() + SCORE_Y_OFFSET;
    let half_width = text.len() as f32 * font_size * 0.25;
    Vec3::new(board_left + half_width, score_y, 0.0)
}

pub fn score_text_world_pos(text: &str, font_size: f32) -> Vec3 {
    score_text_world_pos_for_side(text, font_size, BoardSide::Single)
}

/// Left edge of the stash in Sandbox (depends on board width).
pub fn stash_left_x() -> f32 {
    board_right_edge(BoardSide::Single) + STASH_GAP
}

/// World y for placing pieces below the board (for Draft/Duel stashes).
/// Returns the y for the bottommost tile of the piece to sit just below the board.
pub fn stash_y_below_board(max_y_offset: i32) -> f32 {
    // world y of the centre of cell (0,0) – the board's bottom row
    let bottom_cell_center_y = board_anchor(BoardSide::Single).y;
    // bottom edge of the lowest board tile
    let board_bottom = bottom_cell_center_y - TILE_SIZE / 2.0;
    // gap between board bottom and the top edge of the piece
    let gap = TILE_SIZE * 1.0; // generous gap
    // parent y must place the piece's top tile (at max_y_offset) so that its top edge is at board_bottom - gap
    board_bottom - gap - max_y_offset as f32 * TILE_SIZE - TILE_SIZE / 2.0
}
