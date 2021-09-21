use crate::drawer::Drawer;
use crate::error::Error;
use crate::mesh::Mesh;
use crate::image::Image;
use crate::transform;
use crate::primitive::{
    Color,
    Size
};

use crate::vector::{
    Vec2,
    Vec3
};

use crate::matrix::{
    Matrix3,
    Matrix4
};

pub struct Renderer {
    drawer: Drawer,
    zbuffer: Vec<f32>,

    view_matrix: Matrix4,
    projection_matrix: Matrix4,

    shadow_buffer: Vec<f32>,
    shadow_view_matrix: Matrix4,

    normal_projection_matrix: Matrix4,
    light_vector: Vec3
}

struct BoundingBox {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl Renderer {
    fn create_zbuffer(plane_size: Size) -> Vec<f32> {
        let mut v = Vec::with_capacity((plane_size.width * plane_size.height) as usize);
        for _ in 0..v.capacity() {
            v.push(std::f32::NEG_INFINITY);
        }

        v
    }

    pub fn new() -> Self {
        let drawer = Drawer::new();
        let light_vector = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

        Renderer {
            zbuffer: Self::create_zbuffer(drawer.plane_size()),

            projection_matrix: transform::perspective(3.0),
            view_matrix: Matrix4::IDENTITY,

            shadow_buffer: Self::create_zbuffer(drawer.plane_size()),
            shadow_view_matrix: transform::look_at(
                &light_vector, &Vec3::ZERO, &Vec3 { x: 0.0, y: 1.0, z: 0.0 }
            ),

            normal_projection_matrix: transform::normal_perspective(3.0),
            light_vector,

            drawer
        }
    }

    #[inline(always)]
    fn to_drawer_coordinates(&self, vec: Vec2) -> (i32, i32) {
        (
            (self.drawer.plane_size().width as f32 * (vec.x + 1.0) / 2.0) as i32,
            (self.drawer.plane_size().height as f32 * (-vec.y + 1.0) / 2.0) as i32
        )
    }

    #[inline(always)]
    fn to_renderer_coordinates(&self, x: i32, y: i32) -> Vec2 {
        Vec2 {
            x: x as f32 / self.drawer.plane_size().width as f32 * 2.0 - 1.0,
            y: -y as f32 / self.drawer.plane_size().height as f32 * 2.0 + 1.0
        }
    }

    pub fn refresh(&mut self, color: &Color) {
        let (rows, cols) =  unsafe {
            let mut ws: libc::winsize = std::mem::MaybeUninit::uninit().assume_init();
            libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws);
            (i32::from(ws.ws_row), i32::from(ws.ws_col))
        };

        if rows * 2 != self.drawer.plane_size().height || cols != self.drawer.plane_size().width {
            self.drawer = Drawer::new();
            self.zbuffer = Self::create_zbuffer(self.drawer.plane_size());
        }
        else {
            for p in self.zbuffer.iter_mut() {
                *p = std::f32::NEG_INFINITY;
            }
        }

        self.drawer.clear(color);
    }

    pub fn display(&mut self) -> Result<(), Error> {
        self.drawer.display()?;
        Ok(())
    }

    fn bounding_box(&self, p1: &Vec3, p2: &Vec3, p3: &Vec3) -> BoundingBox {
        let (bbox_min_x, bbox_min_y) = self.to_drawer_coordinates(Vec2 {
            x: Self::min_bounding_box(p1.x, p2.x, p3.x),
            y: Self::max_bounding_box(p1.y, p2.y, p3.y)
        });

        let (bbox_max_x, bbox_max_y) = self.to_drawer_coordinates(Vec2 {
            x: Self::max_bounding_box(p1.x, p2.x, p3.x),
            y: Self::min_bounding_box(p1.y, p2.y, p3.y)
        });

        BoundingBox {
            min_x: std::cmp::max(bbox_min_x, 0),
            max_x: std::cmp::min(bbox_max_x, self.drawer.plane_size().width - 1),
            min_y: std::cmp::max(bbox_min_y, 0),
            max_y: std::cmp::min(bbox_max_y, self.drawer.plane_size().height - 1)
        }
    }

