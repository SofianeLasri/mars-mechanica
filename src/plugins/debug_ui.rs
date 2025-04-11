use bevy::color::palettes::css::BLACK;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::ui::{FlexDirection, Interaction, UiRect};

use crate::components::entity::{DebugRobotText, ExplorerRobot, WorldKnowledge};
use crate::components::{UiAssets, WorldSeed};
use crate::GameState;

// Existing debug text marker components
#[derive(Component)]
pub struct DebugCameraText;
#[derive(Component)]
pub struct DebugHoverText;
#[derive(Component)]
struct FpsCounterText;
#[derive(Component)]
struct WorldSeedText;
#[derive(Component)]
struct GraphicAdapterText;

#[derive(Component)]
pub struct ToolboxToggle {
    pub name: String,
    pub value: bool,
}

#[derive(Resource, Default, Debug)]
pub struct ToolboxState {
    // Cell Mouse Selection
    pub select_solid_objects: bool,
    pub select_entities: bool,
    pub select_empty_cells: bool,
    pub select_all: bool,

    // Click Action
    pub action_destroy: bool,
    pub action_place_solid: bool,
    pub action_place_entity: bool,

    // Solid Objects
    pub solid_rock: bool,
    pub solid_olivine: bool,
    pub solid_basalt: bool,
    pub solid_red_crystal: bool,
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolboxState>()
            .add_systems(
                OnEnter(GameState::InGame),
                (init_debug_bar, init_debug_toolbox),
            )
            .add_systems(
                Update,
                update_fps_counter.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                update_debug_camera_text.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (toolbox_toggle_system, update_toolbox_state)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                update_robot_debug_text.run_if(in_state(GameState::InGame)),
            );
    }
}

fn init_debug_bar(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    adapter: Res<RenderAdapterInfo>,
    world_seed: Res<WorldSeed>,
) {
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

    commands.entity(root_entity).with_children(|child_spawner| {
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
            spawn_bar_text(
                col_spawner,
                &ui_assets,
                &format!("Seed: {}", world_seed.0),
                WorldSeedText,
            );
            spawn_bar_text(
                col_spawner,
                &ui_assets,
                "Robot: No data",
                DebugRobotText,
            );
        });

        let graphic_adapter_name = &adapter.name;
        spawn_bar_column(child_spawner, |col_spawner| {
            spawn_bar_text(col_spawner, &ui_assets, "FPS: --", FpsCounterText);
            spawn_bar_text(
                col_spawner,
                &ui_assets,
                &format!("Adapter: {}", graphic_adapter_name),
                GraphicAdapterText,
            );
        });
    });
}

// TODO: Voir pour utiliser bevy_mod_imgui lorsque le paquet sera compatible avec Bevy 0.16
fn init_debug_toolbox(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let toolbox_root = commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(16.0),
            width: Val::Px(255.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
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

    commands
        .entity(toolbox_root)
        .with_children(|child_spawner| {
            child_spawner.spawn(toolbox_title);
            child_spawner
                .spawn(toolbox_content)
                .with_related::<ChildOf>(|content_spawner| {
                    // Multiple choices authorized
                    spawn_toolbox_section(
                        content_spawner,
                        &ui_assets,
                        "Cell Mouse Selection",
                        |section_spawner, ui_assets| {
                            spawn_toolbox_property(section_spawner, ui_assets, "All", "all", false);
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Solid Objects",
                                "solid_objects",
                                true,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Entities",
                                "entities",
                                false,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Empty Cells",
                                "empty_cells",
                                false,
                            );
                        },
                    );

                    // No multiple choices authorized
                    spawn_toolbox_section(
                        content_spawner,
                        &ui_assets,
                        "Click Action",
                        |section_spawner, ui_assets| {
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Destroy",
                                "detroy",
                                true,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Place Solid Object",
                                "place_solid_objects",
                                false,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Place Entity",
                                "place_entity",
                                false,
                            );
                        },
                    );

                    // No multiple choices authorized
                    spawn_toolbox_section(
                        content_spawner,
                        &ui_assets,
                        "Solid Objects",
                        |section_spawner, ui_assets| {
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Rock",
                                "rock",
                                true,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Olivine",
                                "olivine",
                                false,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Basalt",
                                "basalt",
                                false,
                            );
                            spawn_toolbox_property(
                                section_spawner,
                                ui_assets,
                                "Red Crystal",
                                "red_crystal",
                                false,
                            );
                        },
                    );
                });
        });
}

