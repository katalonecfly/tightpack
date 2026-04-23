use bevy::picking::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// --- Constants & Resources ---
const TILE_SIZE: f32 = 40.0;
const BOARD_SIZE: IVec2 = IVec2::new(8, 8);
const BOARD_OFFSET: Vec3 = Vec3::new(-200.0, 0.0, 0.0);
const INVENTORY_OFFSET: Vec3 = Vec3::new(200.0, 0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .init_resource::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_score_ui, update_stash_labels, update_effect_previews, handle_rotation))
        .run();
}

// --- RON Parsing Structs (The "Raw" Data) ---

#[derive(Deserialize)]
struct RawPieceLibrary {
    pieces: Vec<RawPieceConfig>,
}

#[derive(Deserialize)]
struct RawPieceConfig {
    shape: Vec<IVec2>,
    color: String,
    points: i32,
    effects: Vec<RawGameEffect>,
}

#[derive(Deserialize)]
struct RawGameEffect {
    condition: RawEffectCondition,
    points: i32,
    #[serde(default)] // If missing in RON, this becomes an empty Vec
    offsets: Vec<IVec2>, 
}

#[derive(Deserialize)]
enum RawEffectCondition {
    MatchesColor(String),
    IsEmpty,
    NoColorOnBoard(String),
}

// --- Components & Resources ---

#[derive(Resource, Default)]
struct GameState {
    board_cells: HashMap<IVec2, LinearRgba>,
    score: i32,
}

#[derive(Component, Clone)]
struct Piece {
    type_id: usize,
    shape: Vec<IVec2>,
    original_shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32, // Base points
    effects: Vec<GameEffect>,
    original_effects: Vec<GameEffect>,
    original_pos: Vec3,
    placed_at: Option<IVec2>,
}

#[derive(Clone)]
struct GameEffect {
    condition: EffectCondition,
    points: i32,
    offsets: Option<Vec<IVec2>>,
}

#[derive(Clone, PartialEq)]
enum EffectCondition {
    MatchesColor(LinearRgba),
    IsEmpty,
    NoColorOnBoard(LinearRgba),
}

#[derive(Component)]
struct EffectPreview {
    offset: IVec2,
    condition: EffectCondition,
}

#[derive(Component)]
struct Hovered;

#[derive(Component)]
struct StashLabel(usize);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Dragging;

