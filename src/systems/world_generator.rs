use crate::components::terrain::*;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use std::collections::HashSet;

pub fn generate_world(
    mut commands: Commands,
    world_materials: Res<WorldMaterials>,
    mut chunk_map: ResMut<ChunkMap>,
    mut event_writer: EventWriter<UpdateTerrainEvent>
) {
    let terrain_noise = Perlin::new(random());
    let material_noise = Perlin::new(random());

    info!("Generating world...");

    // Générer le monde chunk par chunk
    for chunk_x in -MAP_SIZE / 2..MAP_SIZE / 2 {
        for chunk_y in -MAP_SIZE / 2..MAP_SIZE / 2 {
            generate_chunk(
                &mut commands,
                &mut chunk_map,
                world_materials.as_ref(),
                terrain_noise,
                material_noise,
                chunk_x,
                chunk_y,
            );
        }
    }

    info!("World generated! Chunks: {}", chunk_map.chunks.len());

    // Envoyer l'événement de mise à jour du terrain
    event_writer.send(UpdateTerrainEvent {
        region: None,
        chunk_coords: None,
    });
}

/// Génère un chunk du terrain avec ses cellules et objets solides
fn generate_chunk(
    commands: &mut Commands,
    chunk_map: &mut ChunkMap,
    world_materials: &WorldMaterials,
    terrain_noise: Perlin,
    material_noise: Perlin,
    chunk_x: i32,
    chunk_y: i32,
) {
    // Créer une entrée pour ce chunk dans la ChunkMap
    chunk_map.chunks.insert((chunk_x, chunk_y), HashSet::new());

    // Générer les cellules du chunk
    for local_x in 0..CHUNK_SIZE {
        for local_y in 0..CHUNK_SIZE {
            let world_x = chunk_x * CHUNK_SIZE + local_x;
            let world_y = chunk_y * CHUNK_SIZE + local_y;

            let (coord_x, coord_y) = calc_cell_coordinates(&world_x, &world_y);

            // Générer la cellule du terrain (sol martien)
            let mut sprite = Sprite::from_color(
                MARS_GROUND_COLOR,
                VEC2_CELL_SIZE,
            );

            commands.spawn((
                sprite,
                Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0),
                TerrainCell { x: world_x, y: world_y },
                TerrainChunk { chunk_x, chunk_y },
            ));

            // Utilise le bruit de Perlin pour déterminer s'il faut placer un objet solide
            let noise_value = terrain_noise.get([world_x as f64 * 0.1, world_y as f64 * 0.1]) as f32;

            // Détermine si on place un objet ici (50% des cellules ont des objets)
            if noise_value > 0.0 {
                // Détermine le type de matériau en fonction d'un autre bruit de Perlin
                let material_value = material_noise.get([world_x as f64 * 0.2, world_y as f64 * 0.2]) as f32;

                let material_id = if material_value > 0.7 {
                    // Olivine (10% de chance)
                    "olivine"
                } else if material_value > 0.4 {
                    // Basalt (30% de chance)
                    "basalt"
                } else {
                    // Roche martienne (60% de chance)
                    "rock"
                };

                if world_materials.materials.len() == 0 {
                    error!("No materials in the world materials");
                    continue;
                }

                let material_def = world_materials.materials.get(material_id).unwrap();
                let mergeable = material_def.can_be_merged;

                // Détermine la santé en fonction du type de matériau
                let health = match material_id {
                    "olivine" => 8.0,
                    "basalt" => 5.0,
                    "rock" => 3.0,
                    _ => 1.0,
                };

                let mut sprite = Sprite::from_image(material_def.plain_texture.clone());
                sprite.custom_size = Some(VEC2_CELL_SIZE);

                // Spawn l'objet solide
                let entity = commands.spawn((
                    sprite,
                    Transform::from_xyz(coord_x as f32, coord_y as f32, 1.0),
                    SolidObject {
                        material_id: material_id.to_string(),
                        health,
                        max_health: health,
                        mergeable,
                        neighbors_pattern: 0, // Sera mis à jour par le système update_neighbors_pattern
                    },
                    TerrainChunk { chunk_x, chunk_y },
                )).id();

                // Ajouter l'entité à la map de chunks
                if let Some(chunk_entities) = chunk_map.chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk_entities.insert(entity);
                }
            }
        }
    }
}

pub fn calc_cell_coordinates(x: &i32, y: &i32) -> (i32, i32) {
    let cell_x = x * CELL_SIZE;
    let cell_y = y * CELL_SIZE;
    (cell_x, cell_y)
}
