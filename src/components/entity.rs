use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct WorldEntity {
    pub entity: Entity,
}