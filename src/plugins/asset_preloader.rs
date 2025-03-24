use crate::components::{
    LoadingBar, LoadingProgress, LOADING_BAR_COLOR, LOADING_BAR_ERROR_COLOR, LOADING_PROGRESS_COLOR,
};
use crate::plugins::ui::UiCamera;
use crate::GameState;
use bevy::app::Update;
use bevy::asset::{AssetServer, Handle, LoadState};
use bevy::audio::AudioSource;
use bevy::color::Color;
use bevy::image::Image;
use bevy::log::error;
use bevy::prelude::{
    default, in_state, BackgroundColor, Camera2d, Commands, Entity, Font, IntoScheduleConfigs,
    NextState, Node, OnEnter, OnExit, Plugin, PositionType, Query, Res, ResMut, Resource, Val,
    With,
};

#[derive(Resource, Default)]
pub struct UiAssets {
    pub(crate) images: Vec<Handle<Image>>,
    pub(crate) sounds: Vec<Handle<AudioSource>>,
    pub(crate) fonts: Vec<Handle<Font>>,
}

#[derive(Resource, Default)]
pub struct LoadingState {
    total: usize,
    loaded: usize,
    error: bool,
}

pub struct AssetPreloaderPlugin;

impl Plugin for AssetPreloaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<UiAssets>()
            .init_resource::<LoadingState>()
            .add_systems(
                OnEnter(GameState::AssetLoading),
                (preload_assets, setup_loading_bar),
            )
            .add_systems(
                Update,
                (check_assets_loaded, update_loading_bar).run_if(in_state(GameState::AssetLoading)),
            )
            .add_systems(OnExit(GameState::AssetLoading), cleanup_loading_bar);
    }
}

fn preload_assets(
    asset_server: Res<AssetServer>,
    mut ui_assets: ResMut<UiAssets>,
    mut loading_state: ResMut<LoadingState>,
) {
    let mut handles = Vec::new();
    for i in 1..=61 {
        let path = format!("textures/animations/intro/{:04}.png", i);
        let handle = asset_server.load(path);
        handles.push(handle);
    }
    handles.push(asset_server.load("textures/ui/background.png"));
    ui_assets.images = handles;

    let mut audio_handles = Vec::new();
    audio_handles.push(asset_server.load("sounds/intro.ogg"));
    audio_handles.push(asset_server.load("sounds/menu-hover.wav"));
    audio_handles.push(asset_server.load("sounds/menu-select.wav"));
    ui_assets.sounds = audio_handles;

    let mut font_handles = Vec::new();
    font_handles.push(asset_server.load("fonts/inter-regular.ttf"));
    font_handles.push(asset_server.load("fonts/inter-bold.ttf"));
    ui_assets.fonts = font_handles;

    loading_state.total = ui_assets.images.len() + ui_assets.sounds.len() + ui_assets.fonts.len();
    loading_state.loaded = 0;
    loading_state.error = false;
}

fn setup_loading_bar(mut commands: Commands) {
    commands.spawn((Camera2d::default(), UiCamera));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(8.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(LOADING_BAR_COLOR),
            LoadingBar,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(LOADING_PROGRESS_COLOR),
                LoadingProgress,
            ));
        });
}

fn check_assets_loaded(
    asset_server: Res<AssetServer>,
    splash_assets: Res<UiAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<LoadingState>,
) {
    let mut loaded = 0;
    let mut has_error = false;

    for handle in &splash_assets.images {
        match asset_server.get_load_state(handle).unwrap() {
            LoadState::Loaded => loaded += 1,
            LoadState::Failed(_) => has_error = true,
            _ => {}
        }
    }

    for handle in &splash_assets.sounds {
        match asset_server.get_load_state(handle).unwrap() {
            LoadState::Loaded => loaded += 1,
            LoadState::Failed(_) => has_error = true,
            _ => {}
        }
    }

    for handle in &splash_assets.fonts {
        match asset_server.get_load_state(handle).unwrap() {
            LoadState::Loaded => loaded += 1,
            LoadState::Failed(_) => has_error = true,
            _ => {}
        }
    }

    loading_state.loaded = loaded;
    loading_state.error = has_error;

    if has_error {
        error!("Erreur de chargement d'un asset!");
        return;
    }

    if loaded == loading_state.total {
        next_state.set(GameState::SplashScreen);
    }
}

fn update_loading_bar(
    loading_state: Res<LoadingState>,
    mut progress_query: Query<&mut Node, With<LoadingProgress>>,
    mut bar_query: Query<&mut BackgroundColor, With<LoadingBar>>,
) {
    if let Ok(mut style) = progress_query.single_mut() {
        let progress = loading_state.loaded as f32 / loading_state.total as f32 * 100.0;
        style.width = Val::Percent(progress);
    }

    if loading_state.error {
        if let Ok(mut color) = bar_query.single_mut() {
            *color = LOADING_BAR_ERROR_COLOR.into();
        }
    }
}

fn cleanup_loading_bar(mut commands: Commands, query: Query<Entity, With<LoadingBar>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
