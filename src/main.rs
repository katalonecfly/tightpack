mod components;
mod config;
mod helpers;
mod resources;
mod systems;
mod puzzles;
mod puzzle_ui;

use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use crate::resources::{GameState, DuelState, TooltipState, PieceLibrary, GameSettings, TempSettings, RoundCounter};
use crate::systems::menu;
use crate::puzzles::*;
use crate::puzzle_ui::*;
use crate::components::Piece;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    Menu,
    Sandbox,
    Draft,
    Duel,
    Settings,
    Controls,
    PuzzlesList,
    Puzzle,
    SolutionList,
    SolutionView,
}

#[derive(Component)]
struct Cleanup;

fn cleanup_system(mut commands: Commands, query: Query<Entity, With<Cleanup>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_escape(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match *current_state.get() {
            AppState::Puzzle => next_state.set(AppState::PuzzlesList),
            AppState::SolutionView => next_state.set(AppState::SolutionList),
            AppState::SolutionList => next_state.set(AppState::PuzzlesList),
            AppState::PuzzlesList => next_state.set(AppState::Menu),
            AppState::Controls => next_state.set(AppState::Menu),
            AppState::Menu => {}
            _ => next_state.set(AppState::Menu),
        }
    }
}

fn load_effects_descriptions(mut commands: Commands) {
    let file_content = include_str!("../assets/effects.ron");
    let descs: config::EffectDescriptions =
        ron::from_str(file_content).expect("Failed to parse effects.ron");
    commands.insert_resource(descs);
}

fn reset_game_state(mut state: ResMut<GameState>) {
    *state = GameState::default();
}

fn reset_duel_state(mut duel_state: ResMut<DuelState>) {
    *duel_state = DuelState::default();
}

fn reset_tooltip_state(mut tooltip: ResMut<TooltipState>) {
    *tooltip = TooltipState::default();
}

fn reset_temp_settings(mut commands: Commands) {
    commands.remove_resource::<TempSettings>();
}

fn reset_round_counter(mut commands: Commands) {
    commands.remove_resource::<RoundCounter>();
}

fn handle_reset(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    puzzle_state: Option<ResMut<PuzzleGameState>>,
    _duel_state: Option<ResMut<DuelState>>,
    _round_counter: Option<ResMut<RoundCounter>>,
    mut piece_query: Query<(&mut Piece, &mut Transform)>,
    _library: Res<PieceLibrary>,
    _settings: Res<GameSettings>,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyN) {
        match *current_state.get() {
            AppState::Sandbox => {
                game_state.board_cells.clear();
                game_state.score = 0;
                for (mut piece, mut transform) in piece_query.iter_mut() {
                    if piece.placed_at.is_some() {
                        piece.placed_at = None;
                        transform.translation = piece.original_pos;
                        transform.translation.z = piece.original_pos.z;
                        transform.rotation = Quat::IDENTITY;
                        piece.shape = piece.original_shape.clone();
                        piece.effects = piece.original_effects.clone();
                    }
                }
            }
            AppState::Draft => {
                next_state.set(AppState::Draft);
            }
            AppState::Duel => {
                next_state.set(AppState::Duel);
            }
            AppState::Puzzle => {
                if let Some(mut puzzle_state) = puzzle_state {
                    puzzle_state.board_cells.clear();
                    puzzle_state.score = 0;
                    for (mut piece, mut transform) in piece_query.iter_mut() {
                        if piece.placed_at.is_some() {
                            piece.placed_at = None;
                            transform.translation = piece.original_pos;
                            transform.translation.z = piece.original_pos.z;
                            transform.rotation = Quat::IDENTITY;
                            piece.shape = piece.original_shape.clone();
                            piece.effects = piece.original_effects.clone();
                        }
                    }
                } else {
                    next_state.set(AppState::Puzzle);
                }
            }
            _ => {}
        }
    }
}