    fn min_bounding_box(a: f32, b: f32, c: f32) -> f32 {
        let mut bb = if a < b { a } else { b };
        bb = if bb < c { bb } else { c };
        bb
    }

    fn max_bounding_box(a: f32, b: f32, c: f32) -> f32 {
        let mut bb = if a > b { a } else { b };
        bb = if bb > c { bb } else { c };
        bb
    }

    pub fn camera(&mut self, eye: &Vec3, center: &Vec3, up: &Vec3) {
        self.view_matrix = transform::look_at(eye, center, up);
    }

    pub fn light(&mut self, light_vector: &Vec3) {
        self.light_vector = *light_vector;
        self.shadow_view_matrix = transform::look_at(
            &light_vector,
            &Vec3::ZERO,
            &Vec3 { x: 0.0, y: 1.0, z: 0.0 }
        );
    }

    fn transform(&self, p: &Vec3) -> Vec3 {
        (self.projection_matrix * (self.view_matrix * p.homo_point())).point_proj()
    }

    fn transform_normal(&self, p: &Vec3) -> Vec3 {
        (
            self.normal_projection_matrix * (self.view_matrix * p.homo_vector())
        ).vector_proj()
    }

    fn transform_shadow(&self, p: &Vec3) -> Vec3 {
        (
           self.projection_matrix * (self.shadow_view_matrix * p.homo_point())
        ).point_proj()
    }

    fn ambient_occlusion(&self, x: i32, y: i32) -> f32{
        let mut ambient_light = 0.0;
        let zbuffer_index = (y * self.drawer.plane_size().width + x) as usize;

        for x_step in -1..=1 {
            for y_step in -1..=1 {
                if x_step == 0 && y_step == 0 {
                    continue;
                }

                let mut zbuffer_x = x;
                let mut zbuffer_y = y;
                let mut max_angle = 0.0;

                while self.ambient_occlusion_step(
                    x, y,
                    zbuffer_index, &mut zbuffer_x, &mut zbuffer_y,
                    x_step, y_step, &mut max_angle
                ) {}

                ambient_light += std::f32::consts::PI / 2.0 - max_angle;
            }
        }

        (ambient_light / 4.0 / std::f32::consts::PI).powi(40)
    }

    fn ambient_occlusion_step(
        &self,
        x: i32, y: i32,
        zbuffer_index: usize, zbuffer_x: &mut i32, zbuffer_y: &mut i32,
        x_step: i32, y_step: i32,
        max_angle: &mut f32
    ) -> bool {
        *zbuffer_x += x_step;
        *zbuffer_y += y_step;

        if *zbuffer_x >= self.drawer.plane_size().width ||
           *zbuffer_y >= self.drawer.plane_size().height ||
           *zbuffer_x < 0 || *zbuffer_y < 0 {
               return false
        }

        let zbuffer_ray_index = (*zbuffer_y * self.drawer.plane_size().width + *zbuffer_x) as usize;

        if self.zbuffer[zbuffer_ray_index] == std::f32::NEG_INFINITY {
            return true;
        }

        let height = self.zbuffer[zbuffer_ray_index] - self.zbuffer[zbuffer_index];
        let length = (((x - *zbuffer_x).pow(2) + (y - *zbuffer_y).pow(2)) as f32).sqrt();

        let angle = (height / length).atan();

        if angle > *max_angle {
            *max_angle = angle;
        }

        true
    }

    pub fn triangle(
        &mut self,
        // Vertices
        v1: &Vec3, v2: &Vec3, v3: &Vec3,
        // UV coordinates
        t1: &Vec2, t2: &Vec2, t3: &Vec2, texture: &Image,
        // Normal vectors
        n1: &Vec3, n2: &Vec3, n3: &Vec3, normal_map: &Image
    ) {
        // vertices used for calculating shadow buffer
        let s1 = self.transform_shadow(v1);
        let s2 = self.transform_shadow(v2);
        let s3 = self.transform_shadow(v3);

        self.fill_in_shadow_buffer(&s1, &s2, &s3);

        // vertices
        let p1 = self.transform(v1);
        let p2 = self.transform(v2);
        let p3 = self.transform(v3);

        // normal vectors
        let n1 = self.transform_normal(n1);
        let n2 = self.transform_normal(n2);
        let n3 = self.transform_normal(n3);

        self.fill_in_triangle(
            &p1, &p2, &p3,
            &t1, &t2, &t3, texture,
            &n1, &n2, &n3, &normal_map,
            &s1, &s2, &s3
        );
    }

