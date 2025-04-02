use crate::components::{ChunkUtils, HoverState, SolidObject, TerrainCell, TerrainChunk, UpdateTerrainEvent, WorldMaterials, CELL_SIZE, VEC2_CELL_SIZE};
use crate::plugins::camera::get_cursor_world_position;
use crate::plugins::debug_ui::{DebugHoverText, ToolboxState};
use crate::GameState;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InteractionPlugin;

#[derive(Component)]
pub struct InteractionSprite;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), init)
            .add_systems(
                FixedUpdate,
                (
                    hover_detection.run_if(in_state(GameState::InGame)),
                    block_click_handler.run_if(in_state(GameState::InGame)),
                ),
            );
    }
}

pub fn init(mut commands: Commands) {
    let semi_transparent_white = Color::srgba(1.0, 1.0, 1.0, 0.5);
    let sprite = Sprite::from_color(semi_transparent_white, VEC2_CELL_SIZE);
    commands.spawn((sprite, InteractionSprite, Transform::from_xyz(0.0, 0.0, -100.0))); // Commencer hors vue
}

/// This system will detect when the cursor is hovering over a terrain cell and update the visual effect
pub fn hover_detection(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection)>,
    terrain_cells_query: Query<(Entity, &Transform, &TerrainCell, Option<&Children>, &TerrainChunk)>,
    solid_objects_query: Query<(Entity, Option<&HoverState>, &SolidObject)>,
    mut interaction_sprite_query: Query<(Entity, &mut Transform), (With<InteractionSprite>, Without<TerrainCell>)>,
    mut text_query: Query<Entity, With<DebugHoverText>>,
    mut writer: TextUiWriter,
    toolbox_state: Res<ToolboxState>,
) {
    let (interaction_sprite, mut interaction_transform) = interaction_sprite_query.single_mut().unwrap();
    let cursor_world_position = get_cursor_world_position(window_query, camera_query);

    // Reset all hover states
    reset_solid_objects_hover_state(&mut commands, &solid_objects_query);

    let mut found_hovered_cell = false;

    // Calculate chunk coordinates
    let cursor_x = (cursor_world_position.x / CELL_SIZE as f32).round() as i32;
    let cursor_y = (cursor_world_position.y / CELL_SIZE as f32).round() as i32;
    let (cursor_chunk_x, cursor_chunk_y) = ChunkUtils::world_to_chunk_coords(cursor_x, cursor_y);

    for (terrain_entity, transform, _, has_children, chunk) in terrain_cells_query.iter() {
        if chunk.chunk_x != cursor_chunk_x || chunk.chunk_y != cursor_chunk_y {
            continue;
        }
        update_debug_text(&mut text_query, &mut writer, Some(transform));

        let block_size = VEC2_CELL_SIZE;
        let block_min = Vec2::new(
            transform.translation.x - block_size.x / 2.0,
            transform.translation.y - block_size.y / 2.0,
        );
        let block_max = Vec2::new(
            transform.translation.x + block_size.x / 2.0,
            transform.translation.y + block_size.y / 2.0,
        );

        let block_is_hovered = cursor_world_position.x >= block_min.x
            && cursor_world_position.x <= block_max.x
            && cursor_world_position.y >= block_min.y
            && cursor_world_position.y <= block_max.y;

        if block_is_hovered {
            let mut should_highlight = false;

            // Check if cell has children
            if let Some(children) = has_children {
                // Check if cell has a solid object
                let has_solid_object = children.iter().any(|child| solid_objects_query.get(child).is_ok());

                // Apply toolbox filtering
                should_highlight = toolbox_state.select_all ||
                    (toolbox_state.select_solid_objects && has_solid_object) ||
                    (toolbox_state.select_empty_cells && children.is_empty());
            } else {
                // No children, so it's empty
                should_highlight = toolbox_state.select_all || toolbox_state.select_empty_cells;
            }

            if should_highlight {
                found_hovered_cell = true;

                // Update debug text
                //update_debug_text(&mut text_query, &mut writer, Some(transform));

                // Move interaction sprite to hover over the cell
                interaction_transform.translation.x = transform.translation.x;
                interaction_transform.translation.y = transform.translation.y;
                interaction_transform.translation.z = 100.0; // Z-index higher than blocks

                // Mark solid objects as hovered if they exist
                if let Some(children) = has_children {
                    for &child in children {
                        if solid_objects_query.get(child).is_ok() {
                            commands.entity(child).insert(HoverState { hovered: true });
                        }
                    }
                }

                break;
            }
        }
    }

    if !found_hovered_cell {
        // Hide interaction sprite if not hovering
        interaction_transform.translation.z = -100.0;
        update_debug_text(&mut text_query, &mut writer, None);
    }
}

