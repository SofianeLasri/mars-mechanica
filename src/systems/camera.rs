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
    pub cursor_start_position: Vec2,
    pub camera_start_position: Vec2,
}

pub fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ControlledCamera {
            zoom_speed: 0.1,
            min_zoom: 0.1, // Zoom maximum (vue très proche)
            max_zoom: 5.0, // Zoom minimum (vue très éloignée)
            pan_speed: 1.0, // Augmenté pour un meilleur contrôle
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
        Err(_) => return, // Sortir si pas de caméra trouvée
    };

    // Gestion du zoom avec la molette
    for event in mouse_wheel_events.read() {
        // Ajuster le zoom en fonction du défilement de la molette
        let zoom_delta = -event.y * camera.zoom_speed;
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }

    // Gestion du déplacement avec le clic droit
    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Right {
            if event.state == ButtonState::Pressed {
                camera.is_panning = true;
                // Réinitialisation pour enregistrer la position au premier frame du drag
                camera.cursor_start_position = Vec2::new(0.0, 0.0);
            } else {
                camera.is_panning = false;
            }
        }
    }

    // Si on est en train de déplacer (clic droit enfoncé)
    if camera.is_panning {
        let window = window_query.single();
        let window_size = Vec2::new(window.width(), window.height());
        let cursor_position = window.cursor_position().unwrap_or(window_size / 2.0) - window_size / 2.0;

        if camera.cursor_start_position == Vec2::new(0.0, 0.0) {
            // Enregistrer la position initiale du curseur et de la caméra
            camera.cursor_start_position = cursor_position;
            // Stocker la position actuelle (sans inverser Y à nouveau)
            camera.camera_start_position = Vec2::new(transform.translation.x, transform.translation.y);
        }

        // Calculer le déplacement
        let delta = (camera.cursor_start_position - cursor_position) * camera.pan_speed;
        let new_position = Vec2::new(
            camera.camera_start_position.x + delta.x,
            camera.camera_start_position.y - delta.y  // Inverser le delta Y pour un mouvement intuitif
        );

        // Appliquer le déplacement à la caméra
        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;
    }
}
