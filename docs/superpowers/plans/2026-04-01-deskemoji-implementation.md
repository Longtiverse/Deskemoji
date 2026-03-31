# Deskemoji 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建一个 Windows 桌面小程序，显示动态小黄脸表情，根据时间、系统状态等情景变化

**Architecture:** 使用 Tauri 框架，Rust 负责系统监控和窗口管理，Web 前端负责 UI 渲染和动画

**Tech Stack:** Tauri, Rust, HTML/CSS/JavaScript, sysinfo (Rust库)

---

## 文件结构

```
Deskemoji/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # Tauri 主入口
│   │   ├── monitor.rs        # 系统监控模块
│   │   └── state.rs          # 应用状态管理
│   ├── Cargo.toml
│   ├── tauri.conf.json       # Tauri 配置
│   └── icons/                # 应用图标
├── src/
│   ├── index.html            # 主页面
│   ├── styles.css            # 样式
│   ├── main.js               # 前端逻辑
│   └── emoji.js              # 表情管理
├── package.json
└── docs/
```

---

### Task 1: 初始化 Tauri 项目

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src/index.html`

- [ ] **Step 1: 创建 package.json**

```json
{
  "name": "deskemoji",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "tauri": "tauri",
    "dev": "tauri dev",
    "build": "tauri build"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.5.0"
  }
}
```

- [ ] **Step 2: 创建 Cargo.toml**

```toml
[package]
name = "deskemoji"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sysinfo = "0.30"
chrono = "0.4"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 3: 创建 tauri.conf.json**

```json
{
  "build": {
    "beforeDevCommand": "",
    "beforeBuildCommand": "",
    "devPath": "../src",
    "distDir": "../src"
  },
  "package": {
    "productName": "Deskemoji",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.deskemoji.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "label": "main",
        "title": "Deskemoji",
        "width": 120,
        "height": 120,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true
      }
    ]
  }
}
```

- [ ] **Step 4: 创建 main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

mod monitor;
mod state;

use state::AppState;

#[tauri::command]
fn get_system_info(state: State<'_, Mutex<AppState>>) -> String {
    let mut app_state = state.lock().unwrap();
    app_state.update();
    serde_json::to_string(&app_state.current_info).unwrap()
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![get_system_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: 创建 index.html**

```html
<!DOCTYPE html>
<html lang="zh">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Deskemoji</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="emoji-container">
    <span id="emoji">😊</span>
  </div>
  <script src="main.js"></script>
</body>
</html>
```

- [ ] **Step 6: 安装依赖并验证**

```bash
npm install
cd src-tauri && cargo check
```

Expected: 无错误

---

### Task 2: 实现系统监控模块

**Files:**
- Create: `src-tauri/src/monitor.rs`
- Create: `src-tauri/src/state.rs`

- [ ] **Step 1: 创建 monitor.rs**

```rust
use sysinfo::System;
use chrono::{Local, Timelike};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemInfo {
    pub hour: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub is_idle: bool,
    pub battery_level: Option<f32>,
    pub is_charging: Option<bool>,
}

pub struct Monitor {
    sys: System,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn get_info(&mut self, idle_minutes: u32) -> SystemInfo {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();

        let now = Local::now();
        let hour = now.hour();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let memory_usage = (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0) as f32;

        SystemInfo {
            hour,
            cpu_usage,
            memory_usage,
            is_idle: idle_minutes >= 5,
            battery_level: None,
            is_charging: None,
        }
    }
}
```

- [ ] **Step 2: 创建 state.rs**

```rust
use crate::monitor::{Monitor, SystemInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct EmojiState {
    pub emoji: String,
    pub scenario: String,
    pub description: String,
}

pub struct AppState {
    pub monitor: Monitor,
    pub current_info: SystemInfo,
    pub current_emoji: EmojiState,
    pub idle_minutes: u32,
}

impl AppState {
    pub fn new() -> Self {
        let mut monitor = Monitor::new();
        let info = monitor.get_info(0);
        let emoji = Self::determine_emoji(&info);

        Self {
            monitor,
            current_info: info,
            current_emoji: emoji,
            idle_minutes: 0,
        }
    }

    pub fn update(&mut self) {
        self.current_info = self.monitor.get_info(self.idle_minutes);
        self.current_emoji = Self::determine_emoji(&self.current_info);
    }

    pub fn set_idle(&mut self, minutes: u32) {
        self.idle_minutes = minutes;
    }

    fn determine_emoji(info: &SystemInfo) -> EmojiState {
        // 优先级：系统状态 > 空闲 > 时间
        if info.cpu_usage > 80.0 {
            EmojiState {
                emoji: "🥵".to_string(),
                scenario: "high_cpu".to_string(),
                description: "CPU使用率过高".to_string(),
            }
        } else if info.memory_usage > 90.0 {
            EmojiState {
                emoji: "💀".to_string(),
                scenario: "high_memory".to_string(),
                description: "内存使用率过高".to_string(),
            }
        } else if info.is_idle {
            EmojiState {
                emoji: "😴".to_string(),
                scenario: "idle".to_string(),
                description: "休息中".to_string(),
            }
        } else {
            // 根据时间
            match info.hour {
                6..=9 => EmojiState {
                    emoji: "🙂".to_string(),
                    scenario: "morning".to_string(),
                    description: "早上好".to_string(),
                },
                10..=11 => EmojiState {
                    emoji: "😊".to_string(),
                    scenario: "late_morning".to_string(),
                    description: "上午好".to_string(),
                },
                12..=13 => EmojiState {
                    emoji: "🤗".to_string(),
                    scenario: "noon".to_string(),
                    description: "中午好".to_string(),
                },
                14..=17 => EmojiState {
                    emoji: "😌".to_string(),
                    scenario: "afternoon".to_string(),
                    description: "下午好".to_string(),
                },
                18..=22 => EmojiState {
                    emoji: "🌙".to_string(),
                    scenario: "evening".to_string(),
                    description: "晚上好".to_string(),
                },
                _ => EmojiState {
                    emoji: "😪".to_string(),
                    scenario: "night".to_string(),
                    description: "该休息了".to_string(),
                },
            }
        }
    }
}
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

---

### Task 3: 实现前端界面和动画

**Files:**
- Create: `src/styles.css`
- Create: `src/main.js`
- Create: `src/emoji.js`

- [ ] **Step 1: 创建 styles.css**

```css
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background: transparent;
  overflow: hidden;
  user-select: none;
  -webkit-app-region: drag;
}

#emoji-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
}

