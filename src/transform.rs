use crate::vector::{
    Vec2,
    Vec3,
    cross
};

use crate::matrix::{
    Matrix2,
    Matrix4
};

// Euclidean -> barycentric
pub fn to_barycentric(a: &Vec2, b: &Vec2, c: &Vec2, p: &Vec2) -> Vec3 {
    // [ABx ACx]
    // [ABy ACy]
    let inv = Matrix2::new([
        [b.x - a.x, c.x - a.x],
        [b.y - a.y, c.y - a.y]
    ]).inverse();

    match inv {
        Some(inv) => {
            let uv = inv * (*p - *a);
            Vec3 { 
                x: 1.0 - uv.x - uv.y,
                y: uv.x,
                z: uv.y,
            }
        },
        None => Vec3 {
            x: -1.0,
            y: -1.0,
            z: -1.0
        }
    }
}

// Barycentric -> euclidean
pub fn to_euclidean(a: &Vec2, b: &Vec2, c: &Vec2, p: &Vec3) -> Vec2 {
    let mat = Matrix2::new([
        [b.x - a.x, c.x - a.x],
        [b.y - a.y, c.y - a.y]
    ]);

    mat * Vec2 { x: p.y, y: p.z } + *a
}

pub fn perspective(c: f32) -> Matrix4 {
    Matrix4::new([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, -1.0 / c, 1.0]
    ])
}

pub fn normal_perspective(c: f32) -> Matrix4 {
    Matrix4::new([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 1.0 / c],
        [0.0, 0.0, 0.0, 1.0]
    ])
}

pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Matrix4 {
    let k = (*eye - *center).normalized();
    let i = cross(up, &k).normalized();
    let j = cross(&k, &i).normalized();

    // i j k are orthonormal so its inverse is equal to its transpose
    Matrix4::new([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, -(*eye - *center).len()],
        [0.0, 0.0, 0.0, 1.0]
    ]) * Matrix4::new([
        [i.x, i.y, i.z, 0.0],
        [j.x, j.y, j.z, 0.0],
        [k.x, k.y, k.z, 0.0],
        [0.0, 0.0, 0.0, 1.0]
    ]) * Matrix4::new([
        [1.0, 0.0, 0.0, -center.x],
        [0.0, 1.0, 0.0, -center.y],
        [0.0, 0.0, 1.0, -center.z],
        [0.0, 0.0, 0.0, 1.0]
    ])
}
