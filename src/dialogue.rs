use crate::emoji_assets::EmojiId;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub mod llm_bridge;

#[derive(Debug, Clone)]
pub struct DialogueManager {
    presets: HashMap<EmojiId, Vec<String>>,
    current_line: Option<String>,
    show_until: Option<Instant>,
    cooldown_until: Instant,
    last_idle_trigger: Instant,
    history: VecDeque<String>,
    pub enabled: bool,
    pub duration_secs: u64,
    pub interval_secs: u64,
    pub cooldown_secs: u64,
}

impl DialogueManager {
    pub fn load_default() -> Self {
        let presets = Self::load_presets();
        Self {
            presets,
            current_line: None,
            show_until: None,
            cooldown_until: Instant::now(),
            last_idle_trigger: Instant::now(),
            history: VecDeque::with_capacity(32),
            enabled: true,
            duration_secs: 5,
            interval_secs: 60,
            cooldown_secs: 3,
        }
    }

    pub fn with_config(enabled: bool, duration_secs: u64, interval_secs: u64) -> Self {
        let mut mgr = Self::load_default();
        mgr.enabled = enabled;
        mgr.duration_secs = duration_secs.max(1);
        mgr.interval_secs = interval_secs;
        mgr
    }

    fn load_presets() -> HashMap<EmojiId, Vec<String>> {
        let path = Path::new("assets/dialogues/presets.json");
        if !path.exists() {
            return Self::builtin_presets();
        }
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<HashMap<String, Vec<String>>>(&text) {
                Ok(map) => {
                    let mut presets = HashMap::new();
                    for (key, lines) in map {
                        if let Some(id) = Self::parse_emoji_id(&key) {
                            presets.insert(id, lines);
                        }
                    }
                    presets
                }
                Err(_) => Self::builtin_presets(),
            },
            Err(_) => Self::builtin_presets(),
        }
    }

    fn parse_emoji_id(key: &str) -> Option<EmojiId> {
        match key.to_lowercase().as_str() {
            "happy" => Some(EmojiId::Happy),
            "sad" => Some(EmojiId::Sad),
            "angry" => Some(EmojiId::Angry),
            "sleepy" => Some(EmojiId::Sleepy),
            "thinking" => Some(EmojiId::Thinking),
            "hot" => Some(EmojiId::Hot),
            "mindblown" => Some(EmojiId::Mindblown),
            "goodnight" => Some(EmojiId::Goodnight),
            _ => None,
        }
    }

    fn builtin_presets() -> HashMap<EmojiId, Vec<String>> {
        let mut map = HashMap::new();
        map.insert(EmojiId::Happy, vec!["今天状态不错！".into(), "继续加油~".into()]);
        map.insert(EmojiId::Sad, vec!["程序卡住了...".into(), "好难过".into()]);
        map.insert(EmojiId::Angry, vec!["别点了！".into(), "我很忙的".into()]);
        map.insert(EmojiId::Sleepy, vec!["Zzz...".into(), "好困啊".into()]);
        map.insert(EmojiId::Thinking, vec!["让我想想...".into(), "这个有意思".into()]);
        map.insert(EmojiId::Hot, vec!["有点烫".into(), "风扇开大点".into()]);
        map.insert(EmojiId::Mindblown, vec!["内存炸了！".into(), "CPU 要冒烟了".into()]);
        map.insert(EmojiId::Goodnight, vec!["早点休息".into(), "晚安~".into()]);
        map
    }

    pub fn update(&mut self) -> Option<&str> {
        if !self.enabled {
            self.current_line = None;
            self.show_until = None;
            return None;
        }
        if let Some(deadline) = self.show_until {
            if Instant::now() >= deadline {
                self.current_line = None;
                self.show_until = None;
                return None;
            }
        }
        self.current_line.as_deref()
    }

    pub fn trigger_by_state(&mut self, emoji: EmojiId) {
        if !self.enabled || Instant::now() < self.cooldown_until {
            return;
        }
        if let Some(line) = self.pick_line(emoji) {
            self.show(line);
        }
    }

    pub fn trigger_by_click(&mut self, emoji: EmojiId, click_count: u32) {
        if !self.enabled {
            return;
        }
        // Special wake-up lines for sleepy/goodnight -> angry
        let line = if emoji == EmojiId::Angry && click_count > 1 {
            Some("还点！？".into())
        } else if emoji == EmojiId::Angry {
            Some("干嘛吵醒我！".into())
        } else {
            self.pick_line(emoji)
        };
        if let Some(l) = line {
            self.show(l);
        }
    }

    pub fn trigger_idle(&mut self, emoji: EmojiId) -> bool {
        if !self.enabled {
            return false;
        }
        let now = Instant::now();
        if now.duration_since(self.last_idle_trigger) < Duration::from_secs(self.interval_secs) {
            return false;
        }
        self.last_idle_trigger = now;
        if let Some(line) = self.pick_line(emoji) {
            self.show(line);
            true
        } else {
            false
        }
    }

    fn pick_line(&mut self, emoji: EmojiId) -> Option<String> {
        let lines: Vec<String> = self.presets.get(&emoji)?.clone();
        if lines.is_empty() {
            return None;
        }
        // Simple round-robin with dedup against recent history
        for line in lines.iter().cycle().take(lines.len() * 2) {
            if !self.history.contains(line) {
                self.push_history(line.clone());
                return Some(line.clone());
            }
        }
        self.history.clear();
        let first = lines.first()?.clone();
        self.push_history(first.clone());
        Some(first)
    }

    fn push_history(&mut self, line: String) {
        if self.history.len() >= self.history.capacity() {
            self.history.pop_front();
        }
        self.history.push_back(line);
    }

    fn show(&mut self, line: String) {
        self.current_line = Some(line);
        self.show_until = Some(Instant::now() + Duration::from_secs(self.duration_secs));
        self.cooldown_until = Instant::now() + Duration::from_secs(self.cooldown_secs);
    }

    pub fn is_showing(&self) -> bool {
        self.show_until.map(|d| Instant::now() < d).unwrap_or(false)
    }

    pub fn dismiss(&mut self) {
        self.current_line = None;
        self.show_until = None;
    }
}