// --- Setup ---

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 1. Central Color Registry
    let mut color_map = HashMap::new();
    color_map.insert("RED".to_string(), LinearRgba::RED);
    color_map.insert("BLUE".to_string(), LinearRgba::BLUE);
    color_map.insert("GREEN".to_string(), LinearRgba::GREEN);
    color_map.insert("YELLOW".to_string(), LinearRgba::new(1.0, 1.0, 0.0, 1.0));    // 2. Load and Parse RON
    let file_content = std::fs::read_to_string("assets/pieces.ron")
        .expect("Missing assets/pieces.ron");
    let lib: RawPieceLibrary = ron::from_str(&file_content)
        .expect("Failed to parse RON");

    // 3. Setup Board (Visuals)
    for x in 0..BOARD_SIZE.x {
        for y in 0..BOARD_SIZE.y {
            commands.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_translation(grid_to_world(IVec2::new(x, y))),
            ));
        }
    }

    // 4. Bake and Spawn Pieces
    for (type_id, raw) in lib.pieces.into_iter().enumerate() {
        let piece_color = *color_map.get(&raw.color).unwrap_or(&LinearRgba::WHITE);
        
        // Convert raw effects (strings) to real effects (RGBA)
        let baked_effects: Vec<GameEffect> = raw.effects.into_iter().map(|re| {
            let condition = match re.condition {
                RawEffectCondition::IsEmpty => EffectCondition::IsEmpty,
                RawEffectCondition::MatchesColor(name) => 
                    EffectCondition::MatchesColor(*color_map.get(&name).unwrap_or(&LinearRgba::WHITE)),
                RawEffectCondition::NoColorOnBoard(name) => 
                    EffectCondition::NoColorOnBoard(*color_map.get(&name).unwrap_or(&LinearRgba::WHITE)),
            };

            // Convert the Vec to Option here: 
            // Empty Vec becomes None (Self-effect), non-empty becomes Some (Target-effect)
            let offsets = if re.offsets.is_empty() { None } else { Some(re.offsets) };

            GameEffect { 
                condition, 
                points: re.points, 
                offsets 
            }
        }).collect();

        let pos = INVENTORY_OFFSET + Vec3::new(0.0, 150.0 - (type_id as f32 * 100.0), 1.0);
        let count = 10; // Hardcoded as requested

        // Spawn Label
        commands.spawn((
            Text2d::new(format!("x{}", count)),
            TextFont { font_size: 24.0, ..default() },
            Transform::from_translation(pos + Vec3::new(-45.0, 35.0, 2.0)),
            StashLabel(type_id),
        ));

        // Spawn instances
        for _ in 0..count {
            spawn_draggable_piece(
                &mut commands,
                type_id,
                raw.shape.clone(),
                piece_color,
                raw.points,
                baked_effects.clone(),
                pos,
            );
        }
    }

    commands.spawn((
        Text::new("Score: 0"),
        TextFont { font_size: 40.0, ..default() },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

fn spawn_draggable_piece(
    commands: &mut Commands,
    type_id: usize,
    shape: Vec<IVec2>,
    color: LinearRgba,
    points: i32,        // Fix: Added missing 'points' argument
    effects: Vec<GameEffect>,
    pos: Vec3,
) {
    let parent = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            Pickable::default(),
            Piece {
                type_id,
                shape: shape.clone(),
                original_shape: shape.clone(),
                color,
                points,      // Fix: Added missing field to the struct initializer
                effects: effects.clone(),
                original_effects: effects.clone(),
                original_pos: pos,
                placed_at: None,
            }
        ))
        .observe(on_drag_start)
        .observe(on_drag)
        .observe(on_drag_end)
        .observe(on_hover_in)
        .observe(on_hover_out)
        .id();

    for offset in shape {
        let child = commands.spawn((
            Sprite::from_color(color, Vec2::splat(TILE_SIZE - 4.0)),
            Transform::from_translation(offset.as_vec2().extend(0.0) * TILE_SIZE),
            Pickable::default(),
        )).id();
        commands.entity(parent).add_child(child);
    }

    for effect in effects {
        if let Some(offsets) = effect.offsets {
            for offset in offsets {
                let preview = commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 1.0, 0.0).into(),
                        custom_size: Some(Vec2::splat(12.0)),
                        ..default()
                    },
                    Transform::from_translation(offset.as_vec2().extend(5.0) * TILE_SIZE),
                    Visibility::Hidden,
                    EffectPreview {
                        offset,
                        condition: effect.condition.clone(),
                    },
                )).id();
                commands.entity(parent).add_child(preview);
            }
        }
    }
}

// --- Interaction ---

fn on_drag_start(
    on: On<Pointer<DragStart>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Piece, &Children)>,
    mut state: ResMut<GameState>,
) {
    if let Ok((mut transform, mut piece, _)) = query.get_mut(on.event_target()) {
        commands.entity(on.event_target()).insert(Dragging);
        
        transform.translation.z = 10.0;
        if let Some(old_pos) = piece.placed_at {
            for offset in &piece.shape {
                state.board_cells.remove(&(old_pos + *offset));
            }
            piece.placed_at = None;
            recalculate_score(&mut state, &query);
        }
    }
}

fn on_drag(on: On<Pointer<Drag>>, mut query: Query<&mut Transform, With<Piece>>) {
    if let Ok(mut transform) = query.get_mut(on.event_target()) {
        transform.translation.x += on.delta.x;
        transform.translation.y -= on.delta.y;
    }
}

fn on_drag_end(
    on: On<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Piece, &Children)>,
    mut preview_query: Query<&mut EffectPreview>,
    mut state: ResMut<GameState>,
) {
    let target = on.event_target();
    commands.entity(target).remove::<Dragging>();

    let Ok((mut transform, mut piece, children)) = query.get_mut(target) else { return };
    let grid_pos = world_to_grid(transform.translation);

    let mut can_place = true;
    for offset in &piece.shape {
        let cell = grid_pos + *offset;
        if cell.x < 0 || cell.x >= BOARD_SIZE.x || cell.y < 0 || cell.y >= BOARD_SIZE.y 
           || state.board_cells.contains_key(&cell) {
            can_place = false;
            break;
        }
    }

    if can_place {
        transform.translation = grid_to_world(grid_pos).with_z(1.0);
        piece.placed_at = Some(grid_pos);
        for offset in &piece.shape {
            state.board_cells.insert(grid_pos + *offset, piece.color);
        }
    } else {
        transform.translation = piece.original_pos;
        transform.translation.z = 1.0;
        transform.rotation = Quat::IDENTITY;
        
        piece.shape = piece.original_shape.clone();
        piece.effects = piece.original_effects.clone();

        let mut effect_idx = 0;
        for effect in &piece.original_effects {
            if let Some(offsets) = &effect.offsets {
                for &orig_offset in offsets {
                    if let Some(&child) = children.get(piece.shape.len() + effect_idx) {
                         if let Ok(mut preview) = preview_query.get_mut(child) {
                             preview.offset = orig_offset;
                         }
                    }
                    effect_idx += 1;
                }
            }
        }
    }
    recalculate_score(&mut state, &query);
}

