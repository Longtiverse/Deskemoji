use crate::emoji_assets::{EmojiId, EmojiImage};
use image::{imageops::FilterType, ImageBuffer, Rgba};
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{copy_nonoverlapping, null_mut};
use std::rc::Rc;
use windows::Win32::Foundation::{COLORREF, HANDLE, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, AC_SRC_ALPHA,
    AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
    HGDIOBJ, RGBQUAD,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, UpdateLayeredWindow, GWL_EXSTYLE, ULW_ALPHA,
    WS_EX_LAYERED,
};
use winit::window::Window;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

const BASE_SPRITE_RATIO: f32 = 0.78;
const BASE_SPRITE_MAX_SIZE: u32 = 180;
const BUBBLE_HEIGHT: u32 = 22;
const FONT_SCALE: u32 = 2;
const FONT_WIDTH: u32 = 5;
const FONT_HEIGHT: u32 = 7;
const FONT_GAP: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmojiTransform {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub alpha: f32,
}

impl EmojiTransform {
    pub const fn new(scale: f32, offset_x: f32, offset_y: f32, alpha: f32) -> Self {
        Self {
            scale,
            offset_x,
            offset_y,
            alpha,
        }
    }

    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    pub fn combine(self, other: Self) -> Self {
        Self::new(
            self.scale * other.scale,
            self.offset_x + other.offset_x,
            self.offset_y + other.offset_y,
            self.alpha * other.alpha,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderLayer<'a> {
    pub image: &'a EmojiImage,
    pub transform: EmojiTransform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GazeDirection {
    Center,
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl GazeDirection {
    pub const fn index(self) -> usize {
        match self {
            GazeDirection::Center => 0,
            GazeDirection::Left => 1,
            GazeDirection::Right => 2,
            GazeDirection::Up => 3,
            GazeDirection::Down => 4,
            GazeDirection::UpLeft => 5,
            GazeDirection::UpRight => 6,
            GazeDirection::DownLeft => 7,
            GazeDirection::DownRight => 8,
        }
    }

    pub fn from_pointer_delta(dx: f32, dy: f32) -> Self {
        let horizontal = if dx <= -18.0 {
            -1
        } else if dx >= 18.0 {
            1
        } else {
            0
        };
        let vertical = if dy <= -22.0 {
            -1
        } else if dy >= 24.0 {
            1
        } else {
            0
        };

        match (horizontal, vertical) {
            (-1, -1) => GazeDirection::UpLeft,
            (0, -1) => GazeDirection::Up,
            (1, -1) => GazeDirection::UpRight,
            (-1, 0) => GazeDirection::Left,
            (0, 0) => GazeDirection::Center,
            (1, 0) => GazeDirection::Right,
            (-1, 1) => GazeDirection::DownLeft,
            (0, 1) => GazeDirection::Down,
            (1, 1) => GazeDirection::DownRight,
            _ => GazeDirection::Center,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteRect {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub center_x: i32,
    pub center_y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BubbleOverlay {
    pub label: String,
}

impl BubbleOverlay {
    pub fn for_state(state: EmojiId, cpu_usage: u8, memory_usage: u8, elapsed_secs: u64) -> Option<Self> {
        match state {
            EmojiId::Hot | EmojiId::Mindblown => {
                let show_cpu = (elapsed_secs / 5) % 2 == 0;
                if show_cpu {
                    Some(Self {
                        label: bubble_label_for_usage("CPU", cpu_usage),
                    })
                } else {
                    Some(Self {
                        label: bubble_label_for_usage("MEM", memory_usage),
                    })
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BubbleRect {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub tail_tip_x: i32,
    pub tail_tip_y: i32,
}

pub struct Renderer {
    presenter: LayeredWindowPresenter,
    scaled_cache: HashMap<ScaledSpriteKey, EmojiImage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ScaledSpriteKey {
    source_id: usize,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(window: Rc<Window>) -> Self {
        let hwnd = window_hwnd(&window).expect("Deskemoji requires a Win32 window handle");
        let presenter = LayeredWindowPresenter::new(hwnd);
        Self {
            presenter,
            scaled_cache: HashMap::new(),
        }
    }

    pub fn render_layers(
        &mut self,
        window: &Window,
        layers: &[RenderLayer<'_>],
        bubble: Option<&BubbleOverlay>,
    ) {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let (center_x, center_y) = compute_sprite_center(size.width, size.height);
        let scaled_cache = &mut self.scaled_cache;
        let mut prepared_layers = Vec::with_capacity(layers.len());
        for layer in layers {
            if layer.transform.alpha <= 0.0 {
                continue;
            }

            let rect = compute_sprite_rect(
                size.width,
                size.height,
                center_x,
                center_y,
                layer.transform.scale,
                layer.transform.offset_x,
                layer.transform.offset_y,
            );
            let key = ensure_scaled_sprite(scaled_cache, layer.image, rect.width, rect.height);
            prepared_layers.push((rect, key, layer.transform.alpha));
        }

        let mut buf = vec![0; (size.width * size.height) as usize];
        buf.fill(0x00000000);

        let mut top_rect = None;

        for (rect, key, alpha) in prepared_layers {
            let image = scaled_cache.get(&key).unwrap();
            composite_sprite(&mut buf, size.width, size.height, image, rect, alpha);
            top_rect = Some(rect);
        }

        if let (Some(bubble), Some(sprite_rect)) = (bubble, top_rect) {
            let (anchor_x, anchor_y) = compute_bubble_anchor(sprite_rect);
            draw_bubble(
                &mut buf,
                size.width,
                size.height,
                bubble,
                anchor_x,
                anchor_y,
                sprite_rect.width,
            );
        }

        self.presenter.present(size.width, size.height, &buf);
    }
}

struct LayeredWindowPresenter {
    hwnd: HWND,
    memory_dc: HDC,
    bitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
    bits: *mut c_void,
    width: u32,
    height: u32,
    upload: Vec<u8>,
}

impl LayeredWindowPresenter {
    fn new(hwnd: HWND) -> Self {
        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as isize);
        }

        let memory_dc = unsafe { CreateCompatibleDC(HDC(0)) };
        assert!(memory_dc.0 != 0, "failed to create layered window memory DC");

        Self {
            hwnd,
            memory_dc,
            bitmap: HBITMAP(0),
            old_bitmap: HGDIOBJ(0),
            bits: null_mut(),
            width: 0,
            height: 0,
            upload: Vec::new(),
        }
    }

    fn present(&mut self, width: u32, height: u32, pixels: &[u32]) {
        self.ensure_bitmap(width, height);
        self.upload = convert_argb_to_premultiplied_bgra(pixels);
        unsafe {
            copy_nonoverlapping(self.upload.as_ptr(), self.bits.cast::<u8>(), self.upload.len());

            let destination = POINT { x: 0, y: 0 };
            let size = SIZE {
                cx: width as i32,
                cy: height as i32,
            };
            let source = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            UpdateLayeredWindow(
                self.hwnd,
                HDC(0),
                Some(&destination),
                Some(&size),
                self.memory_dc,
                Some(&source),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )
            .expect("failed to update layered Deskemoji window");
        }
    }

    fn ensure_bitmap(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height && !self.bits.is_null() {
            return;
        }

        unsafe {
            if self.bitmap.0 != 0 {
                if self.old_bitmap.0 != 0 {
                    SelectObject(self.memory_dc, self.old_bitmap);
                }
                DeleteObject(self.bitmap);
                self.bitmap = HBITMAP(0);
                self.old_bitmap = HGDIOBJ(0);
                self.bits = null_mut();
            }

            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: width * height * 4,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default()],
            };

            let mut bits = null_mut();
            let bitmap = CreateDIBSection(
                self.memory_dc,
                &info,
                DIB_RGB_COLORS,
                &mut bits,
                HANDLE(0),
                0,
            )
            .expect("failed to create layered window DIB section");
            let old_bitmap = SelectObject(self.memory_dc, bitmap);

            self.bitmap = bitmap;
            self.old_bitmap = old_bitmap;
            self.bits = bits;
            self.width = width;
            self.height = height;
        }
    }
}

impl Drop for LayeredWindowPresenter {
    fn drop(&mut self) {
        unsafe {
            if self.memory_dc.0 != 0 && self.old_bitmap.0 != 0 {
                SelectObject(self.memory_dc, self.old_bitmap);
            }
            if self.bitmap.0 != 0 {
                DeleteObject(self.bitmap);
            }
            if self.memory_dc.0 != 0 {
                DeleteDC(self.memory_dc);
            }
        }
    }
}

fn window_hwnd(window: &Window) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get())),
        _ => None,
    }
}

pub fn compute_idle_transform(time_secs: f32) -> EmojiTransform {
    let offset_y = (time_secs * 1.8).sin() * 2.5;
    let scale = 1.0 + (time_secs * 1.8 + 0.7).sin() * 0.02;
    EmojiTransform::new(scale, 0.0, offset_y, 1.0)
}

pub fn compute_closed_eye_response(
    state: EmojiId,
    dx: f32,
    dy: f32,
    time_secs: f32,
) -> EmojiTransform {
    let (max_x, max_y, nod_amount) = match state {
        EmojiId::Sleepy => (3.0, 1.5, 1.0),
        EmojiId::Goodnight => (1.7, 0.85, 0.55),
        _ => return EmojiTransform::identity(),
    };

    let horizontal = normalize_cursor_delta(dx, 24.0, 160.0);
    let vertical = normalize_cursor_delta(dy, 24.0, 180.0);
    let attention = horizontal.abs().max(vertical.abs());
    let nod = (time_secs * 2.2).sin() * nod_amount * attention;

    EmojiTransform::new(1.0, horizontal * max_x, vertical * max_y + nod, 1.0)
}

pub fn compute_state_accent_transform(state: EmojiId, time_secs: f32) -> EmojiTransform {
    let (scale, offset_x, offset_y) = match state {
        EmojiId::Happy => (
            1.0 + (time_secs * 1.4).sin() * 0.004,
            (time_secs * 0.7).sin() * 0.15,
            (time_secs * 1.4).sin() * 0.28,
        ),
        EmojiId::Sad => (
            1.0 - ((time_secs * 0.85).sin() + 1.0) * 0.003,
            (time_secs * 0.55).sin() * 0.08,
            0.22 + (time_secs * 0.85 + 1.2).sin() * 0.18,
        ),
        EmojiId::Angry => (
            1.0 - (time_secs * 1.8).sin().abs() * 0.008,
            (time_secs * 1.1).sin() * 0.18,
            (time_secs * 1.8).sin().abs() * 0.36,
        ),
        EmojiId::Sleepy => (
            1.0 - ((time_secs * 0.72).sin() + 1.0) * 0.004,
            (time_secs * 0.36).sin() * 0.08,
            (time_secs * 0.72).sin() * 0.44,
        ),
        EmojiId::Thinking => (
            1.0 + (time_secs * 0.92).sin() * 0.003,
            (time_secs * 0.92).sin() * 0.3,
            (time_secs * 0.92 + 0.8).sin() * 0.18,
        ),
        EmojiId::Hot => (
            1.0 - (time_secs * 3.2).sin().abs() * 0.012,
            (time_secs * 1.6).sin() * 0.22,
            (time_secs * 3.2).sin() * 0.95,
        ),
        EmojiId::Mindblown => (
            1.0 - 0.006 - (time_secs * 2.0).sin().abs() * 0.01,
            (time_secs * 2.8).sin() * 0.14,
            -0.38 - (time_secs * 2.0).sin().abs() * 0.32,
        ),
        EmojiId::Goodnight => (
            1.0 - ((time_secs * 0.54).sin() + 1.0) * 0.0025,
            (time_secs * 0.28).sin() * 0.04,
            (time_secs * 0.54).sin() * 0.2,
        ),
    };

    EmojiTransform::new(scale, offset_x, offset_y, 1.0)
}

pub fn compute_sprite_rect(
    surface_width: u32,
    surface_height: u32,
    center_x: f32,
    center_y: f32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) -> SpriteRect {
    let base_size =
        surface_width.min(surface_height).min(BASE_SPRITE_MAX_SIZE) as f32 * BASE_SPRITE_RATIO;
    let sprite_size = (base_size * scale.max(0.01)).round().max(1.0) as u32;
    let final_center_x = center_x + offset_x;
    let final_center_y = center_y + offset_y;

    SpriteRect {
        left: (final_center_x - sprite_size as f32 / 2.0).round() as i32,
        top: (final_center_y - sprite_size as f32 / 2.0).round() as i32,
        width: sprite_size,
        height: sprite_size,
        center_x: final_center_x.round() as i32,
        center_y: final_center_y.round() as i32,
    }
}

pub fn compute_sprite_center(surface_width: u32, surface_height: u32) -> (f32, f32) {
    if surface_width > 120 || surface_height > 120 {
        (surface_width as f32 * 0.5, surface_height as f32 * 0.66)
    } else {
        (surface_width as f32 / 2.0, surface_height as f32 / 2.0)
    }
}

pub fn compute_bubble_anchor(sprite_rect: SpriteRect) -> (i32, i32) {
    (sprite_rect.center_x, sprite_rect.top + 4)
}

pub fn bubble_visual_scale(sprite_width: u32) -> f32 {
    (sprite_width as f32 / 94.0).clamp(1.0, 1.8)
}

fn normalize_cursor_delta(delta: f32, deadzone: f32, full_scale: f32) -> f32 {
    let distance = delta.abs();
    if distance <= deadzone {
        return 0.0;
    }

    let usable = (distance - deadzone) / (full_scale - deadzone);
    usable.clamp(0.0, 1.0) * delta.signum()
}

pub fn bubble_label_for_usage(kind: &str, percent: u8) -> String {
    format!("{kind} {}%", percent.min(100))
}

pub fn compute_bubble_rect(
    surface_width: u32,
    _surface_height: u32,
    anchor_x: i32,
    anchor_y: i32,
    bubble_width: u32,
    bubble_height: u32,
) -> BubbleRect {
    let margin = 6;
    let left = (anchor_x - bubble_width as i32 / 2)
        .clamp(margin, surface_width as i32 - bubble_width as i32 - margin);
    let top = (anchor_y - bubble_height as i32 - 8).max(margin);

    BubbleRect {
        left,
        top,
        width: bubble_width,
        height: bubble_height,
        tail_tip_x: anchor_x.clamp(left + 12, left + bubble_width as i32 - 10),
        tail_tip_y: anchor_y,
    }
}

pub fn sample_bilinear_rgba(image: &EmojiImage, x: f32, y: f32) -> [u8; 4] {
    let max_x = (image.width.saturating_sub(1)) as f32;
    let max_y = (image.height.saturating_sub(1)) as f32;
    let x = x.clamp(0.0, max_x);
    let y = y.clamp(0.0, max_y);

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width.saturating_sub(1));
    let y1 = (y0 + 1).min(image.height.saturating_sub(1));
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;

    let top = lerp_premultiplied(
        premultiply_rgba(pixel_rgba(image, x0, y0)),
        premultiply_rgba(pixel_rgba(image, x1, y0)),
        tx,
    );
    let bottom = lerp_premultiplied(
        premultiply_rgba(pixel_rgba(image, x0, y1)),
        premultiply_rgba(pixel_rgba(image, x1, y1)),
        tx,
    );
    unpremultiply_rgba(lerp_premultiplied(top, bottom, ty))
}

pub fn resize_image_high_quality(
    image: &EmojiImage,
    target_width: u32,
    target_height: u32,
) -> EmojiImage {
    if target_width == 0 || target_height == 0 {
        return EmojiImage {
            width: target_width.max(1),
            height: target_height.max(1),
            pixels: vec![0, 0, 0, 0],
        };
    }

    if image.width == target_width && image.height == target_height {
        return image.clone();
    }

    let premultiplied_pixels = premultiply_image_pixels(image);
    let premultiplied =
        ImageBuffer::<Rgba<u8>, _>::from_raw(image.width, image.height, premultiplied_pixels)
            .expect("premultiplied source image should have valid dimensions");
    let resized = image::imageops::resize(
        &premultiplied,
        target_width,
        target_height,
        FilterType::CatmullRom,
    );
    let softened = soften_premultiplied_edges(resized.into_raw(), target_width, target_height);
    let pixels = unpremultiply_image_pixels(softened);

    EmojiImage {
        width: target_width,
        height: target_height,
        pixels,
    }
}

pub fn sample_resampled_rgba(
    image: &EmojiImage,
    x: f32,
    y: f32,
    footprint_x: f32,
    footprint_y: f32,
) -> [u8; 4] {
    let sample_count_x = footprint_x.ceil().clamp(1.0, 6.0) as u32;
    let sample_count_y = footprint_y.ceil().clamp(1.0, 6.0) as u32;

    if sample_count_x == 1 && sample_count_y == 1 {
        return sample_bilinear_rgba(image, x, y);
    }

    let left = x - footprint_x * 0.5;
    let top = y - footprint_y * 0.5;
    let mut accum = [0.0; 4];
    let total_samples = (sample_count_x * sample_count_y) as f32;

    for sample_y in 0..sample_count_y {
        let py = top + ((sample_y as f32 + 0.5) / sample_count_y as f32) * footprint_y;
        for sample_x in 0..sample_count_x {
            let px = left + ((sample_x as f32 + 0.5) / sample_count_x as f32) * footprint_x;
            let premultiplied = premultiply_rgba(sample_bilinear_rgba(image, px, py));
            accum[0] += premultiplied[0];
            accum[1] += premultiplied[1];
            accum[2] += premultiplied[2];
            accum[3] += premultiplied[3];
        }
    }

    unpremultiply_rgba([
        accum[0] / total_samples,
        accum[1] / total_samples,
        accum[2] / total_samples,
        accum[3] / total_samples,
    ])
}

fn draw_bubble(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    bubble: &BubbleOverlay,
    anchor_x: i32,
    anchor_y: i32,
    sprite_width: u32,
) {
    let bubble_scale = bubble_visual_scale(sprite_width);
    let font_scale = (FONT_SCALE as f32 * bubble_scale).round().max(2.0) as u32;
    let padding_x = (8.0 * bubble_scale).round().max(8.0) as i32;
    let padding_y = (5.0 * bubble_scale).round().max(5.0) as i32;
    let bubble_height = (BUBBLE_HEIGHT as f32 * bubble_scale).round() as u32;
    let text_width = bubble_text_width(&bubble.label, font_scale);
    let bubble_width = text_width + (padding_x as u32 * 2);
    let rect = compute_bubble_rect(
        surface_width,
        surface_height,
        anchor_x,
        anchor_y,
        bubble_width,
        bubble_height,
    );

    fill_rounded_rect(
        buf,
        surface_width,
        surface_height,
        rect.left,
        rect.top,
        rect.width,
        rect.height,
        8,
        0xF9F5F1,
        232,
    );
    stroke_rounded_rect(
        buf,
        surface_width,
        surface_height,
        rect.left,
        rect.top,
        rect.width,
        rect.height,
        8,
        0xD5C2B4,
        255,
    );
    fill_tail(buf, surface_width, surface_height, rect, 0xF9F5F1, 232);
    stroke_tail(buf, surface_width, surface_height, rect, 0xD5C2B4, 255);

    let text_x = rect.left + padding_x;
    let text_y = rect.top + padding_y;
    draw_bitmap_text(
        buf,
        surface_width,
        surface_height,
        text_x,
        text_y,
        &bubble.label,
        0xB84747,
        255,
        font_scale,
    );
}

fn ensure_scaled_sprite(
    cache: &mut HashMap<ScaledSpriteKey, EmojiImage>,
    image: &EmojiImage,
    width: u32,
    height: u32,
) -> ScaledSpriteKey {
    let key = ScaledSpriteKey {
        source_id: image.pixels.as_ptr() as usize,
        width: width.max(1),
        height: height.max(1),
    };

    cache
        .entry(key)
        .or_insert_with(|| resize_image_high_quality(image, key.width, key.height));
    key
}

fn composite_sprite(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    image: &EmojiImage,
    rect: SpriteRect,
    opacity: f32,
) {
    if rect.width == 0 || rect.height == 0 || image.width == 0 || image.height == 0 {
        return;
    }

    let opacity = opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }

    for dest_y in 0..rect.height {
        let screen_y = rect.top + dest_y as i32;
        if !(0..surface_height as i32).contains(&screen_y) {
            continue;
        }

        for dest_x in 0..rect.width {
            let screen_x = rect.left + dest_x as i32;
            if !(0..surface_width as i32).contains(&screen_x) {
                continue;
            }
            let sample = pixel_rgba(image, dest_x, dest_y);
            let src_r = sample[0];
            let src_g = sample[1];
            let src_b = sample[2];
            let src_a = ((sample[3] as f32) * opacity).round() as u8;
            if src_a == 0 {
                continue;
            }

            let dst_idx = (screen_y as u32 * surface_width + screen_x as u32) as usize;
            buf[dst_idx] = blend_argb(buf[dst_idx], src_r, src_g, src_b, src_a);
        }
    }
}

fn fill_rounded_rect(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    radius: i32,
    color: u32,
    alpha: u8,
) {
    let radius_sq = radius * radius;
    for y in 0..height as i32 {
        let screen_y = top + y;
        if !(0..surface_height as i32).contains(&screen_y) {
            continue;
        }

        for x in 0..width as i32 {
            let screen_x = left + x;
            if !(0..surface_width as i32).contains(&screen_x) {
                continue;
            }

            let within = if x < radius && y < radius {
                let dx = radius - x - 1;
                let dy = radius - y - 1;
                dx * dx + dy * dy <= radius_sq
            } else if x >= width as i32 - radius && y < radius {
                let dx = x - (width as i32 - radius);
                let dy = radius - y - 1;
                dx * dx + dy * dy <= radius_sq
            } else if x < radius && y >= height as i32 - radius {
                let dx = radius - x - 1;
                let dy = y - (height as i32 - radius);
                dx * dx + dy * dy <= radius_sq
            } else if x >= width as i32 - radius && y >= height as i32 - radius {
                let dx = x - (width as i32 - radius);
                let dy = y - (height as i32 - radius);
                dx * dx + dy * dy <= radius_sq
            } else {
                true
            };

            if within {
                let idx = (screen_y as u32 * surface_width + screen_x as u32) as usize;
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;
                buf[idx] = blend_argb(buf[idx], r, g, b, alpha);
            }
        }
    }
}

fn stroke_rounded_rect(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    radius: i32,
    color: u32,
    alpha: u8,
) {
    fill_rounded_rect(
        buf,
        surface_width,
        surface_height,
        left,
        top,
        width,
        1,
        radius,
        color,
        alpha,
    );
    fill_rounded_rect(
        buf,
        surface_width,
        surface_height,
        left,
        top + height as i32 - 1,
        width,
        1,
        radius,
        color,
        alpha,
    );
    fill_rounded_rect(
        buf,
        surface_width,
        surface_height,
        left,
        top,
        1,
        height,
        radius,
        color,
        alpha,
    );
    fill_rounded_rect(
        buf,
        surface_width,
        surface_height,
        left + width as i32 - 1,
        top,
        1,
        height,
        radius,
        color,
        alpha,
    );
}

fn fill_tail(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    rect: BubbleRect,
    color: u32,
    alpha: u8,
) {
    let base_y = rect.top + rect.height as i32 - 2;
    let base_left = rect.tail_tip_x - 7;
    let base_right = rect.tail_tip_x + 1;
    fill_triangle(
        buf,
        surface_width,
        surface_height,
        (base_left, base_y),
        (base_right, base_y),
        (rect.tail_tip_x, rect.tail_tip_y),
        color,
        alpha,
    );
}

fn stroke_tail(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    rect: BubbleRect,
    color: u32,
    alpha: u8,
) {
    draw_line(
        buf,
        surface_width,
        surface_height,
        rect.tail_tip_x - 7,
        rect.top + rect.height as i32 - 2,
        rect.tail_tip_x,
        rect.tail_tip_y,
        color,
        alpha,
    );
    draw_line(
        buf,
        surface_width,
        surface_height,
        rect.tail_tip_x + 1,
        rect.top + rect.height as i32 - 2,
        rect.tail_tip_x,
        rect.tail_tip_y,
        color,
        alpha,
    );
}

fn fill_triangle(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    a: (i32, i32),
    b: (i32, i32),
    c: (i32, i32),
    color: u32,
    alpha: u8,
) {
    let min_x = a.0.min(b.0).min(c.0);
    let max_x = a.0.max(b.0).max(c.0);
    let min_y = a.1.min(b.1).min(c.1);
    let max_y = a.1.max(b.1).max(c.1);

    for y in min_y..=max_y {
        if !(0..surface_height as i32).contains(&y) {
            continue;
        }
        for x in min_x..=max_x {
            if !(0..surface_width as i32).contains(&x) {
                continue;
            }

            if point_in_triangle((x, y), a, b, c) {
                let idx = (y as u32 * surface_width + x as u32) as usize;
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;
                buf[idx] = blend_argb(buf[idx], r, g, b, alpha);
            }
        }
    }
}

fn point_in_triangle(p: (i32, i32), a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let area = |p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)| {
        (p1.0 * (p2.1 - p3.1) + p2.0 * (p3.1 - p1.1) + p3.0 * (p1.1 - p2.1)) as f32
    };
    let a0 = area(a, b, c);
    let a1 = area(p, b, c);
    let a2 = area(a, p, c);
    let a3 = area(a, b, p);

    let has_neg = (a1 < 0.0) || (a2 < 0.0) || (a3 < 0.0);
    let has_pos = (a1 > 0.0) || (a2 > 0.0) || (a3 > 0.0);
    !(has_neg && has_pos) && a0 != 0.0
}

fn draw_line(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
    alpha: u8,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if (0..surface_width as i32).contains(&x0) && (0..surface_height as i32).contains(&y0) {
            let idx = (y0 as u32 * surface_width + x0 as u32) as usize;
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            buf[idx] = blend_argb(buf[idx], r, g, b, alpha);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn bubble_text_width(text: &str, font_scale: u32) -> u32 {
    if text.is_empty() {
        return 0;
    }

    text.chars().count() as u32 * (FONT_WIDTH * font_scale + FONT_GAP * font_scale)
        - FONT_GAP * font_scale
}

fn draw_bitmap_text(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    left: i32,
    top: i32,
    text: &str,
    color: u32,
    alpha: u8,
    font_scale: u32,
) {
    let mut cursor_x = left;
    let step_x = (FONT_WIDTH + FONT_GAP) as i32 * font_scale as i32;
    for ch in text.chars() {
        draw_glyph(
            buf,
            surface_width,
            surface_height,
            cursor_x,
            top,
            ch,
            color,
            alpha,
            font_scale,
        );
        cursor_x += step_x;
    }
}

fn draw_glyph(
    buf: &mut [u32],
    surface_width: u32,
    surface_height: u32,
    left: i32,
    top: i32,
    ch: char,
    color: u32,
    alpha: u8,
    font_scale: u32,
) {
    let rows = glyph_rows(ch);
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    for (row_idx, row) in rows.iter().enumerate() {
        for col in 0..FONT_WIDTH {
            if (row >> (FONT_WIDTH - 1 - col)) & 1 == 0 {
                continue;
            }

            for sy in 0..font_scale {
                for sx in 0..font_scale {
                    let x = left + (col * font_scale + sx) as i32;
                    let y = top + (row_idx as u32 * font_scale + sy) as i32;
                    if !(0..surface_width as i32).contains(&x)
                        || !(0..surface_height as i32).contains(&y)
                    {
                        continue;
                    }

                    let idx = (y as u32 * surface_width + x as u32) as usize;
                    buf[idx] = blend_argb(buf[idx], r, g, b, alpha);
                }
            }
        }
    }
}

fn glyph_rows(ch: char) -> [u8; FONT_HEIGHT as usize] {
    match ch {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01000, 0b10110, 0b00110, 0b00000,
        ],
        ' ' => [0; FONT_HEIGHT as usize],
        _ => [0; FONT_HEIGHT as usize],
    }
}

fn pixel_rgba(image: &EmojiImage, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * image.width + x) * 4) as usize;
    [
        image.pixels[idx],
        image.pixels[idx + 1],
        image.pixels[idx + 2],
        image.pixels[idx + 3],
    ]
}

fn premultiply_rgba(rgba: [u8; 4]) -> [f32; 4] {
    let alpha = rgba[3] as f32 / 255.0;
    [
        rgba[0] as f32 * alpha,
        rgba[1] as f32 * alpha,
        rgba[2] as f32 * alpha,
        rgba[3] as f32,
    ]
}

fn premultiply_image_pixels(image: &EmojiImage) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(image.pixels.len());
    for rgba in image.pixels.chunks_exact(4) {
        let premultiplied = premultiply_rgba([rgba[0], rgba[1], rgba[2], rgba[3]]);
        pixels.push(premultiplied[0].round().clamp(0.0, 255.0) as u8);
        pixels.push(premultiplied[1].round().clamp(0.0, 255.0) as u8);
        pixels.push(premultiplied[2].round().clamp(0.0, 255.0) as u8);
        pixels.push(rgba[3]);
    }
    pixels
}

fn lerp_premultiplied(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn unpremultiply_rgba(rgba: [f32; 4]) -> [u8; 4] {
    let alpha = rgba[3].clamp(0.0, 255.0);
    if alpha <= 8.0 {
        return [0, 0, 0, 0];
    }

    let alpha_scale = 255.0 / alpha;
    [
        (rgba[0] * alpha_scale).round().clamp(0.0, 255.0) as u8,
        (rgba[1] * alpha_scale).round().clamp(0.0, 255.0) as u8,
        (rgba[2] * alpha_scale).round().clamp(0.0, 255.0) as u8,
        alpha.round() as u8,
    ]
}

fn unpremultiply_image_pixels(pixels: Vec<u8>) -> Vec<u8> {
    let mut output = Vec::with_capacity(pixels.len());
    for rgba in pixels.chunks_exact(4) {
        let unpremultiplied = unpremultiply_rgba([
            rgba[0] as f32,
            rgba[1] as f32,
            rgba[2] as f32,
            rgba[3] as f32,
        ]);
        output.extend_from_slice(&unpremultiplied);
    }
    output
}

fn soften_premultiplied_edges(pixels: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    if width < 3 || height < 3 {
        return pixels;
    }

    let original = pixels;
    let mut output = original.clone();
    let weights = [[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]];

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = ((y * width + x) * 4) as usize;
            let alpha = original[idx + 3];
            if alpha == 0 || alpha == 255 {
                continue;
            }

            let mut accum = [0.0; 4];
            let mut total_weight = 0.0;
            for ky in 0..3 {
                for kx in 0..3 {
                    let sample_x = x + kx - 1;
                    let sample_y = y + ky - 1;
                    let sample_idx = ((sample_y * width + sample_x) * 4) as usize;
                    let weight = weights[ky as usize][kx as usize];
                    accum[0] += original[sample_idx] as f32 * weight;
                    accum[1] += original[sample_idx + 1] as f32 * weight;
                    accum[2] += original[sample_idx + 2] as f32 * weight;
                    accum[3] += original[sample_idx + 3] as f32 * weight;
                    total_weight += weight;
                }
            }

            output[idx] = (accum[0] / total_weight).round().clamp(0.0, 255.0) as u8;
            output[idx + 1] = (accum[1] / total_weight).round().clamp(0.0, 255.0) as u8;
            output[idx + 2] = (accum[2] / total_weight).round().clamp(0.0, 255.0) as u8;
            output[idx + 3] = (accum[3] / total_weight).round().clamp(0.0, 255.0) as u8;
        }
    }

    output
}

fn blend_argb(dst: u32, src_r: u8, src_g: u8, src_b: u8, src_a: u8) -> u32 {
    let src_alpha = src_a as f32 / 255.0;
    let dst_alpha = ((dst >> 24) & 0xFF) as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

    if out_alpha <= 0.01 {
        return 0;
    }

    let dst_r = ((dst >> 16) & 0xFF) as f32;
    let dst_g = ((dst >> 8) & 0xFF) as f32;
    let dst_b = (dst & 0xFF) as f32;

    let out_r = (src_r as f32 * src_alpha + dst_r * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let out_g = (src_g as f32 * src_alpha + dst_g * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let out_b = (src_b as f32 * src_alpha + dst_b * dst_alpha * (1.0 - src_alpha)) / out_alpha;

    ((out_alpha * 255.0).round() as u32) << 24
        | ((out_r.round() as u32) << 16)
        | ((out_g.round() as u32) << 8)
        | (out_b.round() as u32)
}

pub fn convert_argb_to_premultiplied_bgra(pixels: &[u32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        let alpha = ((pixel >> 24) & 0xFF) as u8;
        let red = ((pixel >> 16) & 0xFF) as u8;
        let green = ((pixel >> 8) & 0xFF) as u8;
        let blue = (pixel & 0xFF) as u8;
        let alpha_factor = alpha as u16;

        output.push(((blue as u16 * alpha_factor + 127) / 255) as u8);
        output.push(((green as u16 * alpha_factor + 127) / 255) as u8);
        output.push(((red as u16 * alpha_factor + 127) / 255) as u8);
        output.push(alpha);
    }
    output
}
