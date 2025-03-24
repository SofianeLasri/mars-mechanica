use crate::components::{
    ButtonAction, LoadingText, MenuButton, MenuRoot, BUTTON_HOVER_COLOR, SIDEBAR_COLOR, TEXT_COLOR,
};
use crate::plugins::splash::UiAssets;
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
            );
    }
}

#[derive(Component)]
pub(crate) struct UiCamera;

#[derive(Component)]
struct UiSound;

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
            font: ui_assets.fonts.last().unwrap().clone(),
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

fn handle_menu_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut app_exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_assets: Res<UiAssets>
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = BUTTON_HOVER_COLOR.into();
                commands.spawn((AudioPlayer::new(ui_assets.sounds[2].clone()), UiSound));

                if let ButtonAction::GenerateWorld = button.action {
                    next_state.set(GameState::Loading)
                } else if let ButtonAction::Quit = button.action {
                    app_exit.send(AppExit::Success);
                }
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER_COLOR.into();
                commands.spawn((AudioPlayer::new(ui_assets.sounds[1].clone()), UiSound));
            }
            Interaction::None => *color = Color::NONE.into(),
        }
    }
}

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
        commands.entity(entity).despawn_recursive();
    }
}

fn despawn_ui_camera(mut commands: Commands, query: Query<Entity, With<UiCamera>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
