mod components;
mod plugins;
mod systems;

use crate::plugins::asset_preloader::AssetPreloaderPlugin;
use crate::plugins::discord::DiscordPlugin;
use crate::plugins::game::GamePlugin;
use crate::plugins::splash::SplashPlugin;
use crate::plugins::ui::UiPlugin;
use bevy::prelude::{App, AppExtStates, ClearColor, Color, Entity, NonSend, PluginGroup, Query, Resource, Startup, States, Window, WindowPlugin, With};
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use ::image::open;
use winit::window::Icon;

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
    SeedInput,
    Loading,
    InGame,
}

fn main() {
    let skip_splash = std::env::args().any(|arg| arg == "--skip-splash");

    App::new()
        .insert_resource(CliArgs { skip_splash })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mars mechanica - Pre pre alpha".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .init_state::<GameState>()
        .add_plugins((AssetPreloaderPlugin, SplashPlugin, UiPlugin, GamePlugin, DiscordPlugin))
        .add_systems(Startup, set_window_icon)
        .run();
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window_entity = primary_window_query.single().unwrap();
    let primary_window = windows.get_window(primary_window_entity).unwrap();

    let (icon_rgba, icon_width, icon_height) = {
        let image = open("assets/textures/ui/logo.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary_window.set_window_icon(Some(icon));
}
