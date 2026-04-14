use chrono::{Local, Timelike};
use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Normal,
    Hot,
    Mindblown,
}

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
    resource_state: ResourceState,
    hot_cpu_threshold: f32,
    hot_memory_threshold: f32,
    mindblown_cpu_threshold: f32,
    mindblown_memory_threshold: f32,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            idle_seconds: 0,
            resource_state: ResourceState::Normal,
            hot_cpu_threshold: 70.0,
            hot_memory_threshold: 70.0,
            mindblown_cpu_threshold: 90.0,
            mindblown_memory_threshold: 90.0,
        }
    }

    pub fn with_thresholds(
        mut self,
        hot_cpu: f32,
        hot_mem: f32,
        mindblown_cpu: f32,
        mindblown_mem: f32,
    ) -> Self {
        self.hot_cpu_threshold = hot_cpu;
        self.hot_memory_threshold = hot_mem;
        self.mindblown_cpu_threshold = mindblown_cpu;
        self.mindblown_memory_threshold = mindblown_mem;
        self
    }

    pub fn update(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.update_resource_state();
    }

    fn update_resource_state(&mut self) {
        let cpu = self.sys.global_cpu_info().cpu_usage();
        let memory =
            (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0) as f32;

        let in_hot = cpu > self.hot_cpu_threshold || memory > self.hot_memory_threshold;
        let in_mindblown =
            cpu > self.mindblown_cpu_threshold || memory > self.mindblown_memory_threshold;

        self.resource_state = match self.resource_state {
            ResourceState::Mindblown => {
                if in_mindblown {
                    ResourceState::Mindblown
                } else if in_hot {
                    ResourceState::Hot
                } else {
                    ResourceState::Normal
                }
            }
            ResourceState::Hot => {
                if in_mindblown {
                    ResourceState::Mindblown
                } else if in_hot {
                    ResourceState::Hot
                } else {
                    ResourceState::Normal
                }
            }
            ResourceState::Normal => {
                if in_mindblown {
                    ResourceState::Mindblown
                } else if in_hot {
                    ResourceState::Hot
                } else {
                    ResourceState::Normal
                }
            }
        };
    }

    pub fn set_idle(&mut self, seconds: u64) {
        self.idle_seconds = seconds;
    }

    pub fn get_info(&self) -> SystemInfo {
        let now = Local::now();
        let hour = now.hour();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let memory_usage =
            (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0) as f32;

        SystemInfo {
            hour,
            cpu_usage,
            memory_usage,
            is_idle: self.idle_seconds >= 300,
        }
    }

    pub fn get_resource_state(&self) -> ResourceState {
        self.resource_state
    }

}
