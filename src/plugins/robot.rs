use crate::components::entity::{ExplorerRobot, WorldKnowledge};
use crate::components::terrain::{SolidObject, TerrainCell, CELL_SIZE, VEC2_CELL_SIZE};
use crate::GameState;
use bevy::prelude::*;

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

/// Spawns an explorer robot at a random position on the terrain, near the center of the map and
/// on an empty cell.
///
/// **Note:** We use the asset server and not the asset preloader for the simplicity.
fn spawn_explorer_robot(
    mut commands: Commands,
    terrain_query: Query<(&Transform, &TerrainCell, Option<&Children>)>,
    asset_server: Res<AssetServer>,
) {
    let robot_texture = asset_server.load("textures/robots/robot_blue.png");

    let mut available_position = None;

    // In fact, there is a strong chance that the 0,0 cell isn't empty, so we need to search
    // around it. We will search in a spiral pattern, starting from the center of the map.
    'search: for radius in 0i32..20i32 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let x = dx * CELL_SIZE;
                let y = dy * CELL_SIZE;

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

    // Normally, we should have found an empty cell. If not, we will spawn the robot at (0,0).
    // But in that case, the game can't launch. So we will have to restart the game.

    // TODO: Handle the case where no empty cell is found.

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
            speed: 1.0,
            target_position: IVec2::new(cell_x, cell_y),
            is_moving: false,
            move_timer: 0.0,
            previous_position: None,
            follow_direction: 1,
        },
        Name::new("Explorer Rover"),
    ));

    info!(
        "Explorer rover placed at cell: ({}, {})",
        cell_x, cell_y
    );
}

/// Detects the environment around the robot and updates the world knowledge accordingly.
///
/// The robot detects the terrain cells and solid objects within a radius of 8 cells.
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

/// Plans the movement of the robot based on the world knowledge. It is programmed to follow
/// the walls and explore the environment.
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

        if robot.previous_position.is_none() {
            robot.previous_position = Some(current_cell);
        }

        let directions = [
            IVec2::new(1, 0),  // droite
            IVec2::new(0, 1),  // haut
            IVec2::new(-1, 0), // gauche
            IVec2::new(0, -1), // bas
        ];

        let mut accessible_cells = Vec::new();
        for dir in directions.iter() {
            let neighbor = current_cell + *dir;
            if world_knowledge.discovered_empty.contains(&neighbor)
                && Some(neighbor) != robot.previous_position
            {
                accessible_cells.push(neighbor);
            }
        }

        if accessible_cells.is_empty() {
            for dir in directions.iter() {
                let neighbor = current_cell + *dir;
                if world_knowledge.discovered_empty.contains(&neighbor) {
                    accessible_cells.push(neighbor);
                }
            }
        }

        if accessible_cells.is_empty() {
            continue;
        }

        let mut solid_neighbors = Vec::new();
        for dir in &directions {
            let check_cell = current_cell + *dir;
            if world_knowledge.discovered_solids.contains_key(&check_cell) {
                solid_neighbors.push(*dir);
            }
        }

        let mut next_cell = current_cell;

        if !solid_neighbors.is_empty() {
            solid_neighbors.sort_by_key(|dir| (dir.x.abs() + dir.y.abs()) * 10 + dir.y * 2 + dir.x);

            let solid_dir = solid_neighbors[0];

            let preferred_dir = if robot.follow_direction > 0 {
                IVec2::new(-solid_dir.y, solid_dir.x) // Rotation 90° horaire
            } else {
                IVec2::new(solid_dir.y, -solid_dir.x) // Rotation 90° anti-horaire
            };

            let preferred_cell = current_cell + preferred_dir;

            if world_knowledge.discovered_empty.contains(&preferred_cell)
                && Some(preferred_cell) != robot.previous_position
            {
                next_cell = preferred_cell;
            } else {
                let alt_dir = if robot.follow_direction > 0 {
                    IVec2::new(solid_dir.y, -solid_dir.x)
                } else {
                    IVec2::new(-solid_dir.y, solid_dir.x)
                };

                let alt_cell = current_cell + alt_dir;

                if world_knowledge.discovered_empty.contains(&alt_cell)
                    && Some(alt_cell) != robot.previous_position
                {
                    next_cell = alt_cell;
                    robot.follow_direction = -robot.follow_direction;
                } else if !accessible_cells.is_empty() {
                    next_cell = accessible_cells[0];
                }
            }
        } else {
            let cells_next_to_solid: Vec<IVec2> = accessible_cells
                .iter()
                .filter(|&&cell| {
                    directions.iter().any(|dir| {
                        let adj_cell = cell + *dir;
                        world_knowledge.discovered_solids.contains_key(&adj_cell)
                    })
                })
                .copied()
                .collect();

            if !cells_next_to_solid.is_empty() {
                next_cell = cells_next_to_solid[0];
            } else {
                let unexplored_cells: Vec<IVec2> = accessible_cells
                    .iter()
                    .filter(|&&cell| {
                        directions.iter().any(|dir| {
                            let neighbor = cell + *dir;
                            !world_knowledge.discovered_cells.contains(&neighbor)
                        })
                    })
                    .copied()
                    .collect();

                if !unexplored_cells.is_empty() {
                    next_cell = unexplored_cells[0];
                } else {
                    next_cell = accessible_cells[0];
                }
            }
        }

        if next_cell != current_cell {
            robot.previous_position = Some(current_cell);
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
