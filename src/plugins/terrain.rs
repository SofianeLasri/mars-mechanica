use crate::components::{
    ChunkMap, ChunkUtils, EntityDefinition, MaterialDefinition, SolidObject, UpdateTerrainEvent,
    WorldEntities, WorldMaterials, CELL_SIZE, CHUNK_SIZE, NEIGHBOR_BOTTOM,
    NEIGHBOR_BOTTOM_LEFT, NEIGHBOR_BOTTOM_RIGHT, NEIGHBOR_LEFT, NEIGHBOR_RIGHT, NEIGHBOR_TOP,
    NEIGHBOR_TOP_LEFT, NEIGHBOR_TOP_RIGHT, VEC2_CELL_SIZE,
};
use bevy::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMaterials>()
            .init_resource::<WorldEntities>()
            .init_resource::<ChunkMap>()
            .add_event::<UpdateTerrainEvent>()
            .add_systems(PreStartup, init_world_definitions)
            .add_systems(
                PostStartup,
                (
                    update_neighbors_pattern,
                    update_material_textures,
                    update_sprite_rotations,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_solid_objects,
                    update_neighbors_pattern.run_if(on_event::<UpdateTerrainEvent>),
                    update_material_textures.run_if(on_event::<UpdateTerrainEvent>),
                    update_sprite_rotations.run_if(on_event::<UpdateTerrainEvent>),
                ),
            );
    }
}

/// Initialise les définitions des matériaux et entités.
/// Doit être exécuté avant tout autre système, dans la phase PreStartup !
fn init_world_definitions(
    mut world_materials: ResMut<WorldMaterials>,
    mut world_entities: ResMut<WorldEntities>,
    asset_server: Res<AssetServer>,
) {
    // Initialiser les matériaux martiens
    let mut materials = HashMap::new();

    materials.insert(
        "rock".to_string(),
        MaterialDefinition {
            name: "Martian Rock".to_string(),
            strength: 2.0,
            drop_entity_id: "rock_item".to_string(),
            drop_count_min: 1,
            drop_count_max: 3,
            can_be_merged: true,
            rarity: 0.0, // Très commun
            plain_texture: asset_server.load("textures/terrain/rock/plain.png"),
            side_texture: asset_server.load("textures/terrain/rock/side.png"),
            inter_corner_texture: asset_server.load("textures/terrain/rock/inter-corner.png"),
            outer_corner_texture: asset_server.load("textures/terrain/rock/outer-corner.png"),
        },
    );

    materials.insert(
        "basalt".to_string(),
        MaterialDefinition {
            name: "Basalt".to_string(),
            strength: 3.0,
            drop_entity_id: "basalt_item".to_string(),
            drop_count_min: 1,
            drop_count_max: 2,
            can_be_merged: true,
            rarity: 0.3, // Assez rare
            plain_texture: asset_server.load("textures/terrain/basalt/plain.png"),
            side_texture: asset_server.load("textures/terrain/basalt/side.png"),
            inter_corner_texture: asset_server.load("textures/terrain/basalt/inter-corner.png"),
            outer_corner_texture: asset_server.load("textures/terrain/basalt/outer-corner.png"),
        },
    );

    materials.insert(
        "olivine".to_string(),
        MaterialDefinition {
            name: "Olivine".to_string(),
            strength: 4.0,
            drop_entity_id: "olivine_item".to_string(),
            drop_count_min: 1,
            drop_count_max: 1,
            can_be_merged: false, // Apparaît toujours comme des cristaux individuels
            rarity: 0.7,          // Très rare
            plain_texture: asset_server.load("textures/terrain/olivine/plain.png"),
            side_texture: asset_server.load("textures/terrain/olivine/side.png"),
            inter_corner_texture: asset_server.load("textures/terrain/olivine/inter-corner.png"),
            outer_corner_texture: asset_server.load("textures/terrain/olivine/outer-corner.png"),
        },
    );

    world_materials.materials = materials;

    // Initialiser les entités
    let mut entities = HashMap::new();

    entities.insert(
        "rock_item".to_string(),
        EntityDefinition {
            name: "Martian Rock".to_string(),
            icon: asset_server.load("textures/items/rock.png"),
            max_stack: 64,
        },
    );

    entities.insert(
        "basalt_item".to_string(),
        EntityDefinition {
            name: "Basalt".to_string(),
            icon: asset_server.load("textures/items/basalt.png"),
            max_stack: 32,
        },
    );

    entities.insert(
        "olivine_item".to_string(),
        EntityDefinition {
            name: "Olivine Crystal".to_string(),
            icon: asset_server.load("textures/items/olivine.png"),
            max_stack: 16,
        },
    );

    world_entities.entities = entities;
}

