mod monitor;
mod emoji;
mod renderer;
mod config;
mod settings;

use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent, MouseButton, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowLevel, Window},
};
use winit::platform::windows::WindowBuilderExtWindows;
use tray_icon::{TrayIconBuilder, Icon};

use monitor::Monitor;
use renderer::Renderer;
use config::Config;
use settings::Settings;

const WINDOW_SIZE: u32 = 120;
const EMOJI_RADIUS: f32 = 45.0;
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;

const BOUNCE_SPEED: f32 = 5.0;
const BOUNCE_DECAY: f32 = 0.65;

const EMOJIS: &[(&str, &str)] = &[
    ("\u{1F642}", "开心"),
    ("\u{1F622}", "难过"),
    ("\u{1F621}", "生气"),
    ("\u{1F634}", "困倦"),
    ("\u{1F914}", "思考"),
    ("\u{1F975}", "热"),
    ("\u{1F480}", "崩溃"),
    ("\u{1F319}", "晚安"),
];

struct App {
    window: Rc<Window>,
    renderer: Renderer,
    monitor: Monitor,
    config: Config,
    current_emoji_idx: usize,
    auto_mode: bool,
    manual_emoji: Option<usize>,
    
    bounce_y: f32,
    bounce_vel: f32,
    is_bouncing: bool,
    breath_phase: f32,
    is_hovering: bool,
    
    screen_mouse_x: f32,
    screen_mouse_y: f32,
    window_x: f32,
    window_y: f32,
    eye_x: f32,
    eye_y: f32,
    
    last_update: Instant,
    last_activity: Instant,
}

impl App {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let monitor = Monitor::new();
        let config = Config::load();
        
        let pos = window.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
        
        Self {
            window,
            renderer,
            monitor,
            config,
            current_emoji_idx: 0,
            auto_mode: true,
            manual_emoji: None,
            bounce_y: 0.0,
            bounce_vel: 0.0,
            is_bouncing: false,
            breath_phase: 0.0,
            is_hovering: false,
            screen_mouse_x: 0.0,
            screen_mouse_y: 0.0,
            window_x: pos.x as f32 + 60.0,
            window_y: pos.y as f32 + 60.0,
            eye_x: 0.0,
            eye_y: 0.0,
            last_update: Instant::now(),
            last_activity: Instant::now(),
        }
    }

    fn current_emoji_char(&self) -> char {
        let idx = self.manual_emoji.unwrap_or(self.current_emoji_idx);
        EMOJIS[idx].0.chars().next().unwrap_or('\u{1F642}')
    }

    fn trigger_bounce(&mut self) {
        if !self.is_bouncing {
            self.is_bouncing = true;
            self.bounce_vel = -BOUNCE_SPEED;
        }
    }

    fn update_eye(&mut self) {
        let dx = self.screen_mouse_x - self.window_x;
        let dy = self.screen_mouse_y - self.window_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 10.0 {
            let max_offset = 10.0;
            let strength = (dist / 200.0).min(1.0);
            let target_x = (dx / dist) * strength * max_offset;
            let target_y = (dy / dist) * strength * max_offset;
            
            self.eye_x += (target_x - self.eye_x) * 0.15;
            self.eye_y += (target_y - self.eye_y) * 0.15;
        } else {
            self.eye_x *= 0.9;
            self.eye_y *= 0.9;
        }
    }

    fn update_animation(&mut self) {
        if self.is_bouncing {
            self.bounce_vel += 0.4;
            self.bounce_y += self.bounce_vel;
            
            if self.bounce_y >= 0.0 {
                self.bounce_y = 0.0;
                self.bounce_vel = -self.bounce_vel * BOUNCE_DECAY;
                if self.bounce_vel.abs() < 0.3 {
                    self.is_bouncing = false;
                    self.bounce_vel = 0.0;
                }
            }
        }
        
        if self.is_hovering {
            self.breath_phase += 0.06;
        }
    }

    fn get_breath_offset(&self) -> f32 {
        if self.is_hovering { self.breath_phase.sin() * 2.0 } else { 0.0 }
    }

    fn update(&mut self) {
        self.update_animation();
        self.update_eye();
        
        if let Ok(pos) = self.window.outer_position() {
            self.window_x = pos.x as f32 + 60.0;
            self.window_y = pos.y as f32 + 60.0;
        }
        
        if self.last_update.elapsed() >= Duration::from_secs(self.config.update_interval_secs) {
            self.monitor.update();
            self.monitor.set_idle(self.last_activity.elapsed().as_secs());
            
            if self.auto_mode {
                let info = self.monitor.get_info();
                let new_idx = if info.cpu_usage > self.config.cpu_threshold { 5 }
                    else if info.memory_usage > self.config.memory_threshold { 6 }
                    else if info.is_idle { 3 }
                    else { 0 };
                self.current_emoji_idx = new_idx;
            }
            
            self.last_update = Instant::now();
        }
    }

    fn render(&mut self) {
        let center_y = 60.0 + self.bounce_y + self.get_breath_offset();
        self.renderer.render(
            &self.window,
            self.current_emoji_char(),
            center_y,
            EMOJI_RADIUS,
            self.eye_x,
            self.eye_y,
        );
    }

    fn select_emoji(&mut self, idx: usize) {
        self.manual_emoji = Some(idx);
        self.auto_mode = false;
        self.config.auto_mode = false;
        self.config.save();
        self.trigger_bounce();
    }

    fn toggle_auto(&mut self) {
        self.auto_mode = !self.auto_mode;
        if self.auto_mode {
            self.manual_emoji = None;
        }
        self.config.auto_mode = self.auto_mode;
        self.config.save();
        self.trigger_bounce();
    }

    fn toggle_startup(&mut self) {
        self.config.startup = !self.config.startup;
        self.config.save();
    }
}

