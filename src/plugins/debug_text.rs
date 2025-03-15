use bevy::prelude::*;

#[derive(Component)]
pub struct DebugCameraText;

#[derive(Component)]
pub struct DebugHoverText;

pub struct DebugTextPlugin;

impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, debug_text)
            .add_systems(FixedUpdate, update_debug_camera_text);
    }
}
pub fn debug_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Text::new("Mouse position: (0.0, 0.0)"),
        TextFont {
            font: asset_server.load("fonts/inter-regular.ttf"),
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        DebugCameraText,
    ));
    
    commands.spawn((
        Text::new("Hovered cell: None"),
        TextFont {
            font: asset_server.load("fonts/inter-regular.ttf"),
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(5.0),
            ..default()
        },
        DebugHoverText,
    ));
}

pub fn update_debug_camera_text(
    mut text_query: Query<Entity, With<DebugCameraText>>,
    window_query: Query<&Window>,
    mut writer: TextUiWriter,
) {
    let window = window_query.single();

    let cursor_position = if let Some(position) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let position = position - window_size / 2.0;
        position
    } else {
        Vec2::new(0.0, 0.0)
    };
    
    *writer.text(text_query.single_mut(), 0) = format!("Mouse position: ({:.1}, {:.1})", cursor_position.x, cursor_position.y);
}