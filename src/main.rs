#![windows_subsystem = "windows"]

mod config;
mod monitor;
mod settings;

use std::rc::Rc;
use std::time::{Duration, Instant};
use tray_icon::{Icon, TrayIconBuilder};
use winit::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowLevel},
};

use config::Config;
use deskemoji::app_hang_detector::AppHangDetector;
use deskemoji::dialogue::DialogueManager;
use deskemoji::emoji_assets::{EmojiAssets, EmojiId};
use deskemoji::input_monitor::{InputLevel, InputMonitor};
use deskemoji::renderer::{
    compute_closed_eye_response, compute_idle_transform, compute_state_accent_transform,
    BubbleOverlay, EmojiTransform, GazeDirection, RenderLayer, Renderer,
};
use monitor::{Monitor, ResourceState};
use settings::Settings;

const WINDOW_WIDTH: u32 = 220;
const WINDOW_HEIGHT: u32 = 220;
const GAZE_CENTER_X_RATIO: f32 = 0.50;
const GAZE_CENTER_Y_RATIO: f32 = 0.66;
const WINDOW_MARGIN_RIGHT: i32 = 20;
const WINDOW_MARGIN_BOTTOM: i32 = 60;
const BOUNCE_SPEED: f32 = 5.0;
const BOUNCE_DECAY: f32 = 0.65;
const TRANSITION_DURATION: Duration = Duration::from_millis(180);

const APPROVED_EMOJIS: [EmojiId; 8] = EmojiId::all();

struct App {
    window: Rc<Window>,
    renderer: Renderer,
    assets: EmojiAssets,
    monitor: Monitor,
    input_monitor: InputMonitor,
    hang_detector: AppHangDetector,
    config: Config,
    current_emoji: EmojiId,
    manual_emoji: Option<EmojiId>,
    previous_emoji: Option<EmojiId>,
    transition_started_at: Option<Instant>,
    auto_mode: bool,
    bounce_y: f32,
    bounce_vel: f32,
    is_bouncing: bool,
    is_hovering: bool,
    animation_started_at: Instant,
    last_update: Instant,
    last_activity: Instant,
    last_click_time: Instant,
    last_emoji_change: Instant,
    wake_up_angry_until: Option<Instant>,
    wake_up_click_count: u32,
    dialogue: DialogueManager,
}

impl App {
    fn new(window: Rc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let assets = EmojiAssets::load_default()
            .unwrap_or_else(|err| panic!("failed to load emoji assets: {err}"));
        let config = Config::load();
        let monitor = Monitor::new().with_thresholds(
            config.hot_cpu_threshold,
            config.hot_memory_threshold,
            config.mindblown_cpu_threshold,
            config.mindblown_memory_threshold,
        );
        let auto_mode = config.auto_mode;
        let dialogue = DialogueManager::with_config(
            config.dialogue_enabled,
            config.dialogue_duration_secs,
            config.dialogue_interval_secs,
        );

        let mut app = Self {
            window,
            renderer,
            assets,
            monitor,
            input_monitor: InputMonitor::new(),
            hang_detector: AppHangDetector::new(),
            config,
            current_emoji: EmojiId::Happy,
            manual_emoji: None,
            previous_emoji: None,
            transition_started_at: None,
            auto_mode,
            bounce_y: 0.0,
            bounce_vel: 0.0,
            is_bouncing: false,
            is_hovering: false,
            animation_started_at: Instant::now(),
            last_update: Instant::now(),
            last_activity: Instant::now(),
            last_click_time: Instant::now(),
            last_emoji_change: Instant::now() - Duration::from_secs(10),
            wake_up_angry_until: None,
            wake_up_click_count: 0,
            dialogue,
        };

        if app.auto_mode {
            let initial = app.resolve_auto_emoji();
            app.set_current_emoji(initial, false, true);
            app.dialogue.trigger_by_state(initial);
        }

        app
    }

