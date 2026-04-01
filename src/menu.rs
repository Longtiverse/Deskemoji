use tray_icon::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem, MenuEvent};

pub struct AppMenu {
    pub menu: Menu,
    pub auto_mode_item: MenuItem,
    pub startup_item: MenuItem,
    pub settings_item: MenuItem,
    pub quit_item: MenuItem,
    manual_emojis: Vec<(MenuItem, String)>, // (item, emoji)
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    ManualSelect(String),
    ToggleAutoMode,
    ToggleStartup,
    OpenSettings,
    Quit,
}

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

impl AppMenu {
    pub fn new() -> Self {
        let menu = Menu::new();
        
        // 手动选择子菜单
        let manual_menu = Submenu::new("手动选择", true);
        let mut manual_emojis = Vec::new();
        for (emoji, name) in EMOJI_OPTIONS {
            let item = MenuItem::new(format!("{} {}", emoji, name), true, None);
            manual_menu.append(&item).unwrap();
            manual_emojis.push((item, emoji.to_string()));
        }
        
        menu.append(&manual_menu).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        
        let auto_mode_item = MenuItem::new("自动模式 ✓", true, None);
        menu.append(&auto_mode_item).unwrap();
        
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        
        let settings_item = MenuItem::new("设置", true, None);
        let startup_item = MenuItem::new("开机启动", true, None);
        menu.append(&settings_item).unwrap();
        menu.append(&startup_item).unwrap();
        
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        
        let quit_item = MenuItem::new("退出", true, None);
        menu.append(&quit_item).unwrap();
        
        Self {
            menu,
            auto_mode_item,
            startup_item,
            settings_item,
            quit_item,
            manual_emojis,
        }
    }
    
    pub fn update_auto_mode(&self, enabled: bool) {
        self.auto_mode_item.set_text(if enabled { "自动模式 ✓" } else { "自动模式" });
    }
    
    pub fn update_startup(&self, enabled: bool) {
        self.startup_item.set_text(if enabled { "开机启动 ✓" } else { "开机启动" });
    }
    
    pub fn handle_event(&self, event: &MenuEvent) -> Option<MenuAction> {
        // 检查退出
        if event.id == self.quit_item.id() {
            return Some(MenuAction::Quit);
        }
        
        // 检查自动模式
        if event.id == self.auto_mode_item.id() {
            return Some(MenuAction::ToggleAutoMode);
        }
        
        // 检查开机启动
        if event.id == self.startup_item.id() {
            return Some(MenuAction::ToggleStartup);
        }
        
        // 检查设置
        if event.id == self.settings_item.id() {
            return Some(MenuAction::OpenSettings);
        }
        
        // 检查手动表情
        for (item, emoji) in &self.manual_emojis {
            if event.id == item.id() {
                return Some(MenuAction::ManualSelect(emoji.clone()));
            }
        }
        
        None
    }
}
