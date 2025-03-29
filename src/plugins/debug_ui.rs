use bevy::color::palettes::css::BLACK;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::{FlexDirection, UiRect};

use crate::components::UiAssets;
use crate::GameState;

#[derive(Component)]
pub struct DebugCameraText;

#[derive(Component)]
pub struct DebugHoverText;

#[derive(Component)]
struct FpsCounterText;

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
    let root_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor::from(BLACK),
        ))
        .id();

    commands.entity(root_entity).with_related::<ChildOf>(|child_spawner| {
        spawn_debug_column(child_spawner, |col_spawner| {
            spawn_debug_text(col_spawner, &ui_assets, "Mouse position: (0.0, 0.0)", DebugCameraText);
            spawn_debug_text(col_spawner, &ui_assets, "Hovered cell: None", DebugHoverText);
        });

        spawn_debug_column(child_spawner, |col_spawner| {
            spawn_debug_text(col_spawner, &ui_assets, "FPS: --", FpsCounterText);
        });
    });
}

fn spawn_debug_text<M: Component>(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    ui_assets: &UiAssets,
    text_content: &str,
    marker: M,
) {
    spawner.spawn((
        Text::new(text_content),
        TextFont {
            font: ui_assets.fonts.first().unwrap().clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        marker,
    ));
}

fn spawn_debug_column(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    spawn_contents: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    spawner
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_related::<ChildOf>(|col_spawner| {
            spawn_contents(col_spawner);
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
        position - window_size / 2.0
    } else {
        Vec2::ZERO
    };

    let text_entity = text_query.single().unwrap();
    *writer.text(text_entity, 0) = format!(
        "Mouse position: ({:.1}, {:.1})",
        cursor_position.x, cursor_position.y
    );
}
