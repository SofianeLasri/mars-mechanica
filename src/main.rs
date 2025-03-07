mod components;
mod plugins;
mod systems;

use crate::plugins::EntityPlugin;
use crate::systems::{debug_text, update_debug_text};
use bevy::DefaultPlugins;
use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityPlugin)
        .add_systems(Startup, (setup, debug_text))
        .add_systems(Update, update_debug_text)
        .run();
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Camera2d);
}
