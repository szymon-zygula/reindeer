#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    fn color_f32_to_u8(mut color: f32) -> u8 {
        color = if color < 0.0 { 0.0 } else { color };
        color = if color > 255.0 { 255.0 } else { color };
        color as u8
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        let r = f32::from(self.r) * rhs;
        let g = f32::from(self.g) * rhs;
        let b = f32::from(self.b) * rhs;

        let r = Self::color_f32_to_u8(r);
        let g = Self::color_f32_to_u8(g);
        let b = Self::color_f32_to_u8(b);

        Color { r, g, b }
    }
}

#[allow(dead_code)]
impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const GRAY: Color = Color { r: 127, g: 127, b: 127 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
}

#[derive(Clone)]
pub struct Size {
    pub width: i32,
    pub height: i32
}
