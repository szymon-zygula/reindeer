use crate::error::Error;
use crate::primitive::{
    Color,
    Size
};

#[derive(Clone)]
pub struct WinSize {
    pub cols: i32,
    pub rows: i32
}

pub struct Drawer {
    stdout: std::io::Stdout,
    win_size: WinSize,
    win_buf: Vec<u8>,
    plane_size: Size,
    img_buf: Vec<Color>
}


impl Drawer {
    const DRAWING_BLOCK: &'static [u8] = b"\xE2\x96\x84";
    const DRAWING_SEQUENCE: &'static [u8] = b"\x1b[48;2;000;000;000m\x1b[38;2;000;000;000m";

    pub fn new() -> Self {
        let (cols, rows) = Self::get_terminal_size();

        Drawer {
            stdout: std::io::stdout(),
            win_size: WinSize { cols: cols as i32, rows: rows as i32 },
            win_buf: Self::create_window_buffer(cols, rows),
            plane_size: Size { width: cols as i32, height: rows as i32 * 2 },
            img_buf: Self::create_image_buffer(cols, rows)
        }
    }

    fn get_terminal_size() -> (usize, usize) {
        unsafe {
            let mut ws: libc::winsize = std::mem::MaybeUninit::uninit().assume_init();
            libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws);
            (usize::from(ws.ws_col), usize::from(ws.ws_row))
        }
    }

    fn create_window_buffer(cols: usize, rows: usize) -> Vec<u8> {
        let mut win_buf = Vec::with_capacity(
            rows * cols * (Self::DRAWING_BLOCK.len() + Self::DRAWING_SEQUENCE.len()) + rows - 1
        );

        for _ in 0..rows {
            for _ in 0..cols {
                win_buf.extend_from_slice(Self::DRAWING_SEQUENCE);
                win_buf.extend_from_slice(Self::DRAWING_BLOCK);
            }
        }

        win_buf
    }

    fn create_image_buffer(cols: usize, rows: usize) -> Vec<Color> {
        let mut img_buf = Vec::with_capacity(rows * cols * 2);
        for _ in 0..img_buf.capacity() {
            img_buf.push(Color::BLACK);
        }

        img_buf
    }

    fn set_win_color_value(&mut self, pos: usize, val: u8) {
        let z = b'0';
        let v100 = val / 100;
        let v10 = (val - v100 * 100) / 10;
        let v1 = val - v100 * 100 - v10 * 10;
        self.win_buf[pos] = v100 + z;
        self.win_buf[pos + 1] = v10 + z;
        self.win_buf[pos + 2] = v1 + z;
    }

    fn set_win_vertex(&mut self, x: i32, y: i32, color: &Color) {
        let segment = (2 * x + y % 2 + 2 * self.win_size.cols * (y / 2)) as usize;
        // 7 - length of "\x1b[38;2;"
        let pos = segment * (Self::DRAWING_SEQUENCE.len() / 2) + Self::DRAWING_BLOCK.len() * (segment / 2) + 7;
        // set color every 4 characters ("000;")
        self.set_win_color_value(pos, color.r);
        self.set_win_color_value(pos + 4, color.g);
        self.set_win_color_value(pos + 8, color.b);
    }

    #[inline(always)]
    fn vertex_ref_mut(&mut self, x: i32, y: i32) -> &mut Color {
        &mut self.img_buf[(x + y * self.plane_size.width) as usize]
    }

    #[inline(always)]
    fn vertex_ref(&self, x: i32, y: i32) -> &Color {
        &self.img_buf[(x + y * self.plane_size.width) as usize]
    }

    #[inline(always)]
    pub fn set_vertex(&mut self, x: i32, y: i32, color: &Color) {
        *self.vertex_ref_mut(x, y) = color.clone();
    }

    pub fn clear(&mut self, color: &Color) {
        for vertex in self.img_buf.iter_mut() {
            *vertex = color.clone();
        }
    }

    fn move_cursor_to_origin(&mut self) -> Result<(), Error>{
        use std::io::Write;
        self.stdout.write_all(b"\x1B[0;0H")?;

        Ok(())
    }

    pub fn display(&mut self) -> Result<(), Error> {
        for y in 0..self.plane_size.height {
            for x in 0..self.plane_size.width {
                // Avoid cloning the color by evading the borrow-checker
                let color = self.vertex_ref(x, y) as *const Color;
                unsafe {
                    self.set_win_vertex(x, y, &*color);
                }
            }
        }

        self.print_window_buffer()
    }

    fn print_window_buffer(&mut self) -> Result<(), Error> {
        self.move_cursor_to_origin()?;

        use std::io::Write;
        self.stdout.write_all(&self.win_buf)?;
        self.stdout.flush()?;

        Ok(())
    }

    #[inline(always)]
    pub fn plane_size(&self) -> Size {
        self.plane_size.clone()
    }
}
