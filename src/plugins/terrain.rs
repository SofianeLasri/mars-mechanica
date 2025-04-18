use crate::components::{ChunkMap, ChunkUtils, ControlledCamera, EntitiesToDespawn, EntityDefinition, ItemText, MaskOverlay, MaterialDefinition, SolidObject, TerrainAssets, TerrainCell, TerrainChunk, UpdateTerrainEvent, WorldEntities, WorldEntityItem, WorldMaterials, CELL_SIZE, CHUNK_SIZE, MASK_THICKNESS, NEIGHBOR_BOTTOM, NEIGHBOR_BOTTOM_LEFT, NEIGHBOR_BOTTOM_RIGHT, NEIGHBOR_LEFT, NEIGHBOR_RIGHT, NEIGHBOR_TOP, NEIGHBOR_TOP_LEFT, NEIGHBOR_TOP_RIGHT, VEC2_CELL_SIZE};
use crate::GameState;
use bevy::prelude::*;
use bevy::text::{JustifyText, TextColor, TextFont, TextLayout};
use bevy_sprite::Anchor;
use rand::Rng;
use std::collections::{HashMap, HashSet};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMaterials>()
            .init_resource::<WorldEntities>()
            .init_resource::<ChunkMap>()
            .init_resource::<EntitiesToDespawn>()
            .add_event::<UpdateTerrainEvent>()
            .add_systems(PreStartup, init_world_definitions)
            .add_systems(
                FixedUpdate,
                (
                    update_solid_objects.run_if(in_state(GameState::InGame)),
                    update_neighbors_pattern.run_if(on_event::<UpdateTerrainEvent>),
                    update_material_textures
                        .run_if(on_event::<UpdateTerrainEvent>)
                        .after(update_neighbors_pattern),
                    apply_despawns.after(update_material_textures),
                    update_item_text_visibility.run_if(in_state(GameState::InGame)),
                ),
            );
    }
}

/// This method initialises the world definitions
/// It loads the materials and entities from the asset server.
///
/// **Note:** This method should be executed before any other system, in the PreStartup phase!
fn init_world_definitions(
    mut world_materials: ResMut<WorldMaterials>,
    mut world_entities: ResMut<WorldEntities>,
    asset_server: Res<AssetServer>,
    terrain_assets: Res<TerrainAssets>,
) {
    info!("Initialising world definitions...");

    info!("Loading materials...");
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
            sprites: load_material_sprites(&terrain_assets, "rock"),
            color: Color::srgb(85.0 / 255.0, 51.0 / 255.0, 36.0 / 255.0), // #553324
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
            rarity: 0.6, // Assez rare
            sprites: load_material_sprites(&terrain_assets, "basalt"),
            color: Color::srgb(47.0 / 255.0, 47.0 / 255.0, 47.0 / 255.0), // #2F2F2F
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
            can_be_merged: true, // Apparaît toujours comme des cristaux individuels
            rarity: 0.8,         // Très rare
            sprites: load_material_sprites(&terrain_assets, "olivine"),
            color: Color::srgb(33.0 / 255.0, 72.0 / 255.0, 40.0 / 255.0), // #214828
        },
    );

    materials.insert(
        "red_crystal".to_string(),
        MaterialDefinition {
            name: "Red crystal".to_string(),
            strength: 4.0,
            drop_entity_id: "red_crystal_item".to_string(),
            drop_count_min: 1,
            drop_count_max: 1,
            can_be_merged: false, // Apparaît toujours comme des cristaux individuels
            rarity: 0.95,         // Très très rare
            sprites: load_material_sprites(&terrain_assets, "red_crystal"),
            color: Color::srgb(189.0 / 255.0, 36.0 / 255.0, 36.0 / 255.0), // #bd2424
        },
    );

    world_materials.materials = materials;
    info!("World materials initialised");

    info!("Loading entities...");
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

    entities.insert(
        "red_crystal_item".to_string(),
        EntityDefinition {
            name: "Red Crystal".to_string(),
            icon: asset_server.load("textures/items/red_crystal.png"),
            max_stack: 8,
        },
    );

    world_entities.entities = entities;
    info!("World entities initialised");
    info!("World definitions initialised");
}

/// This method loads the material sprites from the asset server
pub fn load_material_sprites(
    terrain_assets: &Res<TerrainAssets>,
    material_id: &str,
) -> HashMap<String, Handle<Image>> {
    terrain_assets.materials.get(material_id).cloned().unwrap_or_default()
}

