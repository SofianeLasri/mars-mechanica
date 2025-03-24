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

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .add_systems(OnEnter(GameState::AssetLoading), preload_assets)
            .add_systems(
                Update,
                check_assets_loaded.run_if(in_state(GameState::AssetLoading)),
            )
            .add_systems(OnEnter(GameState::SplashScreen), setup_splash)
            .add_systems(
                FixedUpdate,
                update_splash.run_if(in_state(GameState::SplashScreen)),
            )
            .add_systems(OnExit(GameState::SplashScreen), cleanup_splash);
    }
}

fn preload_assets(asset_server: Res<AssetServer>, mut ui_assets: ResMut<UiAssets>) {
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
}

fn check_assets_loaded(
    asset_server: Res<AssetServer>,
    splash_assets: Res<UiAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let images_loaded = splash_assets.images.iter().all(|handle| {
        matches!(asset_server.get_load_state(handle).unwrap(), LoadState::Loaded)
    });

    let audio_loaded = splash_assets.sounds.iter().all(|handle| {
        matches!(asset_server.get_load_state(handle).unwrap(), LoadState::Loaded)
    });

    let fonts_loaded = splash_assets.fonts.iter().all(|handle| {
        matches!(asset_server.get_load_state(handle).unwrap(), LoadState::Loaded)
    });

    if images_loaded && audio_loaded && fonts_loaded {
        info!("All splash assets loaded");
        next_state.set(GameState::SplashScreen);
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

    commands.spawn((Camera2d::default(), UiCamera));
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