    fn fill_in_shadow_buffer(&mut self, s1: &Vec3, s2: &Vec3, s3: &Vec3) {
        let shadow_bbox = self.bounding_box(&s1, &s2, &s3);

        for i in shadow_bbox.min_x..=shadow_bbox.max_x {
            for j in shadow_bbox.min_y..=shadow_bbox.max_y {
                let s = transform::to_barycentric(
                    &Vec2 { x: s1.x, y: s1.y },
                    &Vec2 { x: s2.x, y: s2.y },
                    &Vec2 { x: s3.x, y: s3.y },
                    &self.to_renderer_coordinates(i, j)
                );

                if s.x >= 0.0 && s.y >= 0.0 && s.z >= 0.0 {
                    let pixel_depth = s1.z * s.x + s2.z * s.y + s3.z * s.z;
                    let shadow_buffer_index = (j * self.drawer.plane_size().width + i) as usize;

                    if pixel_depth > self.shadow_buffer[shadow_buffer_index] {
                        self.shadow_buffer[shadow_buffer_index] = pixel_depth;
                    }
                }
            }
        }
    }

    fn fill_in_triangle(
        &mut self,
        // Vertices in barycentric coordinates
        p1: &Vec3, p2: &Vec3, p3: &Vec3,
        // UV coordinates
        t1: &Vec2, t2: &Vec2, t3: &Vec2, texture: &Image,
        // Normal vectors
        n1: &Vec3, n2: &Vec3, n3: &Vec3, normal_map: &Image,
        s1: &Vec3, s2: &Vec3, s3: &Vec3
    ) {
        let bbox = self.bounding_box(&p1, &p2, &p3);
        let light_vector = self.transform_normal(&self.light_vector);

        for i in bbox.min_x..=bbox.max_x {
            for j in bbox.min_y..=bbox.max_y {
                let p = transform::to_barycentric(
                    &Vec2 { x: p1.x, y: p1.y },
                    &Vec2 { x: p2.x, y: p2.y },
                    &Vec2 { x: p3.x, y: p3.y },
                    &self.to_renderer_coordinates(i, j)
                );

                if !(p.x >= 0.0 && p.y >= 0.0 && p.z >= 0.0) || 
                   !self.update_zbuffer_and_check_if_visible(&p, &p1, &p2, &p3, i, j) {
                    continue;
                }

                let texture_coordinates = Self::calc_texture_coords(&t1, &t2, &t3, &p, texture);

                let normal_vector = match Self::calc_normal_vector(
                    n1, n2, n3, &p, &p1, &p2, &p3, &t1, &t2, &t3, texture_coordinates, &normal_map
                    ) {
                    Some(vec) => vec,
                    None => continue,
                };

                let shadow_light = self.calc_shadow_light(&p, s1, s2, s3);
                let light_intensity = self.calc_light_intensity(
                    &light_vector, &normal_vector, shadow_light, i, j
                );

                self.drawer.set_vertex(
                    i, j,
                    &(*texture.at(texture_coordinates.0, texture_coordinates.1) * light_intensity)
                );
            }
        }
    }

    fn update_zbuffer_and_check_if_visible(
        &mut self,
        p: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3,
        i: i32, j: i32
    ) -> bool {
        let pixel_depth = p1.z * p.x + p2.z * p.y + p3.z * p.z;
        let zbuffer_index = (j * self.drawer.plane_size().width + i) as usize;

        if pixel_depth <= self.zbuffer[zbuffer_index] {
            return false;
        }

        self.zbuffer[zbuffer_index] = pixel_depth;
        true
    }

    fn calc_texture_coords(
        t1: &Vec2, t2: &Vec2, t3: &Vec2,
        p: &Vec3, texture: &Image
    ) -> (usize, usize) {
        let uv_coordinates = transform::to_euclidean(t1, t2, t3, &p);
        (
            (uv_coordinates.x * (texture.size().width - 1) as f32) as usize,
            (uv_coordinates.y * (texture.size().height - 1) as f32) as usize,
        )
    }

