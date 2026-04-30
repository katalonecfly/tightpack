mod components;
mod config;
mod helpers;
mod resources;
mod systems;

use bevy::picking::prelude::*;
use bevy::prelude::*;
use resources::{GameState, TooltipState};
use systems::menu;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component)]
struct Cleanup;

fn cleanup_system(mut commands: Commands, query: Query<Entity, With<Cleanup>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .init_resource::<GameState>()
        .init_resource::<TooltipState>()
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Menu), menu::setup_menu)
        .add_systems(Update, menu::menu_interaction.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_system)
        .add_systems(OnEnter(AppState::InGame), systems::setup::setup_game)
        .add_systems(
            Update,
            (
                systems::ui::update_score_ui,
                systems::ui::update_stash_labels,
                systems::ui::update_effect_previews,
                systems::ui::update_tooltip,
                systems::interaction::handle_rotation,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), cleanup_system)
        .run();
}