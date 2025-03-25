use crate::GameState;
use crate::components::ControlledCamera;
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            init.run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            update_camera.run_if(in_state(GameState::InGame)),
        );
    }
}

pub fn init(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ControlledCamera {
            zoom_speed: 0.1,
            min_zoom: 0.75, // Max zoom (+)
            max_zoom: 4.0,  // Max dezoom (-)
            pan_speed: 1.0,
            is_panning: false,
            cursor_start_position: Vec2::new(0.0, 0.0),
            camera_start_position: Vec2::new(0.0, 0.0),
        },
    ));
}

pub fn update_camera(
    mut camera_query: Query<(&mut Projection, &mut Transform, &mut ControlledCamera)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut projection, transform, mut camera) = match camera_query.single_mut() {
        Ok(result) => result,
        Err(_) => return,
    };

    handle_zoom(&mut mouse_wheel_events, &mut projection, &mut camera);

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
        handle_movement(window_query, transform, camera);
    }
}

/// This method handle the zoom of the camera using the mouse wheel events
fn handle_zoom(
    mouse_wheel_events: &mut EventReader<MouseWheel>,
    projection: &mut Mut<Projection>,
    camera: &mut Mut<ControlledCamera>,
) {
    for event in mouse_wheel_events.read() {
        let zoom_delta = -event.y * camera.zoom_speed;

        // On déstructure la Projection pour accéder à la variante Orthographique
        if let Projection::Orthographic(ortho_projection) = &mut **projection {
            ortho_projection.scale =
                (ortho_projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
        } else {
            warn!("The projection is not Orthographic");
        }
    }
}

/// This method handle the movement of the camera using the cursor position, relative to the start position
fn handle_movement(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut transform: Mut<Transform>,
    mut camera: Mut<ControlledCamera>,
) {
    let cursor_position = get_cursor_window_position(window_query);

    if camera.cursor_start_position == Vec2::new(0.0, 0.0) {
        camera.cursor_start_position = cursor_position;
        camera.camera_start_position = Vec2::new(transform.translation.x, transform.translation.y);
    }

    let delta = (camera.cursor_start_position - cursor_position) * camera.pan_speed;
    let new_position = Vec2::new(
        camera.camera_start_position.x + delta.x,
        camera.camera_start_position.y - delta.y,
    );

    transform.translation.x = new_position.x;
    transform.translation.y = new_position.y;
}

/// This method returns the cursor position relative to the center of the window
pub fn get_cursor_window_position(window_query: Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    let window = window_query.single().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let cursor_position = window.cursor_position().unwrap_or(window_size / 2.0) - window_size / 2.0;
    cursor_position
}

/// This method returns the cursor position in the world coordinates
pub fn get_cursor_world_position(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection)>,
) -> Vec2 {
    let cursor_position = get_cursor_window_position(window_query);

    let Ok((_, transform, projection)) = camera_query.single() else {
        return Vec2::ZERO;
    };

    match projection {
        Projection::Orthographic(ortho) => {
            let translation = transform.translation();
            Vec2::new(
                translation.x + cursor_position.x * ortho.scale,
                translation.y - cursor_position.y * ortho.scale,
            )
        }
        _ => Vec2::ZERO,
    }
}
