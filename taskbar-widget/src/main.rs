mod taskbar;
mod win32;

use std::env;
use std::mem::size_of;
use std::panic::{self, PanicHookInfo};
use std::process;
use std::sync::{Mutex, OnceLock};

use taskbar::{AppState, DebugLoopConfig, probe_taskbar};
use taskbar_widget::agent_state::{self, AgentState, HookSummary};
use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreatePen, CreateSolidBrush, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE,
            DT_VCENTER, DeleteObject, DrawTextW, Ellipse, EndPaint, FillRect, InvalidateRect,
            PAINTSTRUCT, PS_NULL, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GetClientRect, GetMessageW, IDC_ARROW, KillTimer, LoadCursorW, MSG, PostQuitMessage,
            RegisterClassW, SW_SHOW, SetCursor, SetTimer, ShowWindow, TranslateMessage,
            WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WM_NCDESTROY, WM_PAINT,
            WM_SETCURSOR, WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
    core::{Error, Result, w},
};

const WINDOW_CLASS_NAME: windows::core::PCWSTR = w!("TaskbarWidgetWindow");
const WINDOW_TITLE: windows::core::PCWSTR = w!("Taskbar Widget");
const WINDOW_WIDTH: i32 = 160;
const WINDOW_HEIGHT: i32 = 48;
const HOOK_STATE_TIMER_ID: usize = 1;
const HOOK_STATE_TIMER_MS: u32 = 1_000;

#[derive(Clone)]
struct PaintState {
    summary: HookSummary,
}

struct PaintStyle {
    label: String,
    background: COLORREF,
    foreground: COLORREF,
    indicator: COLORREF,
    stale_indicator: COLORREF,
}

static PAINT_STATE: OnceLock<Mutex<PaintState>> = OnceLock::new();
static LAST_RUNTIME_STAGE: OnceLock<Mutex<String>> = OnceLock::new();

fn main() -> Result<()> {
    win32::init_runtime_log();
    install_panic_hook();
    set_runtime_stage("bootstrap_start");
    win32::debug_log("phase 1 bootstrap start");
    runtime_log(&format!(
        "startup pid={} state_file={} runtime_log_enabled={}",
        process::id(),
        agent_state::state_file_path().display(),
        win32::runtime_log_enabled()
    ));
    win32::enable_per_monitor_dpi_awareness();
    let debug_config = DebugLoopConfig::from_env();
    taskbar::log_debug_config(&debug_config);
    runtime_log(&format!(
        "config parent={} anchor={} coord_mode={} style_mode={} refresh_mode={} layered={}",
        debug_config.parent_strategy.as_str(),
        debug_config.anchor_strategy.as_str(),
        debug_config.coordinate_mode.as_str(),
        debug_config.style_mode.as_str(),
        debug_config.refresh_mode.as_str(),
        debug_config.layered_mode.as_str()
    ));

    set_runtime_stage("register_window_class");
    let hmodule = unsafe { GetModuleHandleW(None) }?;
    let hinstance = HINSTANCE(hmodule.0);
    register_window_class(hinstance)?;
    set_runtime_stage("probe_taskbar");
    let probe = probe_taskbar(&debug_config);
    taskbar::log_probe(&probe);

    set_runtime_stage("create_window");
    let hwnd = unsafe {
        CreateWindowExW(
            window_ex_style(),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            window_style(),
            0,
            0,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            HWND::default(),
            None,
            hinstance,
            None,
        )
    }?;

    set_runtime_stage("attach_to_taskbar");
    let attach = taskbar::attach_to_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_attach(&attach);
    set_runtime_stage("position_in_taskbar");
    let layout = taskbar::position_in_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_layout(&layout);
    let state = AppState::from_runtime(hwnd, &probe, &attach, &layout);
    taskbar::log_state(&state);
    runtime_log(&format!(
        "window initialized hwnd={} pid={} host_parent={} current_parent={} module_rect={} style_mode={} layered={} refresh_mode={}",
        win32::format_hwnd(hwnd),
        process::id(),
        win32::format_hwnd(probe.host_parent),
        win32::format_hwnd(attach.current_parent),
        win32::format_rect(&layout.module_rect),
        debug_config.style_mode.as_str(),
        debug_config.layered_mode.as_str(),
        debug_config.refresh_mode.as_str()
    ));
    set_runtime_stage("initialize_paint_state");
    initialize_paint_state();
    unsafe {
        let timer = SetTimer(hwnd, HOOK_STATE_TIMER_ID, HOOK_STATE_TIMER_MS, None);
        if timer == 0 {
            win32::debug_log(&format!(
                "[hook-state] SetTimer failed: last_error={}",
                win32::last_error_code()
            ));
            runtime_log(&format!(
                "SetTimer failed hwnd={} last_error={}",
                win32::format_hwnd(hwnd),
                win32::last_error_code()
            ));
        } else {
            runtime_log(&format!(
                "SetTimer armed hwnd={} timer_id={} interval_ms={}",
                win32::format_hwnd(hwnd),
                HOOK_STATE_TIMER_ID,
                HOOK_STATE_TIMER_MS
            ));
        }
    }

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
    set_runtime_stage("refresh_visibility");
    taskbar::refresh_visibility(hwnd, &layout, &debug_config);
    win32::log_window_dpi("main_window", hwnd);
    win32::debug_log("window created and shown");
    runtime_log(&format!(
        "window shown hwnd={} dpi={} awareness={}",
        win32::format_hwnd(hwnd),
        win32::window_dpi(hwnd),
        win32::window_dpi_awareness(hwnd)
    ));
    taskbar::write_diagnostics(
        env::var_os("TASKBAR_MVP_DIAG_FILE"),
        hwnd,
        &debug_config,
        &probe,
        &attach,
        &layout,
    );

    set_runtime_stage("message_loop");
    message_loop()
}

