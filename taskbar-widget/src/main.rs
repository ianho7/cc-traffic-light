#![windows_subsystem = "windows"]

mod autostart;
mod settings_bridge;
mod settings_process;
mod settings_window;
mod taskbar;
mod tauri_settings_ipc;
mod tray_icon;
mod win32;

use std::env;
use std::mem::size_of;
use std::panic::{self, PanicHookInfo};
use std::process;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use taskbar::{AppState, DebugLoopConfig, probe_taskbar};
use taskbar_widget::{
    agent_state::{self},
    app_config, detector,
    runtime_contract::RuntimeContract,
    ui_state::{AppStatusSnapshot, WidgetMountState},
    widget_effects::{self, WidgetEffectsState},
    widget_render::{self, WidgetHotZone},
};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, ScreenToClient,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GetClientRect, GetMessageW, HTCLIENT, HTTRANSPARENT, IDC_ARROW, KillTimer,
            LoadCursorW, MSG, PostQuitMessage, RegisterClassW, SW_HIDE, SW_SHOW, SetCursor,
            SetTimer, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE,
            WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT,
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
const ANIMATION_TIMER_ID: usize = 2;
const HOOK_STATE_TIMER_MS: u32 = 1_000;
const ANIMATION_TIMER_MS: u32 = 80;
const WIDGET_RETRY_INTERVAL_MS: u64 = 5_000;

#[derive(Clone)]
struct PaintState {
    snapshot: AppStatusSnapshot,
    effects: WidgetEffectsState,
    hot_zones: Vec<WidgetHotZone>,
}

#[derive(Clone, Copy)]
struct WidgetRuntimeState {
    mount_state: WidgetMountState,
    last_attach_at: Option<u64>,
    last_retry_at: Option<u64>,
}

