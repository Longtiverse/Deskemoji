use softbuffer::{Context, Surface};
use winit::window::Window;
use std::num::NonZeroU32;
use std::rc::Rc;

use crate::MenuState;

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
        radius: f32,
        menu: &MenuState,
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
        buffer.fill(0x00000000);

        let center_x = size.width as f32 / 2.0;
        let center_y = size.height as f32 / 2.0 + offset_y;
        let scaled_radius = radius * scale;

        // 绘制黄色圆形
        for y in 0..size.height as i32 {
            for x in 0..size.width as i32 {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= scaled_radius * scaled_radius {
                    let idx = (y as u32 * size.width + x as u32) as usize;
                    let dist = dist_sq.sqrt();
                    let gradient = 1.0 - (dist / scaled_radius) * 0.12;
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

        // 绘制菜单
        if menu.visible {
            Self::draw_menu(
                &mut buffer,
                size.width,
                size.height,
                menu,
            );
        }

        buffer.present().unwrap();
    }

    fn draw_menu(buffer: &mut [u32], width: u32, height: u32, menu: &MenuState) {
        let menu_x = menu.cursor_pos.x as i32;
        let menu_y = menu.cursor_pos.y as i32;
        let menu_width = 120;
        let item_height = 28;
        let menu_height = (menu.items.len() as i32) * item_height;
        
        // 绘制背景
        Self::draw_rect(buffer, width, height, menu_x, menu_y, menu_width, menu_height, 0xF0F8F8F8);
        
        // 绘制边框
        Self::draw_rect_outline(buffer, width, height, menu_x, menu_y, menu_width, menu_height, 0xFF808080);
        
        // 绘制菜单项
        for (i, (item, text)) in menu.items.iter().enumerate() {
            let item_y = menu_y + (i as i32) * item_height;
            
            // 悬停效果
            if menu.hovered == Some(i) {
                Self::draw_rect(buffer, width, height, menu_x + 2, item_y + 1, menu_width - 4, item_height - 2, 0xFFD8D8FF);
            }
            
            // 分隔符
            if i == 1 || i == 9 || i == 11 {
                Self::draw_line(buffer, width, height, menu_x + 8, item_y, menu_x + menu_width - 8, item_y, 0xFFC0C0C0);
            }
            
            // 文字
            let text_color = if menu.hovered == Some(i) { 0xFF000080 } else { 0xFF303030 };
            Self::draw_text_simple(buffer, width, height, menu_x + 10, item_y + 8, text, text_color);
            
            // 勾选标记
            match item {
                crate::MenuItem::AutoMode if true => {
                    Self::draw_check(buffer, width, height, menu_x + menu_width - 20, item_y + 8);
                }
                _ => {}
            }
        }
    }

    fn draw_text_simple(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32, text: &str, color: u32) {
        let char_width = 7;
        let text_len = text.len().min(16) as i32;
        
        for i in 0..text_len {
            let px = x + i * char_width;
            for dy in 0..12 {
                for dx in 0..5 {
                    let draw_x = px + dx;
                    let draw_y = y + dy;
                    if draw_x >= 0 && draw_x < width as i32 && draw_y >= 0 && draw_y < height as i32 {
                        let idx = (draw_y as u32 * width + draw_x as u32) as usize;
                        if idx < buffer.len() {
                            buffer[idx] = color;
                        }
                    }
                }
            }
        }
    }

    fn draw_check(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32) {
        let color = 0xFF008000;
        for i in 0..6 {
            let px = x + i;
            let py = y + 3 + (i / 2);
            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let idx = (py as u32 * width + px as u32) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
            }
        }
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
        let left_eye_x = cx - (18.0 * s) as i32 + eye_dx as i32;
        let left_eye_y = cy - (12.0 * s) as i32 + eye_dy as i32;
        let right_eye_x = cx + (18.0 * s) as i32 + eye_dx as i32;
        let right_eye_y = cy - (12.0 * s) as i32 + eye_dy as i32;
        let eye_r = (6.0 * s) as i32;

        match emoji {
            '\u{1F642}' | '\u{1F60A}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (8.0 * s) as i32, (25.0 * s) as i32, 0xFF000000);
            }
            '\u{1F622}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (18.0 * s) as i32, (18.0 * s) as i32, 0xFF000000);
                Self::draw_circle(buffer, width, height, left_eye_x - 4, left_eye_y + 10, 3, 0xFF4FC3F7);
            }
            '\u{1F621}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_line(buffer, width, height, cx - 25, cy - 5, cx - 12, cy - 10, 0xFF000000);
                Self::draw_line(buffer, width, height, cx + 12, cy - 10, cx + 25, cy - 5, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx, cy + (15.0 * s) as i32, (10.0 * s) as i32, 0xFF000000);
            }
            '\u{1F634}' => {
                Self::draw_line(buffer, width, height, left_eye_x - 6, left_eye_y, left_eye_x + 6, left_eye_y, 0xFF000000);
                Self::draw_line(buffer, width, height, right_eye_x - 6, right_eye_y, right_eye_x + 6, right_eye_y, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (12.0 * s) as i32, (15.0 * s) as i32, 0xFF000000);
            }
            '\u{1F914}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_line(buffer, width, height, cx - 8, cy + 10, cx + 8, cy + 10, 0xFF000000);
            }
            '\u{1F975}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx, cy + (15.0 * s) as i32, (10.0 * s) as i32, 0xFF000000);
                Self::draw_circle(buffer, width, height, cx + 30, cy - 8, 4, 0xFF4FC3F7);
            }
            '\u{1F480}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, (7.0 * s) as i32, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, (7.0 * s) as i32, 0xFF000000);
                Self::draw_triangle(buffer, width, height, cx, cy + (10.0 * s) as i32, (6.0 * s) as i32, 0xFF000000);
            }
            '\u{1F319}' => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (12.0 * s) as i32, (15.0 * s) as i32, 0xFF000000);
            }
            _ => {
                Self::draw_circle(buffer, width, height, left_eye_x, left_eye_y, eye_r, 0xFF000000);
                Self::draw_circle(buffer, width, height, right_eye_x, right_eye_y, eye_r, 0xFF000000);
                Self::draw_arc(buffer, width, height, cx, cy + (8.0 * s) as i32, (25.0 * s) as i32, 0xFF000000);
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

    fn draw_rect(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for py in y..(y + h) {
            for px in x..(x + w) {
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let idx = (py as u32 * width + px as u32) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }

    fn draw_rect_outline(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32, color: u32) {
        Self::draw_line(buffer, width, height, x, y, x + w, y, color);
        Self::draw_line(buffer, width, height, x, y + h, x + w, y + h, color);
        Self::draw_line(buffer, width, height, x, y, x, y + h, color);
        Self::draw_line(buffer, width, height, x + w, y, x + w, y + h, color);
    }
}
