use crate::plugins::camera::CameraPlugin;
use crate::plugins::debug_ui::DebugUiPlugin;
use crate::plugins::interaction::InteractionPlugin;
use crate::plugins::robot::RobotPlugin;
use crate::plugins::terrain::TerrainPlugin;
// Nouvelle ligne
use crate::systems::world_generator::generate_world;
use crate::GameState;
use bevy::prelude::{App, OnEnter, Plugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraPlugin,
            TerrainPlugin,
            DebugUiPlugin,
            InteractionPlugin,
            RobotPlugin,
        ))
            .add_systems(OnEnter(GameState::Loading), generate_world);
    }
}
