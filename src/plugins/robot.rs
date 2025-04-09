use crate::GameState;
use crate::components::entity::{ExplorerRobot, WorldKnowledge};
use crate::components::terrain::{CELL_SIZE, SolidObject, TerrainCell, VEC2_CELL_SIZE};
use bevy::prelude::*;
use rand::prelude::*;

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldKnowledge>()
            .add_systems(OnEnter(GameState::InGame), spawn_explorer_robot)
            .add_systems(
                FixedUpdate,
                (detect_environment, plan_robot_movement, move_robot)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn spawn_explorer_robot(
    mut commands: Commands,
    terrain_query: Query<(&Transform, &TerrainCell, Option<&Children>)>,
    asset_server: Res<AssetServer>,
) {
    let robot_texture = asset_server.load("textures/robots/robot_blue.png");

    let mut available_position = None;

    'search: for radius in 0i32..20i32 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Vérifier uniquement les cellules sur le périmètre
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let x = dx * CELL_SIZE;
                let y = dy * CELL_SIZE;

                // Vérifier si cette cellule est vide
                let is_empty = terrain_query
                    .iter()
                    .filter(|(transform, _, _)| {
                        (transform.translation.x - x as f32).abs() < 1.0
                            && (transform.translation.y - y as f32).abs() < 1.0
                    })
                    .all(|(_, _, children)| children.is_none() || children.unwrap().is_empty());

                if is_empty {
                    available_position = Some(Vec2::new(x as f32, y as f32));
                    break 'search;
                }
            }
        }
    }

    let position = available_position.unwrap_or(Vec2::ZERO);

    let cell_x = (position.x / CELL_SIZE as f32).round() as i32;
    let cell_y = (position.y / CELL_SIZE as f32).round() as i32;

    commands.spawn((
        Sprite {
            image: robot_texture,
            custom_size: Some(VEC2_CELL_SIZE * 0.8),
            ..Default::default()
        },
        Transform::from_xyz(position.x, position.y, 10.0),
        ExplorerRobot {
            speed: 1.0, // Une cellule par seconde
            target_position: IVec2::new(cell_x, cell_y),
            is_moving: false,
            move_timer: 0.0,
        },
        Name::new("Explorer Robot"),
    ));

    info!(
        "Robot explorateur placé à la position: ({}, {})",
        cell_x, cell_y
    );
}

fn detect_environment(
    robot_query: Query<(&Transform, &ExplorerRobot)>,
    terrain_query: Query<(&Transform, &TerrainCell, Option<&Children>)>,
    solid_query: Query<&SolidObject>,
    mut world_knowledge: ResMut<WorldKnowledge>,
) {
    for (robot_transform, _) in robot_query.iter() {
        let robot_pos = robot_transform.translation;

        let robot_cell_x = (robot_pos.x / CELL_SIZE as f32).round() as i32;
        let robot_cell_y = (robot_pos.y / CELL_SIZE as f32).round() as i32;

        let detection_radius = 8;

        for dx in -detection_radius..=detection_radius {
            for dy in -detection_radius..=detection_radius {
                if dx * dx + dy * dy > detection_radius * detection_radius {
                    continue;
                }

                let cell_x = robot_cell_x + dx;
                let cell_y = robot_cell_y + dy;
                let cell_pos = IVec2::new(cell_x, cell_y);

                world_knowledge.discovered_cells.insert(cell_pos);

                let terrain_x = cell_x * CELL_SIZE;
                let terrain_y = cell_y * CELL_SIZE;

                for (transform, _, children) in terrain_query.iter() {
                    if (transform.translation.x - terrain_x as f32).abs() < 1.0
                        && (transform.translation.y - terrain_y as f32).abs() < 1.0
                    {
                        if let Some(children) = children {
                            let mut has_solid = false;

                            for child in children.iter() {
                                if let Ok(solid) = solid_query.get(child) {
                                    has_solid = true;
                                    world_knowledge
                                        .discovered_solids
                                        .insert(cell_pos, solid.material_id.clone());
                                    break;
                                }
                            }

                            if !has_solid {
                                world_knowledge.discovered_empty.insert(cell_pos);
                            }
                        } else {
                            world_knowledge.discovered_empty.insert(cell_pos);
                        }

                        break;
                    }
                }
            }
        }
    }
}

fn plan_robot_movement(
    mut robot_query: Query<(&Transform, &mut ExplorerRobot)>,
    world_knowledge: Res<WorldKnowledge>,
    time: Res<Time>,
) {
    for (transform, mut robot) in robot_query.iter_mut() {
        if robot.is_moving {
            robot.move_timer += time.delta_secs();
            if robot.move_timer >= 1.0 / robot.speed {
                robot.is_moving = false;
                robot.move_timer = 0.0;
            }
            continue;
        }

        let current_cell_x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
        let current_cell_y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
        let current_cell = IVec2::new(current_cell_x, current_cell_y);

        let directions = [
            IVec2::new(1, 0),  // droite
            IVec2::new(0, 1),  // haut
            IVec2::new(-1, 0), // gauche
            IVec2::new(0, -1), // bas
        ];

        let mut accessible_cells = Vec::new();

        for dir in directions.iter() {
            let neighbor = current_cell + *dir;

            if world_knowledge.discovered_empty.contains(&neighbor) {
                accessible_cells.push(neighbor);
            }
        }

        let mut unexplored_neighbors = Vec::new();

        for &cell in &accessible_cells {
            for dir in directions.iter() {
                let neighbor = cell + *dir;

                if !world_knowledge.discovered_cells.contains(&neighbor) {
                    unexplored_neighbors.push(cell);
                    break;
                }
            }
        }

        let mut rng = rand::rng();
        let next_cell = if !unexplored_neighbors.is_empty() {
            unexplored_neighbors.choose(&mut rng).copied().unwrap()
        } else if !accessible_cells.is_empty() {
            accessible_cells.choose(&mut rng).copied().unwrap()
        } else {
            current_cell
        };

        if next_cell != current_cell {
            robot.target_position = next_cell;
            robot.is_moving = true;
            robot.move_timer = 0.0;
        }
    }
}

fn move_robot(mut robot_query: Query<(&mut Transform, &ExplorerRobot)>) {
    for (mut transform, robot) in robot_query.iter_mut() {
        if robot.is_moving {
            let current_pos = transform.translation;

            let target_x = robot.target_position.x * CELL_SIZE;
            let target_y = robot.target_position.y * CELL_SIZE;
            let target_pos = Vec3::new(target_x as f32, target_y as f32, current_pos.z);

            let progress = (robot.move_timer * robot.speed).min(1.0);

            transform.translation = current_pos.lerp(target_pos, progress);
        }
    }
}
