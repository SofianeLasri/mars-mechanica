use crate::plugins::ui::UiCamera;
use crate::GameState;
use bevy::prelude::*;

#[derive(Resource)]
struct SplashAnimation {
    frames: Vec<Handle<Image>>,
    current_frame: usize,
    timer: Timer,
}

#[derive(Component)]
struct SplashScreen;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SplashScreen), setup_splash)
            .add_systems(
                Update,
                update_splash.run_if(in_state(GameState::SplashScreen)),
            )
            .add_systems(OnExit(GameState::SplashScreen), cleanup_splash);
    }
}

fn setup_splash(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, UiCamera));
    let mut frames = Vec::new();
    for i in 1..=61 {
        let path = format!("textures/animations/intro/{:04}.png", i);
        frames.push(asset_server.load(path));
    }

    let first_frame = frames[0].clone();

    commands.insert_resource(SplashAnimation {
        frames,
        current_frame: 0,
        timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
    });

    // Création de l'entité d'affichage
    commands.spawn((
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            SplashScreen,
        ),
        children![(
            ImageNode::from(first_frame),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        )],
    ));
}

fn update_splash(
    time: Res<Time>,
    mut splash: ResMut<SplashAnimation>,
    mut images: Query<&mut ImageNode>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    splash.timer.tick(time.delta());

    if splash.timer.just_finished() {
        if let Some(mut image) = images.iter_mut().next() {
            image.image = splash.frames[splash.current_frame].clone();
        }

        splash.current_frame += 1;

        if splash.current_frame >= 61 {
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
