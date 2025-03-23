use crate::components::{
    ButtonAction, LoadingText, MenuButton, MenuRoot, BUTTON_COLOR, BUTTON_HOVER, BUTTON_PRESS,
    TEXT_COLOR,
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

    let menu_root_object = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        MenuRoot,
    );

    commands.spawn(menu_root_object).with_children(|parent| {
        // Titre
        parent.spawn((
            Text::new("Mars Mechanica"),
            TextFont {
                font: asset_server.load("fonts/inter-bold.ttf"),
                font_size: 64.0,
                font_smoothing: Default::default(),
            },
            TextColor(TEXT_COLOR),
            TextLayout::new_with_justify(JustifyText::Center),
        ));

        // Bouton Nouvelle Partie
        parent
            .spawn((
                Button,
                MenuButton {
                    action: ButtonAction::Play,
                },
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Jouer"),
                    TextFont {
                        font: asset_server.load("fonts/inter-regular.ttf"),
                        font_size: 40.0,
                        font_smoothing: Default::default(),
                    },
                    TextColor(TEXT_COLOR),
                ));
            });

        // Bouton Quitter
        parent
            .spawn((
                Button,
                MenuButton {
                    action: ButtonAction::Quit,
                },
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(65.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Quitter"),
                    TextFont {
                        font: asset_server.load("fonts/inter-regular.ttf"),
                        font_size: 40.0,
                        font_smoothing: Default::default(),
                    },
                    TextColor(TEXT_COLOR),
                ));
            });
    });
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
                *color = BUTTON_PRESS.into();

                if let ButtonAction::Play = button.action {
                    next_state.set(GameState::Loading)
                } else {
                    app_exit.send(AppExit::Success);
                }
            }
            Interaction::Hovered => *color = BUTTON_HOVER.into(),
            Interaction::None => *color = BUTTON_COLOR.into(),
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
        commands.entity(entity).despawn_recursive();
    }
}
