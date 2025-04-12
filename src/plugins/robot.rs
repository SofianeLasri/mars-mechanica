use crate::components::entity::{ExplorerRobot, MinerRobot, MinerTask, WorldKnowledge};
use crate::components::terrain::{
    SolidObject, TerrainCell, TerrainChunk, UpdateTerrainEvent, WorldEntityItem, CELL_SIZE,
    VEC2_CELL_SIZE,
};
use crate::GameState;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldKnowledge>()
            .add_systems(
                OnEnter(GameState::InGame),
                (spawn_explorer_robot, spawn_miner_robot),
            )
            .add_systems(
                FixedUpdate,
                (
                    detect_environment,
                    plan_robot_movement,
                    plan_miner_movement,
                    move_robots,
                    check_miner_collection,
                )
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

    info!("Explorer rover placed at cell: ({}, {})", cell_x, cell_y);
}

/// Spawns a miner robot at a random position on the terrain
fn spawn_miner_robot(
    mut commands: Commands,
    terrain_query: Query<(&Transform, &TerrainCell, Option<&Children>)>,
    asset_server: Res<AssetServer>,
) {
    let robot_texture = asset_server.load("textures/robots/robot_red.png");

    let mut available_position = None;

    // Search in a spiral pattern, starting from the center
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
        MinerRobot {
            speed: 1.5,
            target_position: IVec2::new(cell_x, cell_y),
            is_moving: false,
            move_timer: 0.0,
            current_task: MinerTask::Idle,
            spawn_position: IVec2::new(cell_x, cell_y),
            collected_resources: Vec::new(),
        },
        Name::new("Miner Rover"),
    ));

    info!("Miner rover placed at cell: ({}, {})", cell_x, cell_y);
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

