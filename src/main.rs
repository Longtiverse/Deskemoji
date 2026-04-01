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
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder, Icon,
};

use monitor::Monitor;
use emoji::EmojiState;
use renderer::Renderer;

const BOUNCE_GRAVITY: f32 = 1.5;
const BOUNCE_DECAY: f32 = 0.4;
const BOUNCE_INITIAL_VELOCITY: f32 = -10.0;
const BOUNCE_MIN_VELOCITY: f32 = 1.0;
const BREATH_SPEED: f32 = 0.08;
const BREATH_AMPLITUDE: f32 = 2.5;
const BREATH_TIMER_MAX: f32 = 1000.0;
const EYE_TRACK_DISTANCE: f32 = 200.0;
const EYE_TRACK_FACTOR: f32 = 50.0;
const EYE_MAX_OFFSET: f32 = 4.0;
const EYE_DECAY: f32 = 0.9;
const CLICK_DURATION_MS: u128 = 150;
const CLICK_SCALE_MAX: f32 = 0.15;
const UPDATE_INTERVAL_SECS: u64 = 2;
const WINDOW_SIZE: u32 = 120;
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;
const DEFAULT_POSITION: (i32, i32) = (100, 100);

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
        self.bounce_velocity = BOUNCE_INITIAL_VELOCITY;
        self.click_timer = Instant::now();
    }

    fn update(&mut self, cursor_pos: PhysicalPosition<f64>, window_center: (f32, f32)) {
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
        }

        if self.is_hovering && !self.is_bouncing {
            self.breath_timer += BREATH_SPEED;
            if self.breath_timer > BREATH_TIMER_MAX {
                self.breath_timer = 0.0;
            }
        }

        let dx = cursor_pos.x as f32 - window_center.0;
        let dy = cursor_pos.y as f32 - window_center.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist < EYE_TRACK_DISTANCE && dist > 0.0 {
            let factor = (dist / EYE_TRACK_FACTOR).min(1.0);
            self.eye_offset_x = (dx / dist) * factor * EYE_MAX_OFFSET;
            self.eye_offset_y = (dy / dist) * factor * EYE_MAX_OFFSET;
        } else {
            self.eye_offset_x *= EYE_DECAY;
            self.eye_offset_y *= EYE_DECAY;
        }

        if self.click_timer.elapsed() < Duration::from_millis(CLICK_DURATION_MS as u64) {
            let t = self.click_timer.elapsed().as_millis() as f32 / CLICK_DURATION_MS as f32;
            self.click_scale = 1.0 + CLICK_SCALE_MAX * (1.0 - (t * std::f32::consts::PI).cos()) / 2.0;
        } else {
            self.click_scale = 1.0;
        }
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
    need_redraw: bool,
    is_dragging: bool,
    drag_start_cursor: PhysicalPosition<f64>,
    drag_start_window: PhysicalPosition<i32>,
}

impl AppState {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let mut monitor = Monitor::new();
        let current_emoji = EmojiState::from_system_info(&monitor.get_info());

        Self {
            window,
            renderer,
            monitor,
            current_emoji,
            last_update: Instant::now(),
            last_activity: Instant::now(),
            anim: AnimState::new(),
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            need_redraw: true,
            is_dragging: false,
            drag_start_cursor: PhysicalPosition::new(0.0, 0.0),
            drag_start_window: PhysicalPosition::new(0, 0),
        }
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = position;
        self.last_activity = Instant::now();
        
        if self.is_dragging {
            let dx = (position.x - self.drag_start_cursor.x) as i32;
            let dy = (position.y - self.drag_start_cursor.y) as i32;
            let new_x = self.drag_start_window.x + dx;
            let new_y = self.drag_start_window.y + dy;
            self.window.set_outer_position(PhysicalPosition::new(new_x, new_y));
        }
    }

    fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    self.last_activity = Instant::now();
                    self.is_dragging = true;
                    self.drag_start_cursor = self.cursor_pos;
                    if let Ok(pos) = self.window.outer_position() {
                        self.drag_start_window = pos;
                    }
                    self.anim.trigger_bounce();
                    self.need_redraw = true;
                }
                ElementState::Released => {
                    self.is_dragging = false;
                }
            }
        }
    }

    fn handle_hover(&mut self, entered: bool) {
        self.anim.is_hovering = entered;
        if entered {
            self.anim.breath_timer = 0.0;
            self.last_activity = Instant::now();
        }
        self.need_redraw = true;
    }

    fn update(&mut self) {
        if self.last_update.elapsed() >= Duration::from_secs(UPDATE_INTERVAL_SECS) {
            self.monitor.update();
            
            let idle_secs = self.last_activity.elapsed().as_secs();
            self.monitor.set_idle(idle_secs);
            
            let info = self.monitor.get_info();
            let new_emoji = EmojiState::from_system_info(&info);

            if new_emoji.scenario != self.current_emoji.scenario {
                self.current_emoji = new_emoji;
                self.need_redraw = true;
            }

            self.last_update = Instant::now();
        }

        let size = self.window.inner_size();
        let center = (size.width as f32 / 2.0, size.height as f32 / 2.0);
        self.anim.update(self.cursor_pos, center);
        
        if self.anim.is_bouncing || self.anim.is_hovering || self.is_dragging {
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
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {}", e);
            std::process::exit(1);
        }
    };

    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("退出", true, None);
    if let Err(e) = tray_menu.append(&quit_item) {
        eprintln!("Failed to add quit menu item: {:?}", e);
    }

    let mut icon_data = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_data.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    }
    let icon = Icon::from_rgba(icon_data, 16, 16).unwrap_or_else(|e| {
        eprintln!("Failed to create icon: {:?}", e);
        std::process::exit(1);
    });

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Deskemoji")
        .with_icon(icon)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to create tray icon: {:?}", e);
            std::process::exit(1);
        });

    let menu_channel = MenuEvent::receiver();

    let position = event_loop
        .primary_monitor()
        .map(|monitor| {
            let screen_size = monitor.size();
            PhysicalPosition::new(
                (screen_size.width - WINDOW_SIZE - WINDOW_MARGIN_RIGHT as u32) as i32,
                (screen_size.height - WINDOW_SIZE - WINDOW_MARGIN_BOTTOM as u32) as i32,
            )
        })
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not detect primary monitor, using default position");
            PhysicalPosition::new(DEFAULT_POSITION.0, DEFAULT_POSITION.1)
        });

    let window_size = PhysicalSize::new(WINDOW_SIZE, WINDOW_SIZE);

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
            .unwrap_or_else(|e| {
                eprintln!("Failed to create window: {:?}", e);
                std::process::exit(1);
            }),
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
    }).unwrap_or_else(|e| {
        eprintln!("Event loop error: {:?}", e);
        std::process::exit(1);
    });
}
