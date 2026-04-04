use crate::config::Config;

pub struct Settings;

impl Settings {
    pub fn show_settings_dialog(config: &Config) {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
            use windows::Win32::UI::WindowsAndMessaging::{
                MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND,
            };

            let settings_text = format!(
                "Deskemoji 当前设置\r\n\r\n\
                自动模式: {}\r\n\
                开机启动: {}\r\n\r\n\
                更新频率: {} 秒\r\n\
                CPU阈值: {}%\r\n\
                内存阈值: {}%\r\n\
                空闲时间: {} 秒\r\n\r\n\
                设置文件: config.json",
                if config.auto_mode { "开" } else { "关" },
                if config.startup { "开" } else { "关" },
                config.update_interval_secs,
                config.cpu_threshold,
                config.memory_threshold,
                config.idle_threshold_secs
            );

            let text_w: Vec<u16> = settings_text
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let title_w: Vec<u16> = "Deskemoji 设置"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            MessageBoxW(
                windows::Win32::Foundation::HWND(0),
                windows::core::PCWSTR(text_w.as_ptr()),
                windows::core::PCWSTR(title_w.as_ptr()),
                MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND,
            );
        }
    }

    pub fn print_settings(config: &Config) {
        Self::show_settings_dialog(config);
    }

    pub fn toggle_auto_mode(config: &mut Config) {
        config.auto_mode = !config.auto_mode;
        config.save();
    }

    pub fn toggle_startup(config: &mut Config) {
        config.startup = !config.startup;
        if config.startup {
            Self::enable_startup();
        } else {
            Self::disable_startup();
        }
        config.save();
    }

    fn enable_startup() {
        unsafe {
            use windows::Win32::System::Registry::*;

            if let Ok(exe_path) = std::env::current_exe() {
                let exe_path_str = exe_path.to_string_lossy();

                let reg_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
                    .encode_utf16()
                    .collect();
                let value_name: Vec<u16> = "Deskemoji\0".encode_utf16().collect();
                let exe_path_w: Vec<u16> = exe_path_str
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let mut hkey = HKEY(0);
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    windows::core::PCWSTR(reg_path.as_ptr()),
                    0,
                    KEY_SET_VALUE,
                    &mut hkey,
                )
                .is_ok()
                {
                    let data: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            exe_path_w.as_ptr() as *const u8,
                            exe_path_w.len() * 2,
                        )
                    };
                    let _ = RegSetValueExW(
                        hkey,
                        windows::core::PCWSTR(value_name.as_ptr()),
                        0,
                        REG_SZ,
                        Some(data),
                    );
                    let _ = RegCloseKey(hkey);
                }
            }
        }
    }

    fn disable_startup() {
        unsafe {
            use windows::Win32::System::Registry::*;

            let reg_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
                .encode_utf16()
                .collect();
            let value_name: Vec<u16> = "Deskemoji\0".encode_utf16().collect();

            let mut hkey = HKEY(0);
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR(reg_path.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            )
            .is_ok()
            {
                let _ = RegDeleteValueW(hkey, windows::core::PCWSTR(value_name.as_ptr()));
                let _ = RegCloseKey(hkey);
            }
        }
    }
}
