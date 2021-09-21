use crate::error::Error;
use crate::vector::{
    Vec2,
    Vec3
};

pub struct Face {
    pub vertices: [usize; 3],
    pub texture_coords: [usize; 3],
    pub normals: [usize; 3]
}

pub struct Mesh {
    vertices: Vec<Vec3>,
    texture_coords: Vec<Vec2>,
    faces: Vec<Face>,
    normals: Vec<Vec3>
}

impl Mesh {
    fn parse_obj(
        buf_reader: std::io::BufReader<std::fs::File>,
        vertices: &mut Vec<Vec3>,
        faces: &mut Vec<Face>,
        texture_coords: &mut Vec<Vec2>,
        normals: &mut Vec<Vec3>
    ) -> Result<(), Error> {
        use std::io::BufRead;
        for line in buf_reader.lines() {
            let line = line?;

            let line: Vec<&str> = line.split(' ').collect();

            match line[0] {
                "f" => faces.push(Self::parse_f(&line)?),
                "v" => vertices.push(Self::parse_v(&line)?),
                "vt" => texture_coords.push(Self::parse_vt(&line)?),
                "vn" => normals.push(Self::parse_vn(&line)?),
                _ => {}
            }
        }

        Ok(())
    }

    fn parse_f(line: &[&str]) -> Result<Face, Error> {
        let mut vrts = [0, 0, 0];
        let mut txts = [0, 0, 0];
        let mut norms = [0, 0, 0];
        let mut vec: Vec<&str>;

        for i in 0..3 {
            vec = line[i + 1].split('/').collect();
            vrts[i] = vec[0].parse::<usize>()? - 1;
            txts[i] = vec[1].parse::<usize>()? - 1;
            norms[i] = vec[2].parse::<usize>()? - 1;
        }

        Ok(Face {
            vertices: vrts,
            texture_coords: txts,
            normals: norms
        })
    }

    fn parse_v(line: &[&str]) -> Result<Vec3, Error> {
        Ok(Vec3 {
            x: line[1].parse::<f32>()?,
            y: line[2].parse::<f32>()?,
            z: line[3].parse::<f32>()?
        })
    }

    fn parse_vt(line: &[&str]) -> Result<Vec2, Error> {
        Ok(Vec2 {
            x: line[2].parse::<f32>()?,
            y: line[3].parse::<f32>()?
        })
    }

    fn parse_vn(line: &[&str]) -> Result<Vec3, Error> {
        Ok(Vec3 {
            x: line[2].parse::<f32>()?,
            y: line[3].parse::<f32>()?,
            z: line[4].parse::<f32>()?
        })
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let mut vertices = Vec::<Vec3>::new();
        let mut faces = Vec::<Face>::new();
        let mut texture_coords = Vec::<Vec2>::new();
        let mut normals = Vec::<Vec3>::new();

        let file = std::fs::File::open(path)?;
        let buf_reader = std::io::BufReader::new(file);

        Self::parse_obj(buf_reader, &mut vertices, &mut faces, &mut texture_coords, &mut normals)?;

        Ok(Mesh {
            vertices, faces, texture_coords, normals
        })
    }

    #[inline(always)]
    pub fn vertex(&self, num: usize) -> &Vec3 {
        &self.vertices[num]
    }

    #[inline(always)]
    pub fn texture_coord(&self, num: usize) -> &Vec2 {
        &self.texture_coords[num]
    }

    #[inline(always)]
    pub fn normal(&self, num: usize) -> &Vec3 {
        &self.normals[num]
    }

    #[inline(always)]
    pub fn faces(&self) -> std::slice::Iter<Face> {
        self.faces.iter()
    }
}
