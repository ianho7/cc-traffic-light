use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use windows::Win32::{
    Foundation::{HWND, RECT, SetLastError, WIN32_ERROR},
    UI::{
        HiDpi::{
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            DPI_AWARENESS_PER_MONITOR_AWARE, DPI_AWARENESS_SYSTEM_AWARE,
            GetAwarenessFromDpiAwarenessContext, GetDpiForSystem, GetDpiForWindow,
            GetWindowDpiAwarenessContext, PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness,
            SetProcessDpiAwarenessContext,
        },
        WindowsAndMessaging::{GetWindowRect, SetProcessDPIAware},
    },
};

pub const LIVE_DEBUG_PREFIX: &str = "[DEBUG-LIVE-01]";
pub const RUNTIME_LOG_ENV: &str = "TASKBAR_MVP_RUNTIME_LOG_FILE";

static RUNTIME_LOG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static RUNTIME_LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn debug_log(message: &str) {
    println!("[taskbar-mvp] {message}");
}

pub fn init_runtime_log() {
    let Some(path) = runtime_log_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = OpenOptions::new().create(true).append(true).open(path);
    runtime_debug_log(&format!(
        "{LIVE_DEBUG_PREFIX} runtime log enabled path={}",
        path.display()
    ));
}

pub fn runtime_log_enabled() -> bool {
    runtime_log_path().is_some()
}

pub fn runtime_debug_log(message: &str) {
    debug_log(message);
    append_runtime_log(message);
}

pub fn enable_per_monitor_dpi_awareness() {
    let mode = unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_ok() {
            "per_monitor_v2"
        } else if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE).is_ok() {
            "per_monitor_v1"
        } else if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).is_ok() {
            "process_per_monitor"
        } else if SetProcessDPIAware().as_bool() {
            "system_dpi_aware"
        } else {
            "unaware_or_preconfigured"
        }
    };

    let dpi = system_dpi();
    debug_log(&format!(
        "[taskbar-loop] dpi awareness init mode={mode} system_dpi={dpi} scale={:.2}",
        dpi_scale(dpi)
    ));
}

pub fn last_error_code() -> u32 {
    unsafe { windows::Win32::Foundation::GetLastError().0 }
}

pub fn clear_last_error() {
    unsafe { SetLastError(WIN32_ERROR(0)) };
}

pub fn wide_text(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

pub fn format_hwnd(hwnd: HWND) -> String {
    format!("0x{:X}", hwnd.0 as usize)
}

pub fn system_dpi() -> u32 {
    let dpi = unsafe { GetDpiForSystem() };
    if dpi == 0 { 96 } else { dpi }
}

pub fn window_dpi(hwnd: HWND) -> u32 {
    if hwnd.0.is_null() {
        return system_dpi();
    }

    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi == 0 { system_dpi() } else { dpi }
}

pub fn dpi_scale(dpi: u32) -> f32 {
    dpi as f32 / 96.0
}

pub fn window_dpi_awareness(hwnd: HWND) -> &'static str {
    if hwnd.0.is_null() {
        return "invalid_hwnd";
    }

    let awareness = unsafe {
        let context = GetWindowDpiAwarenessContext(hwnd);
        GetAwarenessFromDpiAwarenessContext(context)
    };

    match awareness {
        DPI_AWARENESS_PER_MONITOR_AWARE => "per_monitor",
        DPI_AWARENESS_SYSTEM_AWARE => "system",
        _ => "unaware_or_unknown",
    }
}

pub fn log_window_dpi(label: &str, hwnd: HWND) {
    let dpi = window_dpi(hwnd);
    debug_log(&format!(
        "[taskbar-loop] {label} dpi={dpi} scale={:.2} awareness={}",
        dpi_scale(dpi),
        window_dpi_awareness(hwnd)
    ));
}

pub fn rect_for_window(hwnd: HWND) -> Option<RECT> {
    if hwnd.0.is_null() {
        return None;
    }

    let mut rect = RECT::default();
    let result = unsafe { GetWindowRect(hwnd, &mut rect) };
    if result.is_ok() { Some(rect) } else { None }
}

pub fn format_rect(rect: &RECT) -> String {
    format!("{},{},{},{}", rect.left, rect.top, rect.right, rect.bottom)
}

pub fn log_window(label: &str, hwnd: HWND) {
    match rect_for_window(hwnd) {
        Some(rect) => debug_log(&format!(
            "{label}: hwnd={} rect={}",
            format_hwnd(hwnd),
            format_rect(&rect)
        )),
        None => debug_log(&format!(
            "{label}: hwnd={} rect=<missing>",
            format_hwnd(hwnd)
        )),
    }
}

fn runtime_log_path() -> Option<&'static PathBuf> {
    RUNTIME_LOG_PATH
        .get_or_init(|| env::var_os(RUNTIME_LOG_ENV).map(PathBuf::from))
        .as_ref()
}

fn append_runtime_log(message: &str) {
    let Some(path) = runtime_log_path() else {
        return;
    };
    let lock = RUNTIME_LOG_LOCK.get_or_init(|| Mutex::new(()));
    let Ok(_guard) = lock.lock() else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let _ = writeln!(file, "{} {}", timestamp_ms(), message);
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
