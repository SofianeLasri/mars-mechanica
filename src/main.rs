mod components;
mod plugins;
mod systems;

use crate::plugins::{DebugTextPlugin, EntityPlugin, TerrainPlugin};
use crate::systems::{
    block_click_handler, generate_world, hover_detection, init_camera, update_camera,
};
use bevy::DefaultPlugins;
use bevy::prelude::{App, FixedUpdate, Startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((EntityPlugin, TerrainPlugin, DebugTextPlugin))
        .add_systems(Startup, (init_camera, generate_world))
        .add_systems(
            FixedUpdate,
            (update_camera, hover_detection, block_click_handler),
        )
        .run();
}