    fn resolve_auto_emoji(&mut self) -> EmojiId {
        // 被吵醒的 Angry 优先级最高（强制状态）
        if let Some(until) = self.wake_up_angry_until {
            if Instant::now() < until {
                return EmojiId::Angry;
            }
            self.wake_up_angry_until = None;
            self.wake_up_click_count = 0;
        }

        // 优先级 1: Hot / Mindblown
        match self.monitor.get_resource_state() {
            ResourceState::Mindblown => return EmojiId::Mindblown,
            ResourceState::Hot => return EmojiId::Hot,
            _ => {}
        }

        // 优先级 2: Sad (前台窗口未响应)
        if self.hang_detector.is_hung() {
            return EmojiId::Sad;
        }

        // 优先级 3: Angry (高频输入)
        let input_level = self.input_monitor.current_level();
        if input_level == InputLevel::High {
            return EmojiId::Angry;
        }

        let info = self.monitor.get_info();

        // 优先级 4: Sleepy (系统空闲)
        if info.is_idle {
            return EmojiId::Sleepy;
        }

        // 优先级 5: Goodnight (深夜时段)
        if info.hour >= 22 || info.hour <= 5 {
            return EmojiId::Goodnight;
        }

        // 优先级 6: Thinking / Happy
        match input_level {
            InputLevel::Stable => EmojiId::Thinking,
            InputLevel::Normal | InputLevel::High => EmojiId::Happy,
        }
    }

    fn set_current_emoji(&mut self, emoji: EmojiId, animate: bool, force: bool) {
        if emoji == self.current_emoji {
            return;
        }

        if !force {
            let min_interval = Duration::from_secs(self.config.state_switch_interval_secs);
            if self.last_emoji_change.elapsed() < min_interval {
                return;
            }
        }

        if animate {
            self.previous_emoji = Some(self.current_emoji);
            self.transition_started_at = Some(Instant::now());
        } else {
            self.previous_emoji = None;
            self.transition_started_at = None;
        }

        self.current_emoji = emoji;
        self.last_emoji_change = Instant::now();
        self.dialogue.trigger_by_state(emoji);
    }

    fn trigger_bounce(&mut self) {
        if !self.is_bouncing {
            self.is_bouncing = true;
            self.bounce_vel = -BOUNCE_SPEED;
        }
    }

    fn trigger_shake(&mut self, intensity: f32) {
        if !self.is_bouncing {
            self.is_bouncing = true;
            self.bounce_vel = -BOUNCE_SPEED * intensity;
        }
    }

    fn wake_up_emoji(&mut self) {
        self.wake_up_click_count += 1;
        let base = Duration::from_secs(self.config.angry_base_duration_secs);
        let extend = Duration::from_secs(self.config.angry_click_extend_secs)
            * self.wake_up_click_count.saturating_sub(1);
        let max_dur = Duration::from_secs(self.config.angry_click_max_duration_secs);
        let total = base + extend;
        let total = total.min(max_dur);

        self.wake_up_angry_until = Some(Instant::now() + total);

        let intensity = 1.0 + (self.wake_up_click_count as f32 * 0.3).min(1.5);
        self.trigger_shake(intensity);
        self.set_current_emoji(EmojiId::Angry, true, true);
        self.dialogue.trigger_by_click(EmojiId::Angry, self.wake_up_click_count);
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
    }

    fn update(&mut self) {
        self.update_animation();

        if self.last_update.elapsed() >= Duration::from_secs(self.config.update_interval_secs) {
            self.monitor.update();
            self.monitor
                .set_idle(self.last_activity.elapsed().as_secs());

            if self.auto_mode {
                let next_emoji = self.resolve_auto_emoji();
                self.set_current_emoji(next_emoji, true, false);
            }

            self.dialogue.trigger_idle(self.current_emoji);

            self.last_update = Instant::now();
        }
    }

    fn base_transform(&self) -> EmojiTransform {
        let mut idle = compute_idle_transform(self.animation_started_at.elapsed().as_secs_f32());
        let intensity = if self.is_hovering { 1.0 } else { 0.45 };
        idle.offset_y *= intensity;
        idle.scale = 1.0 + (idle.scale - 1.0) * intensity;

        let bounce_scale = if self.is_bouncing {
            (-self.bounce_y / 28.0).clamp(0.0, 1.0) * 0.05
        } else {
            0.0
        };

        let base = EmojiTransform::new(
            idle.scale + bounce_scale,
            idle.offset_x,
            idle.offset_y + self.bounce_y,
            self.config.opacity.clamp(0.0, 1.0),
        );

        base.combine(compute_state_accent_transform(
            self.current_emoji,
            self.animation_started_at.elapsed().as_secs_f32(),
        ))
    }

