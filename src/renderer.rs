use softbuffer::{Context, Surface};
use winit::window::Window;
use std::num::NonZeroU32;
use std::rc::Rc;

pub struct Renderer {
    context: Context<Rc<Window>>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

impl Renderer {
    pub fn new(window: Rc<Window>) -> Self {
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();
        Self { context, surface }
    }

    pub fn render(
        &mut self,
        window: &Window,
        emoji: char,
        scale: f32,
        offset_y: f32,
        eye_offset_x: f32,
        eye_offset_y: f32,
    ) {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface
            .resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )
            .unwrap();

        let mut buffer = self.surface.buffer_mut().unwrap();

        // 清空为透明
        buffer.fill(0x00000000);

        let center_x = size.width as f32 / 2.0;
        let center_y = size.height as f32 / 2.0 + offset_y;
        let radius = 45.0 * scale;

        // 绘制黄色圆形
        for y in 0..size.height as i32 {
            for x in 0..size.width as i32 {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= radius * radius {
                    let idx = (y as u32 * size.width + x as u32) as usize;
                    // 浅黄色带渐变效果
                    let gradient = 1.0 - (dist_sq.sqrt() / radius) * 0.1;
                    let r = (255.0 * gradient) as u32;
                    let g = (217.0 * gradient) as u32;
                    let b = (61.0 * gradient) as u32;
                    buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }

        // 绘制表情
        Self::draw_face(
            &mut buffer,
            size.width,
            size.height,
            emoji,
            center_x as i32,
            center_y as i32,
            scale,
            eye_offset_x,
            eye_offset_y,
        );

        buffer.present().unwrap();
    }

    fn draw_face(
        buffer: &mut [u32],
        width: u32,
        height: u32,
        emoji: char,
        cx: i32,
        cy: i32,
        scale: f32,
        eye_dx: f32,
        eye_dy: f32,
    ) {
        let s = scale;
        let left_eye_x = cx - (15.0 * s) as i32 + eye_dx as i32;
        let left_eye_y = cy - (10.0 * s) as i32 + eye_dy as i32;
        let right_eye_x = cx + (15.0 * s) as i32 + eye_dx as i32;
        let right_eye_y = cy - (10.0 * s) as i32 + eye_dy as i32;
        let eye_r = (5.0 * s) as i32;

        match emoji {
            '🙂' | '😊' | '🤗' | '😌' => {
                // 微笑表情：眼睛 + 弯弯的嘴
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (5.0 * s) as i32, (20.0 * s) as i32, 0xFF000000);
            }
            '😴' | '😪' => {
                // 闭眼：画横线
                Self::draw_line(buffer, width, height,
                    left_eye_x - 5, left_eye_y,
                    left_eye_x + 5, left_eye_y, 0xFF000000);
                Self::draw_line(buffer, width, height,
                    right_eye_x - 5, right_eye_y,
                    right_eye_x + 5, right_eye_y, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx, cy + (10.0 * s) as i32, (8.0 * s) as i32, 0xFF000000);
            }
            '🥵' => {
                // 热：睁眼 + 张嘴 + 汗滴
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx, cy + (12.0 * s) as i32, (10.0 * s) as i32, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx + (28.0 * s) as i32, cy - (8.0 * s) as i32, (4.0 * s) as i32, 0xFF4FC3F7);
            }
            '💀' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, (7.0 * s) as i32, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, (7.0 * s) as i32, 0xFF000000);
                Self::draw_triangle(buffer, width, height, cx, cy + (8.0 * s) as i32, (6.0 * s) as i32, 0xFF000000);
            }
            '🌙' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (8.0 * s) as i32, (15.0 * s) as i32, 0xFF000000);
            }
            _ => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (5.0 * s) as i32, (20.0 * s) as i32, 0xFF000000);
            }
        }
    }

    fn draw_circle(buffer: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, r: i32, color: u32) {
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                if y < 0 || x < 0 || y >= height as i32 || x >= width as i32 {
                    continue;
                }
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r * r {
                    let idx = (y as u32 * width + x as u32) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }

    fn draw_arc(buffer: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, r: i32, color: u32) {
        for angle in 20..160 {
            let rad = angle as f64 * std::f64::consts::PI / 180.0;
            let x = cx + (r as f64 * rad.cos()) as i32;
            let y = cy + (r as f64 * rad.sin() / 2.0) as i32;
            if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
                // 加粗弧线
                if y + 1 < height as i32 {
                    let idx2 = ((y + 1) as u32 * width + x as u32) as usize;
                    if idx2 < buffer.len() {
                        buffer[idx2] = color;
                    }
                }
            }
        }
    }

    fn draw_line(buffer: &mut [u32], width: u32, height: u32, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
            }
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    fn draw_triangle(buffer: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, size: i32, color: u32) {
        for y in (cy - size)..=(cy + size) {
            for x in (cx - size)..=(cx + size) {
                if y < 0 || x < 0 || y >= height as i32 || x >= width as i32 {
                    continue;
                }
                let dx = x - cx;
                let dy = y - cy;
                if dy >= 0 && dx.abs() <= (size - dy) {
                    let idx = (y as u32 * width + x as u32) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }
}
