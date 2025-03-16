mod components;
mod plugins;
mod systems;

use crate::plugins::{
    CameraPlugin, DebugTextPlugin, EntityPlugin, InteractionPlugin, TerrainPlugin,
};
use crate::systems::generate_world;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Mars mechanica - Pre pre alpha".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            })
        )
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
