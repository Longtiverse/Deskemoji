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

// 动画参数 - 更柔和自然
const BOUNCE_GRAVITY: f32 = 0.8;  // 降低重力
const BOUNCE_DECAY: f32 = 0.6;    // 增加衰减
const BOUNCE_INITIAL_VELOCITY: f32 = -6.0;  // 降低初始速度
const BOUNCE_MIN_VELOCITY: f32 = 0.5;
const BREATH_SPEED: f32 = 0.04;   // 降低呼吸速度
const BREATH_AMPLITUDE: f32 = 1.5;  // 降低幅度
const EYE_TRACK_DISTANCE: f32 = 200.0;
const EYE_TRACK_FACTOR: f32 = 80.0;  // 增加平滑度
const EYE_MAX_OFFSET: f32 = 3.0;     // 降低最大偏移
const EYE_DECAY: f32 = 0.85;         // 增加衰减
const CLICK_DURATION_MS: u128 = 200;  // 延长点击动画
const CLICK_SCALE_MAX: f32 = 0.1;     // 降低缩放幅度
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;
const DEFAULT_POSITION: (i32, i32) = (100, 100);
const FRAME_DURATION: Duration = Duration::from_millis(16);  // ~60fps

// 菜单项
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    AutoMode,
    ManualEmoji(usize),
    Settings,
    Startup,
    Quit,
}

// 右键菜单状态
struct ContextMenu {
    visible: bool,
    position: PhysicalPosition<i32>,
    items: Vec<(MenuItem, String)>,
    hovered: Option<usize>,
}

impl ContextMenu {
    fn new() -> Self {
        let mut items = Vec::new();
        
        // 自动模式
        items.push((MenuItem::AutoMode, "自动模式".to_string()));
        
        // 手动表情
        for (i, (emoji, name)) in EMOJI_OPTIONS.iter().enumerate() {
            items.push((MenuItem::ManualEmoji(i), format!("{} {}", emoji, name)));
        }
        
        // 设置
        items.push((MenuItem::Settings, "设置".to_string()));
        items.push((MenuItem::Startup, "开机启动".to_string()));
        items.push((MenuItem::Quit, "退出".to_string()));
        
        Self {
            visible: false,
            position: PhysicalPosition::new(0, 0),
            items,
            hovered: None,
        }
    }
    
    fn show(&mut self, position: PhysicalPosition<i32>) {
        self.visible = true;
        self.position = position;
        self.hovered = None;
    }
    
    fn hide(&mut self) {
        self.visible = false;
        self.hovered = None;
    }
    
    fn handle_click(&self, x: i32, y: i32) -> Option<MenuItem> {
        if !self.visible {
            return None;
        }
        
        let item_height = 30;
        let menu_width = 150;
        
        // 检查是否在菜单范围内
        if x < self.position.x || x > self.position.x + menu_width {
            return None;
        }
        
        let relative_y = y - self.position.y;
        if relative_y < 0 {
            return None;
        }
        
        let item_index = relative_y as usize / item_height;
        if item_index < self.items.len() {
            return Some(self.items[item_index].0);
        }
        
        None
    }
    
