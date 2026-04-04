use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::window::Window;

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

        let mut buf = self.surface.buffer_mut().unwrap();
        buf.fill(0x00000000);

        let cx = size.width as f32 / 2.0;
        let cy = center_y;

        // 绘制圆形背景
        for y in 0..size.height as i32 {
            for x in 0..size.width as i32 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    let idx = (y as u32 * size.width + x as u32) as usize;
                    let dist = dist_sq.sqrt();
                    let g = 1.0 - dist / radius * 0.12;
                    buf[idx] = 0xFF000000
                        | ((255.0 * g) as u32) << 16
                        | ((217.0 * g) as u32) << 8
                        | ((61.0 * g) as u32);
                }
            }
        }

        // 绘制眼睛 - 跟随鼠标
        let eye_cx = cx + eye_x;
        let eye_cy = cy - 12.0 + eye_y;
        draw_circle(
            &mut buf,
            size.width,
            size.height,
            (eye_cx - 15.0) as i32,
            eye_cy as i32,
            6,
            0xFF000000,
        );
        draw_circle(
            &mut buf,
            size.width,
            size.height,
            (eye_cx + 15.0) as i32,
            eye_cy as i32,
            6,
            0xFF000000,
        );

        // 绘制嘴巴
        draw_arc(
            &mut buf,
            size.width,
            size.height,
            cx as i32,
            (cy + 10.0) as i32,
            20,
            0xFF000000,
        );

        buf.present().unwrap();
    }
}

fn draw_circle(buf: &mut [u32], width: u32, height: u32, cx: i32, cy: i32, r: i32, color: u32) {
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if y < 0 || x < 0 || y >= height as i32 || x >= width as i32 {
                continue;
            }
            if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buf.len() {
                    buf[idx] = color;
                }
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
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}
