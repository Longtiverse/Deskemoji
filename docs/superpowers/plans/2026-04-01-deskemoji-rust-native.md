# Deskemoji Rust 原生实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 使用纯 Rust 创建轻量级单文件桌面 emoji 小程序

**Architecture:** 使用 winit 创建透明窗口，softbuffer 软件渲染，sysinfo 监控系统状态

**Tech Stack:** Rust, winit, softbuffer, sysinfo, tray-icon, chrono

---

## 文件结构

```
Deskemoji/
├── Cargo.toml
├── src/
│   ├── main.rs           # 主入口，窗口管理和事件循环
│   ├── monitor.rs        # 系统监控模块
│   ├── emoji.rs          # emoji 状态管理
│   └── renderer.rs       # 渲染模块
└── docs/
```

## 依赖

```toml
[dependencies]
winit = "0.29"
softbuffer = "0.4"
sysinfo = "0.30"
chrono = "0.4"
tray-icon = "0.11"
```

---

### Task 1: 初始化 Rust 项目

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs` (基础框架)

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "deskemoji"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.29"
softbuffer = "0.4"
sysinfo = "0.30"
chrono = "0.4"
tray-icon = "0.11"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: 创建基础 main.rs**

```rust
mod monitor;
mod emoji;
mod renderer;

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Deskemoji")
        .with_inner_size(PhysicalSize::new(120u32, 120u32))
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top(true)
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    // 渲染逻辑
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
```

- [ ] **Step 3: 验证编译**

```bash
cargo check
```

Expected: 编译成功

---

### Task 2: 实现系统监控模块

**Files:**
- Create: `src/monitor.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 monitor.rs**

```rust
use sysinfo::System;
use chrono::{Local, Timelike};

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hour: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub is_idle: bool,
}

pub struct Monitor {
    sys: System,
    idle_seconds: u64,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            idle_seconds: 0,
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
    }

    pub fn set_idle(&mut self, seconds: u64) {
        self.idle_seconds = seconds;
    }

    pub fn get_info(&self) -> SystemInfo {
        let now = Local::now();
        let hour = now.hour();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let memory_usage = (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0) as f32;

        SystemInfo {
            hour,
            cpu_usage,
            memory_usage,
            is_idle: self.idle_seconds >= 300, // 5分钟
        }
    }
}
```

- [ ] **Step 2: 验证编译**

```bash
cargo check
```

Expected: 编译成功

---

### Task 3: 实现 emoji 状态管理

**Files:**
- Create: `src/emoji.rs`

- [ ] **Step 1: 创建 emoji.rs**

```rust
use crate::monitor::SystemInfo;

#[derive(Debug, Clone)]
pub struct EmojiState {
    pub emoji: char,
    pub scenario: &'static str,
}

impl EmojiState {
    pub fn from_system_info(info: &SystemInfo) -> Self {
        // 优先级：系统状态 > 空闲 > 时间
        if info.cpu_usage > 80.0 {
            Self {
                emoji: '🥵',
                scenario: "high_cpu",
            }
        } else if info.memory_usage > 90.0 {
            Self {
                emoji: '💀',
                scenario: "high_memory",
            }
        } else if info.is_idle {
            Self {
                emoji: '😴',
                scenario: "idle",
            }
        } else {
            match info.hour {
                6..=9 => Self { emoji: '🙂', scenario: "morning" },
                10..=11 => Self { emoji: '😊', scenario: "late_morning" },
                12..=13 => Self { emoji: '🤗', scenario: "noon" },
                14..=17 => Self { emoji: '😌', scenario: "afternoon" },
                18..=22 => Self { emoji: '🌙', scenario: "evening" },
                _ => Self { emoji: '😪', scenario: "night" },
            }
        }
    }
}
```

- [ ] **Step 2: 验证编译**

```bash
cargo check
```

Expected: 编译成功

---

### Task 4: 实现渲染模块

