use crate::components::{
    ButtonAction, LoadingText, MenuButton, MenuButtonComponent, MenuRoot, BUTTON_HOVER_COLOR,
    SIDEBAR_COLOR, TEXT_COLOR,
};
use crate::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(OnEnter(GameState::Loading), setup_loading_screen)
            .add_systems(FixedUpdate, (handle_menu_buttons, handle_loading));
    }
}

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setting up main menu");

    let menu_root_node = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    };

    let background_image_node = ImageNode {
        color: Default::default(),
        image: asset_server.load("textures/background.png"),
        texture_atlas: None,
        flip_x: false,
        flip_y: false,
        rect: None,
        image_mode: Default::default(),
    };

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
            font: asset_server.load("fonts/inter-bold.ttf"),
            font_size: 36.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        },
        TextColor(TEXT_COLOR),
    );

    let game_author = (
        Text::new("Par Sofiane Lasri"),
        TextFont {
            font: asset_server.load("fonts/inter-regular.ttf"),
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

    let button_node = (Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
        ..default()
    }, Button);

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
                                                &asset_server
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
                                                &asset_server
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
                                            children![compute_button("Paramètres", &asset_server)]
                                        ),
                                        (
                                            button_node.clone(),
                                            BackgroundColor(Color::NONE.into()),
                                            MenuButton {
                                                action: ButtonAction::Credits
                                            },
                                            children![compute_button("Crédits", &asset_server)]
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
                            children![compute_button("Quitter", &asset_server)]
                        )
                    ]
                ),
            ],
        )],
    ));
}

fn compute_button(text: &str, asset_server: &Res<AssetServer>) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font: asset_server.load("fonts/inter-regular.ttf"),
            font_size: 24.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        },
        TextColor(TEXT_COLOR),
    )
}

fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut app_exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = BUTTON_HOVER_COLOR.into();

                if let ButtonAction::GenerateWorld = button.action {
                    next_state.set(GameState::Loading)
                } else if let ButtonAction::Quit = button.action {
                    app_exit.send(AppExit::Success);
                }
            }
            Interaction::Hovered => *color = BUTTON_HOVER_COLOR.into(),
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

fn handle_loading(
    mut next_state: ResMut<NextState<GameState>>,
    // Ajouter ici les conditions de fin de chargement
) {
    // Exemple: Attendre que la génération soit terminée
    //next_state.set(GameState::InGame);
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
