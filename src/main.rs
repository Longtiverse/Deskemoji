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

const EMOJI_OPTIONS: &[(&str, &str)] = &[
    ("\u{1F642}", "开心"),
    ("\u{1F622}", "难过"),
    ("\u{1F621}", "生气"),
    ("\u{1F634}", "困倦"),
    ("\u{1F914}", "思考"),
    ("\u{1F975}", "热"),
    ("\u{1F480}", "崩溃"),
    ("\u{1F319}", "晚安"),
];

// 动画参数
const BOUNCE_GRAVITY: f32 = 0.6;
const BOUNCE_DECAY: f32 = 0.65;
const BOUNCE_INITIAL_VELOCITY: f32 = -5.0;
const BOUNCE_MIN_VELOCITY: f32 = 0.3;
const BREATH_SPEED: f32 = 0.04;
const BREATH_AMPLITUDE: f32 = 1.5;
const EYE_MAX_OFFSET: f32 = 4.0;
const EYE_DECAY: f32 = 0.85;
const CLICK_DURATION_MS: u128 = 200;
const CLICK_SCALE_MAX: f32 = 0.1;
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;
const DEFAULT_POSITION: (i32, i32) = (100, 100);
const FRAME_DURATION: Duration = Duration::from_millis(16);

const MAIN_WINDOW_SIZE: u32 = 160;
const EMOJI_RADIUS: f32 = 55.0;
const MAX_BOUNCE_OFFSET: f32 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    AutoMode,
    ManualEmoji(usize),
    Settings,
    Startup,
    Quit,
}

struct MenuState {
    visible: bool,
    items: Vec<(MenuItem, String)>,
    hovered: Option<usize>,
    cursor_pos: PhysicalPosition<f64>,
}

impl MenuState {
    fn new() -> Self {
        let mut items = Vec::new();
        items.push((MenuItem::AutoMode, "自动模式".to_string()));
        for (i, (emoji, name)) in EMOJI_OPTIONS.iter().enumerate() {
            items.push((MenuItem::ManualEmoji(i), format!("{} {}", emoji, name)));
        }
        items.push((MenuItem::Settings, "设置".to_string()));
        items.push((MenuItem::Startup, "开机启动".to_string()));
        items.push((MenuItem::Quit, "退出".to_string()));
        
        Self {
            visible: false,
            items,
            hovered: None,
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    fn show(&mut self, cursor_pos: PhysicalPosition<f64>) {
        self.visible = true;
        self.cursor_pos = cursor_pos;
        self.hovered = None;
    }

    fn hide(&mut self) {
        self.visible = false;
        self.hovered = None;
    }

    fn handle_hover(&mut self, x: f64, y: f64) {
        if !self.visible {
            return;
        }
        
        let item_height = 32.0;
        let relative_y = y - self.cursor_pos.y;
        
        if relative_y < 0.0 {
            self.hovered = None;
            return;
        }
        
        let item_index = (relative_y / item_height) as usize;
        if item_index < self.items.len() {
            self.hovered = Some(item_index);
        } else {
            self.hovered = None;
        }
    }

    fn handle_click(&self, x: f64, y: f64) -> Option<MenuItem> {
        if !self.visible {
            return None;
        }
        
        let item_height = 32.0;
        let relative_y = y - self.cursor_pos.y;
        
        if relative_y < 0.0 {
            return None;
        }
        
        let item_index = (relative_y / item_height) as usize;
        if item_index < self.items.len() {
            return Some(self.items[item_index].0);
        }
        
        None
    }
}

struct AnimState {
    bounce_offset: f32,
    bounce_velocity: f32,
    is_bouncing: bool,
    breath_timer: f32,
    is_hovering: bool,
    eye_offset_x: f32,
    eye_offset_y: f32,
    click_scale: f32,
    click_timer: Instant,
    last_update: Instant,
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
            last_update: Instant::now(),
        }
    }

    fn trigger_bounce(&mut self) {
        if !self.is_bouncing {
            self.is_bouncing = true;
            self.bounce_velocity = BOUNCE_INITIAL_VELOCITY;
            self.click_timer = Instant::now();
        }
    }

