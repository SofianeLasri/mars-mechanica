use crate::GameState;
use crate::components::TEXT_COLOR;
use crate::components::splash::{
    InfoScreen, InfoText, SplashAnimation, SplashFrame, SplashPhase, SplashScreen,
};
use crate::plugins::asset_preloader::UiAssets;
use bevy::prelude::*;

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

    splash_parent.with_children(|parent| {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(96.0)),
                ..default()
            },
            Visibility::Hidden,
            InfoScreen,
            children![(
                Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."),
                TextFont {
                    font: ui_assets.fonts.first().unwrap().clone(),
                    font_size: 24.0,
                    line_height: Default::default(),
                    font_smoothing: Default::default(),
                },
                TextColor(TEXT_COLOR),
                InfoText,
            )],
        ));
    });

    commands.insert_resource(SplashAnimation {
        current_frame: 0,
        phase: SplashPhase::Glitch,
        timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
    });

    commands.spawn((
        AudioPlayer(ui_assets.sounds[0].clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn update_splash(
    time: Res<Time>,
    mut splash: ResMut<SplashAnimation>,
    mut frames: Query<(&mut Visibility, &SplashFrame)>,
    mut images: Query<(&mut ImageNode, &SplashFrame)>,
    mut info_screen: Query<&mut Visibility, (With<InfoScreen>, Without<SplashFrame>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    splash.timer.tick(time.delta());

    match splash.phase {
        SplashPhase::Glitch => {
            if splash.timer.just_finished() {
                splash.current_frame += 1;

                for (mut visibility, frame) in &mut frames {
                    *visibility = if frame.index == splash.current_frame {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }

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
            let alpha = 1.0 - (splash.timer.elapsed_secs() / 0.25).min(1.0);
            for (mut image, frame) in &mut images {
                if frame.index == 29 {
                    image.color.set_alpha(alpha);
                }
            }

            if splash.timer.just_finished() {
                splash.phase = SplashPhase::InfoScreen;
                splash.timer = Timer::from_seconds(4.0, TimerMode::Once);

                if let Ok(mut overlay) = info_screen.single_mut() {
                    *overlay = Visibility::Visible;
                }
            }
        }
        SplashPhase::InfoScreen => {
            if splash.timer.just_finished() {
                next_state.set(GameState::MainMenu);
            }
        }
    }
}

fn cleanup_splash(mut commands: Commands, query: Query<Entity, With<SplashScreen>>) {
    commands.remove_resource::<SplashAnimation>();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
