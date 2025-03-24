use crate::components::{LoadingBar, LoadingProgress};
use crate::plugins::ui::UiCamera;
use crate::GameState;
use bevy::asset::LoadState;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UiAssets {
    pub(crate) images: Vec<Handle<Image>>,
    pub(crate) sounds: Vec<Handle<AudioSource>>,
    pub(crate) fonts: Vec<Handle<Font>>,
}

#[derive(Resource)]
struct SplashAnimation {
    current_frame: usize,
    timer: Timer,
}

#[derive(Component)]
struct SplashScreen;

#[derive(Component)]
struct SplashFrame {
    index: usize,
}

#[derive(Resource, Default)]
struct LoadingState {
    total: usize,
    loaded: usize,
    error: bool,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
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
            .add_systems(OnExit(GameState::AssetLoading), cleanup_loading_bar)
            .add_systems(OnEnter(GameState::SplashScreen), setup_splash)
            .add_systems(
                FixedUpdate,
                update_splash.run_if(in_state(GameState::SplashScreen)),
            )
            .add_systems(OnExit(GameState::SplashScreen), cleanup_splash);
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
            BackgroundColor(Color::srgb(39.0 / 255.0, 39.0 / 255.0, 39.0 / 255.0)),
            LoadingBar,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0)),
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
            *color = Color::srgb(0.5, 0.0, 0.0).into(); // Rouge foncé
        }
    }
}

fn cleanup_loading_bar(mut commands: Commands, query: Query<Entity, With<LoadingBar>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_splash(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let mut splash_parent = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        SplashScreen,
    ));

    for (i, handle) in ui_assets.images.iter().enumerate() {
        splash_parent.with_children(|parent| {
            parent.spawn((
                ImageNode::from(handle.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                if i == 0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                SplashFrame { index: i },
            ));
        });
    }

    commands.insert_resource(SplashAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
    });

    commands.spawn(AudioPlayer::new(ui_assets.sounds[0].clone()));
}

fn update_splash(
    time: Res<Time>,
    mut splash: ResMut<SplashAnimation>,
    mut frames: Query<(&mut Visibility, &SplashFrame)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    splash.timer.tick(time.delta());

    if splash.timer.just_finished() {
        let previous_frame = splash.current_frame;
        splash.current_frame += 1;

        for (mut visibility, frame) in &mut frames {
            if frame.index == previous_frame {
                *visibility = Visibility::Hidden;
            }
        }

        for (mut visibility, frame) in &mut frames {
            if frame.index == splash.current_frame {
                *visibility = Visibility::Visible;
            }
        }

        if splash.current_frame >= 60 {
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_splash(mut commands: Commands, query: Query<Entity, With<SplashScreen>>) {
    commands.remove_resource::<SplashAnimation>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