**Files:**
- Create: `src/renderer.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 renderer.rs**

```rust
use softbuffer::{Context, Surface};
use winit::window::Window;
use std::num::NonZeroU32;
use std::rc::Rc;

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

    pub fn render(&mut self, window: &Window, emoji: char) {
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

        let mut buffer = self.surface.buffer_mut().unwrap();

        // 透明背景
        for pixel in buffer.iter_mut() {
            *pixel = 0x00000000; // ARGB: 全透明
        }

        // 绘制 emoji 背景圆形（浅黄色）
        let center_x = size.width as i32 / 2;
        let center_y = size.height as i32 / 2;
        let radius = 45i32;

        for y in 0..size.height as i32 {
            for x in 0..size.width as i32 {
                let dx = x - center_x;
                let dy = y - center_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= radius * radius {
                    let idx = (y as u32 * size.width + x as u32) as usize;
                    // 浅黄色: #FFD93D -> ARGB: 0xFFFFD93D
                    buffer[idx] = 0xFFFFD93D;
                }
            }
        }

        // 绘制表情特征（简化版）
        Self::draw_face(&mut buffer, size.width, size.height, emoji);

        buffer.present().unwrap();
    }

    fn draw_face(buffer: &mut [u32], width: u32, height: u32, emoji: char) {
        let cx = width as i32 / 2;
        let cy = height as i32 / 2;

        // 根据 emoji 类型绘制不同的表情
        match emoji {
            '🙂' | '😊' => {
                // 微笑：两个眼睛 + 弯弯的嘴
                Self::draw_circle(buffer, width, cx - 15, cy - 10, 5, 0xFF000000); // 左眼
                Self::draw_circle(buffer, width, cx + 15, cy - 10, 5, 0xFF000000); // 右眼
                Self::draw_arc(buffer, width, cx, cy + 5, 20, 0xFF000000); // 嘴巴
            }
            '😴' | '😪' => {
                // 闭眼 + Zzz
                Self::draw_line(buffer, width, cx - 20, cy - 10, cx - 10, cy - 10, 0xFF000000);
                Self::draw_line(buffer, width, cx + 10, cy - 10, cx + 20, cy - 10, 0xFF000000);
                Self::draw_circle(buffer, width, cx, cy + 10, 8, 0xFF000000);
            }
            '🥵' => {
                // 热：眼睛 + 流汗
                Self::draw_circle(buffer, width, cx - 15, cy - 10, 5, 0xFF000000);
                Self::draw_circle(buffer, width, cx + 15, cy - 10, 5, 0xFF000000);
                Self::draw_circle(buffer, width, cx, cy + 10, 10, 0xFF000000);
                Self::draw_circle(buffer, width, cx + 25, cy - 5, 4, 0xFF4FC3F7); // 汗滴
            }
            _ => {
                // 默认：简单笑脸
                Self::draw_circle(buffer, width, cx - 15, cy - 10, 5, 0xFF000000);
                Self::draw_circle(buffer, width, cx + 15, cy - 10, 5, 0xFF000000);
                Self::draw_arc(buffer, width, cx, cy + 5, 20, 0xFF000000);
            }
        }
    }

    fn draw_circle(buffer: &mut [u32], width: u32, cx: i32, cy: i32, r: i32, color: u32) {
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r * r {
                    if x >= 0 && y >= 0 {
                        let idx = (y as u32 * width + x as u32) as usize;
                        if idx < buffer.len() {
                            buffer[idx] = color;
                        }
                    }
                }
            }
        }
    }

    fn draw_arc(buffer: &mut [u32], width: u32, cx: i32, cy: i32, r: i32, color: u32) {
        // 简化：画一条弧线（微笑的嘴巴）
        for angle in 20..160 {
            let rad = angle as f64 * std::f64::consts::PI / 180.0;
            let x = cx + (r as f64 * rad.cos()) as i32;
            let y = cy + (r as f64 * rad.sin()) as i32 / 2;
            if x >= 0 && y >= 0 {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
            }
        }
    }

    fn draw_line(buffer: &mut [u32], width: u32, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && y >= 0 {
                let idx = (y as u32 * width + x as u32) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
            }
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }
}
```

- [ ] **Step 2: 更新 main.rs 集成渲染**

完整 main.rs：

```rust
mod monitor;
mod emoji;
mod renderer;

