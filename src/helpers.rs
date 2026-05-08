use bevy::prelude::*;

pub const TILE_SIZE: f32 = 40.0;
pub const BOARD_SIZE: IVec2 = IVec2::new(8, 8);
pub const BOARD_OFFSET: Vec3 = Vec3::new(-200.0, 0.0, 0.0);
pub const INVENTORY_OFFSET: Vec3 = Vec3::new(200.0, 0.0, 0.0);

pub fn grid_to_world(grid: IVec2) -> Vec3 {
    BOARD_OFFSET + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

pub fn world_to_grid(world: Vec3) -> IVec2 {
    let local = world - BOARD_OFFSET;
    IVec2::new(
        (local.x / TILE_SIZE).round() as i32,
        (local.y / TILE_SIZE).round() as i32,
    )
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
    let board_left = grid_to_world(IVec2::ZERO).x - TILE_SIZE / 2.0;
    let board_top = grid_to_world(IVec2::new(0, BOARD_SIZE.y - 1)).y + TILE_SIZE / 2.0;
    let score_y = board_top + SCORE_Y_OFFSET;
    let half_width = text.len() as f32 * font_size * 0.25;
    Vec3::new(board_left + half_width, score_y, 0.0)
}