    fn update(&mut self, cursor_pos: PhysicalPosition<f64>, window_center: (f32, f32)) -> bool {
        if self.last_update.elapsed() < FRAME_DURATION {
            return false;
        }
        self.last_update = Instant::now();
        
        let mut changed = false;

        if self.is_bouncing {
            self.bounce_velocity += BOUNCE_GRAVITY;
            self.bounce_offset += self.bounce_velocity;
            
            if self.bounce_offset < -MAX_BOUNCE_OFFSET {
                self.bounce_offset = -MAX_BOUNCE_OFFSET;
                self.bounce_velocity = 0.0;
            }
            
            if self.bounce_offset >= 0.0 {
                self.bounce_offset = 0.0;
                self.bounce_velocity = -self.bounce_velocity * BOUNCE_DECAY;
                
                if self.bounce_velocity.abs() < BOUNCE_MIN_VELOCITY {
                    self.is_bouncing = false;
                    self.bounce_velocity = 0.0;
                }
            }
            changed = true;
        }

        if self.is_hovering {
            self.breath_timer += BREATH_SPEED;
            changed = true;
        }

        // 全屏幕眼神跟随
        let dx = cursor_pos.x as f32 - window_center.0;
        let dy = cursor_pos.y as f32 - window_center.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        let old_eye_x = self.eye_offset_x;
        let old_eye_y = self.eye_offset_y;
        
        if dist > 1.0 {
            let factor = 1.0 - (-dist / 500.0).exp();
            let target_x = (dx / dist) * factor * EYE_MAX_OFFSET;
            let target_y = (dy / dist) * factor * EYE_MAX_OFFSET;
            
            self.eye_offset_x += (target_x - self.eye_offset_x) * 0.12;
            self.eye_offset_y += (target_y - self.eye_offset_y) * 0.12;
        } else {
            self.eye_offset_x *= EYE_DECAY;
            self.eye_offset_y *= EYE_DECAY;
        }
        
        if (self.eye_offset_x - old_eye_x).abs() > 0.005 || (self.eye_offset_y - old_eye_y).abs() > 0.005 {
            changed = true;
        }

        let old_scale = self.click_scale;
        let click_elapsed = self.click_timer.elapsed().as_millis() as f32;
        if click_elapsed < CLICK_DURATION_MS as f32 {
            let t = click_elapsed / CLICK_DURATION_MS as f32;
            let eased = 1.0 - (1.0 - t).powi(2);
            self.click_scale = 1.0 + CLICK_SCALE_MAX * (1.0 - eased) * (std::f32::consts::PI * t * 2.0).sin().abs();
        } else {
            self.click_scale = 1.0;
        }
        
        if (self.click_scale - old_scale).abs() > 0.001 {
            changed = true;
        }

        changed
    }

