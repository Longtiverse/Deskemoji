use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auto_mode: bool,
    pub startup: bool,
    pub update_interval_secs: u64,
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub idle_threshold_secs: u64,
    pub opacity: f32,
    pub window_size: u32,
    // 新增：状态系统相关阈值
    pub hot_cpu_threshold: f32,
    pub hot_memory_threshold: f32,
    pub mindblown_cpu_threshold: f32,
    pub mindblown_memory_threshold: f32,
    pub state_switch_interval_secs: u64,
    pub angry_base_duration_secs: u64,
    pub angry_click_extend_secs: u64,
    pub angry_click_max_duration_secs: u64,
    // 新增：对话气泡配置
    pub dialogue_enabled: bool,
    pub dialogue_duration_secs: u64,
    pub dialogue_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_mode: true,
            startup: false,
            update_interval_secs: 2,
            cpu_threshold: 80.0,
            memory_threshold: 90.0,
            idle_threshold_secs: 300,
            opacity: 1.0,
            window_size: 120,
            hot_cpu_threshold: 70.0,
            hot_memory_threshold: 70.0,
            mindblown_cpu_threshold: 90.0,
            mindblown_memory_threshold: 90.0,
            state_switch_interval_secs: 5,
            angry_base_duration_secs: 5,
            angry_click_extend_secs: 2,
            angry_click_max_duration_secs: 15,
            dialogue_enabled: true,
            dialogue_duration_secs: 5,
            dialogue_interval_secs: 60,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    fn config_path() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        exe_path
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .join("config.json")
    }
}
