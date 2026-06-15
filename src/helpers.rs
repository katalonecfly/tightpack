use crate::components::BoardSide;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub const TILE_SIZE: f32 = 40.0;
pub const BOARD_TOP_Y: f32 = 280.0;
pub const STASH_GAP: f32 = 60.0;

pub const SCORE_FONT_SIZE: f32 = 30.0;
pub const SCORE_Y_OFFSET: f32 = 30.0;
pub const STASH_LABEL_FONT_SIZE: f32 = 24.0;
pub const CONFIRM_BUTTON_WIDTH: f32 = 120.0;
pub const CONFIRM_BUTTON_HEIGHT: f32 = 50.0;
pub const CONFIRM_BUTTON_FONT_SIZE: f32 = 28.0;
pub const STASH_VISIBLE_HEIGHT: f32 = 360.0;
pub const STASH_SCROLL_SPEED: f32 = 1.0;
pub const STASH_WIDTH: f32 = 200.0;

/// World position of the bottom‑left cell (0,0) for a given board side and board size.
pub fn board_anchor(side: BoardSide, board_size: IVec2) -> Vec3 {
    let board_width = board_size.x as f32 * TILE_SIZE;
    let bottom_y = BOARD_TOP_Y - (board_size.y - 1) as f32 * TILE_SIZE;
    let x = match side {
        BoardSide::Single => -board_width / 2.0,
        BoardSide::Left => -board_width - 160.0 / 2.0,
        BoardSide::Right => 160.0 / 2.0,
    };
    Vec3::new(x, bottom_y, 0.0)
}

/// World x of the right edge of the board.
pub fn board_right_edge(side: BoardSide, board_size: IVec2) -> f32 {
    board_anchor(side, board_size).x + (board_size.x as f32 - 0.5) * TILE_SIZE
}

/// World y of the top edge of the board (topmost tile's top side).
pub fn board_top_edge(_board_size: IVec2) -> f32 {
    BOARD_TOP_Y + TILE_SIZE / 2.0
}

pub fn grid_to_world_for_side(grid: IVec2, side: BoardSide, board_size: IVec2) -> Vec3 {
    board_anchor(side, board_size)
        + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

pub fn grid_to_world(grid: IVec2, board_size: IVec2) -> Vec3 {
    grid_to_world_for_side(grid, BoardSide::Single, board_size)
}

pub fn world_to_grid_for_side(world: Vec3, side: BoardSide, board_size: IVec2) -> IVec2 {
    let local = world - board_anchor(side, board_size);
    IVec2::new(
        (local.x / TILE_SIZE).round() as i32,
        (local.y / TILE_SIZE).round() as i32,
    )
}

pub fn is_in_bounds(grid: IVec2, board_size: IVec2) -> bool {
    grid.x >= 0 && grid.x < board_size.x && grid.y >= 0 && grid.y < board_size.y
}

pub fn is_cell_available(
    grid: IVec2,
    board_cells: &HashMap<IVec2, LinearRgba>,
    disabled_cells: &HashSet<IVec2>,
    board_size: IVec2,
) -> bool {
    is_in_bounds(grid, board_size)
        && !board_cells.contains_key(&grid)
        && !disabled_cells.contains(&grid)
}

pub fn score_text_world_pos_for_side(
    text: &str,
    font_size: f32,
    side: BoardSide,
    board_size: IVec2,
) -> Vec3 {
    let board_left = board_anchor(side, board_size).x - TILE_SIZE / 2.0;
    let score_y = board_top_edge(board_size) + SCORE_Y_OFFSET;
    let half_width = text.len() as f32 * font_size * 0.25;
    Vec3::new(board_left + half_width, score_y, 0.0)
}

pub fn score_text_world_pos(text: &str, font_size: f32, board_size: IVec2) -> Vec3 {
    score_text_world_pos_for_side(text, font_size, BoardSide::Single, board_size)
}

pub fn stash_left_x(board_size: IVec2) -> f32 {
    board_right_edge(BoardSide::Single, board_size) + STASH_GAP
}

pub fn stash_y_below_board(max_y_offset: i32, board_size: IVec2) -> f32 {
    let bottom_cell_center_y = board_anchor(BoardSide::Single, board_size).y;
    let board_bottom = bottom_cell_center_y - TILE_SIZE / 2.0;
    let gap = TILE_SIZE * 1.0;
    board_bottom - gap - max_y_offset as f32 * TILE_SIZE - TILE_SIZE / 2.0
}