    fn get_total_offset_y(&self) -> f32 {
        let breath_offset = if self.is_hovering { 
            self.breath_timer.sin() * BREATH_AMPLITUDE
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
    last_activity: Instant,
    anim: AnimState,
    cursor_pos: PhysicalPosition<f64>,
    animating: bool,
    config: Config,
    manual_emoji: Option<char>,
    menu: MenuState,
    need_redraw: bool,
}

impl AppState {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let mut monitor = Monitor::new();
        let current_emoji = EmojiState::from_system_info(&monitor.get_info());
        let config = Config::load();

        Self {
            window,
            renderer,
            monitor,
            current_emoji,
            last_update: Instant::now(),
            last_activity: Instant::now(),
            anim: AnimState::new(),
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            animating: false,
            config,
            manual_emoji: None,
            menu: MenuState::new(),
            need_redraw: true,
        }
    }

    fn get_display_emoji(&self) -> char {
        if let Some(emoji) = self.manual_emoji {
            return emoji;
        }
        self.current_emoji.emoji
    }

    fn handle_menu_action(&mut self, action: MenuItem) -> bool {
        let mut changed = false;
        
        match action {
            MenuItem::AutoMode => {
                self.config.auto_mode = !self.config.auto_mode;
                if self.config.auto_mode {
                    self.manual_emoji = None;
                }
                self.config.save();
                changed = true;
            }
            MenuItem::ManualEmoji(idx) => {
                if let Some((emoji, _)) = EMOJI_OPTIONS.get(idx) {
                    self.manual_emoji = emoji.chars().next();
                    self.config.auto_mode = false;
                    self.config.save();
                    changed = true;
                }
            }
            MenuItem::Settings => {
                Settings::print_settings(&self.config);
            }
            MenuItem::Startup => {
                self.config.startup = !self.config.startup;
                self.config.save();
            }
            MenuItem::Quit => {
                std::process::exit(0);
            }
        }
        
        if changed {
            self.need_redraw = true;
            self.anim.trigger_bounce();
        }
        
        changed
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = position;
        self.last_activity = Instant::now();
        
        if self.menu.visible {
            self.menu.handle_hover(position.x, position.y);
        }
    }

    fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => {
                if state == ElementState::Pressed {
                    if self.menu.visible {
                        if let Some(action) = self.menu.handle_click(self.cursor_pos.x, self.cursor_pos.y) {
                            self.handle_menu_action(action);
                        }
                        self.menu.hide();
                    } else {
                        self.last_activity = Instant::now();
                        self.anim.trigger_bounce();
                        self.animating = true;
                        self.window.drag_window();
                    }
                }
            }
            MouseButton::Right => {
                if state == ElementState::Released {
                    if self.menu.visible {
                        self.menu.hide();
                    } else {
                        self.menu.show(self.cursor_pos);
                    }
                    self.need_redraw = true;
                }
            }
            _ => {}
        }
    }

    fn handle_hover(&mut self, entered: bool) {
        self.anim.is_hovering = entered;
        if entered {
            self.anim.breath_timer = 0.0;
            self.last_activity = Instant::now();
            self.animating = true;
        } else {
            self.animating = false;
            if self.menu.visible {
                self.menu.hide();
                self.need_redraw = true;
            }
        }
    }

    fn update(&mut self) -> bool {
        if self.last_update.elapsed() >= Duration::from_secs(self.config.update_interval_secs) {
            self.monitor.update();
            
            let idle_secs = self.last_activity.elapsed().as_secs();
            self.monitor.set_idle(idle_secs);
            
            if self.config.auto_mode {
                let info = self.monitor.get_info();
                let new_emoji = EmojiState::from_system_info(&info);

                if new_emoji.scenario != self.current_emoji.scenario {
                    self.current_emoji = new_emoji;
                    self.need_redraw = true;
                }
            }

            self.last_update = Instant::now();
        }

        let size = self.window.inner_size();
        let center = (size.width as f32 / 2.0, size.height as f32 / 2.0);
        let anim_changed = self.anim.update(self.cursor_pos, center);
        
        if anim_changed {
            self.animating = true;
            self.need_redraw = true;
        } else if !self.anim.is_bouncing && !self.anim.is_hovering && !self.menu.visible {
            self.animating = false;
        }

        if self.menu.visible {
            self.need_redraw = true;
        }

        self.need_redraw
    }

    fn render(&mut self) {
        if self.need_redraw {
            let offset_y = self.anim.get_total_offset_y();
            self.renderer.render(
                &self.window,
                self.get_display_emoji(),
                self.anim.click_scale,
                offset_y,
                self.anim.eye_offset_x,
                self.anim.eye_offset_y,
                EMOJI_RADIUS,
                &self.menu,
            );
            self.need_redraw = false;
        }
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
        .with_tooltip("Deskemoji - 右键弹出菜单")
        .with_icon(icon)
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();

    let position = event_loop
        .primary_monitor()
        .map(|monitor| {
            let screen_size = monitor.size();
            PhysicalPosition::new(
                (screen_size.width - MAIN_WINDOW_SIZE - WINDOW_MARGIN_RIGHT as u32) as i32,
                (screen_size.height - MAIN_WINDOW_SIZE - WINDOW_MARGIN_BOTTOM as u32) as i32,
            )
        })
        .unwrap_or_else(|| {
            PhysicalPosition::new(DEFAULT_POSITION.0, DEFAULT_POSITION.1)
        });

    let window_size = PhysicalSize::new(MAIN_WINDOW_SIZE, MAIN_WINDOW_SIZE);

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
    state.render();

    let menu_channel = tray_icon::menu::MenuEvent::receiver();

    event_loop.run(move |event, elwt| {
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                elwt.exit();
                return;
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
                let should_render = state.update();
                
                if should_render {
                    state.window.request_redraw();
                }
                
                if state.animating || state.menu.visible {
                    elwt.set_control_flow(ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION));
                } else {
                    elwt.set_control_flow(ControlFlow::Wait);
                }
            }
            _ => {}
        }
    });
}
