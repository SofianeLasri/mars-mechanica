use std::collections::{HashMap, HashSet, VecDeque};
use bevy::prelude::*;

#[derive(Component)]
pub struct ExplorerRobot {
    pub speed: f32,
    pub target_position: IVec2,
    pub is_moving: bool,
    pub move_timer: f32,
}

#[derive(Resource, Default)]
pub struct WorldKnowledge {
    pub discovered_cells: HashSet<IVec2>,
    pub discovered_solids: HashMap<IVec2, String>,
    pub discovered_empty: HashSet<IVec2>,
    pub exploration_queue: VecDeque<IVec2>,
}