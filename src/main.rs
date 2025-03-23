mod components;
mod plugins;
mod systems;

use crate::plugins::ui;
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    Loading,
    InGame,
}

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
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_plugins((ui::UiPlugin))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}