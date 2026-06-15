use crate::AppState;
use crate::Cleanup;
use crate::resources::{AIType, BoardSize, GameSettings, TempSettings};
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

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
    SamePieceSet,
}

#[derive(Component)]
pub struct RoundsDisplay;
#[derive(Component)]
pub struct RoundsDecrease;
#[derive(Component)]
pub struct RoundsIncrease;

#[derive(Component)]
pub struct WidthDisplay;
#[derive(Component)]
pub struct WidthDecrease;
#[derive(Component)]
pub struct WidthIncrease;

#[derive(Component)]
pub struct HeightDisplay;
#[derive(Component)]
pub struct HeightDecrease;
#[derive(Component)]
pub struct HeightIncrease;

#[derive(Component)]
struct ConfirmationModal;
#[derive(Component)]
struct YesButton;
#[derive(Component)]
struct NoButton;

pub fn setup_settings(mut commands: Commands, settings: Res<GameSettings>) {
    commands.insert_resource(TempSettings {
        duel_blocking_enabled: settings.duel_blocking_enabled,
        ai_mode: settings.ai_mode,
        rounds: settings.rounds,
        board_width: settings.board_width,
        board_height: settings.board_height,
        same_piece_set: settings.same_piece_set,
    });

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
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .observe(apply_settings);

            // Reset button
            root.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.3, 0.3)),
            ))
            .with_child((
                Text::new("Reset Puzzle Progress"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .observe(open_reset_confirmation);

            // Checkbox row (Destroy mode)
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|row| {
                    let checkbox_text = if settings.duel_blocking_enabled {
                        "[x]"
                    } else {
                        "[ ]"
                    };
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
                        Text::new(checkbox_text),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ))
                    .insert(CheckboxState {
                        value: settings.duel_blocking_enabled,
                        setting_key: SettingKey::DuelBlocking,
                    })
                    .observe(toggle_checkbox);
                    row.spawn((
                        Text::new("Block an opponent's cell after each round (Duel)"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });

            // Checkbox row (Same piece set)
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|row| {
                    let checkbox_text = if settings.same_piece_set {
                        "[x]"
                    } else {
                        "[ ]"
                    };
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
                        Text::new(checkbox_text),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ))
                    .insert(CheckboxState {
                        value: settings.same_piece_set,
                        setting_key: SettingKey::SamePieceSet,
                    })
                    .observe(toggle_checkbox);
                    row.spawn((
                        Text::new("Same piece set for both players each round (Duel)"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });

            // Radio row (AI mode)
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        Text::new("AI Mode:"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));

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

                    let random_color = if settings.ai_mode == AIType::Random {
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
                        BackgroundColor(random_color),
                        RadioState {
                            value: AIType::Random,
                            setting_key: SettingKey::AIMode,
                        },
                    ))
                    .with_child((
                        Text::new("Random"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ))
                    .observe(radio_click);

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

            // Rounds UI
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(15.0),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        Text::new("Rounds (1-99):"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        RoundsDecrease,
                    ))
                    .with_child((
                        Text::new("-"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Text::new(settings.rounds.to_string()),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        RoundsDisplay,
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        RoundsIncrease,
                    ))
                    .with_child((
                        Text::new("+"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Board width UI
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(15.0),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        Text::new("Board Width (7-12):"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        WidthDecrease,
                    ))
                    .with_child((
                        Text::new("-"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Text::new(settings.board_width.to_string()),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        WidthDisplay,
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        WidthIncrease,
                    ))
                    .with_child((
                        Text::new("+"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Board height UI
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(15.0),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        Text::new("Board Height (7-12):"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        HeightDecrease,
                    ))
                    .with_child((
                        Text::new("-"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Text::new(settings.board_height.to_string()),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        HeightDisplay,
                    ));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        HeightIncrease,
                    ))
                    .with_child((
                        Text::new("+"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn toggle_checkbox(
    trigger: On<Pointer<Click>>,
    mut temp_settings: ResMut<TempSettings>,
    mut commands: Commands,
    mut checkbox_query: Query<(&mut CheckboxState, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let entity = trigger.event_target();
    if let Ok((mut state, children)) = checkbox_query.get_mut(entity) {
        state.value = !state.value;
        match state.setting_key {
            SettingKey::DuelBlocking => temp_settings.duel_blocking_enabled = state.value,
            SettingKey::SamePieceSet => temp_settings.same_piece_set = state.value,
            SettingKey::AIMode => {} // handled elsewhere
        }
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
    mut temp_settings: ResMut<TempSettings>,
    mut radio_query: Query<(&RadioState, &mut BackgroundColor, &Children), With<Button>>,
    all_radios: Query<(Entity, &RadioState)>,
) {
    let entity = trigger.event_target();
    let selected_value = if let Ok((state, ..)) = radio_query.get(entity) {
        state.value
    } else {
        return;
    };
    temp_settings.ai_mode = selected_value;
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

pub fn handle_rounds_buttons(
    mut temp_settings: ResMut<TempSettings>,
    mut text_query: Query<&mut Text, With<RoundsDisplay>>,
    decrease_button: Query<&Interaction, (With<RoundsDecrease>, Changed<Interaction>)>,
    increase_button: Query<&Interaction, (With<RoundsIncrease>, Changed<Interaction>)>,
) {
    if let Ok(interaction) = decrease_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.rounds.saturating_sub(1);
            if new_val >= 1 {
                temp_settings.rounds = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
    if let Ok(interaction) = increase_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.rounds.saturating_add(1);
            if new_val <= 99 {
                temp_settings.rounds = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
}

pub fn handle_width_buttons(
    mut temp_settings: ResMut<TempSettings>,
    mut text_query: Query<&mut Text, With<WidthDisplay>>,
    decrease_button: Query<&Interaction, (With<WidthDecrease>, Changed<Interaction>)>,
    increase_button: Query<&Interaction, (With<WidthIncrease>, Changed<Interaction>)>,
) {
    if let Ok(interaction) = decrease_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.board_width.saturating_sub(1);
            if new_val >= 7 {
                temp_settings.board_width = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
    if let Ok(interaction) = increase_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.board_width.saturating_add(1);
            if new_val <= 12 {
                temp_settings.board_width = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
}

pub fn handle_height_buttons(
    mut temp_settings: ResMut<TempSettings>,
    mut text_query: Query<&mut Text, With<HeightDisplay>>,
    decrease_button: Query<&Interaction, (With<HeightDecrease>, Changed<Interaction>)>,
    increase_button: Query<&Interaction, (With<HeightIncrease>, Changed<Interaction>)>,
) {
    if let Ok(interaction) = decrease_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.board_height.saturating_sub(1);
            if new_val >= 7 {
                temp_settings.board_height = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
    if let Ok(interaction) = increase_button.single() {
        if *interaction == Interaction::Pressed {
            let new_val = temp_settings.board_height.saturating_add(1);
            if new_val <= 12 {
                temp_settings.board_height = new_val;
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = new_val.to_string();
                }
            }
        }
    }
}

fn apply_settings(
    _trigger: On<Pointer<Click>>,
    temp_settings: Res<TempSettings>,
    mut settings: ResMut<GameSettings>,
    mut board_size: ResMut<BoardSize>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    settings.duel_blocking_enabled = temp_settings.duel_blocking_enabled;
    settings.ai_mode = temp_settings.ai_mode;
    settings.rounds = temp_settings.rounds;
    settings.board_width = temp_settings.board_width;
    settings.board_height = temp_settings.board_height;
    settings.same_piece_set = temp_settings.same_piece_set;
    board_size.0 = IVec2::new(settings.board_width as i32, settings.board_height as i32);
    next_state.set(AppState::Menu);
}

fn open_reset_confirmation(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let width = window.width();
        let height = window.height();
        let dialog_width = 400.0;
        let dialog_height = 200.0;
        let left = (width - dialog_width) / 2.0;
        let top = (height - dialog_height) / 2.0;

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(width),
                    height: Val::Px(height),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                GlobalZIndex(100),
                ConfirmationModal,
                Cleanup,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(left),
                            top: Val::Px(top),
                            width: Val::Px(dialog_width),
                            height: Val::Px(dialog_height),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(20.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        BorderColor::all(Color::WHITE),
                        GlobalZIndex(101),
                    ))
                    .with_children(|dialog| {
                        dialog.spawn((
                            Text::new("Are you sure?\nAll your saved puzzle solutions will be permanently deleted."),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        dialog
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(30.0),
                                    ..default()
                                },
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(100.0),
                                        height: Val::Px(40.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                                    YesButton,
                                ))
                                .with_child((
                                    Text::new("Yes"),
                                    TextFont::default(),
                                    TextColor(Color::WHITE),
                                ))
                                .observe(confirm_reset);
                                row.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(100.0),
                                        height: Val::Px(40.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
                                    NoButton,
                                ))
                                .with_child((
                                    Text::new("No"),
                                    TextFont::default(),
                                    TextColor(Color::WHITE),
                                ))
                                .observe(cancel_reset);
                            });
                    });
            });
    }
}

fn confirm_reset(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    modal_query: Query<Entity, With<ConfirmationModal>>,
) {
    match crate::puzzles::delete_user_solutions() {
        Ok(count) => println!("Deleted {} user solution files.", count),
        Err(e) => eprintln!("Failed to delete solutions: {}", e),
    }
    for entity in modal_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cancel_reset(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    modal_query: Query<Entity, With<ConfirmationModal>>,
) {
    for entity in modal_query.iter() {
        commands.entity(entity).despawn();
    }
}
