use crate::components::terrain::*;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;

pub const CELL_SIZE: i32 = 64;

pub fn generate_world(mut commands: Commands, world_materials: Res<WorldMaterials>) {
    let terrain_noise = Perlin::new(random());
    let material_noise = Perlin::new(random());

    let width = 50;
    let height = 50;

    let mut occupied_cells = vec![vec![false; height as usize]; width as usize];

    info!("Generating world...");

    generate_cells(&mut commands, width, height);

    info!("Empty cells generated");

    generate_entities_and_world_materials(
        &mut commands,
        world_materials,
        terrain_noise,
        material_noise,
        width,
        height,
        &mut occupied_cells,
    );

    info!("Entities generated");
}

/// Génère les cellules du terrain (sans texture, juste la couleur mars)
fn generate_cells(commands: &mut Commands, width: i32, height: i32) {
    for x in -width / 2..width / 2 {
        for y in -height / 2..height / 2 {
            let (coord_x, coord_y) = calc_cell_coordinates(&x, &y);
            let mut sprite = Sprite::from_color(
                MARS_GROUND_COLOR,
                Vec2::new(CELL_SIZE as f32, CELL_SIZE as f32),
            );

            commands.spawn((
                sprite,
                Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0),
                TerrainCell { x, y },
            ));
        }
    }
}

/// Génère les objets solides (roches, basalte, olivine) et les matériaux du monde
fn generate_entities_and_world_materials(
    commands: &mut Commands,
    world_materials: Res<WorldMaterials>,
    terrain_noise: Perlin,
    material_noise: Perlin,
    width: i32,
    height: i32,
    occupied_cells: &mut Vec<Vec<bool>>,
) {
    for x in -width / 2..width / 2 {
        for y in -height / 2..height / 2 {
            let (coord_x, coord_y) = calc_cell_coordinates(&x, &y);

            // Utilise le bruit de Perlin pour déterminer s'il faut placer un objet
            let noise_value = terrain_noise.get([x as f64 * 0.1, y as f64 * 0.1]) as f32;

            // Détermine si on place un objet ici (50% des cellules ont des objets)
            if noise_value > 0.0 {
                let grid_x = (x + width / 2) as usize;
                let grid_y = (y + height / 2) as usize;
                occupied_cells[grid_x][grid_y] = true;

                // Détermine le type de matériau en fonction d'un autre bruit de Perlin
                let material_value = material_noise.get([x as f64 * 0.2, y as f64 * 0.2]) as f32;

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

                if (world_materials.materials.len() == 0) {
                    error!("No materials in the world materials");
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

                // Spawn l'objet solide
                commands.spawn((
                    Sprite::from_image(material_def.plain_texture.clone()),
                    Transform::from_xyz(coord_x as f32, coord_y as f32, 1.0),
                    SolidObject {
                        material_id: material_id.to_string(),
                        health,
                        max_health: health,
                        mergeable,
                        neighbors_pattern: 0, // Sera mis à jour par le système update_neighbors_pattern
                    },
                ));
            }
        }
    }
}

pub fn calc_cell_coordinates(x: &i32, y: &i32) -> (i32, i32) {
    let cell_x = x * CELL_SIZE;
    let cell_y = y * CELL_SIZE;
    (cell_x, cell_y)
}
