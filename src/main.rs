mod components;
mod plugins;
mod systems;

use crate::plugins::{DebugTextPlugin, EntityPlugin, InteractionPlugin, TerrainPlugin};
use crate::systems::{generate_world, init_camera, update_camera};
use bevy::DefaultPlugins;
use bevy::prelude::{App, FixedUpdate, Startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            EntityPlugin,
            TerrainPlugin,
            DebugTextPlugin,
            InteractionPlugin,
        ))
        .add_systems(Startup, (init_camera, generate_world))
        .add_systems(FixedUpdate, update_camera)
        .run();
}
