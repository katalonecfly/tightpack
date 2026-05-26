use crate::components::{Dragging, Piece, StashPosition};
use crate::helpers::STASH_SCROLL_SPEED;
use crate::resources::{InventoryScroll, StashContentHeight, StashScreenRect};
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn scroll_inventory(
    mut scroll: ResMut<InventoryScroll>,
    mut reader: MessageReader<MouseWheel>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    screen_rect: Res<StashScreenRect>,
    content_height: Res<StashContentHeight>,
) {
    let window = window_query.single().expect("Primary window missing");
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    if cursor_pos.x >= screen_rect.x
        && cursor_pos.x <= screen_rect.x + screen_rect.width
        && cursor_pos.y >= screen_rect.y
        && cursor_pos.y <= screen_rect.y + screen_rect.height
    {
        for event in reader.read() {
            scroll.offset -= event.y * STASH_SCROLL_SPEED;
            let max_scroll = (content_height.0 - screen_rect.height).max(0.0);
            scroll.offset = scroll.offset.clamp(0.0, max_scroll);
        }
    }
}

pub fn apply_inventory_scroll(
    scroll: Res<InventoryScroll>,
    mut piece_query: Query<(&StashPosition, &mut Piece, &mut Transform), Without<Dragging>>,
    mut label_query: Query<(&StashPosition, &mut Transform), (Without<Piece>, Without<Dragging>)>,
) {
    for (stash_pos, mut piece, mut transform) in &mut piece_query {
        if piece.placed_at.is_none() {
            let new_y = stash_pos.desired_world_y + scroll.offset;
            transform.translation.y = new_y;
            piece.original_pos.y = new_y;
        }
    }
    for (stash_pos, mut transform) in &mut label_query {
        transform.translation.y = stash_pos.desired_world_y + scroll.offset;
    }
}
