use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;
use crate::components::{EntityDefinition, MaterialDefinition, SolidObject, WorldEntities, WorldMaterials, NEIGHBOR_BOTTOM, NEIGHBOR_BOTTOM_LEFT, NEIGHBOR_BOTTOM_RIGHT, NEIGHBOR_LEFT, NEIGHBOR_RIGHT, NEIGHBOR_TOP, NEIGHBOR_TOP_LEFT, NEIGHBOR_TOP_RIGHT};

pub struct TerrainPlugin;

const CELL_SIZE: i32 = 64;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMaterials>()
            .init_resource::<WorldEntities>()
            .add_systems(Startup, init_world_definitions)
            .add_systems(
                Update,
                (
                    update_solid_objects,
                    update_neighbors_pattern,
                    update_material_textures,
                    update_sprite_rotations,
                ),
            );
    }
}

// Initialise les définitions des matériaux et entités
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

// Système pour mettre à jour les objets solides (détruire si health = 0)
fn update_solid_objects(
    mut commands: Commands,
    mut solid_objects: Query<(Entity, &SolidObject)>,
    world_materials: Res<WorldMaterials>,
    world_entities: Res<WorldEntities>,
) {
    for (entity, solid_object) in solid_objects.iter() {
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

            commands.entity(entity).despawn();
        }
    }
}

// Système pour mettre à jour les motifs de voisinage de chaque objet solide
fn update_neighbors_pattern(
    mut solid_objects_query: Query<(Entity, &mut SolidObject, &Transform)>,
) {
    // Collecte d'abord toutes les positions et matériaux pour une recherche efficace
    let positions: Vec<(i32, i32, Entity, String)> = solid_objects_query
        .iter()
        .map(|(entity, solid_object, transform)| {
            let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
            let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
            (x, y, entity, solid_object.material_id.clone())
        })
        .collect();

    // Met à jour chaque objet avec son motif de voisinage
    for (entity, mut solid_object, transform) in solid_objects_query.iter_mut() {
        if !solid_object.mergeable {
            continue; // Skip les objets non fusionnables
        }

        let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
        let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
        let material_id = &solid_object.material_id;

        let mut pattern: u8 = 0;

        // Vérification des 8 directions pour les voisins
        // Droite
        if positions.iter().any(|(px, py, e, mat)|
            *px == x + 1 && *py == y && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_RIGHT;
        }
        // Haut-Droite
        if positions.iter().any(|(px, py, e, mat)|
            *px == x + 1 && *py == y + 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_TOP_RIGHT;
        }
        // Haut
        if positions.iter().any(|(px, py, e, mat)|
            *px == x && *py == y + 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_TOP;
        }
        // Haut-Gauche
        if positions.iter().any(|(px, py, e, mat)|
            *px == x - 1 && *py == y + 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_TOP_LEFT;
        }
        // Gauche
        if positions.iter().any(|(px, py, e, mat)|
            *px == x - 1 && *py == y && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_LEFT;
        }
        // Bas-Gauche
        if positions.iter().any(|(px, py, e, mat)|
            *px == x - 1 && *py == y - 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_BOTTOM_LEFT;
        }
        // Bas
        if positions.iter().any(|(px, py, e, mat)|
            *px == x && *py == y - 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_BOTTOM;
        }
        // Bas-Droite
        if positions.iter().any(|(px, py, e, mat)|
            *px == x + 1 && *py == y - 1 && *e != entity && mat == material_id) {
            pattern |= NEIGHBOR_BOTTOM_RIGHT;
        }

        solid_object.neighbors_pattern = pattern;
    }
}

// Vérifie s'il y a un voisin à une position donnée avec le même matériau
fn has_neighbor_at(
    positions: &[(Entity, (i32, i32), &SolidObject)],
    x: i32,
    y: i32,
    material_id: &str,
) -> bool {
    positions
        .iter()
        .any(|(_, pos, obj)| pos.0 == x && pos.1 == y && obj.material_id == material_id)
}

// Système pour mettre à jour les textures des matériaux en fonction des voisins
fn update_material_textures(
    mut commands: Commands,
    solid_objects: Query<(Entity, &SolidObject), Changed<SolidObject>>,
    world_materials: Res<WorldMaterials>,
) {
    for (entity, solid_object) in solid_objects.iter() {
        if let Some(texture) = solid_object.get_texture(&world_materials) {
            // Met à jour la texture de l'entité
            //commands.entity(entity).insert(texture);
        }
    }
}

// Système pour gérer les rotations de sprite en fonction du pattern de voisinage
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
