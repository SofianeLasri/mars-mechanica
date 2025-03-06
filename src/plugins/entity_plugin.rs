use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_entity);
    }
}

fn spawn_entity(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Starting entity plugin");
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(128.)),
    ));
}
