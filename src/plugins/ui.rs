use crate::components::{
    ButtonAction, LoadingText, MenuButton, MenuRoot, SeedCancelButton, SeedDecrementButton,
    SeedIncrementButton, SeedInputScreen, SeedInputValue, SeedInputValueText, SeedRandomizeButton,
    SeedSubmitButton, UiAssets, UiCamera, UiSound, WorldSeed, BUTTON_HOVER_COLOR,
    SIDEBAR_COLOR, TEXT_COLOR,
};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;

use crate::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(OnEnter(GameState::Loading), setup_loading_screen)
            .add_systems(
                FixedUpdate,
                handle_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::Loading),
                (cleanup_loading_screen, despawn_ui_camera),
            )
            .add_systems(OnEnter(GameState::SeedInput), setup_seed_input_screen)
            .add_systems(OnExit(GameState::SeedInput), cleanup_seed_input_screen)
            .add_systems(
                FixedUpdate,
                handle_seed_input.run_if(in_state(GameState::SeedInput)),
            );
    }
}

/// This method sets up the main menu UI by creating the necessary nodes and components.
///
/// **Note:** Here we used the new Bevy 0.16 macros to create the UI tree. But since this is pretty
/// new, it is not used everywhere in the code. Also, the Rust community seems to prefer the
/// old way for compilation performance reasons.
fn setup_main_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    info!("Setting up main menu");

    let menu_root_node = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    };

    let background_image_node = ImageNode::from(ui_assets.images.last().unwrap().clone());

    let side_bar = (
        Node {
            width: Val::Px(512.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            padding: UiRect::new(Val::Px(48.0), Val::Px(48.0), Val::Px(64.0), Val::Px(64.0)),
            ..default()
        },
        BackgroundColor(SIDEBAR_COLOR),
    );

    let game_title = (
        Text::new("Mars Mechanica"),
        TextFont {
            font: ui_assets.fonts[1].clone(),
            font_size: 36.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        },
        TextColor(TEXT_COLOR),
    );

    let game_author = (
        Text::new("Par Sofiane Lasri"),
        TextFont {
            font: ui_assets.fonts.first().unwrap().clone(),
            font_size: 16.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        },
        TextColor(TEXT_COLOR),
    );

    let buttons_container = Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::FlexStart,
        flex_grow: 1.0,
        margin: UiRect::top(Val::Px(64.0)),
        ..default()
    };

    let button_node = (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
            ..default()
        },
        Button,
    );

    let buttons_sub_container = Node {
        flex_direction: FlexDirection::Column,
        ..default()
    };

    commands.spawn((
        menu_root_node,
        background_image_node,
        MenuRoot,
        children![(
            side_bar,
            children![
                (
                    // Parent container for the texts
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    children![game_title, game_author]
                ),
                (
                    buttons_container.clone(),
                    children![
                        (
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(64.0),
                                ..default()
                            },
                            children![
                                (
                                    buttons_sub_container.clone(),
                                    children![
                                        (
                                            button_node.clone(),
                                            BackgroundColor(Color::NONE.into()),
                                            MenuButton {
                                                action: ButtonAction::GenerateWorld
                                            },
                                            children![compute_button(
                                                "Créer un monde",
                                                &ui_assets.fonts.first().unwrap().clone()
                                            )]
                                        ),
                                        (
                                            button_node.clone(),
                                            BackgroundColor(Color::NONE.into()),
                                            MenuButton {
                                                action: ButtonAction::LoadSeed
                                            },
                                            children![compute_button(
                                                "Charger une seed",
                                                &ui_assets.fonts.first().unwrap().clone()
                                            )]
                                        )
                                    ]
                                ),
                                (
                                    buttons_sub_container.clone(),
                                    children![
                                        (
                                            button_node.clone(),
                                            BackgroundColor(Color::NONE.into()),
                                            MenuButton {
                                                action: ButtonAction::Settings
                                            },
                                            children![compute_button(
                                                "Paramètres",
                                                &ui_assets.fonts.first().unwrap().clone()
                                            )]
                                        ),
                                        (
                                            button_node.clone(),
                                            BackgroundColor(Color::NONE.into()),
                                            MenuButton {
                                                action: ButtonAction::Credits
                                            },
                                            children![compute_button(
                                                "Crédits",
                                                &ui_assets.fonts.first().unwrap().clone()
                                            )]
                                        )
                                    ]
                                )
                            ]
                        ),
                        (
                            button_node.clone(),
                            BackgroundColor(Color::NONE.into()),
                            MenuButton {
                                action: ButtonAction::Quit
                            },
                            children![compute_button(
                                "Quitter",
                                &ui_assets.fonts.first().unwrap().clone()
                            )]
                        )
                    ]
                ),
            ],
        )],
    ));
}

/// This function creates a button with the given text, font, and color.
fn compute_button(text: &str, font: &Handle<Font>) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size: 24.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        },
        TextColor(TEXT_COLOR),
    )
}

