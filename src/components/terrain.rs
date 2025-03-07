use bevy::prelude::{Component, Entity};
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
}