fn get_screen_cursor_pos() -> (f32, f32) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        use windows::Win32::Foundation::POINT;
        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);
        (point.x as f32, point.y as f32)
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut icon_data = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_data.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    }
    let icon = Icon::from_rgba(icon_data, 16, 16).unwrap();

    // 完整的托盘菜单
    let tray_menu = tray_icon::menu::Menu::new();
    
    let auto_item = tray_icon::menu::MenuItem::new("自动模式 ✓", true, None);
    tray_menu.append(&auto_item).unwrap();
    tray_menu.append(&tray_icon::menu::PredefinedMenuItem::separator()).unwrap();
    
    let manual_menu = tray_icon::menu::Submenu::new("手动选择", true);
    let mut manual_items = Vec::new();
    for (emoji, name) in EMOJIS {
        let item = tray_icon::menu::MenuItem::new(format!("{} {}", emoji, name), true, None);
        manual_menu.append(&item).unwrap();
        manual_items.push(item);
    }
    tray_menu.append(&manual_menu).unwrap();
    tray_menu.append(&tray_icon::menu::PredefinedMenuItem::separator()).unwrap();
    
    let settings_item = tray_icon::menu::MenuItem::new("设置", true, None);
    let startup_item = tray_icon::menu::MenuItem::new("开机启动", true, None);
    tray_menu.append(&settings_item).unwrap();
    tray_menu.append(&startup_item).unwrap();
    tray_menu.append(&tray_icon::menu::PredefinedMenuItem::separator()).unwrap();
    
    let quit_item = tray_icon::menu::MenuItem::new("退出", true, None);
    tray_menu.append(&quit_item).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Deskemoji")
        .with_icon(icon)
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();

    let menu_channel = tray_icon::menu::MenuEvent::receiver();

    let position = event_loop
        .primary_monitor()
        .map(|m| {
            let s = m.size();
            PhysicalPosition::new(
                (s.width - WINDOW_SIZE - WINDOW_MARGIN_RIGHT as u32) as i32,
                (s.height - WINDOW_SIZE - WINDOW_MARGIN_BOTTOM as u32) as i32,
            )
        })
        .unwrap_or(PhysicalPosition::new(100, 100));

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Deskemoji")
            .with_inner_size(PhysicalSize::new(WINDOW_SIZE, WINDOW_SIZE))
            .with_position(position)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_skip_taskbar(true)
            .build(&event_loop)
            .unwrap(),
    );

    let mut app = App::new(window.clone());
    app.render();

    event_loop.run(move |event, elwt| {
        // 处理托盘菜单
        if let Ok(e) = menu_channel.try_recv() {
            if e.id == quit_item.id() {
                elwt.exit();
            } else if e.id == auto_item.id() {
                app.toggle_auto();
                auto_item.set_text(if app.auto_mode { "自动模式 ✓" } else { "自动模式" });
            } else if e.id == settings_item.id() {
                Settings::print_settings(&app.config);
            } else if e.id == startup_item.id() {
                app.toggle_startup();
                startup_item.set_text(if app.config.startup { "开机启动 ✓" } else { "开机启动" });
            } else {
                // 检查手动表情
                for (i, item) in manual_items.iter().enumerate() {
                    if e.id == item.id() {
                        app.select_emoji(i);
                        break;
                    }
                }
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    app.render();
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left && state == ElementState::Pressed {
                        app.trigger_bounce();
                        app.window.drag_window();
                        app.last_activity = Instant::now();
                    }
                    if button == MouseButton::Right && state == ElementState::Released {
                        // 右键显示托盘菜单
                        let tray = _tray_icon.clone();
                        // 无法直接显示菜单，只能通过托盘图标
                    }
                }
                WindowEvent::CursorEntered { .. } => {
                    app.is_hovering = true;
                    app.breath_phase = 0.0;
                }
                WindowEvent::CursorLeft { .. } => {
                    app.is_hovering = false;
                }
                _ => {}
            },
            Event::AboutToWait => {
                let (sx, sy) = get_screen_cursor_pos();
                app.screen_mouse_x = sx;
                app.screen_mouse_y = sy;
                
                app.update();
                app.window.request_redraw();
                elwt.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(16)
                ));
            }
            _ => {}
        }
    });
}
