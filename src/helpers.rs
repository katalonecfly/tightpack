use crate::components::BoardSide;
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 40.0;
pub const BOARD_SIZE: IVec2 = IVec2::new(8, 8);
pub const BOARD_OFFSET: Vec3 = Vec3::new(-200.0, 0.0, 0.0);
//pub const INVENTORY_OFFSET: Vec3 = Vec3::new(200.0, 0.0, 0.0);

pub fn grid_to_world_for_side(grid: IVec2, side: BoardSide) -> Vec3 {
    let offset = match side {
        BoardSide::Left => LEFT_BOARD_OFFSET,
        BoardSide::Right => RIGHT_BOARD_OFFSET,
        BoardSide::Single => BOARD_OFFSET,
    };
    offset + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

// Keep original for Single mode (existing callers)
pub fn grid_to_world(grid: IVec2) -> Vec3 {
    grid_to_world_for_side(grid, BoardSide::Single)
}

pub fn world_to_grid_for_side(world: Vec3, side: BoardSide) -> IVec2 {
    let offset = match side {
        BoardSide::Left => LEFT_BOARD_OFFSET,
        BoardSide::Right => RIGHT_BOARD_OFFSET,
        BoardSide::Single => BOARD_OFFSET,
    };
    let local = world - offset;
    IVec2::new(
        (local.x / TILE_SIZE).round() as i32,
        (local.y / TILE_SIZE).round() as i32,
    )
}

// Keep original for Single mode
pub fn world_to_grid(world: Vec3) -> IVec2 {
    world_to_grid_for_side(world, BoardSide::Single)
}
pub fn is_in_bounds(grid: IVec2) -> bool {
    grid.x >= 0 && grid.x < BOARD_SIZE.x && grid.y >= 0 && grid.y < BOARD_SIZE.y
}

pub const SCORE_FONT_SIZE: f32 = 30.0;
pub const SCORE_Y_OFFSET: f32 = 30.0;
pub const STASH_LABEL_FONT_SIZE: f32 = 24.0;
pub const CONFIRM_BUTTON_WIDTH: f32 = 120.0;
pub const CONFIRM_BUTTON_HEIGHT: f32 = 50.0;
pub const CONFIRM_BUTTON_FONT_SIZE: f32 = 28.0;

pub fn score_text_world_pos(text: &str, font_size: f32) -> Vec3 {
    score_text_world_pos_for_side(text, font_size, BoardSide::Single)
}

pub fn score_text_world_pos_for_side(text: &str, font_size: f32, side: BoardSide) -> Vec3 {
    let board_offset = match side {
        BoardSide::Left => LEFT_BOARD_OFFSET,
        BoardSide::Right => RIGHT_BOARD_OFFSET,
        BoardSide::Single => BOARD_OFFSET,
    };
    let board_left = board_offset.x - TILE_SIZE / 2.0;
    let board_top = board_offset.y + (BOARD_SIZE.y - 1) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let score_y = board_top + SCORE_Y_OFFSET;
    let half_width = text.len() as f32 * font_size * 0.25;
    Vec3::new(board_left + half_width, score_y, 0.0)
}

// After the existing constants, add:
// New constants (already may be there)
//pub const STASH_ORIGIN_X: f32 = 200.0;
//pub const STASH_ORIGIN_Y: f32 = 280.0;
pub const STASH_LEFT_X: f32 = 160.0;          // world x of stash left edge
pub const STASH_WIDTH: f32 = 200.0;
pub const STASH_VISIBLE_HEIGHT: f32 = 360.0;
pub const STASH_SCROLL_SPEED: f32 = 1.0;     // slow down

pub const LEFT_BOARD_OFFSET: Vec3 = Vec3::new(-340.0, 0.0, 0.0);   // adjust as needed
pub const RIGHT_BOARD_OFFSET: Vec3 = Vec3::new(20.0, 0.0, 0.0);    // gap between boards