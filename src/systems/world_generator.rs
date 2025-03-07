use bevy::prelude::*;
use rand::prelude::*;
use noise::{NoiseFn, Perlin};
use crate::components::terrain::*;

const CELL_SIZE: i32 = 64;

pub fn generate_world(
    mut commands: Commands,
    world_materials: Res<WorldMaterials>,
) {
    // Initialise le générateur de bruit de Perlin pour la génération du terrain
    let terrain_noise = Perlin::new(random());
    let material_noise = Perlin::new(random());

    // Dimensions de la carte
    let width = 50;
    let height = 50;

    // Grille pour tracker les cellules occupées
    let mut occupied_cells = vec![vec![false; height as usize]; width as usize];

    // D'abord, génère les cellules du terrain (sans texture, juste la couleur mars)
    for x in -width/2..width/2 {
        for y in -height/2..height/2 {
            let (coord_x, coord_y) = calc_cell_coordinates(&x, &y);

            // Spawn une cellule de terrain avec la couleur martienne
            commands.spawn((
                /*SpriteBundle {
                    sprite: Sprite {
                        color: MARS_GROUND_COLOR,
                        custom_size: Some(Vec2::new(CELL_SIZE as f32, CELL_SIZE as f32)),
                        ..default()
                    },
                    transform: Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0),
                    ..default()
                },*/
                Sprite::from_color(MARS_GROUND_COLOR, Vec2::new(50.0, 50.0)),
                Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0),
                TerrainCell { x, y },
            ));
        }
    }

    // Ensuite, génère les objets solides (roches, basalte, olivine)
    for x in -width/2..width/2 {
        for y in -height/2..height/2 {
            let (coord_x, coord_y) = calc_cell_coordinates(&x, &y);

            // Utilise le bruit de Perlin pour déterminer s'il faut placer un objet
            let noise_value = terrain_noise.get([x as f64 * 0.1, y as f64 * 0.1]) as f32;

            // Détermine si on place un objet ici (50% des cellules ont des objets)
            if noise_value > 0.0 {
                let grid_x = (x + width/2) as usize;
                let grid_y = (y + height/2) as usize;
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
                    /*SpriteBundle {
                        texture: material_def.plain_texture.clone(), // Texture par défaut, sera mise à jour par le système
                        transform: Transform::from_xyz(coord_x as f32, coord_y as f32, 1.0),
                        ..default()
                    },*/
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