    fn transition_progress(&self) -> f32 {
        self.transition_started_at
            .map(|started| {
                (started.elapsed().as_secs_f32() / TRANSITION_DURATION.as_secs_f32())
                    .clamp(0.0, 1.0)
            })
            .unwrap_or(1.0)
    }

    fn finish_transition_if_needed(&mut self) {
        if self.transition_progress() >= 1.0 {
            self.previous_emoji = None;
            self.transition_started_at = None;
        }
    }

    fn render(&mut self) {
        let base = self
            .base_transform()
            .combine(self.closed_eye_response_transform());
        let progress = ease_out_cubic(self.transition_progress());
        let gaze = self.current_gaze_direction();

        // Compute bubble before building layers to avoid borrow issues
        let bubble = self.current_bubble_overlay();

        let mut layers = Vec::with_capacity(2);
        if let Some(previous) = self.previous_emoji {
            let previous_image = self.assets.get_variant(previous, gaze).unwrap();
            layers.push(RenderLayer {
                image: previous_image,
                transform: EmojiTransform::new(
                    base.scale * (1.0 - 0.04 * progress),
                    base.offset_x,
                    base.offset_y - progress * 2.0,
                    base.alpha * (1.0 - progress),
                ),
            });
        }

        let current_image = self.assets.get_variant(self.current_emoji, gaze).unwrap();
        let current_progress = if self.previous_emoji.is_some() {
            progress
        } else {
            1.0
        };
        layers.push(RenderLayer {
            image: current_image,
            transform: EmojiTransform::new(
                base.scale * (0.96 + 0.04 * current_progress),
                base.offset_x,
                base.offset_y + (1.0 - current_progress) * 2.0,
                base.alpha * current_progress,
            ),
        });

        self.renderer
            .render_layers(&self.window, &layers, bubble.as_ref());
        self.finish_transition_if_needed();
    }

    fn current_gaze_direction(&self) -> GazeDirection {
        let Some((center_x, center_y)) = self.window_gaze_center() else {
            return GazeDirection::Center;
        };

        let (cursor_x, cursor_y) = get_screen_cursor_pos();

        GazeDirection::from_pointer_delta(cursor_x - center_x, cursor_y - center_y)
    }

    fn current_bubble_overlay(&mut self) -> Option<BubbleOverlay> {
        if let Some(line) = self.dialogue.update() {
            return Some(BubbleOverlay {
                label: line.to_string(),
            });
        }

        let info = self.monitor.get_info();
        let elapsed = self.animation_started_at.elapsed().as_secs();
        BubbleOverlay::for_state(
            self.current_emoji,
            info.cpu_usage.round().clamp(0.0, 100.0) as u8,
            info.memory_usage.round().clamp(0.0, 100.0) as u8,
            elapsed,
        )
    }

    fn closed_eye_response_transform(&self) -> EmojiTransform {
        let Some((center_x, center_y)) = self.window_gaze_center() else {
            return EmojiTransform::identity();
        };

        let (cursor_x, cursor_y) = get_screen_cursor_pos();

        compute_closed_eye_response(
            self.current_emoji,
            cursor_x - center_x,
            cursor_y - center_y,
            self.animation_started_at.elapsed().as_secs_f32(),
        )
    }

    fn window_gaze_center(&self) -> Option<(f32, f32)> {
        let window_pos = self.window.outer_position().ok()?;
        let size = self.window.inner_size();
        Some((
            window_pos.x as f32 + size.width as f32 * GAZE_CENTER_X_RATIO,
            window_pos.y as f32 + size.height as f32 * GAZE_CENTER_Y_RATIO,
        ))
    }

    fn select_emoji(&mut self, emoji: EmojiId) {
        self.manual_emoji = Some(emoji);
        self.auto_mode = false;
        self.config.auto_mode = false;
        self.config.save();
        self.set_current_emoji(emoji, true, true);
        self.trigger_bounce();
    }

