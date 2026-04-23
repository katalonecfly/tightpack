use bevy::prelude::*;
use crate::components::*;
use crate::resources::GameState;
use crate::systems::scoring::check_condition;

pub fn update_stash_labels(mut label_query: Query<(&mut Text2d, &StashLabel)>, piece_query: Query<(&Piece, &Transform)>) {
    for (mut text, label) in &mut label_query {
        let count = piece_query.iter().filter(|(p, t)| p.type_id == label.0 && p.placed_at.is_none() && t.translation.z < 5.0).count();
        text.0 = format!("x{}", count);
    }
}

pub fn update_score_ui(state: Res<GameState>, mut query: Query<&mut Text, With<ScoreText>>) {
    if state.is_changed() {
        for mut text in &mut query { text.0 = format!("Score: {}", state.score); }
    }
}

pub fn update_effect_previews(state: Res<GameState>, piece_query: Query<(&Piece, &Children, Has<Hovered>)>, mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>) {
    for (piece, children, is_hovered) in &piece_query {
        for &child in children {
            if let Ok((mut visibility, mut sprite, preview)) = preview_query.get_mut(child) {
                if is_hovered {
                    *visibility = Visibility::Visible;
                    let mut active = false;
                    if let Some(grid_pos) = piece.placed_at {
                        active = check_condition(&preview.condition, Some(grid_pos + preview.offset), &state);
                    }
                    if active {
                        sprite.color = Color::srgb(1.0, 1.0, 0.0).into();
                        sprite.custom_size = Some(Vec2::splat(12.0));
                    } else {
                        sprite.color = Color::srgba(1.0, 1.0, 0.0, 0.4).into();
                        sprite.custom_size = Some(Vec2::splat(8.0));
                    }
                } else { *visibility = Visibility::Hidden; }
            }
        }
    }
}