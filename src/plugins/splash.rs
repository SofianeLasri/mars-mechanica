use crate::plugins::asset_preloader::UiAssets;
use crate::GameState;
use bevy::prelude::*;

#[derive(Resource)]
struct SplashAnimation {
    current_frame: usize,
    phase: SplashPhase,
    timer: Timer,
}

#[derive(Component)]
struct SplashScreen;

#[derive(Component)]
struct SplashFrame {
    index: usize,
}

// En fait comme je n'ai pas trouvé de décodeur vidéo simple d'utilisation,
// j'ai décidé de faire une animation image par image, puis avec du code :)
#[derive(Debug, PartialEq, Eq)]
enum SplashPhase {
    Glitch,   // 30 premières images (1s)
    Hold,     // 1s de pause
    FadeOut,  // 0.25s de fondu
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SplashScreen), setup_splash)
            .add_systems(
                FixedUpdate,
                update_splash.run_if(in_state(GameState::SplashScreen)),
            )
            .add_systems(OnExit(GameState::SplashScreen), cleanup_splash);
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
        phase: SplashPhase::Glitch,
        timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating), // 30 FPS
    });

    commands.spawn(AudioPlayer::new(ui_assets.sounds[0].clone()));
}

fn update_splash(
    time: Res<Time>,
    mut splash: ResMut<SplashAnimation>,
    mut frames: Query<(&mut Visibility, &SplashFrame)>,
    mut images: Query<(&mut ImageNode, &SplashFrame)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    splash.timer.tick(time.delta());

    match splash.phase {
        SplashPhase::Glitch => {
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

                // Après 30 images (1s), passer à la phase de pause
                if splash.current_frame >= 29 {
                    splash.phase = SplashPhase::Hold;
                    splash.timer = Timer::from_seconds(1.0, TimerMode::Once);
                }
            }
        }
        SplashPhase::Hold => {
            if splash.timer.just_finished() {
                splash.phase = SplashPhase::FadeOut;
                splash.timer = Timer::from_seconds(0.25, TimerMode::Once);
            }
        }
        SplashPhase::FadeOut => {
            let alpha = 1.0 - (splash.timer.elapsed_secs() / 0.5).min(1.0);

            for (mut image, frame) in &mut images {
                if frame.index == 29 {
                    image.color.set_alpha(alpha);
                }
            }

            if splash.timer.just_finished() {
                next_state.set(GameState::MainMenu);
            }
        }
    }
}

fn cleanup_splash(mut commands: Commands, query: Query<Entity, With<SplashScreen>>) {
    commands.remove_resource::<SplashAnimation>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
