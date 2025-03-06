use bevy::prelude::*;

pub fn debug_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Text::new("Test"),
        TextFont {
            font: asset_server.load("fonts/inter-regular.ttf"),
            font_size: 50.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }
    ));
}