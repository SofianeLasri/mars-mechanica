use crate::components::terrain::*;
use crate::GameState;
use noise::{NoiseFn, Perlin};
use rand::random;
use std::collections::HashSet;
use bevy::prelude::{info, Commands, EventWriter, NextState, Res, ResMut, Transform};
use bevy_sprite::Sprite;

pub fn generate_world(
    mut commands: Commands,
    world_materials: Res<WorldMaterials>,
    mut chunk_map: ResMut<ChunkMap>,
    mut event_writer: EventWriter<UpdateTerrainEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let terrain_noise = Perlin::new(random());
    let material_noise = Perlin::new(random());

    info!("Generating world...");

    for chunk_x in -MAP_SIZE / 2..MAP_SIZE / 2 {
        for chunk_y in -MAP_SIZE / 2..MAP_SIZE / 2 {
            generate_chunk(
                &mut commands,
                &mut chunk_map,
                world_materials.as_ref(),
                terrain_noise.clone(),
                material_noise.clone(),
                chunk_x,
                chunk_y,
            );
        }
    }

    info!("World generated! Chunks: {}", chunk_map.chunks.len());

    event_writer.write(UpdateTerrainEvent {
        region: None,
        chunk_coords: None,
    });
    next_state.set(GameState::InGame);
}

/// Generates a chunk of terrain with its cells and solid objects
fn generate_chunk(
    commands: &mut Commands,
    chunk_map: &mut ChunkMap,
    world_materials: &WorldMaterials,
    terrain_noise: Perlin,
    material_noise: Perlin,
    chunk_x: i32,
    chunk_y: i32,
) {
    chunk_map.chunks.insert((chunk_x, chunk_y), HashSet::new());

    let radius = (MAP_SIZE * CHUNK_SIZE) / 2;
    let radius_sq = radius.pow(2);

    for local_x in 0..CHUNK_SIZE {
        for local_y in 0..CHUNK_SIZE {
            let world_x = chunk_x * CHUNK_SIZE + local_x;
            let world_y = chunk_y * CHUNK_SIZE + local_y;

            let distance_sq = world_x.pow(2) + world_y.pow(2);

            // Skip cells outside the circular area
            if distance_sq > radius_sq {
                continue;
            }

            let (coord_x, coord_y) = calc_cell_coordinates(&world_x, &world_y);

            // Generate ground cell
            commands.spawn((
                Sprite::from_color(MARS_GROUND_COLOR, VEC2_CELL_SIZE),
                Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0),
                TerrainCell,
                TerrainChunk { chunk_x, chunk_y },
            ));

            // Determine if current cell is on the border
            let is_border = is_border_cell(world_x, world_y, radius_sq);

            let material_id = if is_border {
                "rock".to_string()
            } else {
                // Use noise for internal cells
                let noise_value = terrain_noise.get([world_x as f64 * 0.1, world_y as f64 * 0.1]) as f32;
                if noise_value <= 0.0 {
                    continue;
                }

                let material_value = material_noise.get([world_x as f64 * 0.2, world_y as f64 * 0.2]) as f32;
                if material_value > 0.95 {
                    "red_crystal".to_string()
                } else if material_value > 0.8 {
                    "olivine".to_string()
                } else if material_value > 0.6 {
                    "basalt".to_string()
                } else {
                    "rock".to_string()
                }
            };

            let material_def = world_materials.materials.get(&material_id).unwrap();

            let entity = commands.spawn((
                Sprite::from_color(material_def.color, VEC2_CELL_SIZE),
                Transform::from_xyz(coord_x as f32, coord_y as f32, 1.0),
                SolidObject {
                    material_id: material_id.clone(),
                    health: material_def.strength,
                    mergeable: material_def.can_be_merged,
                    neighbors_pattern: 0,
                },
                TerrainChunk { chunk_x, chunk_y },
            )).id();

            if let Some(chunk_entities) = chunk_map.chunks.get_mut(&(chunk_x, chunk_y)) {
                chunk_entities.insert(entity);
            }
        }
    }
}

/// Checks if a cell is on the circular border by verifying its neighbors
fn is_border_cell(world_x: i32, world_y: i32, radius_sq: i32) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = world_x + dx;
            let ny = world_y + dy;
            if nx.pow(2) + ny.pow(2) > radius_sq {
                return true;
            }
        }
    }
    false
}

pub fn calc_cell_coordinates(x: &i32, y: &i32) -> (i32, i32) {
    (x * CELL_SIZE, y * CELL_SIZE)
}