/// This function handles the button interactions in the main menu.
fn handle_menu_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut app_exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_assets: Res<UiAssets>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = BUTTON_HOVER_COLOR.into();
                commands.spawn((
                    AudioPlayer::new(ui_assets.sounds[2].clone()),
                    PlaybackSettings::DESPAWN,
                    UiSound,
                ));

                if let ButtonAction::GenerateWorld = button.action {
                    let random_seed = rand::random::<u32>();
                    commands.insert_resource(WorldSeed(random_seed));
                    next_state.set(GameState::Loading)
                } else if let ButtonAction::Quit = button.action {
                    app_exit.write(AppExit::Success);
                } else if let ButtonAction::LoadSeed = button.action {
                    next_state.set(GameState::SeedInput);
                }
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER_COLOR.into();
                commands.spawn((
                    AudioPlayer::new(ui_assets.sounds[1].clone()),
                    PlaybackSettings::DESPAWN,
                    UiSound,
                ));
            }
            Interaction::None => *color = Color::NONE.into(),
        }
    }
}

/// I think I will delete this method soon because it is no longer useful. In fact, even in
/// development compilation, the loading time is so fast that it is not worth. Also, it apparence
/// is still in an early stage and I don't like it. So I will remove it soon.
fn setup_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            LoadingText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Génération du monde..."),
                TextFont {
                    font: asset_server.load("fonts/inter-bold.ttf"),
                    font_size: 48.0,
                    line_height: Default::default(),
                    font_smoothing: Default::default(),
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn despawn_ui_camera(mut commands: Commands, query: Query<Entity, With<UiCamera>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_seed_input_screen(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let initial_seed = rand::random::<u32>();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ImageNode::from(ui_assets.images.last().unwrap().clone()),
            SeedInputScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Auto),
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(SIDEBAR_COLOR),
                ))
                .with_children(|modal| {
                    modal.spawn((
                        Text::new("Entrez une seed"),
                        TextFont {
                            font: ui_assets.fonts[1].clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    modal.spawn((
                        Text::new("Utilisez le clavier numérique ou les boutons"),
                        TextFont {
                            font: ui_assets.fonts[0].clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    modal
                        .spawn((
                            Node {
                                width: Val::Px(300.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            SeedInputValue {
                                value: initial_seed.to_string(),
                            },
                        ))
                        .with_children(|controls| {
                            controls
                                .spawn((
                                    Node {
                                        width: Val::Px(40.0),
                                        height: Val::Px(40.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    SeedDecrementButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("-"),
                                        TextFont {
                                            font: ui_assets.fonts[1].clone(),
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    ));
                                });

                            controls
                                .spawn((
                                    Node {
                                        width: Val::Px(160.0),
                                        height: Val::Px(40.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                                ))
                                .with_children(|display| {
                                    display.spawn((
                                        Text::new(initial_seed.to_string()),
                                        TextFont {
                                            font: ui_assets.fonts[0].clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                        SeedInputValueText,
                                    ));
                                });

                            controls
                                .spawn((
                                    Node {
                                        width: Val::Px(40.0),
                                        height: Val::Px(40.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    SeedIncrementButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("+"),
                                        TextFont {
                                            font: ui_assets.fonts[1].clone(),
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    ));
                                });
                        });

                    modal
                        .spawn((
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(40.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Button,
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.4)),
                            SeedRandomizeButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Aléatoire"),
                                TextFont {
                                    font: ui_assets.fonts[0].clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    modal
                        .spawn((Node {
                            width: Val::Px(300.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },))
                        .with_children(|buttons| {
                            buttons
                                .spawn((
                                    Node {
                                        width: Val::Px(130.0),
                                        height: Val::Px(40.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(Color::srgb(0.5, 0.0, 0.0)),
                                    SeedCancelButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Annuler"),
                                        TextFont {
                                            font: ui_assets.fonts[0].clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    ));
                                });

                            buttons
                                .spawn((
                                    Node {
                                        width: Val::Px(130.0),
                                        height: Val::Px(40.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(Color::srgb(0.0, 0.5, 0.0)),
                                    SeedSubmitButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Valider"),
                                        TextFont {
                                            font: ui_assets.fonts[0].clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    ));
                                });
                        });
                });
        });
}

fn handle_seed_input(
    mut commands: Commands,
    mut seed_value_query: Query<&mut SeedInputValue>,
    mut value_text_query: Query<&mut Text, With<SeedInputValueText>>,
    mut increment_button_query: Query<
        &Interaction,
        (With<SeedIncrementButton>, Changed<Interaction>),
    >,
    mut decrement_button_query: Query<
        &Interaction,
        (With<SeedDecrementButton>, Changed<Interaction>),
    >,
    mut randomize_button_query: Query<
        &Interaction,
        (With<SeedRandomizeButton>, Changed<Interaction>),
    >,
    mut submit_button_query: Query<&Interaction, (With<SeedSubmitButton>, Changed<Interaction>)>,
    mut cancel_button_query: Query<&Interaction, (With<SeedCancelButton>, Changed<Interaction>)>,
    mut key_events: EventReader<KeyboardInput>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_assets: Res<UiAssets>,
) {
    let mut seed_updated = false;

    if let Ok(mut seed_value) = seed_value_query.single_mut() {
        for event in key_events.read() {
            if event.state == ButtonState::Pressed {
                match event.key_code {
                    KeyCode::Digit0 | KeyCode::Numpad0 => {
                        seed_value.value.push('0');
                        seed_updated = true;
                    }
                    KeyCode::Digit1 | KeyCode::Numpad1 => {
                        seed_value.value.push('1');
                        seed_updated = true;
                    }
                    KeyCode::Digit2 | KeyCode::Numpad2 => {
                        seed_value.value.push('2');
                        seed_updated = true;
                    }
                    KeyCode::Digit3 | KeyCode::Numpad3 => {
                        seed_value.value.push('3');
                        seed_updated = true;
                    }
                    KeyCode::Digit4 | KeyCode::Numpad4 => {
                        seed_value.value.push('4');
                        seed_updated = true;
                    }
                    KeyCode::Digit5 | KeyCode::Numpad5 => {
                        seed_value.value.push('5');
                        seed_updated = true;
                    }
                    KeyCode::Digit6 | KeyCode::Numpad6 => {
                        seed_value.value.push('6');
                        seed_updated = true;
                    }
                    KeyCode::Digit7 | KeyCode::Numpad7 => {
                        seed_value.value.push('7');
                        seed_updated = true;
                    }
                    KeyCode::Digit8 | KeyCode::Numpad8 => {
                        seed_value.value.push('8');
                        seed_updated = true;
                    }
                    KeyCode::Digit9 | KeyCode::Numpad9 => {
                        seed_value.value.push('9');
                        seed_updated = true;
                    }

                    KeyCode::Backspace => {
                        seed_value.value.pop();
                        seed_updated = true;
                    }

                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        if !seed_value.value.is_empty() {
                            // Convertir la chaîne en u32
                            if let Ok(seed) = seed_value.value.parse::<u32>() {
                                commands.insert_resource(WorldSeed(seed));
                                play_ui_sound(&mut commands, &ui_assets, 2);
                                next_state.set(GameState::Loading);
                                return;
                            }
                        }
                    }
                    // Touche Échap pour annuler
                    KeyCode::Escape => {
                        play_ui_sound(&mut commands, &ui_assets, 2);
                        next_state.set(GameState::MainMenu);
                        return;
                    }
                    _ => {}
                }
            }
        }

        let mut current_value = seed_value.value.parse::<u32>().unwrap_or(0);

        for interaction in increment_button_query.iter_mut() {
            if *interaction == Interaction::Pressed {
                current_value = current_value.wrapping_add(1);
                seed_value.value = current_value.to_string();
                seed_updated = true;
                play_ui_sound(&mut commands, &ui_assets, 1);
            }
        }

        for interaction in decrement_button_query.iter_mut() {
            if *interaction == Interaction::Pressed {
                current_value = current_value.wrapping_sub(1);
                seed_value.value = current_value.to_string();
                seed_updated = true;
                play_ui_sound(&mut commands, &ui_assets, 1);
            }
        }

        for interaction in randomize_button_query.iter_mut() {
            if *interaction == Interaction::Pressed {
                seed_value.value = rand::random::<u32>().to_string();
                seed_updated = true;
                play_ui_sound(&mut commands, &ui_assets, 1);
            }
        }

        if seed_value.value.len() > 10 {
            seed_value.value = seed_value.value[..10].to_string();
            seed_updated = true;
        }

        if seed_updated {
            if let Ok(mut text) = value_text_query.single_mut() {
                text.0 = seed_value.value.clone();
                if seed_value.value.is_empty() {
                    text.0 = "0".to_string();
                }
            }
        }

        for interaction in submit_button_query.iter_mut() {
            if *interaction == Interaction::Pressed {
                if !seed_value.value.is_empty() {
                    if let Ok(seed) = seed_value.value.parse::<u32>() {
                        commands.insert_resource(WorldSeed(seed));
                        play_ui_sound(&mut commands, &ui_assets, 2);
                        next_state.set(GameState::Loading);
                        return;
                    }
                }
            }
        }
    }

    for interaction in cancel_button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            play_ui_sound(&mut commands, &ui_assets, 2);
            next_state.set(GameState::MainMenu);
            return;
        }
    }
}

/// This function plays a UI sound effect based on the provided sound index from the asset preloader.
fn play_ui_sound(commands: &mut Commands, ui_assets: &Res<UiAssets>, sound_index: usize) {
    commands.spawn((
        AudioPlayer::new(ui_assets.sounds[sound_index].clone()),
        PlaybackSettings::DESPAWN,
        UiSound,
    ));
}

fn cleanup_seed_input_screen(mut commands: Commands, query: Query<Entity, With<SeedInputScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
