use crate::Cleanup;
use crate::components::*;
use crate::config::RawPieceConfig;
use crate::helpers::*;
use crate::resources::{PieceLibrary, DuelState};
use crate::systems::draft::DraftConfirmButton;
use bevy::picking::prelude::{Click, Pointer, Pickable};
use bevy::prelude::*;
use rand::RngExt;
use std::collections::HashMap;

pub fn generate_duel_stash(commands: &mut Commands, library: &PieceLibrary) {
    let color_map: HashMap<String, LinearRgba> = [
        ("RED".to_string(), Color::srgb_u8(216, 46, 63).to_linear()),
        ("BLUE".to_string(), Color::srgb_u8(53, 129, 216).to_linear()),
        ("GREEN".to_string(), Color::srgb_u8(40, 204, 45).to_linear()),
        ("YELLOW".to_string(), Color::srgb_u8(255, 225, 53).to_linear()),
    ].into();

    let all_pieces = &library.0;
    let mut rng = rand::rng();
    let mut available: Vec<&RawPieceConfig> = all_pieces.iter().collect();
    let mut chosen = Vec::with_capacity(3);
    for _ in 0..3 {
        if available.is_empty() {
            break;
        }
        let idx = rng.random_range(0..available.len());
        chosen.push(available.remove(idx));
    }

    spawn_side_pieces(commands, &chosen, &color_map, BoardSide::Left, true);
    spawn_side_pieces(commands, &chosen, &color_map, BoardSide::Right, false);
}

fn spawn_side_pieces(
    commands: &mut Commands,
    raw_pieces: &[&RawPieceConfig],
    color_map: &HashMap<String, LinearRgba>,
    side: BoardSide,
    interactive: bool,
) {
    let board_left = grid_to_world_for_side(IVec2::ZERO, side).x;
    let mut next_left = board_left;

    for (i, raw) in raw_pieces.iter().enumerate() {
        let color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        let effects = crate::systems::setup::bake_effects(raw, color_map);
        let type_id = i;

        let min_x = raw.shape.iter().map(|o| o.x).min().unwrap_or(0);
        let max_x = raw.shape.iter().map(|o| o.x).max().unwrap_or(0);
        let max_y = raw.shape.iter().map(|o| o.y).max().unwrap_or(0);
        let width = (max_x - min_x + 1) as f32 * TILE_SIZE;

        let piece_left = next_left;
        let parent_x = piece_left - (min_x as f32) * TILE_SIZE;
        // dynamic y below board
        let parent_y = stash_y_below_board(max_y);
        let pos = Vec3::new(parent_x, parent_y, 1.0);

        let entity = crate::systems::setup::spawn_draggable_piece(
            commands,
            type_id,
            raw.shape.clone(),
            color,
            raw.points,
            effects,
            pos,
            false,   // draft_mode
            interactive,
            side,
        );

        commands.entity(entity).insert(DraftPiece);

        match side {
            BoardSide::Left => {
                commands.entity(entity).insert(PlayerPiece);
            }
            BoardSide::Right => {
                commands.entity(entity).insert(OpponentPiece);
            }
            _ => unreachable!(),
        }

        let label_y = parent_y + (max_y as f32) * TILE_SIZE + TILE_SIZE / 2.0 + 10.0;
        let label_entity = commands.spawn((
            Text2d::new("x1"),
            TextFont {
                font_size: STASH_LABEL_FONT_SIZE,
                ..default()
            },
            Transform::from_translation(Vec3::new(parent_x, label_y, 2.0)),
            StashLabel(type_id),
            Cleanup,
        ))
        .id();
        match side {
            BoardSide::Left => { commands.entity(label_entity).insert(PlayerPiece); }
            BoardSide::Right => { commands.entity(label_entity).insert(OpponentPiece); }
            _ => {}
        }

        next_left = piece_left + width + TILE_SIZE;
    }
}

