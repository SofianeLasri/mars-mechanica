mod components;
mod plugins;
mod systems;

use crate::plugins::EntityPlugin;
use crate::systems::{debug_text, generate_world, init_camera, update_camera, update_debug_text};
use bevy::DefaultPlugins;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityPlugin)
        .add_systems(Startup, (init_camera, debug_text, generate_world))
        .add_systems(Update, (update_debug_text, update_camera))
        .run();
}

