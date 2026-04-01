use crate::config::Config;

#[derive(Debug, Clone)]
pub enum SettingField {
    AutoMode,
    Startup,
    UpdateInterval,
    CpuThreshold,
    MemoryThreshold,
    IdleThreshold,
    WindowSize,
}

pub struct Settings;

impl Settings {
    pub fn print_settings(config: &Config) {
        println!("=== Deskemoji 设置 ===");
        println!("1. 自动模式: {}", if config.auto_mode { "开" } else { "关" });
        println!("2. 开机启动: {}", if config.startup { "开" } else { "关" });
        println!("3. 更新频率: {}秒", config.update_interval_secs);
        println!("4. CPU阈值: {}%", config.cpu_threshold);
        println!("5. 内存阈值: {}%", config.memory_threshold);
        println!("6. 空闲时间: {}秒", config.idle_threshold_secs);
        println!("7. 窗口大小: {}px", config.window_size);
    }

    pub fn toggle_auto_mode(config: &mut Config) {
        config.auto_mode = !config.auto_mode;
        config.save();
    }

    pub fn toggle_startup(config: &mut Config) {
        config.startup = !config.startup;
        // TODO: 实际设置开机启动需要操作注册表
        config.save();
    }
}
