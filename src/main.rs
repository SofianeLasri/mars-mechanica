mod components;
mod plugins;
mod systems;

use crate::plugins::{
    CameraPlugin, DebugTextPlugin, EntityPlugin, InteractionPlugin, TerrainPlugin,
};
use crate::systems::generate_world;
use bevy::prelude::{App, Startup};
use bevy::DefaultPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            EntityPlugin,
            CameraPlugin,
            TerrainPlugin,
            DebugTextPlugin,
            InteractionPlugin,
        ))
        .add_systems(Startup, generate_world)
        .run();
}
