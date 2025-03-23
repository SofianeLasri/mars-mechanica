use crate::plugins::camera::CameraPlugin;
use crate::plugins::debug_text::DebugTextPlugin;
use crate::plugins::entity::EntityPlugin;
use crate::plugins::interaction::InteractionPlugin;
use crate::plugins::terrain::TerrainPlugin;
use crate::systems::world_generator::generate_world;
use crate::GameState;
use bevy::prelude::{App, Commands, NextState, OnEnter, Plugin, ResMut, Resource};

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

#[derive(Resource, Default)]
pub struct WorldGenState {
    pub(crate) generated: bool,
}

fn setup_world_generation(mut commands: Commands) {
    commands.init_resource::<WorldGenState>();
}

fn check_world_generation(
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<WorldGenState>,
) {
    if state.generated {
        next_state.set(GameState::InGame);
        state.generated = false;
    }
}