mod components;
mod plugins;
mod systems;

use bevy::DefaultPlugins;
use bevy::prelude::{App, AppExtStates, ClearColor, Color, PluginGroup, Resource, States, Window, WindowPlugin};
use crate::plugins::asset_preloader::AssetPreloaderPlugin;
use crate::plugins::game::GamePlugin;
use crate::plugins::splash::SplashPlugin;
use crate::plugins::ui::UiPlugin;

#[derive(Resource)]
struct CliArgs {
    skip_splash: bool,
}

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
    let skip_splash = std::env::args().any(|arg| arg == "--skip-splash");

    App::new()
        .insert_resource(CliArgs { skip_splash })
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