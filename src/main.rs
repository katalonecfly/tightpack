mod components;
mod config;
mod helpers;
mod resources;
mod systems;
mod puzzles;

use bevy::picking::prelude::*;
use bevy::prelude::*;
use crate::puzzles::{CurrentPuzzle, PuzzleBoardInfo, PuzzleGameState, SelectedSolution};
use crate::resources::{GameState, DuelState, TooltipState, PieceLibrary, GameSettings, TempSettings};
use systems::menu;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    Menu,
    Sandbox,
    Draft,
    Duel,
    Settings,
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
            AppState::Menu => {}
            _ => next_state.set(AppState::Menu),
        }
    }
}

fn load_effects_descriptions(mut commands: Commands) {
    let file_content =
        std::fs::read_to_string("assets/effects.ron").expect("Missing assets/effects.ron");
    let descs: config::EffectDescriptions =
        ron::from_str(&file_content).expect("Failed to parse effects.ron");
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
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
            )
                .run_if(in_state(AppState::Draft)),
        )
        .add_systems(
            OnExit(AppState::Draft),
            (cleanup_system, reset_game_state, reset_tooltip_state),
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
            )
                .run_if(in_state(AppState::Duel)),
        )
        .add_systems(
            OnExit(AppState::Duel),
            (cleanup_system, reset_duel_state, reset_tooltip_state),
        )
        // PuzzlesList state (no update system, just observers)
        .add_systems(OnEnter(AppState::PuzzlesList), puzzles::setup_puzzle_list)
        .add_systems(OnExit(AppState::PuzzlesList), cleanup_system)
        // Puzzle state
        .add_systems(OnEnter(AppState::Puzzle), puzzles::setup_puzzle)
        .add_systems(
            Update,
            (
                puzzles::update_puzzle_score_ui,
                puzzles::update_puzzle_stash_labels,
                puzzles::update_puzzle_effect_previews,
                puzzles::update_puzzle_tooltip,
                puzzles::handle_puzzle_rotation,
                puzzles::recalculate_puzzle_score_system,
                puzzles::update_puzzle_contributions_system,
            )
                .run_if(in_state(AppState::Puzzle)),
        )
        .add_systems(OnExit(AppState::Puzzle), (cleanup_system, puzzles::reset_puzzle_state))
        // Solution list state
        .add_systems(OnEnter(AppState::SolutionList), puzzles::setup_solution_list)
        .add_systems(
            Update,
            puzzles::solution_list_interaction.run_if(in_state(AppState::SolutionList)),
        )
        .add_systems(OnExit(AppState::SolutionList), cleanup_system)
        // Solution view state
        .add_systems(OnEnter(AppState::SolutionView), puzzles::setup_solution_view)
        .add_systems(OnExit(AppState::SolutionView), (cleanup_system, puzzles::reset_solution_view))
        // Global escape handler
        .add_systems(Update, handle_escape)
        // Settings
        .add_systems(
            OnEnter(AppState::Settings),
            systems::settings::setup_settings,
        )
        .add_systems(OnExit(AppState::Settings), (cleanup_system, reset_temp_settings))
        .run();
}