pub fn on_confirm_click_duel(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    player_drafts: Query<Entity, (With<DraftPiece>, With<PlayerPiece>)>,
    opponent_drafts: Query<Entity, (With<DraftPiece>, With<OpponentPiece>)>,
    library: Res<PieceLibrary>,
    mut duel_state: ResMut<DuelState>,
    player_pieces: Query<&Piece, With<PlayerPiece>>,
    opponent_pieces: Query<&Piece, With<OpponentPiece>>,
    opponent_draft_pieces: Query<(Entity, &Piece), (With<DraftPiece>, With<OpponentPiece>)>,
    player_labels: Query<Entity, (With<StashLabel>, With<PlayerPiece>)>,
    opponent_labels: Query<Entity, (With<StashLabel>, With<OpponentPiece>)>,
) {
    info!("Confirm clicked. Opponent draft count: {}", opponent_draft_pieces.iter().count());
    // Process player side
    for entity in &player_drafts {
        if let Ok(piece) = player_pieces.get(entity) {
            if piece.placed_at.is_some() {
                commands
                    .entity(entity)
                    .remove::<DraftPiece>()
                    .insert(LockedPiece);
            } else {
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
    for label in &player_labels {
        commands.entity(label).despawn();
    }

    // AI placement
    // Collect opponent draft pieces data for AI, then despawn them all
    let draft_data: Vec<(Entity, Piece)> = opponent_draft_pieces
        .iter()
        .map(|(e, p)| (e, p.clone()))
        .collect();
    for entity in &opponent_drafts {
        commands.entity(entity).despawn();
    }
    for label in &opponent_labels {
        commands.entity(label).despawn();
    }

    // AI placement
    if let Some(placement) = crate::systems::ai::first_free_placement(
        &draft_data.iter().map(|(e, p)| (*e, p)).collect::<Vec<_>>(),
        &duel_state.opponent,
    ) {
        // Spawn a new locked piece on opponent's board
        let world_pos = grid_to_world_for_side(placement.origin, BoardSide::Right);
        let entity = crate::systems::setup::spawn_draggable_piece(
            &mut commands,
            0, // type_id not important for opponent
            placement.shape.clone(),
            placement.color,
            placement.raw_config.points,
            placement.effects.clone(),
            world_pos.with_z(1.0),
            false, // draft_mode
            false, // interactive
            BoardSide::Right,
        );
        commands.entity(entity).insert(LockedPiece);
        commands.entity(entity).insert(OpponentPiece);
        // Insert a fresh Piece component with placed_at set
        let placed_piece = Piece {
            type_id: 0,
            shape: placement.shape.clone(),
            original_shape: placement.shape.clone(),
            color: placement.color,
            points: placement.raw_config.points,
            effects: placement.effects.clone(),
            original_effects: placement.effects.clone(),
            original_pos: world_pos.with_z(1.0),
            placed_at: Some(placement.origin),
            board_side: BoardSide::Right,
        };
        commands.entity(entity).insert(placed_piece);
        // Update opponent board cells
        for offset in &placement.shape {
            duel_state.opponent.board_cells.insert(
                placement.origin + *offset,
                placement.color,
            );
        }
    }

    // Update scores
    duel_state.player.score =
        crate::systems::scoring::recalculate_score(&duel_state.player.board_cells, &player_pieces);
    duel_state.opponent.score =
        crate::systems::scoring::recalculate_score(&duel_state.opponent.board_cells, &opponent_pieces);

    // Generate new stash
    generate_duel_stash(&mut commands, &library);
}

pub fn setup_duel(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    let file_content = std::fs::read_to_string("assets/pieces.ron").expect("Missing pieces.ron");
    let lib: crate::config::RawPieceLibrary = ron::from_str(&file_content).expect("Failed to parse RON");
    let pieces = lib.pieces.clone();
    commands.insert_resource(PieceLibrary(pieces.clone()));

    spawn_board(&mut commands, BoardSide::Left);
    spawn_board(&mut commands, BoardSide::Right);

    commands.spawn((
        Text2d::new("Player: 0"),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos_for_side(
            "Player: 0",
            SCORE_FONT_SIZE,
            BoardSide::Left,
        )),
        ScoreText,
        PlayerScoreText,
        Cleanup,
    ));
    commands.spawn((
        Text2d::new("Opponent: 0"),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(score_text_world_pos_for_side(
            "Opponent: 0",
            SCORE_FONT_SIZE,
            BoardSide::Right,
        )),
        ScoreText,
        OpponentScoreText,
        Cleanup,
    ));

    spawn_confirm_button(&mut commands, BoardSide::Left);

    commands.insert_resource(DuelState::default());
    generate_duel_stash(&mut commands, &PieceLibrary(pieces));
}

fn spawn_board(commands: &mut Commands, side: BoardSide) {
    let board_root = commands.spawn((Transform::default(), Cleanup)).id();
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            let tile = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                    Transform::from_translation(grid_to_world_for_side(IVec2::new(x, y), side)),
                ))
                .id();
            commands.entity(board_root).add_child(tile);
        }
    }
}

fn spawn_confirm_button(commands: &mut Commands, side: BoardSide) {
    let board_right = board_right_edge(side);
    let board_top = board_top_edge();
    let score_y = board_top + SCORE_Y_OFFSET;
    let button_pos = Vec3::new(
        board_right - CONFIRM_BUTTON_WIDTH / 2.0,
        score_y,
        0.0,
    );
    commands
        .spawn((
            Sprite::from_color(
                Color::srgb(0.3, 0.8, 0.3),
                Vec2::new(CONFIRM_BUTTON_WIDTH, CONFIRM_BUTTON_HEIGHT),
            ),
            Transform::from_translation(button_pos),
            Pickable::default(),
            DraftConfirmButton,
            Cleanup,
        ))
        .with_child((
            Text2d::new("Confirm"),
            TextFont {
                font_size: CONFIRM_BUTTON_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::default(),
        ))
        .observe(on_confirm_click_duel);
}