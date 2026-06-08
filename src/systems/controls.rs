use crate::Cleanup;
use crate::AppState;
use bevy::prelude::*;

pub fn setup_controls(mut commands: Commands) {
    commands.spawn((Camera2d, Cleanup));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            Cleanup,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Controls & Shortcuts"),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // Table container (grid-like)
            let controls = vec![
                ("Left click on a piece and drag", "Pick up a piece from stash or board and move it around"),
                ("Release (drop)", "Place on board; valid placement is highlighted"),
                ("Right click on a piece (or invalid drop)", "Piece returns to stash"),
                ("R key (while dragging)", "Rotate piece 90 degrees clockwise"),
                ("Hover over a piece", "Show its info (points, effects)"),
                ("ESC", "Go back to previous screen"),
                ("Ctrl + N", "Reset current puzzle / game mode"),
            ];

            for (action, description) in controls {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(1100.0),      // total width
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        // Fixed width for the action column (250px)
                        row.spawn((
                            Node {
                                width: Val::Px(450.0),
                                ..default()
                            },
                        ))
                        .with_child((
                            Text::new(action),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        // Remaining space for description (left‑aligned)
                        row.spawn((
                            Text::new(description),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
            }

            // Back button (width matches menu button)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.6)),
                ))
                .with_child((
                    Text::new("Back to Menu"),
                    TextFont { font_size: 28.0, ..default() },
                    TextColor(Color::WHITE),
                ))
                .observe(|_trigger: On<Pointer<Click>>, mut next_state: ResMut<NextState<AppState>>| {
                    next_state.set(AppState::Menu);
                });
        });
}