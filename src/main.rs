mod components;
mod config;
mod helpers;
mod resources;
mod systems;

use bevy::picking::prelude::*;
use bevy::prelude::*;
use resources::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .init_resource::<GameState>()
        .add_systems(Startup, systems::setup::setup)
        .add_systems(Update, (
            systems::ui::update_score_ui,
            systems::ui::update_stash_labels,
            systems::ui::update_effect_previews,
            systems::interaction::handle_rotation,
        ))
        .run();
}