static PAINT_STATE: OnceLock<Mutex<PaintState>> = OnceLock::new();
static LAST_RUNTIME_STAGE: OnceLock<Mutex<String>> = OnceLock::new();
static APP_STATUS_SNAPSHOT: OnceLock<Mutex<AppStatusSnapshot>> = OnceLock::new();
static SETTINGS_HWND: OnceLock<Mutex<isize>> = OnceLock::new();
static DEBUG_CONFIG: OnceLock<DebugLoopConfig> = OnceLock::new();
static WIDGET_RUNTIME_STATE: OnceLock<Mutex<WidgetRuntimeState>> = OnceLock::new();

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
    let mut config_result = app_config::load_config_diagnostic();
    autostart::sync_config_flag(&mut config_result.config);
    let runtime_contract = RuntimeContract::v1_default();
    runtime_log(&format!(
        "config load_status={} path={} schema_version={} modules={} signals={}",
        config_result.outcome.as_str(),
        config_result.path.display(),
        config_result.config.schema_version,
        runtime_contract.module_names(),
        runtime_contract.signal_names()
    ));
    win32::enable_per_monitor_dpi_awareness();
    let debug_config = DebugLoopConfig::from_env();
    let _ = DEBUG_CONFIG.set(debug_config.clone());
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
    let _ = SETTINGS_HWND.set(Mutex::new(0));
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
    set_runtime_stage("create_settings_window");
    let settings_hwnd = settings_window::create_window(
        hinstance,
        AppStatusSnapshot::empty(),
        config_result.config.clone(),
    )?;
    settings_bridge::bind_main_window(hwnd);
    tauri_settings_ipc::ensure_server_started();
    store_settings_hwnd(settings_hwnd);

    set_runtime_stage("attach_to_taskbar");
    let attach = taskbar::attach_to_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_attach(&attach);
    set_runtime_stage("position_in_taskbar");
    let layout = taskbar::position_in_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_layout(&layout);
    initialize_widget_runtime_state(widget_mount_state_from_results(&attach, &layout));
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
    sync_settings_hosts();
    if let Err(error) = tray_icon::add_tray_icon(hwnd, &settings_bridge::current_config()) {
        runtime_log(&format!(
            "tray icon add failed; continuing without tray icon: {error}"
        ));
    }
    unsafe {
        let hook_timer = SetTimer(hwnd, HOOK_STATE_TIMER_ID, HOOK_STATE_TIMER_MS, None);
        if hook_timer == 0 {
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

        let animation_timer = SetTimer(hwnd, ANIMATION_TIMER_ID, ANIMATION_TIMER_MS, None);
        if animation_timer == 0 {
            runtime_log(&format!(
                "animation SetTimer failed hwnd={} last_error={}",
                win32::format_hwnd(hwnd),
                win32::last_error_code()
            ));
        }
    }

    sync_widget_visibility(hwnd, &layout, &debug_config);
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
    run_message_loop()
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
                attempt_widget_recovery(hwnd);
                poll_hook_state(hwnd);
                LRESULT(0)
            } else if wparam.0 == ANIMATION_TIMER_ID {
                tick_widget_animation(hwnd);
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_COMMAND => {
            if let Some(action) = tray_command_from_wparam(wparam) {
                if matches!(action, tray_icon::TrayAction::Exit) {
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                            hwnd,
                            WM_CLOSE,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                } else {
                    handle_tray_action(hwnd, action);
                }
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_NCHITTEST => handle_hit_test(hwnd, lparam),
        WM_LBUTTONUP => {
            handle_widget_click(hwnd, lparam);
            LRESULT(0)
        }
        tray_icon::TRAY_CALLBACK_MESSAGE => {
            if let Some(action) =
                tray_icon::handle_callback(hwnd, wparam, lparam, &settings_bridge::current_config())
            {
                handle_tray_action(hwnd, action);
            }
            LRESULT(0)
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
            tray_icon::remove_tray_icon(hwnd);
            settings_process::shutdown_managed_tauri_settings();
            runtime_log(&format!(
                "WM_DESTROY received hwnd={}",
                win32::format_hwnd(hwnd)
            ));
            unsafe {
                let _ = KillTimer(hwnd, HOOK_STATE_TIMER_ID);
                let _ = KillTimer(hwnd, ANIMATION_TIMER_ID);
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
    let config = settings_bridge::refresh_config_from_disk();
    let (mount_state, last_attach_at) = current_widget_runtime_state();
    let snapshot = detector::build_status_snapshot(&config, &result, mount_state, last_attach_at);
    runtime_log(&format!(
        "initialize_paint_state load_status={} path={} overall={} codex={} claude={}",
        result.outcome.as_str(),
        result.path.display(),
        snapshot.overall_state.as_str(),
        snapshot
            .sources
            .get("codex")
            .map(|source| source.state.as_str())
            .unwrap_or("idle"),
        snapshot
            .sources
            .get("claude")
            .map(|source| source.state.as_str())
            .unwrap_or("idle"),
    ));
    let _ = PAINT_STATE.set(Mutex::new(PaintState {
        snapshot: snapshot.clone(),
        effects: WidgetEffectsState::default(),
        hot_zones: Vec::new(),
    }));
    if let Some(lock) = PAINT_STATE.get()
        && let Ok(mut state) = lock.lock()
    {
        state.effects.sync_snapshot(&snapshot, widget_effects::now_ms());
    }
    runtime_log(&format!(
        "initialize_app_snapshot overall={} codex={} claude={}",
        snapshot.overall_state.as_str(),
        snapshot
            .sources
            .get("codex")
            .map(|source| source.state.as_str())
            .unwrap_or("idle"),
        snapshot
            .sources
            .get("claude")
            .map(|source| source.state.as_str())
            .unwrap_or("idle")
    ));
    if let Some(hwnd) = current_settings_hwnd() {
        let _ = hwnd;
    }
    let _ = APP_STATUS_SNAPSHOT.set(Mutex::new(snapshot));
}

fn poll_hook_state(hwnd: HWND) {
    let result = agent_state::load_state_for_display_diagnostic();
    let previous_config = settings_bridge::current_config();
    let config = settings_bridge::refresh_config_from_disk();
    let config_changed = previous_config != config;
    let (mount_state, last_attach_at) = current_widget_runtime_state();
    let next_snapshot =
        detector::build_status_snapshot(&config, &result, mount_state, last_attach_at);
    runtime_log(&format!(
        "WM_TIMER tick load_status={} path={} overall={} codex={} claude={}",
        result.outcome.as_str(),
        result.path.display(),
        next_snapshot.overall_state.as_str(),
        next_snapshot
            .sources
            .get("codex")
            .map(|source| source.state.as_str())
            .unwrap_or("idle"),
        next_snapshot
            .sources
            .get("claude")
            .map(|source| source.state.as_str())
            .unwrap_or("idle")
    ));
    let Some(lock) = PAINT_STATE.get() else {
        return;
    };
    let Ok(mut current) = lock.lock() else {
        return;
    };

    if display_snapshot_changed(&current.snapshot, &next_snapshot) || config_changed {
        win32::debug_log(&format!(
            "[hook-state] snapshot/config changed overall={} codex={} claude={} config_changed={}",
            next_snapshot.overall_state.as_str(),
            next_snapshot
                .sources
                .get("codex")
                .map(|source| source.state.as_str())
                .unwrap_or("idle"),
            next_snapshot
                .sources
                .get("claude")
                .map(|source| source.state.as_str())
                .unwrap_or("idle"),
            config_changed,
        ));
        runtime_log(&format!(
            "snapshot/config changed old_overall={} new_overall={} config_changed={} invalidate_requested=true",
            current.snapshot.overall_state.as_str(),
            next_snapshot.overall_state.as_str(),
            config_changed,
        ));
        current.snapshot = next_snapshot.clone();
        let snapshot_for_effects = current.snapshot.clone();
        current
            .effects
            .sync_snapshot(&snapshot_for_effects, widget_effects::now_ms());
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    } else {
        current
            .effects
            .sync_snapshot(&next_snapshot, widget_effects::now_ms());
        runtime_log(&format!(
            "snapshot unchanged overall={}",
            next_snapshot.overall_state.as_str()
        ));
    }

    if let Some(snapshot_lock) = APP_STATUS_SNAPSHOT.get()
        && let Ok(mut snapshot) = snapshot_lock.lock()
    {
        let mut effective_snapshot = next_snapshot;
        effective_snapshot.last_widget_attach_at = snapshot.last_widget_attach_at;
        *snapshot = effective_snapshot.clone();
        tray_icon::sync_tray_state(hwnd, &snapshot, &settings_bridge::current_config());
    }
    sync_settings_hosts();
}

fn paint_window(hwnd: HWND) -> LRESULT {
    let mut paint = PAINTSTRUCT::default();
    let mut client_rect = RECT::default();
    let snapshot = PAINT_STATE
        .get()
        .and_then(|state| {
            state
                .lock()
                .ok()
                .map(|state| (state.snapshot.clone(), state.effects.clone()))
        })
        .unwrap_or_else(|| (AppStatusSnapshot::empty(), WidgetEffectsState::default()));
    runtime_log(&format!(
        "WM_PAINT enter hwnd={} overall={}",
        win32::format_hwnd(hwnd),
        snapshot.0.overall_state.as_str()
    ));

    unsafe {
        let _ = BeginPaint(hwnd, &mut paint);
        let _ = GetClientRect(hwnd, &mut client_rect);
    }
    let frame = widget_render::build_widget_frame(
        &snapshot.0,
        &snapshot.1,
        widget_effects::now_ms(),
        &settings_bridge::current_config(),
        &client_rect,
    );
    widget_render::apply_widget_frame(hwnd, &frame);
    if let Some(lock) = PAINT_STATE.get()
        && let Ok(mut state) = lock.lock()
    {
        state.hot_zones = frame.hot_zones.clone();
    }
    unsafe {
        let _ = EndPaint(hwnd, &paint);
    }

    runtime_log(&format!(
        "WM_PAINT exit hwnd={} overall={}",
        win32::format_hwnd(hwnd),
        snapshot.0.overall_state.as_str()
    ));
    LRESULT(0)
}

fn tick_widget_animation(hwnd: HWND) {
    let Some(lock) = PAINT_STATE.get() else {
        return;
    };
    let Ok(state) = lock.lock() else {
        return;
    };
    let now_ms = widget_effects::now_ms();
    if state.effects.needs_animation_frame(&state.snapshot, now_ms) {
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
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

fn display_snapshot_changed(current: &AppStatusSnapshot, next: &AppStatusSnapshot) -> bool {
    current.overall_state != next.overall_state
        || current.last_error_summary != next.last_error_summary
        || current.sources != next.sources
}

fn handle_hit_test(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let mut point = POINT {
        x: low_word(lparam.0 as u32) as i16 as i32,
        y: high_word(lparam.0 as u32) as i16 as i32,
    };
    unsafe {
        let _ = ScreenToClient(hwnd, &mut point);
    }

    if current_hot_group(point).is_some() {
        LRESULT(HTCLIENT as isize)
    } else {
        LRESULT(HTTRANSPARENT as isize)
    }
}

fn handle_widget_click(hwnd: HWND, lparam: LPARAM) {
    let point = POINT {
        x: low_word(lparam.0 as u32) as i16 as i32,
        y: high_word(lparam.0 as u32) as i16 as i32,
    };
    if current_hot_group(point).is_some() {
        handle_tray_action(hwnd, tray_icon::TrayAction::OpenSettings);
    }
}

fn current_hot_group(point: POINT) -> Option<widget_render::WidgetGroupId> {
    PAINT_STATE
        .get()
        .and_then(|lock| lock.lock().ok())
        .and_then(|state| widget_render::hit_test(&state.hot_zones, point))
}

fn low_word(value: u32) -> u16 {
    (value & 0xFFFF) as u16
}

fn high_word(value: u32) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

fn handle_tray_action(hwnd: HWND, action: tray_icon::TrayAction) {
    match action {
        tray_icon::TrayAction::OpenSettings => {
            match settings_process::open_or_focus_tauri_settings() {
                Ok(true) => {
                    if let Some(settings_hwnd) = current_settings_hwnd() {
                        settings_window::hide_window(settings_hwnd);
                    }
                    return;
                }
                Ok(false) => {}
                Err(error) => {
                    runtime_log(&format!(
                        "tauri settings open failed; falling back to Win32 settings window: {error}"
                    ));
                }
            }
            if let Some(settings_hwnd) = current_settings_hwnd() {
                settings_window::show_window(settings_hwnd);
            }
        }
        tray_icon::TrayAction::Refresh => {
            if let Some(snapshot_lock) = APP_STATUS_SNAPSHOT.get()
                && let Ok(snapshot) = snapshot_lock.lock()
            {
                runtime_log(&format!(
                    "manual_refresh requested overall={} codex={} claude={}",
                    snapshot.overall_state.as_str(),
                    snapshot
                .sources
                .get("codex")
                .map(|source| source.state.as_str())
                .unwrap_or("idle"),
                    snapshot
                .sources
                .get("claude")
                .map(|source| source.state.as_str())
                .unwrap_or("idle")
                ));
            }
            poll_hook_state(hwnd);
            relayout_widget(hwnd);
        }
        tray_icon::TrayAction::Exit => {}
    }
}

fn run_message_loop() -> Result<()> {
    let mut message = MSG::default();

    loop {
        let status = unsafe { GetMessageW(&mut message, HWND::default(), 0, 0) }.0;
        if status == -1 {
            runtime_log("Win32 message loop failed");
            return Err(Error::from_win32());
        }
        if status == 0 {
            break;
        }

        unsafe {
            let _ = TranslateMessage(&message);
            let _ = DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn initialize_widget_runtime_state(mount_state: WidgetMountState) {
    let now = agent_state::now_ms();
    let state = WidgetRuntimeState {
        mount_state,
        last_attach_at: (mount_state == WidgetMountState::Attached).then_some(now),
        last_retry_at: None,
    };
    let _ = WIDGET_RUNTIME_STATE.set(Mutex::new(state));
}

fn current_widget_runtime_state() -> (WidgetMountState, Option<u64>) {
    WIDGET_RUNTIME_STATE
        .get()
        .and_then(|lock| {
            lock.lock()
                .ok()
                .map(|state| (state.mount_state, state.last_attach_at))
        })
        .unwrap_or((WidgetMountState::Attached, None))
}

fn widget_mount_state_from_results(
    attach: &taskbar::TaskbarAttachResult,
    layout: &taskbar::TaskbarLayoutResult,
) -> WidgetMountState {
    if attach.set_parent_succeeded && layout.moved {
        WidgetMountState::Attached
    } else {
        WidgetMountState::TrayOnly
    }
}

fn sync_widget_visibility(
    hwnd: HWND,
    layout: &taskbar::TaskbarLayoutResult,
    debug_config: &DebugLoopConfig,
) {
    let config = settings_bridge::current_config();
    let has_visible_group = config.monitoring.codex_enabled || config.monitoring.claude_enabled;
    let (mount_state, _) = current_widget_runtime_state();
    unsafe {
        let _ = ShowWindow(
            hwnd,
            if mount_state == WidgetMountState::Attached && has_visible_group {
                SW_SHOW
            } else {
                SW_HIDE
            },
        );
    }
    if mount_state == WidgetMountState::Attached && has_visible_group && layout.width > 0 {
        set_runtime_stage("refresh_visibility");
        taskbar::refresh_visibility(hwnd, layout, debug_config);
    }
}

fn relayout_widget(hwnd: HWND) {
    let Some(debug_config) = DEBUG_CONFIG.get() else {
        return;
    };
    let probe = probe_taskbar(debug_config);
    let layout = taskbar::position_in_taskbar(hwnd, &probe, debug_config);
    sync_widget_visibility(hwnd, &layout, debug_config);
}

fn attempt_widget_recovery(hwnd: HWND) {
    let Some(lock) = WIDGET_RUNTIME_STATE.get() else {
        return;
    };
    let Ok(mut state) = lock.lock() else {
        return;
    };
    if state.mount_state == WidgetMountState::Attached {
        return;
    }

    let now = agent_state::now_ms();
    if state
        .last_retry_at
        .is_some_and(|last_retry| now.saturating_sub(last_retry) < WIDGET_RETRY_INTERVAL_MS)
    {
        return;
    }

    state.mount_state = WidgetMountState::Retrying;
    state.last_retry_at = Some(now);
    drop(state);

    let Some(debug_config) = DEBUG_CONFIG.get() else {
        return;
    };
    let probe = probe_taskbar(debug_config);
    let attach = taskbar::attach_to_taskbar(hwnd, &probe, debug_config);
    let layout = taskbar::position_in_taskbar(hwnd, &probe, debug_config);
    let recovered = attach.set_parent_succeeded && layout.moved;

    if let Some(lock) = WIDGET_RUNTIME_STATE.get()
        && let Ok(mut state) = lock.lock()
    {
        if recovered {
            state.mount_state = WidgetMountState::Attached;
            state.last_attach_at = Some(now);
        } else {
            state.mount_state = WidgetMountState::Retrying;
        }
    }

    unsafe {
        let _ = ShowWindow(hwnd, if recovered { SW_SHOW } else { SW_HIDE });
    }
    if recovered {
        taskbar::refresh_visibility(hwnd, &layout, debug_config);
    }
}

fn store_settings_hwnd(hwnd: HWND) {
    if let Some(lock) = SETTINGS_HWND.get()
        && let Ok(mut current) = lock.lock()
    {
        *current = hwnd.0 as isize;
    }
}

fn current_settings_hwnd() -> Option<HWND> {
    SETTINGS_HWND
        .get()
        .and_then(|lock| {
            lock.lock()
                .ok()
                .map(|value| HWND(*value as *mut std::ffi::c_void))
        })
        .filter(|hwnd| hwnd.0 != ptr::null_mut())
}

fn current_app_status_snapshot() -> AppStatusSnapshot {
    APP_STATUS_SNAPSHOT
        .get()
        .and_then(|lock| lock.lock().ok().map(|snapshot| snapshot.clone()))
        .unwrap_or_else(AppStatusSnapshot::empty)
}

fn sync_settings_hosts() {
    let snapshot = current_app_status_snapshot();

    if let Some(settings_hwnd) = current_settings_hwnd() {
        settings_window::update_snapshot(settings_hwnd, snapshot);
    } else {
        settings_bridge::update_snapshot(snapshot);
    }
}

fn tray_command_from_wparam(wparam: WPARAM) -> Option<tray_icon::TrayAction> {
    match (wparam.0 & 0xFFFF) as u16 {
        tray_icon::TRAY_CMD_OPEN_SETTINGS => Some(tray_icon::TrayAction::OpenSettings),
        tray_icon::TRAY_CMD_REFRESH => Some(tray_icon::TrayAction::Refresh),
        tray_icon::TRAY_CMD_EXIT => Some(tray_icon::TrayAction::Exit),
        _ => None,
    }
}