    fn handle_hover(&mut self, x: i32, y: i32) {
        if !self.visible {
            return;
        }
        
        let item_height = 30;
        let menu_width = 150;
        
        if x < self.position.x || x > self.position.x + menu_width {
            self.hovered = None;
            return;
        }
        
        let relative_y = y - self.position.y;
        if relative_y < 0 {
            self.hovered = None;
            return;
        }
        
        let item_index = relative_y as usize / item_height;
        if item_index < self.items.len() {
            self.hovered = Some(item_index);
        } else {
            self.hovered = None;
        }
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
        // 限制更新频率
        if self.last_update.elapsed() < FRAME_DURATION {
            return false;
        }
        self.last_update = Instant::now();
        
        let mut changed = false;

        // 弹跳物理 - 更自然的衰减
        if self.is_bouncing {
            self.bounce_velocity += BOUNCE_GRAVITY;
            self.bounce_offset += self.bounce_velocity;
            
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

        // 呼吸效果 - 更平滑
        if self.is_hovering {
            self.breath_timer += BREATH_SPEED;
            changed = true;
        }

        // 眼神跟随 - 更平滑的跟踪
        let dx = cursor_pos.x as f32 - window_center.0;
        let dy = cursor_pos.y as f32 - window_center.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        let old_eye_x = self.eye_offset_x;
        let old_eye_y = self.eye_offset_y;
        
        if dist < EYE_TRACK_DISTANCE && dist > 1.0 {
            let factor = (dist / EYE_TRACK_FACTOR).min(1.0);
            let target_x = (dx / dist) * factor * EYE_MAX_OFFSET;
            let target_y = (dy / dist) * factor * EYE_MAX_OFFSET;
            
            // 平滑插值
            self.eye_offset_x += (target_x - self.eye_offset_x) * 0.15;
            self.eye_offset_y += (target_y - self.eye_offset_y) * 0.15;
        } else {
            self.eye_offset_x *= EYE_DECAY;
            self.eye_offset_y *= EYE_DECAY;
        }
        
        if (self.eye_offset_x - old_eye_x).abs() > 0.005 || (self.eye_offset_y - old_eye_y).abs() > 0.005 {
            changed = true;
        }

        // 点击缩放效果 - 更平滑
        let old_scale = self.click_scale;
        let click_elapsed = self.click_timer.elapsed().as_millis() as f32;
        if click_elapsed < CLICK_DURATION_MS as f32 {
            let t = click_elapsed / CLICK_DURATION_MS as f32;
            // 使用平滑的 easing 函数
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
    context_menu: ContextMenu,
    need_redraw: bool,
}

impl AppState {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let mut monitor = Monitor::new();
        let current_emoji = EmojiState::from_system_info(&monitor.get_info());
        let config = Config::load();
        let context_menu = ContextMenu::new();

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
            context_menu,
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
        
        // 更新菜单悬停状态
        if self.context_menu.visible {
            self.context_menu.handle_hover(position.x as i32, position.y as i32);
            self.need_redraw = true;
        }
    }

    fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => {
                if state == ElementState::Pressed {
                    // 如果菜单可见，检查是否点击了菜单项
                    if self.context_menu.visible {
                        if let Some(action) = self.context_menu.handle_click(self.cursor_pos.x as i32, self.cursor_pos.y as i32) {
                            self.handle_menu_action(action);
                        }
                        self.context_menu.hide();
                        self.need_redraw = true;
                    } else {
                        // 否则开始拖动
                        self.last_activity = Instant::now();
                        self.anim.trigger_bounce();
                        self.animating = true;
                        self.window.drag_window();
                    }
                }
            }
            MouseButton::Right => {
                if state == ElementState::Released {
                    // 显示右键菜单
                    self.context_menu.show(PhysicalPosition::new(
                        self.cursor_pos.x as i32,
                        self.cursor_pos.y as i32,
                    ));
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
        }
        self.need_redraw = true;
    }

    fn update(&mut self) -> bool {
        // 更新系统信息
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

        // 更新动画
        let size = self.window.inner_size();
        let center = (size.width as f32 / 2.0, size.height as f32 / 2.0);
        let anim_changed = self.anim.update(self.cursor_pos, center);
        
        if anim_changed {
            self.animating = true;
            self.need_redraw = true;
        } else if !self.anim.is_bouncing && !self.anim.is_hovering && !self.context_menu.visible {
            self.animating = false;
        }

        // 始终需要重绘以显示菜单
        if self.context_menu.visible {
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
                &self.context_menu,
            );
            self.need_redraw = false;
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // 系统托盘（仅用于退出）
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

    let position = event_loop
        .primary_monitor()
        .map(|monitor| {
            let screen_size = monitor.size();
            PhysicalPosition::new(
                (screen_size.width - 120 - WINDOW_MARGIN_RIGHT as u32) as i32,
                (screen_size.height - 120 - WINDOW_MARGIN_BOTTOM as u32) as i32,
            )
        })
        .unwrap_or_else(|| {
            PhysicalPosition::new(DEFAULT_POSITION.0, DEFAULT_POSITION.1)
        });

    let window_size = PhysicalSize::new(120u32, 120u32);

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
        // 处理托盘菜单
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
                    // 鼠标离开时隐藏菜单
                    if state.context_menu.visible {
                        state.context_menu.hide();
                        state.need_redraw = true;
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                let should_render = state.update();
                
                if should_render {
                    state.window.request_redraw();
                }
                
                // 使用 Wait 而不是 Poll，只在需要时唤醒
                if state.animating || state.context_menu.visible {
                    elwt.set_control_flow(ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION));
                } else {
                    elwt.set_control_flow(ControlFlow::Wait);
                }
            }
            _ => {}
        }
    });
}
