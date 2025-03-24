mod components;
mod plugins;
mod systems;

use crate::plugins::asset_preloader::AssetPreloaderPlugin;
use crate::plugins::game::GamePlugin;
use crate::plugins::splash::SplashPlugin;
use crate::plugins::ui::UiPlugin;
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    AssetLoading,
    SplashScreen,
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
        .insert_resource(ClearColor(Color::BLACK))
        .init_state::<GameState>()
        .add_plugins((AssetPreloaderPlugin, SplashPlugin, UiPlugin, GamePlugin))
        .run();
}