use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use monitor::Monitor;
use emoji::EmojiState;
use renderer::Renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Deskemoji")
            .with_inner_size(PhysicalSize::new(120u32, 120u32))
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .build(&event_loop)
            .unwrap(),
    );

    let mut renderer = Renderer::new(window.clone());
    let mut monitor = Monitor::new();
    let mut current_emoji = EmojiState::from_system_info(&monitor.get_info());
    let mut last_update = Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    renderer.render(&window, current_emoji.emoji);
                }
                _ => {}
            },
            Event::AboutToWait => {
                // 每秒更新一次
                if last_update.elapsed() >= Duration::from_secs(1) {
                    monitor.update();
                    let info = monitor.get_info();
                    let new_emoji = EmojiState::from_system_info(&info);
                    
                    if new_emoji.scenario != current_emoji.scenario {
                        current_emoji = new_emoji;
                    }
                    
                    last_update = Instant::now();
                }
                
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
```

- [ ] **Step 3: 验证编译**

```bash
cargo check
```

Expected: 编译成功

---

### Task 5: 添加系统托盘

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新 main.rs 添加托盘**

```rust
mod monitor;
mod emoji;
mod renderer;

use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, Icon,
};

use monitor::Monitor;
use emoji::EmojiState;
use renderer::Renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // 创建托盘菜单
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("退出", true, None);
    tray_menu.append(&quit_item).unwrap();

    // 创建托盘图标（简单的 emoji 图标）
    let icon = Icon::from_rgba(vec![0; 16 * 16 * 4], 16, 16).unwrap_or_default();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Deskemoji")
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();

    // 创建窗口
    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Deskemoji")
            .with_inner_size(PhysicalSize::new(120u32, 120u32))
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .build(&event_loop)
            .unwrap(),
    );

    let mut renderer = Renderer::new(window.clone());
    let mut monitor = Monitor::new();
    let mut current_emoji = EmojiState::from_system_info(&monitor.get_info());
    let mut last_update = Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        // 检查托盘菜单事件
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                elwt.exit();
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    renderer.render(&window, current_emoji.emoji);
                }
                _ => {}
            },
            Event::AboutToWait => {
                if last_update.elapsed() >= Duration::from_secs(1) {
                    monitor.update();
                    let info = monitor.get_info();
                    let new_emoji = EmojiState::from_system_info(&info);
                    
                    if new_emoji.scenario != current_emoji.scenario {
                        current_emoji = new_emoji;
                    }
                    
                    last_update = Instant::now();
                }
                
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
```

- [ ] **Step 2: 验证编译**

```bash
cargo check
```

Expected: 编译成功

---

### Task 6: 编译发布版本

**Files:**
- Verify: `Cargo.toml`

- [ ] **Step 1: 编译发布版本**

```bash
cargo build --release
```

Expected: 编译成功

- [ ] **Step 2: 检查文件大小**

```bash
dir target\release\deskemoji.exe
```

Expected: < 5MB

- [ ] **Step 3: 运行测试**

```bash
target\release\deskemoji.exe
```

Expected:
- 透明窗口显示 emoji
- 表情随时间/状态变化
- 系统托盘图标存在
- 右键托盘可退出

---

## 验证清单

- [ ] 程序启动后显示 emoji（黄色圆形 + 表情）
- [ ] 透明背景，无边框
- [ ] 表情随时间变化（早/中/晚）
- [ ] CPU 高时显示 🥵
- [ ] 空闲时显示 😴
- [ ] 系统托盘图标
- [ ] 右键托盘可退出
- [ ] 单文件 exe < 5MB
