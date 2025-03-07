use bevy::asset::{AssetServer, Assets};
use bevy::math::Vec3;
use bevy::prelude::{ColorMaterial, Commands, Mesh, Mesh2d, MeshMaterial2d, Rectangle, Res, ResMut, Transform};

const CELL_SIZE: i32 = 64;

pub fn generate_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
)
{
    let positions = [
        (0,0),
        (0,-1),
        (1,-1),
        (2, -1),
        (2,-2),
    ];

    for (x, y) in positions.iter() {
        let (coord_x, coord_y) = calc_cell_coordinates(x, y);
        println!("Placing object on cell ({}, {})", coord_x, coord_y);

        let image = asset_server.load("textures/terrain/plain.png");
        let material = materials.add(ColorMaterial {
            texture: Some(image),
            ..Default::default()
        });

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(material.clone()),
            //Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(64.)),
            Transform::from_xyz(coord_x as f32, coord_y as f32, 0.0).with_scale(Vec3::splat(CELL_SIZE as f32)),
        ));
    }
}

pub fn calc_cell_coordinates(x: &i32, y: &i32) -> (i32, i32) {
    let cell_x = x * CELL_SIZE;
    let cell_y = y * CELL_SIZE;
    (cell_x, cell_y)
}