/// Système pour mettre à jour les objets solides (détruire si health = 0)
fn update_solid_objects(
    mut commands: Commands,
    solid_objects: Query<(Entity, &SolidObject, &Transform)>,
    world_materials: Res<WorldMaterials>,
    world_entities: Res<WorldEntities>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    for (entity, solid_object, transform) in solid_objects.iter() {
        if solid_object.health <= 0.0 {
            // Logique pour faire apparaître des items lorsqu'un objet est détruit
            if let Some(material) = world_materials.materials.get(&solid_object.material_id) {
                if let Some(_entity_def) = world_entities.entities.get(&material.drop_entity_id) {
                    // Déterminer combien d'objets vont être droppés
                    let drop_count = if material.drop_count_min == material.drop_count_max {
                        material.drop_count_min
                    } else {
                        let mut rng = rand::thread_rng();
                        rng.gen_range(material.drop_count_min..=material.drop_count_max)
                    };

                    // Logique pour créer les items drops
                    // ...
                }
            }

            // Supprimer l'entité du chunk map
            let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
            let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
            let (chunk_x, chunk_y) = ChunkUtils::world_to_chunk_coords(x, y);

            if let Some(entities) = chunk_map.chunks.get_mut(&(chunk_x, chunk_y)) {
                entities.remove(&entity);
            }

            commands.entity(entity).despawn();
        }
    }
}

/// This method updates the neighbors pattern for all solid objects
fn update_neighbors_pattern(
    mut solid_objects_query: Query<(Entity, &mut SolidObject, &Transform)>,
    mut event_reader: EventReader<UpdateTerrainEvent>,
    chunk_map: Res<ChunkMap>,
) {
    info!("Update neighbors pattern");

    let chunks_to_update = find_chunks_to_update(&mut event_reader, &chunk_map);

    if chunks_to_update.is_empty() {
        return;
    }

    let mut entities_to_update: HashSet<Entity> = HashSet::new();
    for chunk_coords in &chunks_to_update {
        info!("Updating chunk {:?}", chunk_coords);
        if let Some(entities) = chunk_map.chunks.get(chunk_coords) {
            entities_to_update.extend(entities);
        }
    }

    let positions: Vec<(i32, i32, Entity, String)> = solid_objects_query
        .iter()
        .filter(|(entity, _, _)| chunks_to_update.is_empty() || entities_to_update.contains(entity))
        .map(|(entity, solid_object, transform)| {
            let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
            let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
            (x, y, entity, solid_object.material_id.clone())
        })
        .collect();

    for (entity, mut solid_object, transform) in solid_objects_query.iter_mut() {
        if !solid_object.mergeable {
            continue;
        }

        let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
        let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
        let (chunk_x, chunk_y) = ChunkUtils::world_to_chunk_coords(x, y);

        if !chunks_to_update.is_empty() && !chunks_to_update.contains(&(chunk_x, chunk_y)) {
            continue;
        }

        let material_id = &solid_object.material_id;

        let pattern = get_neighbors_pattern(&positions, entity, x, y, material_id);

        solid_object.neighbors_pattern = pattern;
    }
    info!("Neighbors pattern updated");
}

/// This method returns the pattern of neighbors for a given bloc/Entity at position (x, y).
///
/// It takes care of checking the 8 directions for neighbors and returns a u8 pattern.
///
/// It also checks if the neighbors are of the same material.
fn get_neighbors_pattern(positions: &Vec<(i32, i32, Entity, String)>, entity: Entity, x: i32, y: i32, material_id: &String) -> u8 {
    let mut pattern: u8 = 0;

    // Vérification des 8 directions pour les voisins
    // Droite
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x + 1 && *py == y && *e != entity && mat == material_id)
    {
        pattern |= NEIGHBOR_RIGHT;
    }
    // Haut-Droite
    if positions.iter().any(|(px, py, e, mat)| {
        *px == x + 1 && *py == y + 1 && *e != entity && mat == material_id
    }) {
        pattern |= NEIGHBOR_TOP_RIGHT;
    }
    // Haut
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x && *py == y + 1 && *e != entity && mat == material_id)
    {
        pattern |= NEIGHBOR_TOP;
    }
    // Haut-Gauche
    if positions.iter().any(|(px, py, e, mat)| {
        *px == x - 1 && *py == y + 1 && *e != entity && mat == material_id
    }) {
        pattern |= NEIGHBOR_TOP_LEFT;
    }
    // Gauche
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x - 1 && *py == y && *e != entity && mat == material_id)
    {
        pattern |= NEIGHBOR_LEFT;
    }
    // Bas-Gauche
    if positions.iter().any(|(px, py, e, mat)| {
        *px == x - 1 && *py == y - 1 && *e != entity && mat == material_id
    }) {
        pattern |= NEIGHBOR_BOTTOM_LEFT;
    }
    // Bas
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x && *py == y - 1 && *e != entity && mat == material_id)
    {
        pattern |= NEIGHBOR_BOTTOM;
    }
    // Bas-Droite
    if positions.iter().any(|(px, py, e, mat)| {
        *px == x + 1 && *py == y - 1 && *e != entity && mat == material_id
    }) {
        pattern |= NEIGHBOR_BOTTOM_RIGHT;
    }
    pattern
}

