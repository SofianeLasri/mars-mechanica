use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Component)]
pub struct ExplorerRobot {
    pub speed: f32,
    pub target_position: IVec2,
    pub is_moving: bool,
    pub move_timer: f32,
    pub previous_position: Option<IVec2>,
    pub follow_direction: i8, // 1 pour sens horaire, -1 pour anti-horaire
}

#[derive(Resource, Default)]
pub struct WorldKnowledge {
    pub discovered_cells: HashSet<IVec2>,
    pub discovered_solids: HashMap<IVec2, String>,
    pub discovered_empty: HashSet<IVec2>,
    pub exploration_queue: VecDeque<IVec2>,
}