fn register_window_class(hinstance: HINSTANCE) -> Result<()> {
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: hinstance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        lpszClassName: WINDOW_CLASS_NAME,
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        win32::debug_log(&format!(
            "RegisterClassW failed: last_error={}",
            win32::last_error_code()
        ));
        return Err(Error::from_win32());
    }

    win32::debug_log(&format!(
        "registered window class (size={} atom={atom})",
        size_of::<WNDCLASSW>()
    ));
    Ok(())
}

fn message_loop() -> Result<()> {
    let mut message = MSG::default();
    runtime_log("message loop start");

    loop {
        let status = unsafe { GetMessageW(&mut message, None, 0, 0) };
        match status.0 {
            -1 => {
                win32::debug_log(&format!(
                    "GetMessageW failed: last_error={}",
                    win32::last_error_code()
                ));
                runtime_log(&format!(
                    "GetMessageW failed last_error={}",
                    win32::last_error_code()
                ));
                return Err(Error::from_win32());
            }
            0 => break,
            _ => unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            },
        }
    }

    win32::debug_log("message loop exited cleanly");
    runtime_log("message loop exited cleanly");
    Ok(())
}

fn window_style() -> WINDOW_STYLE {
    WINDOW_STYLE(WS_POPUP.0)
}

fn window_ex_style() -> WINDOW_EX_STYLE {
    WS_EX_TOOLWINDOW
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_PAINT => {
            set_runtime_stage("wm_paint");
            paint_window(hwnd)
        }
        WM_TIMER => {
            if wparam.0 == HOOK_STATE_TIMER_ID {
                set_runtime_stage("wm_timer");
                poll_hook_state(hwnd);
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CLOSE => {
            set_runtime_stage("wm_close");
            runtime_log(&format!(
                "WM_CLOSE received hwnd={}",
                win32::format_hwnd(hwnd)
            ));
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_SETCURSOR => {
            unsafe {
                let _ = SetCursor(LoadCursorW(None, IDC_ARROW).unwrap_or_default());
            }
            LRESULT(1)
        }
        WM_DESTROY => {
            set_runtime_stage("wm_destroy");
            win32::debug_log("WM_DESTROY received");
            runtime_log(&format!(
                "WM_DESTROY received hwnd={}",
                win32::format_hwnd(hwnd)
            ));
            unsafe {
                let _ = KillTimer(hwnd, HOOK_STATE_TIMER_ID);
            }
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_NCDESTROY => {
            set_runtime_stage("wm_ncdestroy");
            runtime_log(&format!(
                "WM_NCDESTROY received hwnd={}",
                win32::format_hwnd(hwnd)
            ));
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn initialize_paint_state() {
    let result = agent_state::load_state_for_display_diagnostic();
    let summary = result.state.global_summary.clone();
    runtime_log(&format!(
        "initialize_paint_state load_status={} path={} summary={}",
        result.outcome.as_str(),
        result.path.display(),
        format_summary(&summary)
    ));
    let _ = PAINT_STATE.set(Mutex::new(PaintState { summary }));
}

fn poll_hook_state(hwnd: HWND) {
    let result = agent_state::load_state_for_display_diagnostic();
    let next = result.state.global_summary.clone();
    runtime_log(&format!(
        "WM_TIMER tick load_status={} path={} summary={}",
        result.outcome.as_str(),
        result.path.display(),
        format_summary(&next)
    ));
    let Some(lock) = PAINT_STATE.get() else {
        return;
    };
    let Ok(mut current) = lock.lock() else {
        return;
    };

    if display_summary_changed(&current.summary, &next) {
        win32::debug_log(&format!(
            "[hook-state] summary changed state={} active={} stale={}",
            next.state.as_str(),
            next.active_task_count,
            next.stale_task_count
        ));
        runtime_log(&format!(
            "summary changed old={} new={} invalidate_requested=true",
            format_summary(&current.summary),
            format_summary(&next)
        ));
        current.summary = next;
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    } else {
        runtime_log(&format!("summary unchanged {}", format_summary(&next)));
    }
}

fn paint_window(hwnd: HWND) -> LRESULT {
    let mut paint = PAINTSTRUCT::default();
    let mut client_rect = RECT::default();
    let summary = PAINT_STATE
        .get()
        .and_then(|state| state.lock().ok().map(|state| state.summary.clone()))
        .unwrap_or_else(|| {
            agent_state::HookMonitorState::default_at(agent_state::now_ms()).global_summary
        });
    let style = paint_style(&summary);
    let mut text = win32::wide_text(&style.label);
    runtime_log(&format!(
        "WM_PAINT enter hwnd={} label={}",
        win32::format_hwnd(hwnd),
        style.label
    ));

    unsafe {
        let hdc = BeginPaint(hwnd, &mut paint);
        let _ = GetClientRect(hwnd, &mut client_rect);

        let background_brush = CreateSolidBrush(style.background);
        let _ = FillRect(hdc, &client_rect, background_brush);

        // Keep the layout intentionally simple: a bright status light on the left
        // and a short label on the right, so the widget still reads at taskbar size.
        let indicator_rect = RECT {
            left: client_rect.left + 12,
            top: client_rect.top + 11,
            right: client_rect.left + 34,
            bottom: client_rect.top + 33,
        };
        let mut text_rect = RECT {
            left: indicator_rect.right + 10,
            top: client_rect.top,
            right: client_rect.right - 12,
            bottom: client_rect.bottom,
        };

        let indicator_brush = CreateSolidBrush(style.indicator);
        let null_pen = CreatePen(PS_NULL, 0, COLORREF(0));
        let previous_brush = SelectObject(hdc, indicator_brush);
        let previous_pen = SelectObject(hdc, null_pen);
        let _ = Ellipse(
            hdc,
            indicator_rect.left,
            indicator_rect.top,
            indicator_rect.right,
            indicator_rect.bottom,
        );

        if summary.has_stale {
            let stale_rect = RECT {
                left: client_rect.right - 14,
                top: client_rect.top + 8,
                right: client_rect.right - 8,
                bottom: client_rect.top + 14,
            };
            let stale_brush = CreateSolidBrush(style.stale_indicator);
            let _ = FillRect(hdc, &stale_rect, stale_brush);
            let _ = DeleteObject(stale_brush);
        }

        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, style.foreground);
        let _ = DrawTextW(
            hdc,
            &mut text,
            &mut text_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );

        let _ = SelectObject(hdc, previous_pen);
        let _ = SelectObject(hdc, previous_brush);
        let _ = DeleteObject(null_pen);
        let _ = DeleteObject(indicator_brush);
        let _ = DeleteObject(background_brush);
        let _ = EndPaint(hwnd, &paint);
    }

    runtime_log(&format!(
        "WM_PAINT exit hwnd={} label={}",
        win32::format_hwnd(hwnd),
        style.label
    ));
    LRESULT(0)
}

fn paint_style(summary: &HookSummary) -> PaintStyle {
    let suffix = if summary.active_task_count > 0 {
        format!(" {}", summary.active_task_count)
    } else {
        String::new()
    };

    let (state_label, indicator, background) = match summary.state {
        AgentState::Idle => ("IDLE", rgb(126, 138, 150), rgb(32, 37, 44)),
        AgentState::Done => ("DONE", rgb(92, 220, 128), rgb(27, 68, 44)),
        AgentState::Working => ("RUN", rgb(82, 180, 255), rgb(26, 60, 101)),
        AgentState::Waiting => ("WAIT", rgb(255, 205, 74), rgb(103, 74, 23)),
        AgentState::Error => ("ERR", rgb(255, 120, 104), rgb(108, 37, 32)),
    };

    let stale = if summary.has_stale { " !" } else { "" };

    PaintStyle {
        label: format!("{state_label}{suffix}{stale}"),
        background,
        foreground: rgb(244, 244, 244),
        indicator,
        stale_indicator: rgb(255, 205, 74),
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16))
}

fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        runtime_log(&format!(
            "panic last_stage={} message={}",
            last_runtime_stage(),
            panic_message(info)
        ));
    }));
}

