use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_multiple_squares);
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
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(64.)),
    ));
}

fn spawn_multiple_squares(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Définir les positions où tu veux placer tes carrés
    let positions = [
        Vec2::new(100.0, 100.0),
        Vec2::new(-100.0, 150.0),
        Vec2::new(200.0, -100.0),
        Vec2::new(-150.0, -150.0),
        Vec2::new(0.0, 200.0),
    ];

    let image = asset_server.load("textures/terrain/inter-corner.png");
    let material = materials.add(ColorMaterial {
        texture: Some(image),
        ..Default::default()
    });

    // Créer un carré à chaque position
    for position in positions.iter() {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(material.clone()),
            //Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(64.)),
            Transform::from_xyz(position.x, position.y, 0.0).with_scale(Vec3::splat(64.)),
        ));
    }
}