/// Système pour mettre à jour les objets solides (détruire si health = 0)
fn update_solid_objects(
    mut commands: Commands,
    terrain_cells_query: Query<(Entity, &Transform, &Children, &TerrainChunk)>,
    solid_objects_query: Query<(Entity, &SolidObject)>,
    world_materials: Res<WorldMaterials>,
    world_entities: Res<WorldEntities>,
    mut chunk_map: ResMut<ChunkMap>,
    asset_server: Res<AssetServer>,
    mut to_despawn: ResMut<EntitiesToDespawn>,
) {
    let mut to_destroy: Vec<(Entity, Entity, &Transform, &TerrainChunk)> = Vec::new();

    for (parent_entity, parent_transform, children, chunk) in terrain_cells_query.iter() {
        for child in children.iter() {
            if let Ok((child_entity, solid_object)) = solid_objects_query.get(child) {
                if solid_object.health <= 0.0 {
                    to_destroy.push((child_entity, parent_entity, parent_transform, chunk));
                }
            }
        }
    }

    for (child_entity, _parent_entity, parent_transform, chunk) in to_destroy {
        if let Ok((_, solid_object)) = solid_objects_query.get(child_entity) {
            if let Some(material) = world_materials.materials.get(&solid_object.material_id) {
                if let Some(entity_def) = world_entities.entities.get(&material.drop_entity_id) {
                    let drop_count = if material.drop_count_min == material.drop_count_max {
                        material.drop_count_min
                    } else {
                        let mut rng = rand::rng();
                        rng.random_range(material.drop_count_min..=material.drop_count_max)
                    };

                    commands
                        .spawn((
                            Sprite {
                                image: entity_def.icon.clone(),
                                custom_size: Some(Vec2::splat(64.0)),
                                ..Default::default()
                            },
                            Transform::from_translation(parent_transform.translation),
                            Visibility::Visible,
                            WorldEntityItem {
                                entity_id: material.drop_entity_id.clone(),
                                quantity: drop_count,
                            },
                        ));

                    let text_position = Vec3::new(
                        parent_transform.translation.x,
                        parent_transform.translation.y - (CELL_SIZE as f32 / 2.0) + 10.0,
                        parent_transform.translation.z + 1.0,
                    );

                    let text = drop_count.to_string();

                    commands.spawn((
                        Text2d::new(text),
                        TextFont {
                            font: asset_server.load("fonts/inter-regular.ttf"),
                            font_size: 14.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Anchor::BottomCenter,
                        Transform::from_translation(text_position),
                        Visibility::Hidden,
                        ItemText,
                    ));
                }
            }

            info!("Planing despawn for solid object entity {:?}", child_entity);
            to_despawn.0.push(child_entity);

            let (chunk_x, chunk_y) = (chunk.chunk_x, chunk.chunk_y);
            if let Some(_entities) = chunk_map.chunks.get_mut(&(chunk_x, chunk_y)) {
                // _entities.remove(&child_entity);

                info!(
                    "Entity {:?} processed in chunk {:?}",
                    child_entity,
                    (chunk_x, chunk_y)
                );
            }
        }
    }
}

/// This method updates the neighbors pattern for all solid objects
fn update_neighbors_pattern(
    terrain_cells_query: Query<(Entity, &Transform, &TerrainCell, &Children)>,
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

    let positions: Vec<(i32, i32, Entity, String)> = terrain_cells_query
        .iter()
        .filter(|(entity, _, _, _)| chunks_to_update.is_empty() || entities_to_update.contains(entity))
        .filter_map(|(_, transform, _, children)| {
            let child = children.iter().next()?;
            let solid_object = solid_objects_query.get(child).ok()?;

            let x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
            let y = (transform.translation.y / CELL_SIZE as f32).round() as i32;

            Some((x, y, child, solid_object.1.material_id.clone()))
        })
        .collect();


    for (_parent_entity, parent_transform, _, children) in terrain_cells_query.iter() {
        let x = (parent_transform.translation.x / CELL_SIZE as f32).round() as i32;
        let y = (parent_transform.translation.y / CELL_SIZE as f32).round() as i32;
        let (chunk_x, chunk_y) = ChunkUtils::world_to_chunk_coords(x, y);

        if !chunks_to_update.is_empty() && !chunks_to_update.contains(&(chunk_x, chunk_y)) {
            continue;
        }

        for child in children.iter() {
            if let Ok((entity, mut solid_object, _)) = solid_objects_query.get_mut(child) {
                if !solid_object.mergeable {
                    continue;
                }

                let material_id = &solid_object.material_id;
                let pattern = get_neighbors_pattern(&positions, entity, x, y, material_id);
                solid_object.neighbors_pattern = pattern;
            }
        }
    }
    info!("Neighbors pattern updated");
}

/// This method returns the pattern of neighbors for a given bloc/Entity at position (x, y).
///
/// It takes care of checking the 8 directions for neighbors and returns a u8 pattern.
///
/// It also checks if the neighbors are of the same material.
fn get_neighbors_pattern(
    positions: &Vec<(i32, i32, Entity, String)>,
    entity: Entity,
    x: i32,
    y: i32,
    material_id: &String,
) -> u8 {
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
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x + 1 && *py == y + 1 && *e != entity && mat == material_id)
    {
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
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x - 1 && *py == y + 1 && *e != entity && mat == material_id)
    {
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
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x - 1 && *py == y - 1 && *e != entity && mat == material_id)
    {
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
    if positions
        .iter()
        .any(|(px, py, e, mat)| *px == x + 1 && *py == y - 1 && *e != entity && mat == material_id)
    {
        pattern |= NEIGHBOR_BOTTOM_RIGHT;
    }
    pattern
}

/// This method finds the chunks to update based on the given event
fn find_chunks_to_update(
    event_reader: &mut EventReader<UpdateTerrainEvent>,
    chunk_map: &Res<ChunkMap>,
) -> HashSet<(i32, i32)> {
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

            info!(
                "Update region: ({}, {}) to ({}, {})",
                min_x, min_y, max_x, max_y
            );
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

/// This system updates the material textures for the given solid objects
fn update_material_textures(
    mut commands: Commands,
    solid_objects: Query<(Entity, &SolidObject), Changed<SolidObject>>,
    world_materials: Res<WorldMaterials>,
    children_query: Query<&Children>,
    mask_overlay_query: Query<(), With<MaskOverlay>>,
) {
    info!("Update material textures");
    for (entity, solid_object) in solid_objects.iter() {
        let material_def = match world_materials.materials.get(&solid_object.material_id) {
            Some(mat) => mat,
            None => continue,
        };

        if let Some(texture) = solid_object.get_texture(&world_materials) {
            let mut sprite = Sprite::from_image(texture);
            sprite.custom_size = Some(VEC2_CELL_SIZE);
            commands.entity(entity).insert(sprite);
        } else {
            let sprite = Sprite::from_color(material_def.color, VEC2_CELL_SIZE);
            commands.entity(entity).insert(sprite);
        }

        remove_mask_overlays_from_parent(
            entity,
            &mut commands,
            &children_query,
            &mask_overlay_query,
        );

        if solid_object.mergeable {
            spawn_border_masks(
                &mut commands,
                entity,
                material_def.color,
                solid_object.neighbors_pattern,
            );
        }
    }
    info!("Material textures updated");
}

fn remove_mask_overlays_from_parent(
    parent: Entity,
    commands: &mut Commands,
    children_query: &Query<&Children>,
    mask_overlay_query: &Query<(), With<MaskOverlay>>,
) {
    if let Ok(children) = children_query.get(parent) {
        for child in children.iter() {
            if mask_overlay_query.get(child).is_ok() {
                commands.entity(child).despawn();
            }
        }
    }
}

pub fn trigger_terrain_update(
    chunk_x: i32,
    chunk_y: i32,
    mut event_writer: EventWriter<UpdateTerrainEvent>,
) {
    event_writer.write(UpdateTerrainEvent {
        region: None,
        chunk_coords: Some((chunk_x, chunk_y)),
    });
}

fn spawn_border_masks(
    commands: &mut Commands,
    parent: Entity,
    mask_color: Color,
    neighbors_pattern: u8,
) {
    let half_cell: f32 = (CELL_SIZE / 2) as f32; // half_cell
    let inner_offset = MASK_THICKNESS;

    // Pour faciliter le test, on définit pour chaque côté s'il est présent ou non.
    let has_top = (neighbors_pattern & NEIGHBOR_TOP) != 0;
    let has_bottom = (neighbors_pattern & NEIGHBOR_BOTTOM) != 0;
    let has_left = (neighbors_pattern & NEIGHBOR_LEFT) != 0;
    let has_right = (neighbors_pattern & NEIGHBOR_RIGHT) != 0;

    let has_top_left = (neighbors_pattern & NEIGHBOR_TOP_LEFT) != 0;
    let has_top_right = (neighbors_pattern & NEIGHBOR_TOP_RIGHT) != 0;
    let has_bottom_left = (neighbors_pattern & NEIGHBOR_BOTTOM_LEFT) != 0;
    let has_bottom_right = (neighbors_pattern & NEIGHBOR_BOTTOM_RIGHT) != 0;

    // --- Coins : si deux côtés adjacents sont présents, on ajoute un masque carré pour masquer le coin intérieur.
    // Par exemple, si il n'y a ni voisin en haut ni en gauche, on affiche un carré dans le coin supérieur gauche.
    if !has_top && !has_left && has_right && has_bottom && has_bottom_right {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                    Transform::from_xyz(inner_offset, -inner_offset, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
    if has_top && !has_left && !has_bottom && has_right && has_top_right {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                    Transform::from_xyz(inner_offset, inner_offset, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
    if has_top && has_left && !has_bottom && !has_right && has_top_left {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                    Transform::from_xyz(-inner_offset, inner_offset, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
    if !has_top && has_left && has_bottom && !has_right && has_bottom_left {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                    Transform::from_xyz(-inner_offset, -inner_offset, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }

    if has_top && has_bottom && has_right && has_left {
        // 1 missing corner
        if !has_top_left && has_bottom_left && has_bottom_right && has_top_right {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                        Transform::from_xyz(-inner_offset, -inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        if !has_bottom_left & has_bottom_right && has_top_right && has_top_left {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                        Transform::from_xyz(-inner_offset, inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }
        if !has_bottom_right && has_top_right && has_top_left && has_bottom_left {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(-inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                        Transform::from_xyz(inner_offset, inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        if !has_top_right && has_top_left && has_bottom_left && has_bottom_right {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(-inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::splat(half_cell)),
                        Transform::from_xyz(inner_offset, -inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        // Two missing corners
        if !has_top_left && !has_bottom_left && has_bottom_right && has_top_right {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        if !has_bottom_left && !has_bottom_right && has_top_right && has_top_left {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(CELL_SIZE as f32, half_cell)),
                        Transform::from_xyz(0.0, inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        if !has_bottom_right && !has_top_right && has_top_left && has_bottom_left {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                        Transform::from_xyz(-inner_offset, 0.0, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }

        if !has_top_right && !has_top_left && has_bottom_left && has_bottom_right {
            commands.entity(parent).with_children(|parent| {
                parent
                    .spawn((
                        Sprite::from_color(mask_color, Vec2::new(CELL_SIZE as f32, half_cell)),
                        Transform::from_xyz(0.0, -inner_offset, 0.1),
                    ))
                    .insert(MaskOverlay);
            });
        }
    }

    // --- Bords simples : si un seul côté manque ET que l'opposé est présent (ce qui évite de doubler avec le coin déjà traité),
    // on ajoute un sprite rectangulaire couvrant toute la largeur (ou hauteur) de la cellule.
    if !has_top && has_left && has_right && has_bottom {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::new(CELL_SIZE as f32, half_cell)),
                    Transform::from_xyz(0.0, -inner_offset, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
    if !has_bottom && has_left && has_right && has_top {
        commands.entity(parent).with_children(|parent| {
            let sprite = Sprite::from_color(mask_color, Vec2::new(CELL_SIZE as f32, half_cell));
            parent
                .spawn((sprite, Transform::from_xyz(0.0, inner_offset, 0.1)))
                .insert(MaskOverlay);
        });
    }
    if !has_left && has_top && has_bottom && has_right {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                    Transform::from_xyz(inner_offset, 0.0, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
    if !has_right && has_top && has_bottom && has_left {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite::from_color(mask_color, Vec2::new(half_cell, CELL_SIZE as f32)),
                    Transform::from_xyz(-inner_offset, 0.0, 0.1),
                ))
                .insert(MaskOverlay);
        });
    }
}

/// This system updates the visibility of the item text based on the camera zoom level
///
/// If the camera zoom level is less than 1.0, the text is hidden.
fn update_item_text_visibility(
    camera_query: Query<&Projection, With<ControlledCamera>>,
    mut text_query: Query<&mut Visibility, With<ItemText>>,
) {
    if let Ok(projection) = camera_query.single() {
        if let Projection::Orthographic(ortho_projection) = projection {
            let zoom = ortho_projection.scale;
            for mut visibility in text_query.iter_mut() {
                *visibility = if zoom <= 1.5 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// This system despawns the entities in the EntitiesToDespawn resource
fn apply_despawns(mut commands: Commands, mut to_despawn: ResMut<EntitiesToDespawn>) {
    for entity in to_despawn.0.drain(..) {
        commands.entity(entity).despawn();
        info!("Entity {:?} despawned", entity);
    }
}