#emoji-container:active {
  cursor: grabbing;
}

#emoji {
  font-size: 64px;
  animation: float 3s ease-in-out infinite;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.2));
  transition: transform 0.3s ease;
}

#emoji:hover {
  transform: scale(1.1);
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-8px);
  }
}

@keyframes bounce {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

@keyframes shake {
  0%, 100% {
    transform: translateX(0);
  }
  25% {
    transform: translateX(-3px);
  }
  75% {
    transform: translateX(3px);
  }
}

.bounce {
  animation: bounce 0.5s ease-in-out infinite;
}

.shake {
  animation: shake 0.3s ease-in-out infinite;
}
```

- [ ] **Step 2: 创建 emoji.js**

```javascript
export class EmojiManager {
  constructor(elementId) {
    this.element = document.getElementById(elementId);
    this.currentEmoji = '';
    this.currentAnimation = 'float';
  }

  setEmoji(emoji, animation = 'float') {
    if (this.currentEmoji === emoji) return;
    
    this.currentEmoji = emoji;
    this.element.textContent = emoji;
    
    // 移除所有动画类
    this.element.classList.remove('bounce', 'shake');
    
    // 添加新动画
    if (animation !== 'float') {
      this.element.classList.add(animation);
    }
  }

  setAnimation(animation) {
    this.element.classList.remove('bounce', 'shake');
    if (animation !== 'float') {
      this.element.classList.add(animation);
    }
  }
}
```

- [ ] **Step 3: 创建 main.js**

```javascript
import { EmojiManager } from './emoji.js';

const emojiManager = new EmojiManager('emoji');
let lastState = null;

async function updateEmoji() {
  try {
    const { invoke } = window.__TAURI__.tauri;
    const stateJson = await invoke('get_system_info');
    const state = JSON.parse(stateJson);
    
    if (!lastState || lastState.scenario !== state.scenario) {
      const animation = getAnimation(state.scenario);
      emojiManager.setEmoji(state.emoji, animation);
      lastState = state;
    }
  } catch (error) {
    console.error('Failed to update emoji:', error);
  }
}

function getAnimation(scenario) {
  switch (scenario) {
    case 'high_cpu':
    case 'high_memory':
      return 'shake';
    case 'notification':
      return 'bounce';
    default:
      return 'float';
  }
}

// 每秒更新一次
setInterval(updateEmoji, 1000);

// 初始更新
updateEmoji();
```

- [ ] **Step 4: 验证前端资源**

确认文件存在：
```bash
ls src/
```

Expected: index.html, styles.css, main.js, emoji.js

---

### Task 4: 添加右键菜单和系统托盘

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 更新 Cargo.toml 添加 tray 功能**

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open", "system-tray"] }
```

- [ ] **Step 2: 更新 main.rs 添加系统托盘**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

mod monitor;
mod state;

use state::AppState;

#[tauri::command]
fn get_system_info(state: State<'_, Mutex<AppState>>) -> String {
    let mut app_state = state.lock().unwrap();
    app_state.update();
    serde_json::to_string(&app_state.current_info).unwrap()
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let settings = CustomMenuItem::new("settings".to_string(), "设置");
    let tray_menu = SystemTrayMenu::new()
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "settings" => {
                    // TODO: 打开设置窗口
                }
                _ => {}
            },
            _ => {}
        })
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![get_system_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功

---

### Task 5: 添加窗口拖拽和右键菜单

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`
- Modify: `src/main.js`

- [ ] **Step 1: 更新 index.html 添加右键菜单**

```html
<!DOCTYPE html>
<html lang="zh">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Deskemoji</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="emoji-container">
    <span id="emoji">😊</span>
  </div>
  
  <div id="context-menu" class="hidden">
    <div class="menu-item" data-action="settings">设置</div>
    <div class="menu-item" data-action="about">关于</div>
    <div class="menu-divider"></div>
    <div class="menu-item" data-action="quit">退出</div>
  </div>
  
  <script src="main.js"></script>
</body>
</html>
```

- [ ] **Step 2: 更新 styles.css 添加菜单样式**

```css
/* 添加到现有样式后面 */

.hidden {
  display: none !important;
}

#context-menu {
  position: fixed;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 8px;
  padding: 4px 0;
  min-width: 120px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  backdrop-filter: blur(10px);
}

