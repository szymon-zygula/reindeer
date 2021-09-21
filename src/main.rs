#![warn(clippy::all)]

mod primitive;
mod mesh;
mod image;
mod error;
mod drawer;
mod renderer;
mod transform;
mod vector;
mod matrix;

use crate::error::Error;
use crate::primitive::Color;
use crate::renderer::Renderer;
use crate::mesh::Mesh;
use crate::image::Image;
use crate::vector::Vec3;

// Demo scene setup
fn main() -> Result<(), Error> {
    let mut renderer = Renderer::new();

    // Animation variables
    let mut v: f32 = 0.0;
    let mut u: f32 = 0.0;
    let mut w: f32 = 0.0;
    let dynamic_camera = true;

    let light_vector = Vec3 { x: 2.0, y: 5.0, z: 1.0 }.normalized();
    renderer.light(&light_vector);

    let camera_position = Vec3 { x: 0.5, y: 0.3, z: 1.0 };
    renderer.camera(
        &camera_position,
        &Vec3::ZERO,
        &Vec3 { x: 0.0, y: 1.0, z: 0.0 }
    );

    let head_mesh = Mesh::from_file("head.obj")?;
    let head_texture = Image::from_file("head_diffuse.tga")?;
    let head_normal_map = Image::from_file("head_nm_tangent.tga")?;

    loop {
        if dynamic_camera {
            v += 0.06;
            u -= 0.1;
            w += 0.03;

            renderer.camera(
                &Vec3 { x: v.cos() * u.sin(), y: 1.0, z: w.sin() * 2.5 },
                &Vec3 { x: v.cos(), y: u.sin(), z: w.cos() },
                &Vec3 { x: 0.0, y: 1.0, z: 0.0 }
            );
        }

        renderer.refresh(&Color::BLUE);
        renderer.model(&head_mesh, &head_texture, &head_normal_map, &Vec3 {x: 0.0, y: 0.0, z: 0.0});
        renderer.display()?;
    }
}
