use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Component, Clone)]
pub struct ExplorerRobot {
    pub speed: f32,
    pub target_position: IVec2,
    pub is_moving: bool,
    pub move_timer: f32,
    pub previous_position: Option<IVec2>,
    pub follow_direction: i8, // 1 pour sens horaire, -1 pour anti-horaire
}

#[derive(Component, Clone)]
pub struct MinerRobot {
    pub speed: f32,
    pub target_position: IVec2,
    pub is_moving: bool,
    pub move_timer: f32,
    pub current_task: MinerTask,
    pub spawn_position: IVec2,
    pub collected_resources: Vec<(String, i32)>, // (entity_id, quantity)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinerTask {
    Idle,
    MovingToTarget,
    Mining,
    ReturningToSpawn,
}

#[derive(Resource, Default, Clone)]
pub struct WorldKnowledge {
    pub discovered_cells: HashSet<IVec2>,
    pub discovered_solids: HashMap<IVec2, String>,
    pub discovered_empty: HashSet<IVec2>,
    pub exploration_queue: VecDeque<IVec2>,
}

#[derive(Component)]
pub struct DebugRobotText;

pub enum RobotCommand {
    PlanExplorerMovement {
        robot_entity: Entity,
        position: IVec2,
        robot_data: ExplorerRobot,
        delta_time: f32,
    },
    PlanMinerMovement {
        robot_entity: Entity,
        position: IVec2,
        robot_data: MinerRobot,
        delta_time: f32,
    },
    UpdateWorldKnowledge(Arc<WorldKnowledge>),
    Shutdown,
}

pub enum RobotResult {
    ExplorerMovementPlan {
        entity: Entity,
        new_target: IVec2,
        is_moving: bool,
        previous_position: Option<IVec2>,
        follow_direction: i8,
    },
    MinerMovementPlan {
        entity: Entity,
        new_target: IVec2,
        is_moving: bool,
        current_task: MinerTask,
    },
}

#[derive(Resource)]
pub struct RobotThreadManager {
    pub(crate) command_sender: Sender<RobotCommand>,
    pub(crate) result_receiver: Receiver<RobotResult>,
    pub(crate) shared_knowledge: Arc<Mutex<WorldKnowledge>>,
}