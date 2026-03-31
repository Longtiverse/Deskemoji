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
            is_idle: self.idle_seconds >= 300,
        }
    }
}
