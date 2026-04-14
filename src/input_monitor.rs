use std::collections::VecDeque;
use std::time::{Duration, Instant};

const WINDOW_DURATION: Duration = Duration::from_secs(10);
const MOUSE_MOVE_DEBOUNCE: Duration = Duration::from_millis(100);

/// 每分钟输入事件数的阈值档位
pub const THINKING_MIN_CPM: u32 = 30;
pub const THINKING_MAX_CPM: u32 = 150;
pub const THINKING_VARIATION_MAX: f32 = 0.40; // 40%
pub const ANGRY_CPM_THRESHOLD: u32 = 250;
pub const ANGRY_BURST_THRESHOLD: u32 = 50; // 10 秒内

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLevel {
    /// 低活跃或中高活跃但波动大 → Happy
    Normal,
    /// 持续稳定输入 → Thinking
    Stable,
    /// 高频狂暴输入 → Angry
    High,
}

/// 基于滑动窗口的输入频率监控器
pub struct InputMonitor {
    events: VecDeque<Instant>,
    last_mouse_move: Option<Instant>,
}

impl InputMonitor {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            last_mouse_move: None,
        }
    }

    /// 记录一次键盘按下或鼠标点击
    pub fn record_key_or_click(&mut self) {
        self.events.push_back(Instant::now());
        self.trim_old();
    }

    /// 记录鼠标移动（带防抖）
    pub fn record_mouse_move(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_mouse_move {
            if now.duration_since(last) < MOUSE_MOVE_DEBOUNCE {
                return;
            }
        }
        self.last_mouse_move = Some(now);
        self.events.push_back(now);
        self.trim_old();
    }

    fn trim_old(&mut self) {
        let cutoff = Instant::now() - WINDOW_DURATION;
        while let Some(front) = self.events.front() {
            if *front < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// 当前 10 秒窗口内的总事件数
    pub fn current_count(&mut self) -> u32 {
        self.trim_old();
        self.events.len() as u32
    }

    /// 估算每分钟事件数（CPM）
    pub fn cpm(&mut self) -> u32 {
        let count = self.current_count();
        // 10 秒窗口 * 6 = 1 分钟
        count * 6
    }

    /// 计算波动系数（标准差 / 均值），基于 1 秒分桶
    fn variation_ratio(&mut self) -> f32 {
        self.trim_old();
        if self.events.len() < 10 {
            return 1.0; // 数据不足，视为高波动
        }

        let now = Instant::now();
        let mut buckets = [0u32; 10];
        for t in &self.events {
            let age = now.duration_since(*t).as_secs() as usize;
            if age < 10 {
                buckets[9 - age] += 1;
            }
        }

        let mean = buckets.iter().sum::<u32>() as f32 / buckets.len() as f32;
        if mean <= 0.1 {
            return 1.0;
        }

        let variance: f32 = buckets
            .iter()
            .map(|v| {
                let diff = *v as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / buckets.len() as f32;

        variance.sqrt() / mean
    }

    pub fn current_level(&mut self) -> InputLevel {
        let cpm = self.cpm();
        let count = self.current_count();

        if cpm > ANGRY_CPM_THRESHOLD || count > ANGRY_BURST_THRESHOLD {
            return InputLevel::High;
        }

        if cpm >= THINKING_MIN_CPM
            && cpm <= THINKING_MAX_CPM
            && self.variation_ratio() <= THINKING_VARIATION_MAX
        {
            return InputLevel::Stable;
        }

        InputLevel::Normal
    }
}

impl Default for InputMonitor {
    fn default() -> Self {
        Self::new()
    }
}