/// Plans the movement of the miner robot based on the world knowledge. It is programmed to
/// mine red crystals and return to the spawn point.
fn plan_miner_movement(
    mut miner_query: Query<(&Transform, &mut MinerRobot)>,
    world_knowledge: Res<WorldKnowledge>,
    terrain_query: Query<(&Transform, &TerrainCell, Option<&Children>)>,
    solid_query: Query<&SolidObject>,
    time: Res<Time>,
) {
    for (transform, mut miner) in miner_query.iter_mut() {
        if miner.is_moving {
            miner.move_timer += time.delta_secs();
            if miner.move_timer >= 1.0 / miner.speed {
                miner.is_moving = false;
                miner.move_timer = 0.0;
            }
            continue;
        }

        let current_cell_x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
        let current_cell_y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
        let current_cell = IVec2::new(current_cell_x, current_cell_y);

        match miner.current_task {
            MinerTask::Idle => {
                let red_crystal_positions: Vec<IVec2> = world_knowledge
                    .discovered_solids
                    .iter()
                    .filter(|(_, material_id)| *material_id == "red_crystal")
                    .map(|(pos, _)| *pos)
                    .collect();

                /*// Echo all discovered solids material ids
                for (pos, material_id) in &world_knowledge.discovered_solids {
                    info!("Discovered solid at {:?} with material id: {}", pos, material_id);
                }*/

                if red_crystal_positions.is_empty() {
                    continue;
                }

                if let Some(closest_pos) =
                    find_closest_position(current_cell, &red_crystal_positions)
                {
                    let crystal_exists = terrain_query
                        .iter()
                        .filter(|(transform, _, _)| {
                            let cell_x =
                                (transform.translation.x / CELL_SIZE as f32).round() as i32;
                            let cell_y =
                                (transform.translation.y / CELL_SIZE as f32).round() as i32;
                            IVec2::new(cell_x, cell_y) == closest_pos
                        })
                        .any(|(_, _, children)| {
                            if let Some(children) = children {
                                for child in children.iter() {
                                    if let Ok(solid) = solid_query.get(child) {
                                        return solid.material_id == "red_crystal";
                                    }
                                }
                            }
                            false
                        });

                    if crystal_exists {
                        let path = find_path(current_cell, closest_pos, &world_knowledge);
                        if !path.is_empty() {
                            miner.target_position = path[0];
                            miner.is_moving = true;
                            miner.move_timer = 0.0;
                            miner.current_task = MinerTask::MovingToTarget;
                        }
                    }
                }
            }
            MinerTask::MovingToTarget => {
                let is_at_crystal = terrain_query
                    .iter()
                    .filter(|(transform, _, _)| {
                        let cell_x = (transform.translation.x / CELL_SIZE as f32).round() as i32;
                        let cell_y = (transform.translation.y / CELL_SIZE as f32).round() as i32;
                        IVec2::new(cell_x, cell_y) == current_cell
                    })
                    .any(|(_, _, children)| {
                        if let Some(children) = children {
                            for child in children.iter() {
                                if let Ok(solid) = solid_query.get(child) {
                                    return solid.material_id == "red_crystal";
                                }
                            }
                        }
                        false
                    });

                if is_at_crystal {
                    miner.current_task = MinerTask::Mining;
                    continue;
                }

                let red_crystal_positions: Vec<IVec2> = world_knowledge
                    .discovered_solids
                    .iter()
                    .filter(|(_, material_id)| *material_id == "red_crystal")
                    .map(|(pos, _)| *pos)
                    .collect();

                if red_crystal_positions.is_empty() {
                    miner.current_task = MinerTask::ReturningToSpawn;
                    continue;
                }

                if let Some(closest_pos) =
                    find_closest_position(current_cell, &red_crystal_positions)
                {
                    let path = find_path(current_cell, closest_pos, &world_knowledge);
                    if !path.is_empty() {
                        if path[0] != miner.target_position {
                            miner.target_position = path[0];
                            miner.is_moving = true;
                            miner.move_timer = 0.0;
                        }
                    }
                } else {
                    miner.current_task = MinerTask::ReturningToSpawn;
                }
            }
            MinerTask::Mining => {
                // Le minage est géré dans un système séparé
                // Ce système gère juste les déplacements entre les tâches
            }
            MinerTask::ReturningToSpawn => {
                if current_cell == miner.spawn_position {
                    info!(
                        "Miner rover returned to spawn with resources: {:?}",
                        miner.collected_resources
                    );
                    miner.collected_resources.clear();
                    miner.current_task = MinerTask::Idle;
                    continue;
                }

                let path = find_path(current_cell, miner.spawn_position, &world_knowledge);
                if !path.is_empty() {
                    miner.target_position = path[0];
                    miner.is_moving = true;
                    miner.move_timer = 0.0;
                }
            }
        }
    }
}