    fn toggle_auto(&mut self) {
        self.auto_mode = !self.auto_mode;
        if self.auto_mode {
            self.manual_emoji = None;
            self.monitor.update();
            self.monitor
                .set_idle(self.last_activity.elapsed().as_secs());
            let next_emoji = self.resolve_auto_emoji();
            self.set_current_emoji(next_emoji, true, true);
        }
        self.config.auto_mode = self.auto_mode;
        self.config.save();
        self.trigger_bounce();
    }

    fn toggle_startup(&mut self) {
        let next = !self.config.startup;
        Settings::set_startup(&mut self.config, next);
    }

    fn handle_menu_cmd(&mut self, cmd: i32) {
        match cmd {
            100 => self.toggle_auto(),
            200..=207 => {
                let index = (cmd - 200) as usize;
                if let Some(&emoji) = APPROVED_EMOJIS.get(index) {
                    self.select_emoji(emoji);
                }
            }
            300 => Settings::print_settings(&self.config),
            301 => self.toggle_startup(),
            999 => std::process::exit(0),
            _ => {}
        }
    }
}

fn ease_out_cubic(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn get_screen_cursor_pos() -> (f32, f32) {
    unsafe {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

        let mut pt = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut pt);
        (pt.x as f32, pt.y as f32)
    }
}

fn window_hwnd(window: &Window) -> Option<windows::Win32::Foundation::HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(windows::Win32::Foundation::HWND(handle.hwnd.get())),
        _ => None,
    }
}

fn show_popup_menu(
    owner: windows::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    auto_mode: bool,
    startup: bool,
) -> i32 {
    unsafe {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        let hmenu = CreatePopupMenu().unwrap();

        let auto_text = if auto_mode {
            "自动模式 ✓\0"
        } else {
            "自动模式\0"
        };
        let auto_w: Vec<u16> = auto_text.encode_utf16().collect();
        AppendMenuW(
            hmenu,
            MF_STRING,
            100,
            windows::core::PCWSTR(auto_w.as_ptr()),
        )
        .unwrap();

        AppendMenuW(hmenu, MF_SEPARATOR, 0, windows::core::PCWSTR::null()).unwrap();

        for (i, emoji_id) in APPROVED_EMOJIS.iter().enumerate() {
            let text = format!("{} {}\0", emoji_id.emoji_char(), emoji_id.label_zh());
            let w: Vec<u16> = text.encode_utf16().collect();
            AppendMenuW(
                hmenu,
                MF_STRING,
                200usize + i,
                windows::core::PCWSTR(w.as_ptr()),
            )
            .unwrap();
        }

        AppendMenuW(hmenu, MF_SEPARATOR, 0, windows::core::PCWSTR::null()).unwrap();

        let settings_w: Vec<u16> = "设置\0".encode_utf16().collect();
        AppendMenuW(
            hmenu,
            MF_STRING,
            300,
            windows::core::PCWSTR(settings_w.as_ptr()),
        )
        .unwrap();

        let startup_text = if startup {
            "开机启动 ✓\0"
        } else {
            "开机启动\0"
        };
        let startup_w: Vec<u16> = startup_text.encode_utf16().collect();
        AppendMenuW(
            hmenu,
            MF_STRING,
            301,
            windows::core::PCWSTR(startup_w.as_ptr()),
        )
        .unwrap();

        AppendMenuW(hmenu, MF_SEPARATOR, 0, windows::core::PCWSTR::null()).unwrap();

        let quit_w: Vec<u16> = "退出\0".encode_utf16().collect();
        AppendMenuW(
            hmenu,
            MF_STRING,
            999,
            windows::core::PCWSTR(quit_w.as_ptr()),
        )
        .unwrap();

        let _ = SetForegroundWindow(owner);
        let flags = TPM_RIGHTBUTTON.0 | TPM_RETURNCMD.0;
        let cmd = TrackPopupMenuEx(hmenu, flags, x, y, owner, None);
        let _ = PostMessageW(owner, WM_NULL, WPARAM(0), LPARAM(0));

        DestroyMenu(hmenu).ok();
        cmd.0
    }
}

