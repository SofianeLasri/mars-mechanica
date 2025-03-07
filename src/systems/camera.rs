use crate::components::ControlledCamera;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;

pub fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ControlledCamera {
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 3.0,
            pan_speed: 1.0,
            is_panning: false,
            cursor_start_position: Vec2::new(0.0, 0.0),
            camera_start_position: Vec2::new(0.0, 0.0),
        },
    ));
}

pub fn update_camera(
    mut camera_query: Query<(
        &mut OrthographicProjection,
        &mut Transform,
        &mut ControlledCamera,
    )>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    window_query: Query<&Window>,
) {
    let (mut projection, mut transform, mut camera) = match camera_query.get_single_mut() {
        Ok(result) => result,
        Err(_) => return,
    };

    for event in mouse_wheel_events.read() {
        let zoom_delta = -event.y * camera.zoom_speed;
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }

    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Right {
            if event.state == ButtonState::Pressed {
                camera.is_panning = true;
                camera.cursor_start_position = Vec2::new(0.0, 0.0);
            } else {
                camera.is_panning = false;
            }
        }
    }

    if camera.is_panning {
        let window = window_query.single();
        let window_size = Vec2::new(window.width(), window.height());
        let cursor_position = window.cursor_position().unwrap_or(window_size / 2.0) - window_size / 2.0;

        if camera.cursor_start_position == Vec2::new(0.0, 0.0) {
            camera.cursor_start_position = cursor_position;
            camera.camera_start_position = Vec2::new(transform.translation.x, transform.translation.y);
        }

        let delta = (camera.cursor_start_position - cursor_position) * camera.pan_speed;
        let new_position = Vec2::new(
            camera.camera_start_position.x + delta.x,
            camera.camera_start_position.y - delta.y
        );

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;
    }
}