/// This method finds the chunks to update based on the given event
fn find_chunks_to_update(event_reader: &mut EventReader<UpdateTerrainEvent>, chunk_map: &Res<ChunkMap>) -> HashSet<(i32, i32)> {
    let mut chunks_to_update = HashSet::new();

    for event in event_reader.read() {
        if let Some(chunk_coords) = event.chunk_coords {
            info!("Update chunk {:?}", chunk_coords);
            for neighbor_chunk in ChunkUtils::get_neighbor_chunks(chunk_coords.0, chunk_coords.1) {
                chunks_to_update.insert(neighbor_chunk);
            }
        } else if let Some(region) = event.region {
            let min_x = (region.0.x / (CHUNK_SIZE * CELL_SIZE) as f32).floor() as i32;
            let min_y = (region.0.y / (CHUNK_SIZE * CELL_SIZE) as f32).floor() as i32;
            let max_x = (region.1.x / (CHUNK_SIZE * CELL_SIZE) as f32).ceil() as i32;
            let max_y = (region.1.y / (CHUNK_SIZE * CELL_SIZE) as f32).ceil() as i32;

            info!("Update region: ({}, {}) to ({}, {})", min_x, min_y, max_x, max_y);
            for chunk_x in min_x..=max_x {
                for chunk_y in min_y..=max_y {
                    for neighbor_chunk in ChunkUtils::get_neighbor_chunks(chunk_x, chunk_y) {
                        chunks_to_update.insert(neighbor_chunk);
                    }
                }
            }
        } else {
            info!("Update all chunks");
            for &chunk_coords in chunk_map.chunks.keys() {
                chunks_to_update.insert(chunk_coords);
            }
        }
    }
    chunks_to_update
}

/// Système pour mettre à jour les textures des matériaux en fonction des voisins
fn update_material_textures(
    mut commands: Commands,
    solid_objects: Query<(Entity, &SolidObject), Changed<SolidObject>>,
    world_materials: Res<WorldMaterials>,
) {
    for (entity, solid_object) in solid_objects.iter() {
        if let Some(texture) = solid_object.get_texture(&world_materials) {
            // L'ancien sprite est remplacé par le nouveau
            let mut sprite = Sprite::from_image(texture);
            sprite.custom_size = Some(VEC2_CELL_SIZE);
            commands.entity(entity).insert(sprite);
        }
    }
}

/// Système pour gérer les rotations de sprite en fonction du pattern de voisinage
fn update_sprite_rotations(mut query: Query<(&SolidObject, &mut Transform), Changed<SolidObject>>) {
    for (solid_object, mut transform) in query.iter_mut() {
        // Si l'objet n'est pas fusionnable, garde sa rotation par défaut
        if !solid_object.mergeable {
            continue;
        }

        // Définis la rotation en fonction du pattern de voisinage
        let pattern = solid_object.neighbors_pattern;

        if solid_object.is_side() {
            // Rotation pour les côtés
            match pattern {
                NEIGHBOR_RIGHT => transform.rotation = Quat::from_rotation_z(0.0),
                NEIGHBOR_TOP => {
                    transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)
                }
                NEIGHBOR_LEFT => transform.rotation = Quat::from_rotation_z(std::f32::consts::PI),
                NEIGHBOR_BOTTOM => {
                    transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                }
                _ => {}
            }
        } else if solid_object.is_outer_corner() {
            // Rotation pour les coins extérieurs
            match pattern {
                NEIGHBOR_TOP_RIGHT => transform.rotation = Quat::from_rotation_z(0.0),
                NEIGHBOR_TOP_LEFT => {
                    transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)
                }
                NEIGHBOR_BOTTOM_LEFT => {
                    transform.rotation = Quat::from_rotation_z(std::f32::consts::PI)
                }
                NEIGHBOR_BOTTOM_RIGHT => {
                    transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                }
                _ => {}
            }
        } else if solid_object.is_inner_corner() {
            // Rotation pour les coins intérieurs
            if (pattern & (NEIGHBOR_RIGHT | NEIGHBOR_TOP) == (NEIGHBOR_RIGHT | NEIGHBOR_TOP))
                && (pattern & NEIGHBOR_TOP_RIGHT == 0)
            {
                transform.rotation = Quat::from_rotation_z(0.0);
            } else if (pattern & (NEIGHBOR_TOP | NEIGHBOR_LEFT) == (NEIGHBOR_TOP | NEIGHBOR_LEFT))
                && (pattern & NEIGHBOR_TOP_LEFT == 0)
            {
                transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            } else if (pattern & (NEIGHBOR_LEFT | NEIGHBOR_BOTTOM)
                == (NEIGHBOR_LEFT | NEIGHBOR_BOTTOM))
                && (pattern & NEIGHBOR_BOTTOM_LEFT == 0)
            {
                transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
            } else if (pattern & (NEIGHBOR_BOTTOM | NEIGHBOR_RIGHT)
                == (NEIGHBOR_BOTTOM | NEIGHBOR_RIGHT))
                && (pattern & NEIGHBOR_BOTTOM_RIGHT == 0)
            {
                transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            }
        }
    }
}

pub fn trigger_terrain_update(
    chunk_x: i32,
    chunk_y: i32,
    mut event_writer: EventWriter<UpdateTerrainEvent>,
) {
    event_writer.send(UpdateTerrainEvent {
        region: None,
        chunk_coords: Some((chunk_x, chunk_y)),
    });
}