/// Checks if the miner robot is at the position of a red crystal or an item to collect.
fn check_miner_collection(
    mut commands: Commands,
    mut miner_query: Query<(&Transform, &mut MinerRobot)>,
    item_query: Query<(Entity, &Transform, &WorldEntityItem)>,
    mut solid_query: Query<(Entity, &mut SolidObject)>,
    terrain_query: Query<(&Transform, &TerrainCell, &TerrainChunk, Option<&Children>)>,
    mut world_knowledge: ResMut<WorldKnowledge>,
    mut update_terrain_events: EventWriter<UpdateTerrainEvent>,
) {
    for (miner_transform, mut miner) in miner_query.iter_mut() {
        let miner_pos_x = (miner_transform.translation.x / CELL_SIZE as f32).round() as i32;
        let miner_pos_y = (miner_transform.translation.y / CELL_SIZE as f32).round() as i32;
        let miner_pos = IVec2::new(miner_pos_x, miner_pos_y);

        if miner.current_task == MinerTask::Mining {
            for (terrain_transform, _, chunk, children) in terrain_query.iter() {
                let cell_x = (terrain_transform.translation.x / CELL_SIZE as f32).round() as i32;
                let cell_y = (terrain_transform.translation.y / CELL_SIZE as f32).round() as i32;
                let cell_pos = IVec2::new(cell_x, cell_y);

                if cell_pos == miner_pos && children.is_some() {
                    for child in children.unwrap().iter() {
                        if let Ok((_solid_entity, mut solid)) = solid_query.get_mut(child) {
                            if solid.material_id == "red_crystal" {
                                solid.health = 0.0;

                                world_knowledge.discovered_solids.remove(&miner_pos);
                                world_knowledge.discovered_empty.insert(miner_pos);

                                update_terrain_events.write(UpdateTerrainEvent {
                                    region: None,
                                    chunk_coords: Some((chunk.chunk_x, chunk.chunk_y)),
                                });

                                miner.current_task = MinerTask::Idle;
                                break;
                            }
                        }
                    }
                }
            }
        }

        for (item_entity, item_transform, item) in item_query.iter() {
            let item_pos_x = (item_transform.translation.x / CELL_SIZE as f32).round() as i32;
            let item_pos_y = (item_transform.translation.y / CELL_SIZE as f32).round() as i32;
            let item_pos = IVec2::new(item_pos_x, item_pos_y);

            if miner_pos == item_pos && item.entity_id == "red_crystal_item" {
                miner
                    .collected_resources
                    .push((item.entity_id.clone(), item.quantity));
                commands.entity(item_entity).despawn();

                if miner.current_task == MinerTask::Idle {
                    miner.current_task = MinerTask::ReturningToSpawn;
                }
            }
        }
    }
}

/// Moves the robots based on their target position and speed.
fn move_robots(
    mut explorer_query: Query<(&mut Transform, &ExplorerRobot)>,
    mut miner_query: Query<(&mut Transform, &MinerRobot), Without<ExplorerRobot>>,
) {
    for (mut transform, robot) in explorer_query.iter_mut() {
        if robot.is_moving {
            let current_pos = transform.translation;

            let target_x = robot.target_position.x * CELL_SIZE;
            let target_y = robot.target_position.y * CELL_SIZE;
            let target_pos = Vec3::new(target_x as f32, target_y as f32, current_pos.z);

            let progress = (robot.move_timer * robot.speed).min(1.0);

            transform.translation = current_pos.lerp(target_pos, progress);
        }
    }

    for (mut transform, robot) in miner_query.iter_mut() {
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

/// Finds the closest position to the current position from a list of target positions.
fn find_closest_position(current_pos: IVec2, target_positions: &[IVec2]) -> Option<IVec2> {
    target_positions
        .iter()
        .min_by_key(|pos| {
            let dx = pos.x - current_pos.x;
            let dy = pos.y - current_pos.y;
            dx * dx + dy * dy
        })
        .copied()
}

/// Finds a path from the start position to the goal position using BFS.
fn find_path(start: IVec2, goal: IVec2, world_knowledge: &WorldKnowledge) -> Vec<IVec2> {
    if start == goal {
        return Vec::new();
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut came_from = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    let directions = [
        IVec2::new(1, 0),
        IVec2::new(0, 1),
        IVec2::new(-1, 0),
        IVec2::new(0, -1),
    ];

    while let Some(current) = queue.pop_front() {
        if current == goal {
            break;
        }

        for dir in &directions {
            let next = current + *dir;

            if visited.contains(&next) {
                continue;
            }

            let is_walkable = world_knowledge.discovered_empty.contains(&next)
                || (world_knowledge.discovered_cells.contains(&next)
                && !world_knowledge.discovered_solids.contains_key(&next));

            if !is_walkable {
                continue;
            }

            visited.insert(next);
            came_from.insert(next, current);
            queue.push_back(next);
        }
    }

    let mut path = Vec::new();
    let mut current = goal;

    while current != start {
        path.push(current);
        match came_from.get(&current) {
            Some(&prev) => current = prev,
            None => return Vec::new(), // No path found
        }
    }

    path.reverse();
    path
}