    fn calc_normal_vector(
        n1: &Vec3, n2: &Vec3, n3: &Vec3,
        p: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3,
        t1: &Vec2, t2: &Vec2, t3: &Vec2,
        texture_coordinates: (usize, usize), normal_map: &Image
    ) -> Option<Vec3> {
        // Tangent basis
        let n_vector = (*n1 * p.x + *n2 * p.y + *n3 * p.z).normalized();

        let darboux_matrix = match Self::calc_darboux_matrix(&p1, &p2, &p3, &n_vector) {
            Some(matrix) => matrix,
            None => return None
        };

        let i_vector = darboux_matrix * Vec3 { x: t2.x - t1.x, y: t3.x - t1.x, z: 0.0 };
        let j_vector = darboux_matrix * Vec3 { x: t2.y - t1.y, y: t3.y - t1.y, z: 0.0 };

        let normal_color = normal_map.at(texture_coordinates.0, texture_coordinates.1);

        Some((
            (f32::from(normal_color.r) / 255.0).powi(3) * i_vector.normalized() +
            (f32::from(normal_color.g) / 255.0).powi(3) * j_vector.normalized() +
            (f32::from(normal_color.b) / 255.0).powi(3) * n_vector
        ).normalized())
    }

    fn calc_darboux_matrix(p1: &Vec3, p2: &Vec3, p3: &Vec3, n_vector: &Vec3) -> Option<Matrix3> {
        Matrix3::new([
            [p2.x - p1.x, p2.y - p1.y, p2.z - p1.z],
            [p3.x - p1.x, p3.y - p1.y, p3.z - p1.z],
            [n_vector.x, n_vector.y, n_vector.z]
        ]).inverse()
    }

    fn calc_shadow_light(&self, p: &Vec3, s1: &Vec3, s2: &Vec3, s3: &Vec3) -> f32 {
        let shadow_vector = p.x * *s1 + p.y * *s2 + p.z * *s3;

        let shadow_coordinates = self.to_drawer_coordinates(
            Vec2 { x: shadow_vector.x, y: shadow_vector.y }
        );

        let shadow_buffer_index = (
            shadow_coordinates.1 * self.drawer.plane_size().width +
            shadow_coordinates.0) as usize;

        if self.shadow_buffer[shadow_buffer_index] > shadow_vector.z + 0.2 {
            -1.0
        }
        else {
            0.0
        }
    }

    fn calc_light_intensity(
        &self,
        light_vector: &Vec3,
        normal_vector: &Vec3,
        shadow_light: f32,
        i: i32, j: i32
    ) -> f32 {
        let reflection_vector =
            2.0 * *normal_vector * (*normal_vector * *light_vector) - *light_vector;

        let specular_light = (reflection_vector * Vec3 { x: 0.0, y: 0.0, z: 1.0 }).powi(35);
        let diffuse_light = *normal_vector * *light_vector;
        let ambient_light = self.ambient_occlusion(i, j);

        specular_light * 0.7 +
        diffuse_light * 1.0 +
        ambient_light * 0.4 +
        shadow_light * 0.2
    }

    pub fn model(&mut self, mesh: &Mesh, texture: &Image, normal_map: &Image, pos: &Vec3) {
        for face in mesh.faces() {
            let vertices = [
                *mesh.vertex(face.vertices[0]) + *pos,
                *mesh.vertex(face.vertices[1]) + *pos,
                *mesh.vertex(face.vertices[2]) + *pos
            ];

            let uv_coordinates = [
                mesh.texture_coord(face.texture_coords[0]),
                mesh.texture_coord(face.texture_coords[1]),
                mesh.texture_coord(face.texture_coords[2])
            ];

            let normal_vectors = [
                mesh.normal(face.normals[0]),
                mesh.normal(face.normals[1]),
                mesh.normal(face.normals[2]),
            ];

            self.triangle(
                &vertices[0], &vertices[1], &vertices[2],
                &uv_coordinates[0], &uv_coordinates[1], &uv_coordinates[2], &texture,
                &normal_vectors[0], &normal_vectors[1], &normal_vectors[2], &normal_map
            );
        }
    }
} 
