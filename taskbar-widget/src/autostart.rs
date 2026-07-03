use std::env;

use crate::win32;
use taskbar_widget::app_config::AppConfig;
use windows::{
    Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE,
        REG_SAM_FLAGS, REG_SZ, RRF_RT_REG_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteValueW,
        RegGetValueW, RegOpenKeyExW, RegSetValueExW,
    },
    core::{PCWSTR, PWSTR, Result, w},
};

const RUN_KEY_PATH: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE_NAME: &str = "CcTrafficLight";

pub fn sync_config_flag(config: &mut AppConfig) {
    if let Ok(enabled) = is_enabled() {
        config.general.autostart_enabled = enabled;
    }
}

pub fn is_enabled() -> Result<bool> {
    let Some(key) = open_run_key(KEY_QUERY_VALUE) else {
        return Ok(false);
    };
    let _guard = RegistryKey(key);

    let value_name = wide_null(RUN_VALUE_NAME);
    let mut value_type = REG_SZ;
    let mut size = 0u32;
    let status = unsafe {
        RegGetValueW(
            key,
            PCWSTR::null(),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            Some(&mut value_type),
            None,
            Some(&mut size),
        )
    };

    if status.is_err() {
        return Ok(false);
    }

    Ok(size > 2)
}

pub fn set_enabled(enabled: bool) -> Result<()> {
    if enabled {
        let key = create_run_key()?;
        let _guard = RegistryKey(key);
        let command = quoted_command_line();
        let bytes = wide_null(&command);
        unsafe {
            RegSetValueExW(
                key,
                PCWSTR(wide_null(RUN_VALUE_NAME).as_ptr()),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    bytes.as_ptr().cast::<u8>(),
                    bytes.len() * std::mem::size_of::<u16>(),
                )),
            )
            .ok()
        }
    } else {
        let Some(key) = open_run_key(KEY_SET_VALUE) else {
            return Ok(());
        };
        let _guard = RegistryKey(key);
        let value_name = wide_null(RUN_VALUE_NAME);
        unsafe { RegDeleteValueW(key, PCWSTR(value_name.as_ptr())).ok() }
    }
}

fn quoted_command_line() -> String {
    let path = env::current_exe().unwrap_or_else(|_| "taskbar-widget.exe".into());
    format!("\"{}\"", path.display())
}

fn create_run_key() -> Result<HKEY> {
    let mut key = HKEY::default();
    unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY_PATH,
            0,
            PWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()?;
    }
    Ok(key)
}

fn open_run_key(access: REG_SAM_FLAGS) -> Option<HKEY> {
    let mut key = HKEY::default();
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, 0, access, &mut key) };
    if status.is_ok() { Some(key) } else { None }
}

fn wide_null(value: &str) -> Vec<u16> {
    let mut wide = win32::wide_text(value);
    wide.push(0);
    wide
}

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}
