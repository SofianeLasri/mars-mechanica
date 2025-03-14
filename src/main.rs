mod components;
mod plugins;
mod systems;

use crate::plugins::{EntityPlugin, TerrainPlugin};
use crate::systems::{
    debug_text, generate_world, init_camera, update_camera, update_debug_camera_text,
    hover_detection, block_click_handler
};
use bevy::DefaultPlugins;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityPlugin)
        .add_plugins(TerrainPlugin)
        .add_systems(Startup, (debug_text, init_camera, generate_world))
        .add_systems(
            FixedUpdate,
            (
                update_debug_camera_text,
                update_camera,
                hover_detection,
                block_click_handler,
            )
        )
        .run();
}
