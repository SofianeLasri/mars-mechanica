use bevy::math::Vec2;
use bevy::prelude::Component;

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