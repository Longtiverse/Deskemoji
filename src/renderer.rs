use softbuffer::{Context, Surface};
use winit::window::Window;
use std::num::NonZeroU32;
use std::rc::Rc;

use crate::MenuItem;

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
        center_y: f32,
        radius: f32,
        eye_x: f32,
        eye_y: f32,
        show_menu: bool,
        menu_items: &[MenuItem],
        menu_hover: Option<usize>,
        menu_start_y: i32,
        menu_item_height: i32,
        menu_width: i32,
        auto_mode: bool,
        startup: bool,
    ) {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 { return; }

        self.surface.resize(
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        ).unwrap();

        let mut buf = self.surface.buffer_mut().unwrap();
        buf.fill(0x00000000);

        let cx = size.width as f32 / 2.0;
        let cy = center_y;

        // 圆形背景
        for y in 0..size.height as i32 {
            for x in 0..size.width as i32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx * dx + dy * dy <= radius * radius {
                    let idx = (y as u32 * size.width + x as u32) as usize;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let g = 1.0 - dist / radius * 0.1;
                    buf[idx] = 0xFF000000 
                        | ((255.0 * g) as u32) << 16 
                        | ((217.0 * g) as u32) << 8 
                        | ((61.0 * g) as u32);
                }
            }
        }

        // 眼睛 - 大幅度跟随
        let eye_cx = cx + eye_x;
        let eye_cy = cy - 12.0 + eye_y;
        draw_circle(&mut buf, size.width, size.height, 
            (eye_cx - 15.0) as i32, eye_cy as i32, 6, 0xFF000000);
        draw_circle(&mut buf, size.width, size.height, 
            (eye_cx + 15.0) as i32, eye_cy as i32, 6, 0xFF000000);

        // 嘴巴
        draw_arc(&mut buf, size.width, size.height,
            cx as i32, (cy + 10.0) as i32, 20, 0xFF000000);

        // 菜单
        if show_menu {
            let menu_x = (size.width as i32 - menu_width) / 2;
            let menu_height = menu_items.len() as i32 * menu_item_height;
            
            // 背景
            draw_rect(&mut buf, size.width, size.height, 
                menu_x, menu_start_y, menu_width, menu_height, 0xF5F5F5F5);
            
            // 边框
            draw_rect_outline(&mut buf, size.width, size.height, 
                menu_x, menu_start_y, menu_width, menu_height, 0xFF888888);
            
            // 菜单项
            for (i, item) in menu_items.iter().enumerate() {
                let y = menu_start_y + i as i32 * menu_item_height;
                
                if menu_hover == Some(i) {
                    draw_rect(&mut buf, size.width, size.height, 
                        menu_x + 1, y, menu_width - 2, menu_item_height, 0xFFE0E8FF);
                }
                
                if item.text.contains("───") || item.text.contains("────") {
                    draw_line(&mut buf, size.width, size.height, 
                        menu_x + 5, y + menu_item_height/2, 
                        menu_x + menu_width - 5, y + menu_item_height/2, 0xFFAAAAAA);
                    continue;
                }
                
                let color = if menu_hover == Some(i) { 0xFF000080 } else { 0xFF333333 };
                draw_text(&mut buf, size.width, size.height, menu_x + 8, y + 4, &item.text, color);
                
                let checked = match &item.action {
                    crate::MenuAction::ToggleAuto => auto_mode,
                    crate::MenuAction::ToggleStartup => startup,
                    _ => false,
                };
                if checked {
                    draw_check(&mut buf, size.width, size.height, menu_x + menu_width - 15, y + 4);
                }
            }
        }

        buf.present().unwrap();
    }
}

fn draw_text(buf: &mut [u32], width: u32, height: u32, x: i32, y: i32, text: &str, color: u32) {
    for (i, _) in text.chars().enumerate().take(14) {
        let px = x + i as i32 * 7;
        for dy in 0..11 {
            for dx in 0..5 {
                let rx = px + dx;
                let ry = y + dy;
                if rx >= 0 && rx < width as i32 && ry >= 0 && ry < height as i32 {
                    let idx = (ry as u32 * width + rx as u32) as usize;
                    if idx < buf.len() { buf[idx] = color; }
                }
            }
        }
    }
}

fn draw_check(buf: &mut [u32], width: u32, height: u32, x: i32, y: i32) {
    for i in 0..5 {
        let px = x + i;
        let py = y + 3 + i/2;
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as u32 * width + px as u32) as usize;
            if idx < buf.len() { buf[idx] = 0xFF008800; }
        }
    }
}

fn draw_circle(buf: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, r: i32, color: u32) {
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if y < 0 || x < 0 || y >= height as i32 || x >= width as i32 { continue; }
            if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buf.len() { buf[idx] = color; }
            }
        }
    }
}

fn draw_arc(buf: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, r: i32, color: u32) {
    for angle in 20..160 {
        let rad = angle as f64 * std::f64::consts::PI / 180.0;
        let x = cx + (r as f64 * rad.cos()) as i32;
        let y = cy + (r as f64 * rad.sin() * 0.5) as i32;
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            let idx = (y as u32 * width + x as u32) as usize;
            if idx < buf.len() { buf[idx] = color; }
        }
    }
}

fn draw_line(buf: &mut [u32], width: u32, height: u32, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x1, y1);
    loop {
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            let idx = (y as u32 * width + x as u32) as usize;
            if idx < buf.len() { buf[idx] = color; }
        }
        if x == x2 && y == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_rect(buf: &mut [u32], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32, color: u32) {
    for py in y..y+h {
        for px in x..x+w {
            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let idx = (py as u32 * width + px as u32) as usize;
                if idx < buf.len() { buf[idx] = color; }
            }
        }
    }
}

fn draw_rect_outline(buf: &mut [u32], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32, color: u32) {
    draw_line(buf, width, height, x, y, x+w, y, color);
    draw_line(buf, width, height, x, y+h, x+w, y+h, color);
    draw_line(buf, width, height, x, y, x, y+h, color);
    draw_line(buf, width, height, x+w, y, x+w, y+h, color);
}
