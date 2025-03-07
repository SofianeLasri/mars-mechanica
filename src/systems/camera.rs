use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;

// Composant pour marquer et configurer notre caméra contrôlable
#[derive(Component)]
pub struct ControlledCamera {
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub pan_speed: f32,
    pub is_panning: bool,
}

pub fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ControlledCamera {
            zoom_speed: 0.1,
            min_zoom: 0.1, // Zoom maximum (vue très proche)
            max_zoom: 5.0, // Zoom minimum (vue très éloignée)
            pan_speed: 0.1,
            is_panning: false,
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
    mut mouse_motion_events: EventReader<MouseMotion>,
    window_query: Query<&Window>,
) {
    let (mut projection, mut transform, mut camera) = match camera_query.get_single_mut() {
        Ok(result) => result,
        Err(_) => return, // Sortir si pas de caméra trouvée
    };

    // Gestion du zoom avec la molette
    for event in mouse_wheel_events.read() {
        // Ajuster le zoom en fonction du défilement de la molette
        let zoom_delta = -event.y * camera.zoom_speed;
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }

    // Gestion du déplacement avec le clic droit
    /*for event in mouse_button_input_events.read() {
        info!("{:?}", event);
    }*/
    let mouse_button_input_event = mouse_button_input_events.read().last().cloned();
    let is_right_mouse_button_pressed = mouse_button_input_event.map_or(false, |event| {
        event.button == MouseButton::Right && event.state == ButtonState::Pressed
    });

    let is_right_mouse_button_released = mouse_button_input_event.map_or(false, |event| {
        event.button == MouseButton::Right && event.state == ButtonState::Released
    });

    if is_right_mouse_button_pressed {
        camera.is_panning = true;
    } else if is_right_mouse_button_released {
        camera.is_panning = false;
    }

    if camera.is_panning {
        for event in mouse_motion_events.read() {

            // We need to get the window size and the current zoom to calculate the pan speed
            let window = window_query.single();
            let window_size = Vec2::new(window.width(), window.height());

            // The mouse delta must be normalized and the origin must be at the center of the window
            let mouse_delta = if let Some(position) = window.cursor_position() {
                let window_size = Vec2::new(window.width(), window.height());
                let position = (position - window_size / 2.0) / -2.0;
                position
            } else {
                Vec2::new(0.0, 0.0)
            };

            info!("{:?}", mouse_delta);

            /*let pan_delta = Vec2::new(
                -event.delta.x * camera.pan_speed / projection.scale,
                event.delta.y * camera.pan_speed / projection.scale,
            );*/

            let pan_delta = Vec2::new(
                -mouse_delta.x * camera.pan_speed / projection.scale,
                mouse_delta.y * camera.pan_speed / projection.scale,
            );

            // Appliquer le déplacement à la caméra
            transform.translation.x += pan_delta.x;
            transform.translation.y += pan_delta.y;
        }
    }
}