fn main() {
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let event_loop = EventLoop::new().unwrap();

    let mut icon_data = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_data.extend_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    }
    let icon = Icon::from_rgba(icon_data, 16, 16).unwrap();

    let tray_menu = tray_icon::menu::Menu::new();

    let auto_item = tray_icon::menu::MenuItem::new("自动模式 ✓", true, None);
    tray_menu.append(&auto_item).unwrap();
    tray_menu
        .append(&tray_icon::menu::PredefinedMenuItem::separator())
        .unwrap();

    let manual_menu = tray_icon::menu::Submenu::new("手动选择", true);
    for emoji_id in APPROVED_EMOJIS {
        let text = format!("{} {}", emoji_id.emoji_char(), emoji_id.label_zh());
        let item = tray_icon::menu::MenuItem::new(text, true, None);
        manual_menu.append(&item).unwrap();
    }
    tray_menu.append(&manual_menu).unwrap();
    tray_menu
        .append(&tray_icon::menu::PredefinedMenuItem::separator())
        .unwrap();

    let settings_item = tray_icon::menu::MenuItem::new("设置", true, None);
    let startup_item = tray_icon::menu::MenuItem::new("开机启动", true, None);
    tray_menu.append(&settings_item).unwrap();
    tray_menu.append(&startup_item).unwrap();
    tray_menu
        .append(&tray_icon::menu::PredefinedMenuItem::separator())
        .unwrap();

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
        .map(|monitor| {
            let size = monitor.size();
            PhysicalPosition::new(
                (size.width - WINDOW_WIDTH - WINDOW_MARGIN_RIGHT as u32) as i32,
                (size.height - WINDOW_HEIGHT - WINDOW_MARGIN_BOTTOM as u32) as i32,
            )
        })
        .unwrap_or(PhysicalPosition::new(100, 100));

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Deskemoji")
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_position(position)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_skip_taskbar(true)
            .build(&event_loop)
            .unwrap(),
    );
    window.set_undecorated_shadow(false);

    let mut app = App::new(window.clone());
    app.render();

    let _ = event_loop.run(move |event, elwt| {
        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_item.id() {
                elwt.exit();
                return;
            }

            if menu_event.id == auto_item.id() {
                app.toggle_auto();
                auto_item.set_text(if app.auto_mode {
                    "自动模式 ✓"
                } else {
                    "自动模式"
                });
            } else if menu_event.id == settings_item.id() {
                Settings::print_settings(&app.config);
            } else if menu_event.id == startup_item.id() {
                app.toggle_startup();
            }
        }

        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(_) | DeviceEvent::Button { .. } => {
                    app.input_monitor.record_key_or_click();
                    app.last_activity = Instant::now();
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => app.render(),
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Left && state == ElementState::Pressed {
                        let now = Instant::now();
                        let click_interval = now.duration_since(app.last_click_time);
                        let is_double_click = click_interval < Duration::from_millis(300);
                        app.last_click_time = now;

                        if !is_double_click {
                            if [EmojiId::Sleepy, EmojiId::Goodnight].contains(&app.current_emoji) {
                                app.wake_up_emoji();
                            } else {
                                app.trigger_bounce();
                                let _ = app.window.drag_window();
                                app.last_activity = Instant::now();
                                app.dialogue.trigger_by_click(app.current_emoji, 0);
                            }
                        }
                    }

                    if button == MouseButton::Right && state == ElementState::Pressed {
                        let (screen_x, screen_y) = get_screen_cursor_pos();
                        if let Some(owner) = window_hwnd(app.window.as_ref()) {
                            let cmd = show_popup_menu(
                                owner,
                                screen_x as i32,
                                screen_y as i32,
                                app.auto_mode,
                                app.config.startup,
                            );
                            if cmd > 0 {
                                app.handle_menu_cmd(cmd);
                            }
                        }
                    }
                }
                WindowEvent::CursorEntered { .. } => {
                    app.is_hovering = true;
                }
                WindowEvent::CursorLeft { .. } => {
                    app.is_hovering = false;
                }
                WindowEvent::CursorMoved { .. } => {}
                _ => {}
            },
            Event::AboutToWait => {
                app.update();
                app.window.request_redraw();
                elwt.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(16),
                ));
            }
            _ => {}
        }
    });
}
