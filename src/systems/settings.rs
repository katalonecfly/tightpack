use bevy::prelude::*;
use bevy::picking::prelude::{Click, Pointer};
use crate::resources::GameSettings;
use crate::Cleanup;
use crate::AppState;

#[derive(Component)]
struct SettingsRoot;

#[derive(Component, Clone)]
struct CheckboxState {
    value: bool,
    setting_key: SettingKey,
}

#[derive(Clone, PartialEq, Eq)]
enum SettingKey {
    DuelBlocking,
}

pub fn setup_settings(
    mut commands: Commands,
    settings: Res<GameSettings>,
) {
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
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Cleanup,
            SettingsRoot,
        ))
        .with_children(|root| {
            // Apply button - now at the top
            root.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(30.0)), // space below button
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
            ))
            .with_child((
                Text::new("Apply"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ))
            .observe(apply_settings);

            // General section
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|section| {
                section.spawn((
                    Text::new("General"),
                    TextFont { font_size: 30.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                section.spawn((
                    Text::new("No settings yet."),
                    TextFont::default(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Sandbox section
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|section| {
                section.spawn((
                    Text::new("Sandbox"),
                    TextFont { font_size: 30.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                section.spawn((
                    Text::new("No settings yet."),
                    TextFont::default(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Draft section
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|section| {
                section.spawn((
                    Text::new("Draft"),
                    TextFont { font_size: 30.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                section.spawn((
                    Text::new("No settings yet."),
                    TextFont::default(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Duel section
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|section| {
                section.spawn((
                    Text::new("Duel"),
                    TextFont { font_size: 30.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                // Row for checkbox
                section
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        ))
                        .with_child((
                            Text::new(if settings.duel_blocking_enabled { "[x]" } else { "[ ]" }),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(Color::WHITE),
                        ))
                        .insert(CheckboxState {
                            value: settings.duel_blocking_enabled,
                            setting_key: SettingKey::DuelBlocking,
                        })
                        .observe(toggle_checkbox);
                        row.spawn((
                            Text::new("Block opponent's cells (Destroy mode)"),
                            TextFont::default(),
                            TextColor(Color::WHITE),
                        ));
                    });
            });

            // Puzzles section
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|section| {
                section.spawn((
                    Text::new("Puzzles"),
                    TextFont { font_size: 30.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                section.spawn((
                    Text::new("No settings yet."),
                    TextFont::default(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });
        });
}

fn toggle_checkbox(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    mut checkbox_query: Query<(&mut CheckboxState, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let entity = trigger.event_target();
    if let Ok((mut state, children)) = checkbox_query.get_mut(entity) {
        state.value = !state.value;
        for &child in children {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if state.value { "[x]" } else { "[ ]" }.to_string();
                break;
            }
        }
        commands.entity(entity).insert(state.clone());
    }
}

fn apply_settings(
    _trigger: On<Pointer<Click>>,
    checkbox_query: Query<&CheckboxState>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for state in checkbox_query.iter() {
        match state.setting_key {
            SettingKey::DuelBlocking => {
                settings.duel_blocking_enabled = state.value;
            }
        }
    }
    next_state.set(AppState::Menu);
}