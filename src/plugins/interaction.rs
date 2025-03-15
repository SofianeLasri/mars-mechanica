use crate::components::{HoverState, SolidObject, UpdateTerrainEvent, CELL_SIZE, VEC2_CELL_SIZE};
use crate::plugins::camera::get_cursor_world_position;
use crate::plugins::debug_text::DebugHoverText;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InteractionPlugin;

#[derive(Component)]
pub struct InteractionSprite;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .add_systems(FixedUpdate, (hover_detection, block_click_handler));
    }
}

pub fn init(mut commands: Commands) {
    let semi_transparent_white = Color::srgba(1.0, 1.0, 1.0, 0.5);
    let sprite = Sprite::from_color(semi_transparent_white, VEC2_CELL_SIZE);
    commands.spawn((sprite, InteractionSprite));
}

/// This system will detect when the cursor is hovering over a block and update the visual effect
pub fn hover_detection(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
    mut solid_objects_query: Query<
        (Entity, &Transform, Option<&HoverState>, &SolidObject),
        With<SolidObject>,
    >,
    mut interaction_sprite_query: Query<Entity, With<InteractionSprite>>,
    mut text_query: Query<Entity, With<DebugHoverText>>,
    mut writer: TextUiWriter,
) {
    let interaction_sprite = interaction_sprite_query.single_mut();
    let cursor_world_position = get_cursor_world_position(window_query, camera_query);

    reset_solid_objects_hover_state(&mut commands, &mut solid_objects_query);

    let mut a_block_has_been_hovered = false;

    for (entity, transform, hover_state, _solid_object) in solid_objects_query.iter_mut() {
        let block_size = VEC2_CELL_SIZE;
        let block_min = Vec2::new(
            transform.translation.x - block_size.x / 2.0,
            transform.translation.y - block_size.y / 2.0,
        );
        let block_max = Vec2::new(
            transform.translation.x + block_size.x / 2.0,
            transform.translation.y + block_size.y / 2.0,
        );

        let block_is_hovered_by_cursor = cursor_world_position.x >= block_min.x
            && cursor_world_position.x <= block_max.x
            && cursor_world_position.y >= block_min.y
            && cursor_world_position.y <= block_max.y;

        if block_is_hovered_by_cursor {
            a_block_has_been_hovered = true;
            block_hover_action(
                &mut commands,
                &mut text_query,
                &mut writer,
                interaction_sprite,
                entity,
                transform,
                hover_state,
            );
        }
    }

    if !a_block_has_been_hovered {
        commands
            .entity(interaction_sprite)
            .insert(Transform::from_xyz(
                0.0, 0.0, -100.0, // Z-index plus bas que les blocs
            ));
        update_debug_text(&mut text_query, &mut writer, None);
    }
}

/// This method handles the visual effect when a block is hovered by the cursor
fn block_hover_action(
    commands: &mut Commands,
    text_query: &mut Query<Entity, With<DebugHoverText>>,
    writer: &mut TextUiWriter,
    interaction_sprite: Entity,
    entity: Entity,
    transform: &Transform,
    hover_state: Option<&HoverState>,
) {
    update_debug_text(text_query, writer, Some(transform));

    commands
        .entity(interaction_sprite)
        .insert(Transform::from_xyz(
            transform.translation.x,
            transform.translation.y,
            100.0, // Z-index plus élevé que les blocs
        ));

    // Ajouter ou mettre à jour le composant HoverState
    if hover_state.is_none() || !hover_state.unwrap().hovered {
        commands.entity(entity).insert(HoverState { hovered: true });
    }
}

fn update_debug_text(
    text_query: &mut Query<Entity, With<DebugHoverText>>,
    writer: &mut TextUiWriter,
    transform: Option<&Transform>,
) {
    if let Some(transform) = transform {
        let cell_position = Vec2::new(
            transform.translation.x / CELL_SIZE as f32,
            transform.translation.y / CELL_SIZE as f32,
        );
        *writer.text(text_query.single_mut(), 0) = format!(
            "Hovered cell: ({:.1}, {:.1})",
            cell_position.x, cell_position.y
        );
    } else {
        *writer.text(text_query.single_mut(), 0) = "Hovered cell: None".to_string();
    }
}

fn reset_solid_objects_hover_state(
    commands: &mut Commands,
    solid_objects_query: &mut Query<
        (Entity, &Transform, Option<&HoverState>, &SolidObject),
        With<SolidObject>,
    >,
) {
    for (entity, _, hover_state, _solid_object) in solid_objects_query.iter_mut() {
        if hover_state.is_some() && hover_state.unwrap().hovered {
            commands.entity(entity).remove::<HoverState>();
        }
    }
}

// Système pour gérer les clics sur les blocs
pub fn block_click_handler(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut solid_objects_query: Query<(Entity, &mut SolidObject, Option<&HoverState>)>,
    mut update_terrain_events: EventWriter<UpdateTerrainEvent>,
) {
    for event in mouse_button_events.read() {
        // Vérifier si c'est un clic gauche
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            // Trouver et détruire le bloc survolé
            for (_entity, mut solid_object, hover_state) in solid_objects_query.iter_mut() {
                if let Some(hover) = hover_state {
                    if hover.hovered {
                        // Détruire le bloc en réduisant sa santé à 0
                        solid_object.health = 0.0;

                        // Déclencher une mise à jour du terrain
                        update_terrain_events.send(UpdateTerrainEvent {
                            region: None, // Mettre à jour tout le terrain
                        });

                        break; // Ne détruire qu'un seul bloc par clic
                    }
                }
            }
        }
    }
}
