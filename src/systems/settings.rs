use bevy::prelude::*;
use bevy::picking::prelude::{Click, Pointer};
use crate::resources::{GameSettings, AIType};
use crate::Cleanup;
use crate::AppState;

#[derive(Component)]
struct SettingsRoot;

#[derive(Component, Clone)]
struct CheckboxState {
    value: bool,
    setting_key: SettingKey,
}

#[derive(Component, Clone)]
struct RadioState {
    value: AIType,
    setting_key: SettingKey,
}

#[derive(Clone, PartialEq, Eq)]
enum SettingKey {
    DuelBlocking,
    AIMode,
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
            // Apply button
            root.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(30.0)),
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

            // Checkbox row (Destroy mode)
            root
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

            // Radio row (AI mode)
            root
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new("AI Mode:"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));

                    // Dummy button
                    let dummy_color = if settings.ai_mode == AIType::Dummy {
                        Color::srgb(0.4, 0.6, 0.4)
                    } else {
                        Color::srgb(0.3, 0.3, 0.3)
                    };
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(dummy_color),
                        RadioState {
                            value: AIType::Dummy,
                            setting_key: SettingKey::AIMode,
                        },
                    ))
                    .with_child((
                        Text::new("Dummy"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ))
                    .observe(radio_click);

                    // Greedy button
                    let greedy_color = if settings.ai_mode == AIType::Greedy {
                        Color::srgb(0.4, 0.6, 0.4)
                    } else {
                        Color::srgb(0.3, 0.3, 0.3)
                    };
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(greedy_color),
                        RadioState {
                            value: AIType::Greedy,
                            setting_key: SettingKey::AIMode,
                        },
                    ))
                    .with_child((
                        Text::new("Greedy"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ))
                    .observe(radio_click);
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

fn radio_click(
    trigger: On<Pointer<Click>>,
    mut settings: ResMut<GameSettings>,
    mut radio_query: Query<(&RadioState, &mut BackgroundColor, &Children), With<Button>>,
    all_radios: Query<(Entity, &RadioState)>,
    _text_query: Query<&mut Text>,
) {
    let entity = trigger.event_target();
    let selected_value = if let Ok((state, ..)) = radio_query.get(entity) {
        state.value
    } else {
        return;
    };

    // Update the global settings immediately
    settings.ai_mode = selected_value;

    // Update background colors of all radios with the same key
    for (e, _) in all_radios.iter() {
        if let Ok((state, mut bg, _)) = radio_query.get_mut(e) {
            if state.setting_key == SettingKey::AIMode {
                *bg = if state.value == selected_value {
                    Color::srgb(0.4, 0.6, 0.4).into()
                } else {
                    Color::srgb(0.3, 0.3, 0.3).into()
                };
            }
        }
    }
}

fn apply_settings(
    _trigger: On<Pointer<Click>>,
    checkbox_query: Query<&CheckboxState>,
    radio_query: Query<&RadioState>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for state in checkbox_query.iter() {
        match state.setting_key {
            SettingKey::DuelBlocking => {
                settings.duel_blocking_enabled = state.value;
            }
            _ => {}
        }
    }
    for state in radio_query.iter() {
        if state.setting_key == SettingKey::AIMode {
            settings.ai_mode = state.value;
        }
    }
    next_state.set(AppState::Menu);
}