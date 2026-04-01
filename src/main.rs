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
use emoji::EmojiState;
use renderer::Renderer;
use config::Config;
use settings::Settings;

const WINDOW_SIZE: u32 = 180;
const EMOJI_CENTER_Y: f32 = 55.0;
const EMOJI_RADIUS: f32 = 40.0;
const MENU_START_Y: i32 = 100;
const MENU_ITEM_HEIGHT: i32 = 20;
const MENU_WIDTH: i32 = 100;
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;
const BOUNCE_SPEED: f32 = 4.0;
const BOUNCE_DECAY: f32 = 0.7;
const EYE_SENSITIVITY: f32 = 8.0;

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub text: String,
    pub action: MenuAction,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    ToggleAuto,
    SelectEmoji(usize),
    ShowSettings,
    ToggleStartup,
    Quit,
}

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
    show_menu: bool,
    menu_items: Vec<MenuItem>,
    menu_hover: Option<usize>,
    bounce_y: f32,
    bounce_vel: f32,
    is_bouncing: bool,
    breath_phase: f32,
    is_hovering: bool,
    mouse_x: f32,
    mouse_y: f32,
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
        
        let menu_items = vec![
            MenuItem { text: "自动模式".into(), action: MenuAction::ToggleAuto },
            MenuItem { text: "────────".into(), action: MenuAction::ShowSettings },
            MenuItem { text: "🙂 开心".into(), action: MenuAction::SelectEmoji(0) },
            MenuItem { text: "😢 难过".into(), action: MenuAction::SelectEmoji(1) },
            MenuItem { text: "😡 生气".into(), action: MenuAction::SelectEmoji(2) },
            MenuItem { text: "😴 困倦".into(), action: MenuAction::SelectEmoji(3) },
            MenuItem { text: "🤔 思考".into(), action: MenuAction::SelectEmoji(4) },
            MenuItem { text: "🥵 热".into(), action: MenuAction::SelectEmoji(5) },
            MenuItem { text: "💀 崩溃".into(), action: MenuAction::SelectEmoji(6) },
            MenuItem { text: "🌙 晚安".into(), action: MenuAction::SelectEmoji(7) },
            MenuItem { text: "────────".into(), action: MenuAction::ShowSettings },
            MenuItem { text: "设置".into(), action: MenuAction::ShowSettings },
            MenuItem { text: "开机启动".into(), action: MenuAction::ToggleStartup },
            MenuItem { text: "退出".into(), action: MenuAction::Quit },
        ];
        
        Self {
            window, renderer, monitor, config,
            current_emoji_idx: 0,
            auto_mode: true,
            manual_emoji: None,
            show_menu: false,
            menu_items,
            menu_hover: None,
            bounce_y: 0.0,
            bounce_vel: 0.0,
            is_bouncing: false,
            breath_phase: 0.0,
            is_hovering: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            eye_x: 0.0,
            eye_y: 0.0,
            last_update: Instant::now(),
            last_activity: Instant::now(),
        }
    }

    fn current_emoji_char(&self) -> char {
        let idx = self.manual_emoji.unwrap_or(self.current_emoji_idx);
        EMOJIS[idx].0.chars().next().unwrap_or('🙂')
    }

    fn trigger_bounce(&mut self) {
        if !self.is_bouncing {
            self.is_bouncing = true;
            self.bounce_vel = -BOUNCE_SPEED;
        }
    }

    fn update_eye(&mut self) {
        let cx = (WINDOW_SIZE as f32) / 2.0;
        let cy = EMOJI_CENTER_Y;
        
        let dx = self.mouse_x - cx;
        let dy = self.mouse_y - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 5.0 {
            let nx = dx / dist;
            let ny = dy / dist;
            let strength = (dist / 80.0).min(1.0);
            
            let target_x = nx * strength * EYE_SENSITIVITY;
            let target_y = ny * strength * EYE_SENSITIVITY;
            
            self.eye_x += (target_x - self.eye_x) * 0.25;
            self.eye_y += (target_y - self.eye_y) * 0.25;
        } else {
            self.eye_x *= 0.8;
            self.eye_y *= 0.8;
        }
    }

    fn update_animation(&mut self) {
        if self.is_bouncing {
            self.bounce_vel += 0.3;
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
            self.breath_phase += 0.05;
        }
    }

    fn get_breath_offset(&self) -> f32 {
        if self.is_hovering { self.breath_phase.sin() * 2.0 } else { 0.0 }
    }

    fn handle_menu_click(&mut self, idx: usize) {
        if let Some(item) = self.menu_items.get(idx) {
            match &item.action {
                MenuAction::ToggleAuto => {
                    self.auto_mode = !self.auto_mode;
                    if self.auto_mode { self.manual_emoji = None; }
                    self.config.auto_mode = self.auto_mode;
                    self.config.save();
                }
                MenuAction::SelectEmoji(i) => {
                    self.manual_emoji = Some(*i);
                    self.auto_mode = false;
                    self.config.auto_mode = false;
                    self.config.save();
                }
                MenuAction::ShowSettings => {
                    Settings::print_settings(&self.config);
                }
                MenuAction::ToggleStartup => {
                    self.config.startup = !self.config.startup;
                    self.config.save();
                }
                MenuAction::Quit => {
                    std::process::exit(0);
                }
            }
            self.trigger_bounce();
        }
        self.show_menu = false;
    }

    fn update(&mut self) {
        self.update_animation();
        self.update_eye();
        
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
        let emoji = self.current_emoji_char();
        let center_y = EMOJI_CENTER_Y + self.bounce_y + self.get_breath_offset();
        
        self.renderer.render(
            &self.window,
            emoji,
            center_y,
            EMOJI_RADIUS,
            self.eye_x,
            self.eye_y,
            self.show_menu,
            &self.menu_items,
            self.menu_hover,
            MENU_START_Y,
            MENU_ITEM_HEIGHT,
            MENU_WIDTH,
            self.auto_mode,
            self.config.startup,
        );
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut icon_data = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_data.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    }
    let icon = Icon::from_rgba(icon_data, 16, 16).unwrap();

    let tray_menu = tray_icon::menu::Menu::new();
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
        if let Ok(e) = menu_channel.try_recv() {
            if e.id == quit_item.id() {
                elwt.exit();
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    app.render();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    app.mouse_x = position.x as f32;
                    app.mouse_y = position.y as f32;
                    app.last_activity = Instant::now();
                    
                    if app.show_menu {
                        let rel_y = position.y as i32 - MENU_START_Y;
                        if rel_y >= 0 {
                            let idx = (rel_y / MENU_ITEM_HEIGHT) as usize;
                            if idx < app.menu_items.len() {
                                app.menu_hover = Some(idx);
                            } else {
                                app.menu_hover = None;
                            }
                        } else {
                            app.menu_hover = None;
                        }
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left && state == ElementState::Pressed {
                        if app.show_menu {
                            if let Some(idx) = app.menu_hover {
                                app.handle_menu_click(idx);
                            } else {
                                app.show_menu = false;
                            }
                        } else {
                            app.trigger_bounce();
                            app.window.drag_window();
                        }
                        app.last_activity = Instant::now();
                    }
                    if button == MouseButton::Right && state == ElementState::Released {
                        app.show_menu = !app.show_menu;
                        app.menu_hover = None;
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
