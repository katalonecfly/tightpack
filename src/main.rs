mod components;
mod config;
mod helpers;
mod resources;
mod systems;
mod puzzles;

use bevy::picking::prelude::*;
use bevy::prelude::*;
use resources::{DuelState, GameSettings, GameState, PieceLibrary, TooltipState};
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
            AppState::PuzzlesList => next_state.set(AppState::Menu),
            AppState::Menu => {} // do nothing
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
                systems::ui::update_contributions_system, // <-- added
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
                systems::ui::update_contributions_system, // <-- added
            )
                .run_if(in_state(AppState::Draft)),
        )
        .add_systems(
            OnExit(AppState::Draft),
            (cleanup_system, reset_game_state, reset_tooltip_state),
        )
        // Duel:
        .add_systems(OnEnter(AppState::Duel), systems::duel::setup_duel)
        .add_systems(
            Update,
            (
                systems::ui::update_duel_score_ui,
                systems::ui::update_stash_labels,
                systems::ui::update_duel_effect_previews,
                systems::ui::update_duel_tooltip, // <-- changed
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
        // PuzzlesList state
        .add_systems(OnEnter(AppState::PuzzlesList), puzzles::setup_puzzle_list)
        .add_systems(
            Update,
            puzzles::puzzle_list_interaction.run_if(in_state(AppState::PuzzlesList)),
        )
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
        .add_systems(Update, handle_escape)
        .add_systems(
            OnEnter(AppState::Settings),
            systems::settings::setup_settings,
        )
        .add_systems(OnExit(AppState::Settings), cleanup_system)
        .run();
}
