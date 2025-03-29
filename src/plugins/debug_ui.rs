use crate::components::UiAssets;
use crate::GameState;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::ui::{FlexDirection, UiRect};

#[derive(Component)]
pub struct DebugCameraText;

#[derive(Component)]
pub struct DebugHoverText;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), init)
            .add_systems(
                FixedUpdate,
                update_debug_camera_text.run_if(in_state(GameState::InGame)),
            );
    }
}
pub fn init(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor::from(BLACK),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Mouse position: (0.0, 0.0)"),
                        TextFont {
                            font: ui_assets.fonts.first().unwrap().clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        DebugCameraText,
                    ));
                    parent.spawn((
                        Text::new("Hovered cell: None"),
                        TextFont {
                            font: ui_assets.fonts.first().unwrap().clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        DebugHoverText,
                    ));
                });
        });
}

pub fn update_debug_camera_text(
    text_query: Query<Entity, With<DebugCameraText>>,
    window_query: Query<&Window>,
    mut writer: TextUiWriter,
) {
    let window = window_query.single().unwrap();

    let cursor_position = if let Some(position) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let position = position - window_size / 2.0;
        position
    } else {
        Vec2::new(0.0, 0.0)
    };

    let text_entity = text_query.single().unwrap();
    *writer.text(text_entity, 0) = format!(
        "Mouse position: ({:.1}, {:.1})",
        cursor_position.x, cursor_position.y
    );
}