.menu-item {
  padding: 8px 16px;
  cursor: pointer;
  color: #fff;
  font-size: 13px;
  transition: background 0.15s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 4px 0;
}
```

- [ ] **Step 3: 更新 main.js 添加右键菜单逻辑**

```javascript
import { EmojiManager } from './emoji.js';

const emojiManager = new EmojiManager('emoji');
const contextMenu = document.getElementById('context-menu');
let lastState = null;

// 右键菜单
document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  contextMenu.style.left = e.pageX + 'px';
  contextMenu.style.top = e.pageY + 'px';
  contextMenu.classList.remove('hidden');
});

document.addEventListener('click', () => {
  contextMenu.classList.add('hidden');
});

document.querySelectorAll('.menu-item').forEach(item => {
  item.addEventListener('click', async () => {
    const action = item.dataset.action;
    const { invoke } = window.__TAURI__.tauri;
    
    switch (action) {
      case 'quit':
        const { app } = window.__TAURI__;
        await app.exit();
        break;
      case 'settings':
        // TODO: 打开设置窗口
        break;
      case 'about':
        alert('Deskemoji v0.1.0\n桌面动态表情助手');
        break;
    }
  });
});

// 更新emoji逻辑...
// (保持之前的 updateEmoji 函数)
```

- [ ] **Step 4: 验证功能**

运行应用测试右键菜单：
```bash
npm run dev
```

Expected: 右键显示菜单，点击退出可关闭应用

---

### Task 6: 打包测试

**Files:**
- Verify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 构建应用**

```bash
npm run build
```

Expected: 构建成功

- [ ] **Step 2: 检查打包体积**

```bash
du -sh src-tauri/target/release/bundle/msi/*.msi
```

Expected: < 15MB

- [ ] **Step 3: 运行打包后的应用**

双击运行生成的 exe 文件

Expected:
- 透明窗口显示emoji
- 表情随时间变化
- 可拖拽移动
- 右键菜单可用
- 系统托盘图标存在

---

## 验证清单

- [ ] 程序启动后显示emoji
- [ ] 透明背景，无边框
- [ ] 表情随时间变化（早/中/晚）
- [ ] CPU高时显示🥵
- [ ] 空闲时显示😴
- [ ] 可拖拽移动位置
- [ ] 右键菜单可退出
- [ ] 系统托盘图标
- [ ] 打包体积 < 15MB
