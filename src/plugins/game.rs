use crate::plugins::camera::CameraPlugin;
use crate::plugins::debug_ui::DebugUiPlugin;
use crate::plugins::entity::EntityPlugin;
use crate::plugins::interaction::InteractionPlugin;
use crate::plugins::terrain::TerrainPlugin;
use crate::plugins::robot::RobotPlugin; // Nouvelle ligne
use crate::systems::world_generator::generate_world;
use crate::GameState;
use bevy::prelude::{App, OnEnter, Plugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EntityPlugin,
            CameraPlugin,
            TerrainPlugin,
            DebugUiPlugin,
            InteractionPlugin,
            RobotPlugin,
        ))
            .add_systems(OnEnter(GameState::Loading), generate_world);
    }
}
