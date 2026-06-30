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

pub fn debug_log(message: &str) {
    println!("[taskbar-mvp] {message}");
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
