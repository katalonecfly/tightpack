// systems/draft.rs
use bevy::prelude::*;
use std::collections::HashMap;
use rand::RngExt;      // provides random_range()
use crate::config::RawPieceConfig;     // for the vector of references
use crate::components::*;
use crate::helpers::*;
use crate::resources::PieceLibrary;
use crate::Cleanup;

#[derive(Component)]
pub struct DraftConfirmButton;

pub fn refresh_draft_stash(commands: &mut Commands, library: &PieceLibrary) {
    let mut color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ]
    .into();

    let mut rng = rand::rng();
    let all_pieces = &library.0;

    // Pick 3 distinct pieces
    let mut available: Vec<&RawPieceConfig> = all_pieces.iter().collect();
    let mut chosen = Vec::with_capacity(3);
    for _ in 0..3 {
        let idx = rng.random_range(0..available.len());
        chosen.push(available.remove(idx));
    }

    let start_x = -((chosen.len() as f32 - 1.0) * TILE_SIZE * 1.5) / 2.0;
    let stash_y = -TILE_SIZE;

    for (i, raw) in chosen.iter().enumerate() {
        let type_id = all_pieces
            .iter()
            .position(|p| std::ptr::eq(p, *raw))
            .unwrap_or(i);
        let color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        let effects = crate::systems::setup::bake_effects(raw, &color_map);
        let pos = Vec3::new(start_x + i as f32 * TILE_SIZE * 1.5, stash_y, 1.0);

        // Label
        commands.spawn((
            Text2d::new("x1"),
            TextFont { font_size: 20.0, ..default() },
            Transform::from_translation(pos + Vec3::new(0.0, TILE_SIZE / 2.0 + 10.0, 2.0)),
            StashLabel(type_id),
            DraftPiece,
            Cleanup,
        ));

        crate::systems::setup::spawn_draggable_piece(
            commands,
            type_id,
            raw.shape.clone(),
            color,
            raw.points,
            effects,
            pos,
            true,
        );
    }
}

pub fn generate_draft_stash(mut commands: Commands, library: Res<PieceLibrary>) {
    refresh_draft_stash(&mut commands, &library);
}

pub fn confirm_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (With<DraftConfirmButton>, Changed<Interaction>)>,
    draft_entities: Query<Entity, With<DraftPiece>>,
    library: Res<PieceLibrary>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            for entity in &draft_entities {
                commands.entity(entity).despawn();
            }
            refresh_draft_stash(&mut commands, &library);
        }
    }
}