fn build_app(window_plugin: WindowPlugin) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(window_plugin))
        .add_plugins(MeshPickingPlugin)
        .init_resource::<GameState>()
        .init_resource::<TooltipState>()
        .init_resource::<PieceLibrary>()
        .insert_resource(GameSettings::default())
        .init_state::<AppState>()
        .add_systems(Startup, load_effects_descriptions)
        // Menu
        .add_systems(OnEnter(AppState::Menu), menu::setup_menu)
        .add_systems(
            Update,
            menu::menu_interaction.run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnExit(AppState::Menu), cleanup_system)
        // Sandbox
        .add_systems(OnEnter(AppState::Sandbox), systems::setup::setup_sandbox)
        .add_systems(
            Update,
            (
                systems::ui::update_score_ui,
                systems::ui::update_stash_labels,
                systems::ui::update_effect_previews,
                systems::ui::update_tooltip,
                systems::interaction::handle_rotation,
                systems::scoring::recalculate_score_system,
                systems::ui::update_contributions_system,
                systems::inventory::scroll_inventory,
                systems::inventory::apply_inventory_scroll,
                handle_reset,
            )
                .run_if(in_state(AppState::Sandbox)),
        )
        .add_systems(
            OnExit(AppState::Sandbox),
            (cleanup_system, reset_game_state, reset_tooltip_state),
        )
        // Draft
        .add_systems(
            OnEnter(AppState::Draft),
            (
                systems::setup::setup_draft,
                systems::draft::generate_draft_stash,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                systems::ui::update_score_ui,
                systems::ui::update_stash_labels,
                systems::ui::update_effect_previews,
                systems::ui::update_tooltip,
                systems::interaction::handle_rotation,
                systems::scoring::recalculate_score_system,
                systems::ui::update_contributions_system,
                systems::draft::update_draft_round_display,
                handle_reset,
            )
                .run_if(in_state(AppState::Draft)),
        )
        .add_systems(
            OnExit(AppState::Draft),
            (cleanup_system, reset_game_state, reset_tooltip_state, reset_round_counter),
        )
        // Duel
        .add_systems(OnEnter(AppState::Duel), systems::duel::setup_duel)
        .add_systems(
            Update,
            (
                systems::ui::update_duel_score_ui,
                systems::ui::update_stash_labels,
                systems::ui::update_duel_effect_previews,
                systems::ui::update_duel_tooltip,
                systems::interaction::handle_rotation,
                systems::scoring::recalculate_duel_score_system,
                systems::ui::update_duel_contributions_system,
                systems::duel::handle_destroy_input,
                systems::duel::update_duel_round_display,
                handle_reset,
            )
                .run_if(in_state(AppState::Duel)),
        )
        .add_systems(
            OnExit(AppState::Duel),
            (cleanup_system, reset_duel_state, reset_tooltip_state, reset_round_counter),
        )
        // Controls
        .add_systems(OnEnter(AppState::Controls), systems::controls::setup_controls)
        .add_systems(OnExit(AppState::Controls), cleanup_system)
        // PuzzlesList
        .add_systems(OnEnter(AppState::PuzzlesList), setup_puzzle_list)
        .add_systems(OnExit(AppState::PuzzlesList), cleanup_system)
        // Puzzle
        .add_systems(OnEnter(AppState::Puzzle), setup_puzzle)
        .add_systems(
            Update,
            (
                update_puzzle_score_ui,
                update_puzzle_stash_labels,
                update_puzzle_effect_previews,
                update_puzzle_tooltip,
                handle_puzzle_rotation,
                recalculate_puzzle_score_system,
                update_puzzle_contributions_system,
                systems::inventory::scroll_inventory,
                systems::inventory::apply_inventory_scroll,
                handle_reset,
            )
                .run_if(in_state(AppState::Puzzle)),
        )
        .add_systems(
            Update,
            update_help_tooltip.run_if(in_state(AppState::PuzzlesList)),
        )
        .add_systems(OnExit(AppState::Puzzle), (cleanup_system, reset_puzzle_state))
        // Solution list
        .add_systems(OnEnter(AppState::SolutionList), setup_solution_list)
        .add_systems(
            Update,
            solution_list_interaction.run_if(in_state(AppState::SolutionList)),
        )
        .add_systems(OnExit(AppState::SolutionList), cleanup_system)
        // Solution view
        .add_systems(OnEnter(AppState::SolutionView), setup_solution_view)
        .add_systems(
            Update,
            (
                update_puzzle_tooltip,
                update_puzzle_effect_previews,
                update_puzzle_contributions_system,
            )
                .run_if(in_state(AppState::SolutionView)),
        )        
        .add_systems(OnExit(AppState::SolutionView), (cleanup_system, reset_solution_view))
        // Global escape
        .add_systems(Update, handle_escape)
        // Settings
        .add_systems(
            OnEnter(AppState::Settings),
            systems::settings::setup_settings,
        )
        .add_systems(
            Update,
            systems::settings::handle_rounds_buttons.run_if(in_state(AppState::Settings)),
        )
        .add_systems(OnExit(AppState::Settings), (cleanup_system, reset_temp_settings));
    
    app
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    };
    build_app(window_plugin).run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let window_plugin = WindowPlugin::default();
    build_app(window_plugin).run();
}