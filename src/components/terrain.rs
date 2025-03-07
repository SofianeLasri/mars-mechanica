use bevy::prelude::{Component, Entity, Image};
use crate::components::WorldEntity;

#[derive(Component)]
pub struct TerrainCell {
    pub x: i32,
    pub y: i32,
    pub solid_object: SolidObject,
    pub entity: WorldEntity,
}

#[derive(Component)]
pub struct SolidObject {
    pub material: WorldMaterial,
    pub health: f32,
}

#[derive(Component)]
pub struct WorldMaterial {
    pub name: String,
    pub strength: f32,
    pub entity: WorldEntity,
    pub drop_count_min: i32,
    pub drop_count_max: i32,
    pub can_be_merged: bool,
    pub plain_texture: Image,
    pub side_texture: Image,
    pub inter_corner_texture: Image,
    pub outer_corner_texture: Image,
}