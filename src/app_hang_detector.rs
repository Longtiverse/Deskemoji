use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsHungAppWindow};

/// 检测前台窗口是否进入"未响应"状态
pub struct AppHangDetector;

impl AppHangDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn is_hung(&self) -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return false;
            }
            IsHungAppWindow(hwnd).as_bool()
        }
    }
}

impl Default for AppHangDetector {
    fn default() -> Self {
        Self::new()
    }
}
