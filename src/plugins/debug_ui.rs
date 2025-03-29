use bevy::color::palettes::css::BLACK;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::{FlexDirection, UiRect};

use crate::components::UiAssets;
use crate::GameState;

// Existing debug text marker components
#[derive(Component)]
pub struct DebugCameraText;
#[derive(Component)]
pub struct DebugHoverText;
#[derive(Component)]
struct FpsCounterText;

#[derive(Component)]
pub struct ToolboxToggle {
    pub value: bool,
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            (init_debug_bar, init_debug_toolbox),
        )
            .add_systems(
                FixedUpdate,
                update_debug_camera_text.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                update_fps_counter.run_if(in_state(GameState::InGame)),
            );
    }
}

fn init_debug_bar(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let root_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor::from(BLACK),
        ))
        .id();

    commands.entity(root_entity).with_related::<ChildOf>(|child_spawner| {
        spawn_bar_column(child_spawner, |col_spawner| {
            spawn_bar_text(
                col_spawner,
                &ui_assets,
                "Mouse position: (0.0, 0.0)",
                DebugCameraText,
            );
            spawn_bar_text(
                col_spawner,
                &ui_assets,
                "Hovered cell: None",
                DebugHoverText,
            );
        });

        spawn_bar_column(child_spawner, |col_spawner| {
            spawn_bar_text(col_spawner, &ui_assets, "FPS: --", FpsCounterText);
        });
    });
}

// TODO: Voir pour utiliser bevy_mod_imgui lorsque le paquet sera compatible avec Bevy 0.16
fn init_debug_toolbox(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let toolbox_root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(16.0),
                width: Val::Px(255.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();

    let toolbox_title = (
        Node {
            width: Val::Percent(100.0),
            padding: UiRect {
                left: Val::Px(8.0),
                right: Val::Px(8.0),
                top: Val::Px(4.0),
                bottom: Val::Px(4.0),
            },
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        children![(
            Text::new("Debug Toolbox"),
            TextFont {
                font: ui_assets.fonts.last().unwrap().clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    );

    let toolbox_content = (
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(76.0 / 255.0, 76.0 / 255.0, 76.0 / 255.0, 0.9)),
    );

    commands.entity(toolbox_root).with_related::<ChildOf>(|child_spawner| {
        child_spawner.spawn(toolbox_title);
        child_spawner
            .spawn(toolbox_content)
            .with_related::<ChildOf>(|content_spawner| {
                spawn_toolbox_section(content_spawner, &ui_assets, "Cell Mouse Selection", |section_spawner, ui_assets| {
                    spawn_toolbox_property(section_spawner, ui_assets, "Solid Objects", true);
                    spawn_toolbox_property(section_spawner, ui_assets, "Entities", false);
                });

                spawn_toolbox_section(content_spawner, &ui_assets, "Click Action", |section_spawner, ui_assets| {
                    spawn_toolbox_property(section_spawner, ui_assets, "Destroy", true);
                    spawn_toolbox_property(section_spawner, ui_assets, "Place Solid Object", false);
                    spawn_toolbox_property(section_spawner, ui_assets, "Place Entity", false);
                });

                spawn_toolbox_section(content_spawner, &ui_assets, "Solid Objects", |section_spawner, ui_assets| {
                    spawn_toolbox_property(section_spawner, ui_assets, "Rock", true);
                    spawn_toolbox_property(section_spawner, ui_assets, "Olivine", false);
                    spawn_toolbox_property(section_spawner, ui_assets, "Basalt", false);
                    spawn_toolbox_property(section_spawner, ui_assets, "Red Crystal", false);
                });
            });
    });
}

fn spawn_bar_text<M: Component>(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    ui_assets: &UiAssets,
    text_content: &str,
    marker: M,
) {
    spawner.spawn((
        Text::new(text_content),
        TextFont {
            font: ui_assets.fonts.last().unwrap().clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        marker,
    ));
}

fn spawn_bar_column(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    spawn_contents: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    spawner
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_related::<ChildOf>(|col_spawner| {
            spawn_contents(col_spawner);
        });
}

/// Spawns a section in the toolbox with a title and a builder function.
///
/// # Arguments
/// * `spawner` - The spawner to use for creating the UI elements.
/// * `ui_assets` - The UI assets to use for the font.
/// * `section_title` - The title of the section.
/// * `build_section` - A closure that builds the contents of the section.
///
/// The closure receives a mutable reference to the spawner and the UI assets.
/// It is expected to spawn the UI elements for the section.
///
/// # Example:
/// ```rust
/// spawn_toolbox_section(content_spawner, &ui_assets, "SectionTitle", |section_spawner, ui_assets| {
///     spawn_toolbox_property(section_spawner, ui_assets, "Property 1", true);
///     spawn_toolbox_property(section_spawner, ui_assets, "Property 2", false);
/// });
/// ````
fn spawn_toolbox_section(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    ui_assets: &UiAssets,
    section_title: &str,
    build_section: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>, &UiAssets),
) {
    spawner
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_related::<ChildOf>(|section_spawner| {
            section_spawner.spawn((
                Text::new(section_title),
                TextFont {
                    font: ui_assets.fonts.last().unwrap().clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            section_spawner
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_related::<ChildOf>(|props_spawner| {
                    build_section(props_spawner, ui_assets);
                });
        });
}

/// Spawns a property in the toolbox with a checkbox and label.
///
/// # Arguments
/// * `spawner` - The spawner to use for creating the UI elements.
/// * `ui_assets` - The UI assets to use for the font.
/// * `label_text` - The text to display next to the checkbox.
/// * `initial_state` - The initial state of the checkbox (checked or unchecked).
///
/// # Example:
/// ```rust
/// spawn_toolbox_property(spawner, &ui_assets, "Property Name", true);
/// ```
fn spawn_toolbox_property(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    ui_assets: &UiAssets,
    label_text: &str,
    initial_state: bool,
) {
    spawner
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_related::<ChildOf>(|row_spawner| {
            let checkbox_color = if initial_state {
                Color::srgba(0.0, 1.0, 0.0, 1.0)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 1.0)
            };

            row_spawner.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(checkbox_color),
                ToolboxToggle { value: initial_state },
            ));

            row_spawner.spawn((
                Text::new(label_text),
                TextFont {
                    font: ui_assets.fonts.last().unwrap().clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn update_debug_camera_text(
    text_query: Query<Entity, With<DebugCameraText>>,
    window_query: Query<&Window>,
    mut writer: TextUiWriter,
) {
    let window = window_query.single().unwrap();

    let cursor_position = if let Some(position) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        position - window_size / 2.0
    } else {
        Vec2::ZERO
    };

    let text_entity = text_query.single().unwrap();
    *writer.text(text_entity, 0) = format!(
        "Mouse position: ({:.1}, {:.1})",
        cursor_position.x, cursor_position.y
    );
}

fn update_fps_counter(
    text_query: Query<Entity, With<FpsCounterText>>,
    time: Res<Time>,
    mut writer: TextUiWriter,
) {
    let text_entity = text_query.single().unwrap();
    let fps = 1.0 / time.delta_secs();
    let color = if fps < 30.0 {
        Color::srgb(1.0, 0.0, 0.0)
    } else if fps < 40.0 {
        Color::srgb(1.0, 0.5, 0.0)
    } else {
        Color::WHITE
    };

    *writer.text(text_entity, 0) = format!("FPS: {:.1}", fps);
    *writer.color(text_entity, 0) = TextColor::from(color);
}