fn update_toolbox_state(
    mut state: ResMut<ToolboxState>,
    query: Query<&ToolboxToggle>,
) {
    *state = ToolboxState::default();

    for toggle in query.iter() {
        if !toggle.value {
            continue;
        }

        let label = &toggle.name.as_str();

        match *label {
            "all" => state.select_all = true,
            "solid_objects" => state.select_solid_objects = true,
            "entities" => state.select_entities = true,
            "empty_cells" => state.select_empty_cells = true,

            // Click Action section
            "destroy" => state.action_destroy = true,
            "place_solid_objects" => state.action_place_solid = true,
            "place_entity" => state.action_place_entity = true,

            // Solid Objects section
            "rock" => state.solid_rock = true,
            "olivine" => state.solid_olivine = true,
            "basalt" => state.solid_basalt = true,
            "red_crystal" => state.solid_red_crystal = true,
            _ => {}
        }
    }

    if state.select_all {
        state.select_solid_objects = false;
        state.select_entities = false;
        state.select_empty_cells = false;
    } else if state.select_solid_objects {
        state.select_entities = false;
        state.select_empty_cells = false;
    } else if state.select_entities {
        state.select_empty_cells = false;
    }

    if state.action_destroy {
        state.action_place_solid = false;
        state.action_place_entity = false;
    } else if state.action_place_solid {
        state.action_place_entity = false;
    }

    if state.solid_rock {
        state.solid_olivine = false;
        state.solid_basalt = false;
        state.solid_red_crystal = false;
    } else if state.solid_olivine {
        state.solid_basalt = false;
        state.solid_red_crystal = false;
    } else if state.solid_basalt {
        state.solid_red_crystal = false;
    }

    if !state.select_all
        && !state.select_solid_objects
        && !state.select_entities
        && !state.select_empty_cells
    {
        state.select_solid_objects = true;
    }

    if !state.action_destroy && !state.action_place_solid && !state.action_place_entity {
        state.action_destroy = true;
    }

    if !state.solid_rock && !state.solid_olivine && !state.solid_basalt && !state.solid_red_crystal
    {
        state.solid_rock = true;
    }
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
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },))
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
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },))
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
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },))
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
/// * `name` - The name of the property (used for identification).
/// * `initial_state` - The initial state of the checkbox (checked or unchecked).
///
/// # Example:
/// ```rust
/// spawn_toolbox_property(spawner, &ui_assets, "Property Name", "property_name", true);
/// ```
fn spawn_toolbox_property(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    ui_assets: &UiAssets,
    label_text: &str,
    name: &str,
    initial_state: bool,
) {
    spawner
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            ..default()
        },))
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
                Button,
                BackgroundColor(checkbox_color),
                ToolboxToggle {
                    name: name.to_string(),
                    value: initial_state,
                },
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

/// Système qui gère l'interaction sur les cases à cocher de la toolbox.
/// Pour chaque entité possédant ToolboxToggle, lorsque l'état Interaction change et qu'il est Pressed,
/// on inverse la valeur et on met à jour la couleur de fond.
fn toolbox_toggle_system(
    mut query: Query<
        (&Interaction, &mut ToolboxToggle, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut toggle, mut bg_color) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Inverser la valeur
            toggle.value = !toggle.value;
            // Mettre à jour la couleur en fonction de la nouvelle valeur
            if toggle.value {
                *bg_color = BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 1.0));
            } else {
                *bg_color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 1.0));
            }
        }
    }
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

pub fn update_robot_debug_text(
    text_query: Query<Entity, With<DebugRobotText>>,
    world_knowledge: Res<WorldKnowledge>,
    robot_query: Query<&ExplorerRobot>,
    mut writer: TextUiWriter,
) {
    if let Ok(text_entity) = text_query.single() {
        if let Ok(robot) = robot_query.single() {
            let discovered_cells = world_knowledge.discovered_cells.len();
            let discovered_solids = world_knowledge.discovered_solids.len();
            let discovered_empty = world_knowledge.discovered_empty.len();

            let text = format!(
                "Robot: Pos({},{}), Cells: {}, Solids: {}, Empty: {}",
                robot.target_position.x,
                robot.target_position.y,
                discovered_cells,
                discovered_solids,
                discovered_empty
            );

            *writer.text(text_entity, 0) = text;
        } else {
            *writer.text(text_entity, 0) = "Robot: Not spawned".to_string();
        }
    }
}