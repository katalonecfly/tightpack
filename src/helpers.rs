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
    IVec2::new((local.x / TILE_SIZE).round() as i32, (local.y / TILE_SIZE).round() as i32)
}