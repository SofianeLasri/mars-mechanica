use crate::components::{LoadingBar, LoadingProgress, TerrainAssets, UiAssets, UiCamera, LOADING_BAR_COLOR, LOADING_BAR_ERROR_COLOR, LOADING_PROGRESS_COLOR};
use crate::{CliArgs, GameState};
use bevy::app::Update;
use bevy::asset::{AssetServer, Handle, LoadState};
use bevy::image::Image;
use bevy::prelude::{
    default, error, in_state, App, BackgroundColor, Camera2d, Commands, Entity, Font,
    IntoScheduleConfigs, NextState, Node, OnEnter, OnExit, Plugin, PositionType, Query, Res, ResMut, Resource, Val,
    With,
};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct LoadingState {
    total: usize,
    loaded: usize,
    error: bool,
}

pub struct AssetPreloaderPlugin;

impl Plugin for AssetPreloaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainAssets>()
            .init_resource::<UiAssets>()
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
    mut terrain_assets: ResMut<TerrainAssets>,
    mut loading_state: ResMut<LoadingState>,
) {
    let mut ui_images: Vec<Handle<Image>> = Vec::new();
    for i in 1..=30 {
        let path = format!("textures/animations/intro/{:04}.png", i);
        ui_images.push(asset_server.load(path));
    }
    ui_images.push(asset_server.load("textures/ui/background.png"));
    ui_assets.images = ui_images;

    ui_assets.sounds = vec![
        asset_server.load("sounds/intro.ogg"),
        asset_server.load("sounds/menu-hover.wav"),
        asset_server.load("sounds/menu-select.wav"),
    ];

    ui_assets.fonts = vec![
        asset_server.load("fonts/inter-regular.ttf"),
        asset_server.load("fonts/inter-bold.ttf"),
    ];

    let material_ids = vec!["rock", "basalt", "olivine", "red_crystal"];
    let sprite_names = vec![
        "alone",
        "bottom-right",
        "bottom",
        "left-bottom-right",
        "left-bottom",
        "left-right",
        "top-left",
        "left",
        "right",
        "top-bottom-right",
        "top-bottom",
        "top-left-bottom-right",
        "top-left-bottom",
        "top-left-right",
        "top-right",
        "top",
    ];

    let mut terrain_materials: HashMap<String, HashMap<String, Handle<Image>>> = HashMap::new();
    let mut terrain_asset_count = 0;
    for material in material_ids.iter() {
        let mut sprites: HashMap<String, Handle<Image>> = HashMap::new();
        for sprite_name in sprite_names.iter() {
            let path = format!("textures/terrain/{}/{}.png", material, sprite_name);
            sprites.insert(sprite_name.to_string(), asset_server.load(path));
            terrain_asset_count += 1;
        }
        terrain_materials.insert(material.to_string(), sprites);
    }
    terrain_assets.materials = terrain_materials;

    loading_state.total = ui_assets.images.len()
        + ui_assets.sounds.len()
        + ui_assets.fonts.len()
        + terrain_asset_count;
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
    terrain_assets: Res<TerrainAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<LoadingState>,
    cli_args: Res<CliArgs>,
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

    for (_material, sprites_map) in terrain_assets.materials.iter() {
        for (_sprite_name, handle) in sprites_map.iter() {
            match asset_server.get_load_state(handle).unwrap() {
                LoadState::Loaded => loaded += 1,
                LoadState::Failed(_) => has_error = true,
                _ => {}
            }
        }
    }

    loading_state.loaded = loaded;
    loading_state.error = has_error;

    if has_error {
        error!("Erreur de chargement d'un asset!");
        return;
    }

    if loaded == loading_state.total {
        if cli_args.skip_splash {
            next_state.set(GameState::MainMenu);
        } else {
            next_state.set(GameState::SplashScreen);
        }
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
        commands.entity(entity).despawn();
    }
}
