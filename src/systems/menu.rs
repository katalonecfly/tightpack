use crate::AppState;
use crate::Cleanup;
use bevy::prelude::*;

const BUTTON_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const TEXT_COLOR: Color = Color::WHITE;

#[derive(Component)]
pub(crate) struct MenuButton;

pub fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tightpack"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            for label in &["Sandbox", "Draft", "Duel", "Puzzles", "Settings"] {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(250.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR),
                        MenuButton,
                    ))
                    .with_child((
                        Text::new(*label),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
            }
        });

    commands.spawn((Camera2d, Cleanup));
}

pub fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<(&Interaction, &Children), (With<MenuButton>, Changed<Interaction>)>,
    text_query: Query<&mut Text>,
) {
    for (interaction, children) in &query {
        if *interaction == Interaction::Pressed {
            let label = children
                .iter()
                .find_map(|child| text_query.get(child).ok())
                .map(|t| t.0.clone())
                .unwrap_or_default();
            match label.as_str() {
                "Sandbox" => next_state.set(AppState::Sandbox),
                "Draft" => next_state.set(AppState::Draft),
                "Duel" => next_state.set(AppState::Duel),
                "Puzzles" => next_state.set(AppState::PuzzlesList),
                "Settings" => next_state.set(AppState::Settings),
                _ => {} // Puzzles still do nothing
            }
        }
    }
}
