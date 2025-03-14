use bevy::prelude::*;
use crate::components::{SolidObject, HoverState, UpdateTerrainEvent, WorldMaterials};
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ButtonState;
use bevy::window::PrimaryWindow;
use crate::systems::{DebugCameraText, DebugHoverText, CELL_SIZE};

// Système pour détecter le survol des blocs et appliquer un effet visuel
pub fn hover_detection(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut solid_objects_query: Query<(Entity, &Transform, &mut Sprite, Option<&HoverState>, &SolidObject), With<SolidObject>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    world_materials: Res<WorldMaterials>,
    mut text_query: Query<Entity, With<DebugHoverText>>,
    mut writer: TextUiWriter
) {
    // Récupérer la position de la souris
    let window = window_query.single();
    let window_size = Vec2::new(window.width(), window.height());
    let cursor_position = window.cursor_position().unwrap_or(window_size / 2.0) - window_size / 2.0;

    let (camera, camera_transform) = camera_query.single();

    //let cursor_world_position = camera_transform.translation().truncate() + cursor_position;
    // Y is inverted
    // TODO: Gérer le dézoom de la caméra
    let cursor_world_position = Vec2::new(
        camera_transform.translation().x + cursor_position.x,
        camera_transform.translation().y - cursor_position.y,
    );

    // Réinitialiser tous les états de survol
    for (entity, _, mut sprite, hover_state, solid_object) in solid_objects_query.iter_mut() {
        if hover_state.is_some() && hover_state.unwrap().hovered {
            if let Some(texture) = solid_object.get_texture(&world_materials) {
                let mut sprite = Sprite::from_image(texture);
                sprite.custom_size = Some(Vec2::new(CELL_SIZE as f32, CELL_SIZE as f32));
                commands.entity(entity).insert(sprite);
            }

            // Supprimer le composant HoverState
            commands.entity(entity).remove::<HoverState>();
        }
    }

    // Détecter quels blocs sont sous la souris
    for (entity, transform, mut sprite, hover_state, solid_object) in solid_objects_query.iter_mut() {
        // Vérifier si la position de la souris est dans le bloc
        let block_size = Vec2::new(CELL_SIZE as f32, CELL_SIZE as f32); // Utiliser CELL_SIZE
        let block_min = Vec2::new(
            transform.translation.x - block_size.x / 2.0,
            transform.translation.y - block_size.y / 2.0,
        );
        let block_max = Vec2::new(
            transform.translation.x + block_size.x / 2.0,
            transform.translation.y + block_size.y / 2.0,
        );

        if cursor_world_position.x >= block_min.x && cursor_world_position.x <= block_max.x &&
            cursor_world_position.y >= block_min.y && cursor_world_position.y <= block_max.y {
            // Debug Hover Text
            let cell_position = Vec2::new(transform.translation.x / CELL_SIZE as f32, transform.translation.y / CELL_SIZE as f32);
            *writer.text(text_query.single_mut(), 0) = format!("Hovered cell: ({:.1}, {:.1})", cell_position.x, cell_position.y);

            // Bloc survolé, appliquer l'effet visuel (overlay gris clair)
            //sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.0);
            commands.entity(entity).insert(Sprite::from_color(Color::WHITE, Vec2::new(CELL_SIZE as f32, CELL_SIZE as f32)));

            // Ajouter ou mettre à jour le composant HoverState
            if hover_state.is_none() || !hover_state.unwrap().hovered {
                commands.entity(entity).insert(HoverState { hovered: true });
            }
        }
    }
}

// Système pour gérer les clics sur les blocs
pub fn block_click_handler(
    mut commands: Commands,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut solid_objects_query: Query<(Entity, &mut SolidObject, Option<&HoverState>)>,
    mut update_terrain_events: EventWriter<UpdateTerrainEvent>,
) {
    for event in mouse_button_events.read() {
        // Vérifier si c'est un clic gauche
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            // Trouver et détruire le bloc survolé
            for (entity, mut solid_object, hover_state) in solid_objects_query.iter_mut() {
                if let Some(hover) = hover_state {
                    if hover.hovered {
                        // Détruire le bloc en réduisant sa santé à 0
                        solid_object.health = 0.0;

                        // Déclencher une mise à jour du terrain
                        update_terrain_events.send(UpdateTerrainEvent {
                            region: None // Mettre à jour tout le terrain
                        });

                        break; // Ne détruire qu'un seul bloc par clic
                    }
                }
            }
        }
    }
}
