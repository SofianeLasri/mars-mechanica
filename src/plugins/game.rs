use crate::plugins::camera::CameraPlugin;
use crate::plugins::debug_text::DebugTextPlugin;
use crate::plugins::entity::EntityPlugin;
use crate::plugins::interaction::InteractionPlugin;
use crate::plugins::terrain::TerrainPlugin;
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
            DebugTextPlugin,
            InteractionPlugin,
        ))
            .add_systems(OnEnter(GameState::Loading), (generate_world));
    }
}