use crate::monitor::SystemInfo;

#[derive(Debug, Clone)]
pub struct EmojiState {
    pub emoji: char,
    pub scenario: &'static str,
}

impl EmojiState {
    pub fn from_system_info(info: &SystemInfo) -> Self {
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