/// This system destroys or places blocks when the player clicks
pub fn block_click_handler(
    mut commands: Commands,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection)>,
    terrain_cells_query: Query<(Entity, &Transform, &Children, &TerrainChunk), With<TerrainCell>>,
    mut solid_objects_query: Query<(Entity, Option<&HoverState>, &mut SolidObject)>,
    mut update_terrain_events: EventWriter<UpdateTerrainEvent>,
    world_materials: Res<WorldMaterials>,
    toolbox_state: Res<ToolboxState>,
) {
    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            // First try to handle destroy action
            if toolbox_state.action_destroy {
                // Find and destroy any hovered solid object
                for (entity, hover_state, mut solid_object) in solid_objects_query.iter_mut() {
                    if let Some(hover) = hover_state {
                        if hover.hovered {
                            // Find the parent terrain cell to get its chunk
                            for (_, _, children, chunk) in terrain_cells_query.iter() {
                                if children.contains(&entity) {
                                    /*commands.entity(entity).insert(SolidObject {
                                        health: 0.0,
                                        ..solid_object
                                    });*/
                                    solid_object.health = 0.0;

                                    update_terrain_events.write(UpdateTerrainEvent {
                                        region: None,
                                        chunk_coords: Some((chunk.chunk_x, chunk.chunk_y)),
                                    });

                                    return; // We've handled the click
                                }
                            }
                        }
                    }
                }
            }

            // If we're placing and didn't destroy anything, find a hovered empty cell
            if toolbox_state.action_place_solid {
                let cursor_world_position = get_cursor_world_position(window_query, camera_query);
                let cursor_x = (cursor_world_position.x / CELL_SIZE as f32).round() as i32;
                let cursor_y = (cursor_world_position.y / CELL_SIZE as f32).round() as i32;
                let (cursor_chunk_x, cursor_chunk_y) = ChunkUtils::world_to_chunk_coords(cursor_x, cursor_y);

                for (terrain_entity, transform, children, chunk) in terrain_cells_query.iter() {
                    if chunk.chunk_x != cursor_chunk_x || chunk.chunk_y != cursor_chunk_y {
                        continue;
                    }

                    // Check if this cell is empty
                    if !children.is_empty() {
                        continue;
                    }

                    // Check if this cell is hovered by the cursor
                    let block_size = VEC2_CELL_SIZE;
                    let block_min = Vec2::new(
                        transform.translation.x - block_size.x / 2.0,
                        transform.translation.y - block_size.y / 2.0,
                    );
                    let block_max = Vec2::new(
                        transform.translation.x + block_size.x / 2.0,
                        transform.translation.y + block_size.y / 2.0,
                    );

                    let is_hovered = cursor_world_position.x >= block_min.x
                        && cursor_world_position.x <= block_max.x
                        && cursor_world_position.y >= block_min.y
                        && cursor_world_position.y <= block_max.y;

                    if is_hovered {
                        // Determine which material to place based on toolbox state
                        let material_id = if toolbox_state.solid_rock {
                            "rock"
                        } else if toolbox_state.solid_olivine {
                            "olivine"
                        } else if toolbox_state.solid_basalt {
                            "basalt"
                        } else if toolbox_state.solid_red_crystal {
                            "red_crystal"
                        } else {
                            "rock" // Default
                        }.to_string();

                        // Get material definition
                        if let Some(material_def) = world_materials.materials.get(&material_id) {
                            // Place new solid object
                            commands.entity(terrain_entity).with_children(|cell| {
                                cell.spawn((
                                    Sprite::from_color(material_def.color, VEC2_CELL_SIZE),
                                    SolidObject {
                                        material_id: material_id.clone(),
                                        health: material_def.strength,
                                        mergeable: material_def.can_be_merged,
                                        neighbors_pattern: 0,
                                    },
                                    TerrainChunk {
                                        chunk_x: chunk.chunk_x,
                                        chunk_y: chunk.chunk_y
                                    },
                                ));
                            });

                            update_terrain_events.write(UpdateTerrainEvent {
                                region: None,
                                chunk_coords: Some((chunk.chunk_x, chunk.chunk_y)),
                            });

                            return; // We've handled the click
                        }
                    }
                }
            }
        }
    }
}

fn update_debug_text(
    text_query: &mut Query<Entity, With<DebugHoverText>>,
    writer: &mut TextUiWriter,
    transform: Option<&Transform>,
) {
    let text_entity = text_query.single_mut().unwrap();
    if let Some(transform) = transform {
        let cell_position = Vec2::new(
            transform.translation.x / CELL_SIZE as f32,
            transform.translation.y / CELL_SIZE as f32,
        );
        *writer.text(text_entity, 0) = format!(
            "Hovered cell: ({:.1}, {:.1})",
            cell_position.x, cell_position.y
        );
    } else {
        *writer.text(text_entity, 0) = "Hovered cell: None".to_string();
    }
}

fn reset_solid_objects_hover_state(
    commands: &mut Commands,
    solid_objects_query: &Query<(Entity, Option<&HoverState>, &SolidObject)>,
) {
    for (entity, hover_state, _) in solid_objects_query.iter() {
        if hover_state.is_some() && hover_state.unwrap().hovered {
            commands.entity(entity).remove::<HoverState>();
        }
    }
}
