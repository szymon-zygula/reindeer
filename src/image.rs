use crate::error::Error;
use crate::primitive::{
    Color,
    Size
};

pub struct Image {
    buffer: Vec<Color>,
    size: Size
}

impl Image {
    const TGA_HEADER_SIZE: usize = 18;
    fn parse_tga_file(
        file_buffer: &[u8],
        size: &mut Size
    ) -> Result<Vec<Color>, Error> {
        if file_buffer.len() <= Self::TGA_HEADER_SIZE {
            return Err(Error::Parse);
        }

        let id_length = file_buffer[0];

        let colormap_type = file_buffer[1];
        if colormap_type != 0 {
            return Err(Error::UnsupportedFormat);
        }

        let image_type = file_buffer[2];
        size.width = i32::from(u16::from_le((
            u16::from(file_buffer[13]) << 0b1000) | u16::from(file_buffer[12])
        ));
        size.height = i32::from(u16::from_le((
            u16::from(file_buffer[15]) << 0b1000) | u16::from(file_buffer[14])
        ));

        // 0 - do nothing, 1 - ignore last byte of every pixel, otherwise fail
        let alpha_depth = (file_buffer[17] as usize & 0b1111) / 8;

        // 3 BRG bytes + alpha bytes
        let step = 3 + alpha_depth;

        Ok(match image_type {
            2 => Self::load_uncompressed_truecolor(id_length, size, step, file_buffer),
            10 => Self::load_runlength_encoded_truecolor(id_length, size, step, file_buffer),
            _ => return Err(Error::UnsupportedFormat)
        })
    }

    fn load_uncompressed_truecolor(
        id_length: u8,
        size: &Size,
        step: usize,
        file_buffer: &[u8]
    ) -> Vec<Color> {
        let mut color_buffer =  Vec::<Color>::new();
        color_buffer.reserve((size.width * size.height) as usize);
        let start = Self::TGA_HEADER_SIZE + id_length as usize;
        let end = start + (size.width * size.height) as usize * step;
        for i in (start..end).step_by(step) {
            // TGA uses BRGa color encoding
            color_buffer.push(Color {
                r: file_buffer[i + 2],
                g: file_buffer[i + 1],
                b: file_buffer[i]
            });
        }

        color_buffer
    }

    fn load_runlength_encoded_truecolor(
        id_length: u8,
        size: &Size,
        step: usize,
        file_buffer: &[u8]
    ) -> Vec<Color> {
        let mut color_buffer =  Vec::<Color>::new();
        color_buffer.reserve((size.width * size.height) as usize);
        let mut byte_index = Self::TGA_HEADER_SIZE + id_length as usize;
        let mut pixels_read = 0usize;

        while pixels_read < (size.width * size.height) as usize {
            Self::read_encoded_pixels(step, &mut byte_index, &mut pixels_read, file_buffer, &mut color_buffer)
        }

        color_buffer
    }

    fn read_encoded_pixels(
        step: usize,
        byte_index: &mut usize,
        pixels_read: &mut usize,
        file_buffer: &[u8],
        color_buffer: &mut Vec<Color>
    ) {
        let encoding_type = (file_buffer[*byte_index] & 0b1000_0000) >> 7;
        let encoding_length = (file_buffer[*byte_index] & 0b0111_1111) + 1;
        *byte_index += 1;

        // following pixels are not compressed
        if encoding_type == 0 {
            Self::read_uncompressed_pixels(
                step, encoding_length, pixels_read, byte_index, file_buffer, color_buffer
            );
        }
        // following pixels are compressed
        else {
            Self::read_compressed_pixels(
                step, encoding_length, pixels_read, byte_index, file_buffer, color_buffer
            );
        }
    }

    fn read_uncompressed_pixels(
        step: usize,
        encoding_length: u8,
        pixels_read: &mut usize,
        byte_index: &mut usize,
        file_buffer: &[u8],
        color_buffer: &mut Vec<Color>
    ) {
        for j in (0..(step * encoding_length as usize)).step_by(step) {
            color_buffer.push(Color {
                r: file_buffer[*byte_index + j + 2],
                g: file_buffer[*byte_index + j + 1],
                b: file_buffer[*byte_index + j]
            });
        }

        *pixels_read += encoding_length as usize;
        *byte_index += encoding_length as usize * step;
    }

    fn read_compressed_pixels(
        step: usize,
        encoding_length: u8,
        pixels_read: &mut usize,
        byte_index: &mut usize,
        file_buffer: &[u8],
        color_buffer: &mut Vec<Color>
    ) {
        for _ in 0..encoding_length {
            color_buffer.push(Color {
                r: file_buffer[*byte_index + 2],
                g: file_buffer[*byte_index + 1],
                b: file_buffer[*byte_index]
            });
        }

        *pixels_read += encoding_length as usize;
        *byte_index += step;
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut file_buffer = Vec::<u8>::new();

        use std::io::Read;
        file.read_to_end(&mut file_buffer)?;

        let mut image_size = Size { width: 0, height: 0 };
        let color_buffer = Self::parse_tga_file(&file_buffer, &mut image_size)?;

        Ok(Image {
            buffer: color_buffer,
            size: image_size
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn at(&self, x: usize, y: usize) -> &Color {
        &self.buffer[x + y * self.size.width as usize]
    }
}
