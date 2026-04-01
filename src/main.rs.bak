mod monitor;
mod emoji;
mod renderer;

use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent, MouseButton, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowLevel, Window},
};
use winit::platform::windows::WindowBuilderExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder, Icon,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageW, WM_NCLBUTTONDOWN, HTCAPTION,
};
use windows::Win32::Foundation::HWND;

use monitor::Monitor;
use emoji::EmojiState;
use renderer::Renderer;

// 动画状态
struct AnimState {
    // 弹跳
    bounce_offset: f32,
    bounce_velocity: f32,
    is_bouncing: bool,
    // 呼吸
    breath_timer: f32,
    is_hovering: bool,
    // 眼神跟随
    eye_offset_x: f32,
    eye_offset_y: f32,
    // 点击效果
    click_scale: f32,
    click_timer: Instant,
}

impl AnimState {
    fn new() -> Self {
        Self {
            bounce_offset: 0.0,
            bounce_velocity: 0.0,
            is_bouncing: false,
            breath_timer: 0.0,
            is_hovering: false,
            eye_offset_x: 0.0,
            eye_offset_y: 0.0,
            click_scale: 1.0,
            click_timer: Instant::now(),
        }
    }

    fn trigger_bounce(&mut self) {
        self.is_bouncing = true;
        self.bounce_velocity = -10.0;
        self.click_timer = Instant::now();
    }

    fn update(&mut self, cursor_pos: PhysicalPosition<f64>, window_center: (f32, f32)) {
        // 弹跳物理模拟
        if self.is_bouncing {
            self.bounce_velocity += 1.5; // 重力
            self.bounce_offset += self.bounce_velocity;
            
            if self.bounce_offset >= 0.0 {
                self.bounce_offset = 0.0;
                self.bounce_velocity = -self.bounce_velocity * 0.4; // 反弹衰减
                
                if self.bounce_velocity.abs() < 1.0 {
                    self.is_bouncing = false;
                    self.bounce_velocity = 0.0;
                }
            }
        }

        // 呼吸效果
        if self.is_hovering && !self.is_bouncing {
            self.breath_timer += 0.08;
        }

        // 眼神跟随
        let dx = cursor_pos.x as f32 - window_center.0;
        let dy = cursor_pos.y as f32 - window_center.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist < 200.0 && dist > 0.0 {
            let max_offset = 4.0;
            let factor = (dist / 50.0).min(1.0);
            self.eye_offset_x = (dx / dist) * factor * max_offset;
            self.eye_offset_y = (dy / dist) * factor * max_offset;
        } else {
            self.eye_offset_x *= 0.9;
            self.eye_offset_y *= 0.9;
        }

        // 点击缩放效果
        if self.click_timer.elapsed() < Duration::from_millis(150) {
            let t = self.click_timer.elapsed().as_millis() as f32 / 150.0;
            self.click_scale = 1.0 + 0.15 * (1.0 - (t * std::f32::consts::PI).cos()) / 2.0;
        } else {
            self.click_scale = 1.0;
        }
    }

    fn get_total_offset_y(&self) -> f32 {
        let breath_offset = if self.is_hovering { 
            (self.breath_timer.sin() * 2.5) 
        } else { 
            0.0 
        };
        self.bounce_offset + breath_offset
    }
}

struct AppState {
    window: Rc<Window>,
    renderer: Renderer,
    monitor: Monitor,
    current_emoji: EmojiState,
    last_update: Instant,
    anim: AnimState,
    cursor_pos: PhysicalPosition<f64>,
    need_redraw: bool,
    hwnd: Option<HWND>,
}

impl AppState {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let mut monitor = Monitor::new();
        let current_emoji = EmojiState::from_system_info(&monitor.get_info());
        
        // 获取 HWND 用于原生窗口移动
        let hwnd = window.window_handle().ok().and_then(|handle| {
            match handle.as_raw() {
                RawWindowHandle::Win32(win32_handle) => {
                    Some(HWND(isize::from(win32_handle.hwnd) as _))
                }
                _ => None,
            }
        });

        Self {
            window,
            renderer,
            monitor,
            current_emoji,
            last_update: Instant::now(),
            anim: AnimState::new(),
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            need_redraw: true,
            hwnd,
        }
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = position;
    }

    fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left && state == ElementState::Pressed {
            // 使用 Windows 原生窗口移动 - 完美流畅
            if let Some(hwnd) = self.hwnd {
                unsafe {
                    use windows::Win32::Foundation::WPARAM;
                    let _ = SendMessageW(hwnd, WM_NCLBUTTONDOWN, WPARAM(HTCAPTION as usize), None);
                }
            }
            // 触发弹跳动画
            self.anim.trigger_bounce();
            self.need_redraw = true;
        }
    }

    fn handle_hover(&mut self, entered: bool) {
        self.anim.is_hovering = entered;
        if entered {
            self.anim.breath_timer = 0.0;
        }
        self.need_redraw = true;
    }

    fn update(&mut self) {
        // 更新系统信息
        if self.last_update.elapsed() >= Duration::from_secs(2) {
            self.monitor.update();
            let info = self.monitor.get_info();
            let new_emoji = EmojiState::from_system_info(&info);

            if new_emoji.scenario != self.current_emoji.scenario {
                self.current_emoji = new_emoji;
                self.need_redraw = true;
            }

            self.last_update = Instant::now();
        }

        // 更新动画
        let size = self.window.inner_size();
        let center = (size.width as f32 / 2.0, size.height as f32 / 2.0);
        self.anim.update(self.cursor_pos, center);
        
        // 检查是否需要重绘
        if self.anim.is_bouncing || self.anim.is_hovering {
            self.need_redraw = true;
        }
    }

    fn render(&mut self) {
        if self.need_redraw {
            let offset_y = self.anim.get_total_offset_y();
            self.renderer.render(
                &self.window,
                self.current_emoji.emoji,
                self.anim.click_scale,
                offset_y,
                self.anim.eye_offset_x,
                self.anim.eye_offset_y,
            );
            self.need_redraw = false;
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("退出", true, None);
    tray_menu.append(&quit_item).unwrap();

    let mut icon_data = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_data.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    }
    let icon = Icon::from_rgba(icon_data, 16, 16).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Deskemoji")
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();

    let primary_monitor = event_loop.primary_monitor().unwrap();
    let screen_size = primary_monitor.size();
    let window_size = PhysicalSize::new(120u32, 120u32);
    let position = PhysicalPosition::new(
        (screen_size.width - window_size.width - 20) as i32,
        (screen_size.height - window_size.height - 60) as i32,
    );

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Deskemoji")
            .with_inner_size(window_size)
            .with_position(position)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_skip_taskbar(true)
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = AppState::new(window.clone());

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                elwt.exit();
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    state.render();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    state.handle_cursor_moved(position);
                }
                WindowEvent::MouseInput { state: mouse_state, button, .. } => {
                    state.handle_mouse_input(button, mouse_state);
                }
                WindowEvent::CursorEntered { .. } => {
                    state.handle_hover(true);
                }
                WindowEvent::CursorLeft { .. } => {
                    state.handle_hover(false);
                }
                _ => {}
            },
            Event::AboutToWait => {
                state.update();
                state.window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