fn panic_message(info: &PanicHookInfo<'_>) -> String {
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn set_runtime_stage(stage: &str) {
    let lock = LAST_RUNTIME_STAGE.get_or_init(|| Mutex::new(String::new()));
    if let Ok(mut current) = lock.lock() {
        *current = stage.to_string();
    }
}

fn last_runtime_stage() -> String {
    LAST_RUNTIME_STAGE
        .get()
        .and_then(|lock| lock.lock().ok().map(|stage| stage.clone()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn runtime_log(message: &str) {
    win32::runtime_debug_log(&format!("{} {message}", win32::LIVE_DEBUG_PREFIX));
}

fn format_summary(summary: &HookSummary) -> String {
    format!(
        "state={} active={} stale={} stale_count={} top_task={}",
        summary.state.as_str(),
        summary.active_task_count,
        summary.has_stale,
        summary.stale_task_count,
        summary
            .highest_priority_task
            .clone()
            .unwrap_or_else(|| "<none>".to_string())
    )
}

fn display_summary_changed(current: &HookSummary, next: &HookSummary) -> bool {
    current.state != next.state
        || current.active_task_count != next.active_task_count
        || current.has_stale != next.has_stale
        || current.stale_task_count != next.stale_task_count
        || current.highest_priority_task != next.highest_priority_task
}