fn on_hover_in(on: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(on.event_target()).insert(Hovered);
}

fn on_hover_out(on: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(on.event_target()).remove::<Hovered>();
}

// --- Systems ---

fn update_stash_labels(
    mut label_query: Query<(&mut Text2d, &StashLabel)>,
    piece_query: Query<(&Piece, &Transform)>,
) {
    for (mut text, label) in &mut label_query {
        let count = piece_query
            .iter()
            .filter(|(p, t)| {
                p.type_id == label.0 && 
                p.placed_at.is_none() && 
                t.translation.z < 5.0
            })
            .count();
        
        text.0 = format!("x{}", count);
    }
}

fn recalculate_score(state: &mut GameState, query: &Query<(&mut Transform, &mut Piece, &Children)>) {
    let mut total = 0;
    for (_, piece, _) in query.iter() {
        if let Some(pos) = piece.placed_at {
            // Add base placement points
            total += piece.points;

            for effect in &piece.effects {
                match &effect.offsets {
                    Some(offsets) => {
                        // Check specific neighbor tiles
                        for offset in offsets {
                            if check_condition(&effect.condition, Some(pos + *offset), state) {
                                total += effect.points;
                            }
                        }
                    }
                    None => {
                        // No offsets? Check the piece's own position (or global state)
                        if check_condition(&effect.condition, Some(pos), state) {
                            total += effect.points;
                        }
                    }
                }
            }
        }
    }
    state.score = total;
}

fn check_condition(cond: &EffectCondition, target: Option<IVec2>, state: &GameState) -> bool {
    match cond {
        EffectCondition::MatchesColor(c) => target.map_or(false, |cell| state.board_cells.get(&cell) == Some(c)),
        EffectCondition::IsEmpty => target.map_or(false, |cell| !state.board_cells.contains_key(&cell)),
        EffectCondition::NoColorOnBoard(c) => !state.board_cells.values().any(|color| color == c),
    }
}

fn update_score_ui(state: Res<GameState>, mut query: Query<&mut Text, With<ScoreText>>) {
    if state.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Score: {}", state.score);
        }
    }
}

fn update_effect_previews(
    state: Res<GameState>,
    piece_query: Query<(&Piece, &Children, Has<Hovered>)>,
    mut preview_query: Query<(&mut Visibility, &mut Sprite, &EffectPreview)>,
) {
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
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn handle_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut piece_query: Query<(Entity, &mut Transform, &mut Piece, &Children), With<Dragging>>,
    mut preview_query: Query<&mut EffectPreview>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Right) {
        for (_entity, mut transform, mut piece, children) in &mut piece_query {
            transform.rotate_z(-std::f32::consts::FRAC_PI_2);

            for offset in &mut piece.shape {
                let old = *offset;
                *offset = IVec2::new(old.y, -old.x);
            }

            for effect in &mut piece.effects {
                if let Some(offsets) = &mut effect.offsets {
                    for offset in offsets {
                        let old = *offset;
                        *offset = IVec2::new(old.y, -old.x);
                    }
                }
            }

            for &child in children {
                if let Ok(mut preview) = preview_query.get_mut(child) {
                    let old = preview.offset;
                    preview.offset = IVec2::new(old.y, -old.x);
                }
            }
        }
    }
}

// --- Helpers ---

fn grid_to_world(grid: IVec2) -> Vec3 {
    BOARD_OFFSET + Vec3::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE, 0.0)
}

fn world_to_grid(world: Vec3) -> IVec2 {
    let local = world - BOARD_OFFSET;
    IVec2::new((local.x / TILE_SIZE).round() as i32, (local.y / TILE_SIZE).round() as i32)
}