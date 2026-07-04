use std::{ffi::OsString, fs, path::PathBuf, thread, time::Duration};

use shared_core::app_config::WidgetPlacement;
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, POINT, RECT},
        Graphics::Gdi::{InvalidateRect, ScreenToClient, UpdateWindow},
        UI::WindowsAndMessaging::{
            FindWindowExW, FindWindowW, GWL_EXSTYLE, GWL_STYLE, GetClientRect, GetParent,
            GetClassNameW, GetWindowLongPtrW, HWND_TOP, IsWindowVisible, LWA_ALPHA, MoveWindow,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            SWP_SHOWWINDOW, SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW,
            SetWindowPos, WS_BORDER, WS_CAPTION, WS_CHILD, WS_DLGFRAME, WS_EX_LAYERED,
            WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
        },
    },
    core::{PCWSTR, w},
};

use crate::win32;
use crate::settings_bridge;

#[derive(Clone, Copy, Debug)]
pub enum ParentStrategy {
    None,
    ShellTray,
    Rebar,
    TaskSwitch,
    CompositionBridge,
}

impl ParentStrategy {
    fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("shell").to_ascii_lowercase().as_str() {
            "none" => Self::None,
            "rebar" => Self::Rebar,
            "task_switch" | "taskswitch" | "mstaskswwclass" => Self::TaskSwitch,
            "composition" | "composition_bridge" | "bridge" => Self::CompositionBridge,
            _ => Self::ShellTray,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ShellTray => "shell",
            Self::Rebar => "rebar",
            Self::TaskSwitch => "task_switch",
            Self::CompositionBridge => "composition_bridge",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AnchorStrategy {
    TrayNotify,
    TaskSwitch,
    Start,
    ShellTray,
}

impl AnchorStrategy {
    fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("tray_notify").to_ascii_lowercase().as_str() {
            "task_switch" | "taskswitch" | "mstaskswwclass" => Self::TaskSwitch,
            "start" => Self::Start,
            "shell" | "shell_tray" => Self::ShellTray,
            _ => Self::TrayNotify,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TrayNotify => "tray_notify",
            Self::TaskSwitch => "task_switch",
            Self::Start => "start",
            Self::ShellTray => "shell",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CoordinateMode {
    RectDelta,
    ScreenToClient,
}

impl CoordinateMode {
    fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("rect_delta").to_ascii_lowercase().as_str() {
            "screen_to_client" | "client" | "screen2client" => Self::ScreenToClient,
            _ => Self::RectDelta,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RectDelta => "rect_delta",
            Self::ScreenToClient => "screen_to_client",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StyleMode {
    Child,
    PopupParented,
    ParentThenChild,
}

impl StyleMode {
    fn from_env(value: Option<&str>) -> Self {
        match value
            .unwrap_or("popup_parented")
            .to_ascii_lowercase()
            .as_str()
        {
            "child" => Self::Child,
            "popup_parented" | "popupparented" | "popup" => Self::PopupParented,
            "parent_then_child" | "parentthenchild" => Self::ParentThenChild,
            _ => Self::PopupParented,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::PopupParented => "popup_parented",
            Self::ParentThenChild => "parent_then_child",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RefreshMode {
    None,
    Redraw,
    TopmostPulse,
}

impl RefreshMode {
    fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("none").to_ascii_lowercase().as_str() {
            "redraw" => Self::Redraw,
            "topmost_pulse" | "topmostpulse" | "top" => Self::TopmostPulse,
            _ => Self::None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Redraw => "redraw",
            Self::TopmostPulse => "topmost_pulse",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LayeredMode {
    Off,
    Opaque,
    PerPixel,
}

impl LayeredMode {
    fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("per_pixel").to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "opaque" => Self::Opaque,
            "per_pixel" | "perpixel" | "alpha" => Self::PerPixel,
            _ => Self::PerPixel,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Opaque => "opaque",
            Self::PerPixel => "per_pixel",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DebugLoopConfig {
    pub parent_strategy: ParentStrategy,
    pub anchor_strategy: AnchorStrategy,
    pub coordinate_mode: CoordinateMode,
    pub style_mode: StyleMode,
    pub refresh_mode: RefreshMode,
    pub layered_mode: LayeredMode,
    pub uses_env_overrides: bool,
}

impl DebugLoopConfig {
    pub fn from_env() -> Self {
        let parent_override = std::env::var("TASKBAR_MVP_PARENT").ok();
        let anchor_override = std::env::var("TASKBAR_MVP_ANCHOR").ok();
        let coord_override = std::env::var("TASKBAR_MVP_COORD_MODE").ok();
        let style_override = std::env::var("TASKBAR_MVP_STYLE_MODE").ok();
        let refresh_override = std::env::var("TASKBAR_MVP_REFRESH_MODE").ok();
        let layered_override = std::env::var("TASKBAR_MVP_LAYERED").ok();

        Self {
            parent_strategy: ParentStrategy::from_env(parent_override.as_deref()),
            anchor_strategy: AnchorStrategy::from_env(anchor_override.as_deref()),
            coordinate_mode: CoordinateMode::from_env(coord_override.as_deref()),
            style_mode: StyleMode::from_env(style_override.as_deref()),
            refresh_mode: RefreshMode::from_env(refresh_override.as_deref()),
            layered_mode: LayeredMode::from_env(layered_override.as_deref()),
            uses_env_overrides: parent_override.is_some()
                || anchor_override.is_some()
                || coord_override.is_some()
                || style_override.is_some()
                || refresh_override.is_some()
                || layered_override.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AppState {
    pub hwnd: HWND,
    pub taskbar_hwnd: HWND,
    pub parent_hwnd: HWND,
    pub module_rect: RECT,
}

impl AppState {
    pub fn from_runtime(
        hwnd: HWND,
        probe: &TaskbarProbe,
        attach: &TaskbarAttachResult,
        layout: &TaskbarLayoutResult,
    ) -> Self {
        Self {
            hwnd,
            taskbar_hwnd: probe.shell_tray,
            parent_hwnd: first_valid_window(attach.current_parent, probe.host_parent),
            module_rect: layout.module_rect,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TaskbarProbe {
    pub shell_tray: HWND,
    pub start_button: HWND,
    pub tray_dummy_search: HWND,
    pub tray_notify: HWND,
    pub rebar: HWND,
    pub task_switch: HWND,
    pub task_list: HWND,
    pub composition_bridge: HWND,
    pub input_site: HWND,
    pub host_parent: HWND,
    pub position_anchor: HWND,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskbarAttachResult {
    pub attempted: bool,
    pub target_parent: HWND,
    pub previous_parent: HWND,
    pub current_parent: HWND,
    pub style_before: isize,
    pub style_after: isize,
    pub ex_style_before: isize,
    pub ex_style_after: isize,
    pub layered_attempted: bool,
    pub layered_ok: bool,
    pub layered_last_error: u32,
    pub set_parent_api_ok: bool,
    pub set_parent_last_error: u32,
    pub parent_relation_verified: bool,
    pub set_parent_succeeded: bool,
    pub residual_top_level_style_bits: u32,
    pub top_level_style_cleared: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskbarLayoutResult {
    pub attempted: bool,
    pub anchor: HWND,
    pub parent_rect: RECT,
    pub parent_client_rect: RECT,
    pub anchor_rect: RECT,
    pub module_rect: RECT,
    pub moved: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl TaskbarProbe {
    pub fn probe(config: &DebugLoopConfig) -> Self {
        let shell_tray = find_top_level_window(w!("Shell_TrayWnd"));
        let start_button = find_child_window(shell_tray, w!("Start"));
        let tray_dummy_search = find_child_window(shell_tray, w!("TrayDummySearchControl"));
        let tray_notify = find_child_window(shell_tray, w!("TrayNotifyWnd"));
        let rebar = find_child_window(shell_tray, w!("ReBarWindow32"));
        let task_switch = first_valid_window(
            find_child_window(rebar, w!("MSTaskSwWClass")),
            find_child_window(shell_tray, w!("MSTaskSwWClass")),
        );
        let task_list = first_valid_window(
            find_child_window(task_switch, w!("MSTaskListWClass")),
            find_child_window(shell_tray, w!("MSTaskListWClass")),
        );
        let composition_bridge = find_child_window(
            shell_tray,
            w!("Windows.UI.Composition.DesktopWindowContentBridge"),
        );
        let input_site = first_valid_window(
            find_child_window(
                composition_bridge,
                w!("Windows.UI.Input.InputSite.WindowClass"),
            ),
            find_child_window(shell_tray, w!("Windows.UI.Input.InputSite.WindowClass")),
        );

        let host_parent = choose_parent(
            config.parent_strategy,
            shell_tray,
            rebar,
            task_switch,
            composition_bridge,
        );
        let position_anchor = choose_anchor(
            config.anchor_strategy,
            tray_notify,
            task_switch,
            start_button,
            shell_tray,
        );

        Self {
            shell_tray,
            start_button,
            tray_dummy_search,
            tray_notify,
            rebar,
            task_switch,
            task_list,
            composition_bridge,
            input_site,
            host_parent,
            position_anchor,
        }
    }

    pub fn is_shell_tray_available(&self) -> bool {
        is_valid_window(self.shell_tray)
    }
}

pub fn log_debug_config(config: &DebugLoopConfig) {
    win32::debug_log(&format!(
        "[taskbar-loop] config mode={} parent={} anchor={} coord_mode={} style_mode={} refresh_mode={} layered={}",
        if config.uses_env_overrides {
            "env_override"
        } else {
            "runtime_default"
        },
        config.parent_strategy.as_str(),
        config.anchor_strategy.as_str(),
        config.coordinate_mode.as_str(),
        config.style_mode.as_str(),
        config.refresh_mode.as_str(),
        config.layered_mode.as_str()
    ));
}

pub fn probe_taskbar(config: &DebugLoopConfig) -> TaskbarProbe {
    let probe = TaskbarProbe::probe(config);
    if !probe.is_shell_tray_available() {
        win32::debug_log(&format!(
            "[taskbar-loop] taskbar probe failed: Shell_TrayWnd missing last_error={}",
            win32::last_error_code()
        ));
    }
    probe
}

pub fn log_state(state: &AppState) {
    println!(
        "[taskbar-mvp] state hwnd={:?} taskbar={:?} parent={:?} rect=({}, {}, {}, {})",
        state.hwnd,
        state.taskbar_hwnd,
        state.parent_hwnd,
        state.module_rect.left,
        state.module_rect.top,
        state.module_rect.right,
        state.module_rect.bottom
    );
}

pub fn log_probe(probe: &TaskbarProbe) {
    win32::debug_log("[taskbar-loop] taskbar probe snapshot");
    win32::log_window("shell_tray", probe.shell_tray);
    win32::log_window("start_button", probe.start_button);
    win32::log_window("tray_dummy_search", probe.tray_dummy_search);
    win32::log_window("tray_notify", probe.tray_notify);
    win32::log_window("rebar", probe.rebar);
    win32::log_window("task_switch", probe.task_switch);
    win32::log_window("task_list", probe.task_list);
    win32::log_window("composition_bridge", probe.composition_bridge);
    win32::log_window("input_site", probe.input_site);
    win32::debug_log(&format!(
        "[taskbar-loop] taskbar probe choices host_parent={} position_anchor={}",
        win32::format_hwnd(probe.host_parent),
        win32::format_hwnd(probe.position_anchor)
    ));
}

pub fn attach_to_taskbar(
    hwnd: HWND,
    probe: &TaskbarProbe,
    config: &DebugLoopConfig,
) -> TaskbarAttachResult {
    if !is_valid_window(hwnd) || !is_valid_window(probe.host_parent) {
        return TaskbarAttachResult {
            attempted: false,
            target_parent: probe.host_parent,
            previous_parent: HWND::default(),
            current_parent: HWND::default(),
            style_before: 0,
            style_after: 0,
            ex_style_before: 0,
            ex_style_after: 0,
            layered_attempted: false,
            layered_ok: false,
            layered_last_error: 0,
            set_parent_api_ok: false,
            set_parent_last_error: 0,
            parent_relation_verified: false,
            set_parent_succeeded: false,
            residual_top_level_style_bits: 0,
            top_level_style_cleared: false,
        };
    }

    let previous_parent = unsafe { GetParent(hwnd).unwrap_or_default() };
    let style_before = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    let ex_style_before = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    let layered_attempted = !matches!(config.layered_mode, LayeredMode::Off);
    let mut layered_ok = false;
    let mut layered_last_error = 0;

    if layered_attempted {
        unsafe {
            let _ = SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (ex_style_before as u32 | WS_EX_LAYERED.0) as isize,
            );
        }
        if matches!(config.layered_mode, LayeredMode::Opaque) {
            win32::clear_last_error();
            layered_ok =
                unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA).is_ok() };
            layered_last_error = win32::last_error_code();
        } else {
            layered_ok = true;
        }
    }
    let ex_style_after = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };

    if matches!(config.style_mode, StyleMode::Child) {
        unsafe {
            let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, (WS_CHILD.0 | WS_VISIBLE.0) as isize);
        }
    }

    win32::clear_last_error();
    let set_parent_api_ok = unsafe { SetParent(hwnd, probe.host_parent).is_ok() };
    let set_parent_last_error = win32::last_error_code();

    if matches!(config.style_mode, StyleMode::ParentThenChild) {
        unsafe {
            let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, (WS_CHILD.0 | WS_VISIBLE.0) as isize);
        }
    }

    let style_after = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    let residual_top_level_style_bits = top_level_style_bits(style_after);
    let top_level_style_cleared = residual_top_level_style_bits == 0;
    if matches!(
        config.style_mode,
        StyleMode::Child | StyleMode::ParentThenChild
    ) {
        let _ = unsafe {
            SetWindowPos(
                hwnd,
                HWND::default(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
    let current_parent = unsafe { GetParent(hwnd).unwrap_or_default() };
    let parent_relation_verified = current_parent == probe.host_parent;
    let set_parent_succeeded = if matches!(config.style_mode, StyleMode::PopupParented) {
        set_parent_api_ok
    } else {
        parent_relation_verified
    };

    let result = TaskbarAttachResult {
        attempted: true,
        target_parent: probe.host_parent,
        previous_parent,
        current_parent,
        style_before,
        style_after,
        ex_style_before,
        ex_style_after,
        layered_attempted,
        layered_ok,
        layered_last_error,
        set_parent_api_ok,
        set_parent_last_error,
        parent_relation_verified,
        set_parent_succeeded,
        residual_top_level_style_bits,
        top_level_style_cleared,
    };

    if win32::runtime_log_enabled() {
        win32::runtime_debug_log(&format!(
            "{} attach target_parent={} previous_parent={} current_parent={} api_ok={} verified={} layered_ok={} style_mode={} layered_mode={} residual_top_level_style_bits=0x{:X}",
            win32::LIVE_DEBUG_PREFIX,
            win32::format_hwnd(result.target_parent),
            win32::format_hwnd(result.previous_parent),
            win32::format_hwnd(result.current_parent),
            result.set_parent_api_ok,
            result.parent_relation_verified,
            result.layered_ok,
            config.style_mode.as_str(),
            config.layered_mode.as_str(),
            result.residual_top_level_style_bits
        ));
    }

    result
}

pub fn log_attach(result: &TaskbarAttachResult) {
    if !result.attempted {
        win32::debug_log("[taskbar-loop] taskbar attach skipped: no host parent");
        return;
    }

    win32::debug_log(&format!(
        "[taskbar-loop] taskbar attach result success={} api_ok={} last_error={} parent_relation_verified={} target_parent={} previous_parent={} current_parent={} style_before=0x{:X} style_after=0x{:X} ex_style_before=0x{:X} ex_style_after=0x{:X} layered_attempted={} layered_ok={} layered_last_error={} top_level_cleared={} residual_top_level_style_bits=0x{:X}",
        result.set_parent_succeeded,
        result.set_parent_api_ok,
        result.set_parent_last_error,
        result.parent_relation_verified,
        win32::format_hwnd(result.target_parent),
        win32::format_hwnd(result.previous_parent),
        win32::format_hwnd(result.current_parent),
        result.style_before,
        result.style_after,
        result.ex_style_before,
        result.ex_style_after,
        result.layered_attempted,
        result.layered_ok,
        result.layered_last_error,
        result.top_level_style_cleared,
        result.residual_top_level_style_bits
    ));
}

pub fn position_in_taskbar(
    hwnd: HWND,
    probe: &TaskbarProbe,
    config: &DebugLoopConfig,
) -> TaskbarLayoutResult {
    if !is_valid_window(hwnd) || !is_valid_window(probe.host_parent) {
        return TaskbarLayoutResult {
            attempted: false,
            anchor: probe.position_anchor,
            parent_rect: RECT::default(),
            parent_client_rect: RECT::default(),
            anchor_rect: RECT::default(),
            module_rect: RECT::default(),
            moved: false,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
    }

    let parent_rect = win32::rect_for_window(probe.host_parent).unwrap_or_default();
    let parent_client_rect = client_rect_for_window(probe.host_parent).unwrap_or_default();
    let anchor = first_valid_window(probe.position_anchor, probe.host_parent);
    let anchor_rect = win32::rect_for_window(anchor).unwrap_or(parent_rect);
    let parent_width = (parent_client_rect.right - parent_client_rect.left)
        .max(parent_rect.right - parent_rect.left)
        .max(0);
    let parent_height = (parent_client_rect.bottom - parent_client_rect.top)
        .max(parent_rect.bottom - parent_rect.top)
        .max(1);
    let enabled_group_count = enabled_group_count();
    let runtime_config = settings_bridge::current_config();
    let module_width = match enabled_group_count {
        0 => 0,
        1 => 80.min(parent_width.max(1)),
        _ => 160.min(parent_width.max(1)),
    };
    let margin = 8;
    let occupied_rects = collect_peer_widget_rects(hwnd, probe, &parent_rect);
    let desired_screen_left = resolve_module_left(
        runtime_config.widget_visual.placement,
        module_width,
        margin,
        &parent_rect,
        &anchor_rect,
        probe.start_button,
        &occupied_rects,
    );
    let desired_screen_top = parent_rect.top;

    let (x, y, width, height) = match config.coordinate_mode {
        CoordinateMode::RectDelta => (
            (desired_screen_left - parent_rect.left).max(0),
            0,
            module_width,
            parent_height,
        ),
        CoordinateMode::ScreenToClient => {
            let mut origin = POINT {
                x: desired_screen_left,
                y: desired_screen_top,
            };
            let mut far = POINT {
                x: desired_screen_left + module_width,
                y: desired_screen_top + parent_height,
            };
            unsafe {
                let _ = ScreenToClient(probe.host_parent, &mut origin);
                let _ = ScreenToClient(probe.host_parent, &mut far);
            }

            (
                origin.x.max(0),
                origin.y.max(0),
                (far.x - origin.x).max(1),
                (far.y - origin.y).max(1),
            )
        }
    };

    let moved = if module_width > 0 {
        unsafe { MoveWindow(hwnd, x, y, width, height, true).is_ok() }
    } else {
        false
    };
    let module_rect = win32::rect_for_window(hwnd).unwrap_or(RECT {
        left: parent_rect.left + x,
        top: parent_rect.top + y,
        right: parent_rect.left + x + width,
        bottom: parent_rect.top + y + height,
    });

    let result = TaskbarLayoutResult {
        attempted: true,
        anchor,
        parent_rect,
        parent_client_rect,
        anchor_rect,
        module_rect,
        moved,
        x,
        y,
        width,
        height,
    };

    if win32::runtime_log_enabled() {
        win32::runtime_debug_log(&format!(
            "{} layout anchor={} parent_rect={} module_rect={} move_args=({}, {}, {}, {}) moved={}",
            win32::LIVE_DEBUG_PREFIX,
            win32::format_hwnd(result.anchor),
            win32::format_rect(&result.parent_rect),
            win32::format_rect(&result.module_rect),
            result.x,
            result.y,
            result.width,
            result.height,
            result.moved
        ));
    }

    result
}

fn resolve_module_left(
    placement: WidgetPlacement,
    module_width: i32,
    margin: i32,
    parent_rect: &RECT,
    right_anchor_rect: &RECT,
    start_button: HWND,
    occupied_rects: &[RECT],
) -> i32 {
    if module_width <= 0 {
        return parent_rect.left;
    }

    let base_left = match placement {
        WidgetPlacement::Right => {
            if rect_has_area(right_anchor_rect) {
                right_anchor_rect.left - module_width - margin
            } else {
                parent_rect.right - module_width - margin
            }
        }
        WidgetPlacement::Left => {
            let _ = start_button;
            parent_rect.left + margin
        }
    };

    adjust_for_occupied_widgets(
        base_left,
        placement,
        module_width,
        margin,
        parent_rect,
        occupied_rects,
    )
}

fn adjust_for_occupied_widgets(
    initial_left: i32,
    placement: WidgetPlacement,
    module_width: i32,
    margin: i32,
    parent_rect: &RECT,
    occupied_rects: &[RECT],
) -> i32 {
    let min_left = parent_rect.left.max(0);
    let max_left = (parent_rect.right - module_width).max(min_left);
    let mut left = initial_left.clamp(min_left, max_left);

    for _ in 0..occupied_rects.len().max(1) {
        let Some(overlap) = occupied_rects
            .iter()
            .find(|rect| horizontal_overlap(left, module_width, rect))
        else {
            break;
        };

        left = match placement {
            WidgetPlacement::Right => (overlap.left - module_width - margin).max(min_left),
            WidgetPlacement::Left => (overlap.right + margin).min(max_left),
        };
    }

    left.clamp(min_left, max_left)
}

fn collect_peer_widget_rects(hwnd: HWND, probe: &TaskbarProbe, parent_rect: &RECT) -> Vec<RECT> {
    let mut rects = Vec::new();
    let mut roots = vec![probe.shell_tray, probe.rebar, probe.task_switch, probe.host_parent];
    roots.retain(|root| is_valid_window(*root));
    roots.dedup_by(|a, b| a.0 == b.0);

    for root in roots {
        collect_peer_widget_rects_recursive(hwnd, root, parent_rect, &mut rects);
    }

    rects.sort_by_key(|rect| rect.left);
    rects.dedup_by(|a, b| {
        a.left == b.left && a.top == b.top && a.right == b.right && a.bottom == b.bottom
    });
    rects
}

fn collect_peer_widget_rects_recursive(
    hwnd: HWND,
    parent: HWND,
    parent_rect: &RECT,
    rects: &mut Vec<RECT>,
) {
    let mut child = HWND::default();
    loop {
        child = unsafe { FindWindowExW(parent, child, PCWSTR::null(), None).unwrap_or_default() };
        if !is_valid_window(child) {
            break;
        }
        if child != hwnd
            && unsafe { IsWindowVisible(child).as_bool() }
            && let Some(rect) = win32::rect_for_window(child)
            && is_candidate_peer_widget(child, &rect, parent_rect)
        {
            rects.push(rect);
        }
        collect_peer_widget_rects_recursive(hwnd, child, parent_rect, rects);
    }
}

fn is_candidate_peer_widget(hwnd: HWND, rect: &RECT, parent_rect: &RECT) -> bool {
    if !rect_has_area(rect) {
        return false;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    let parent_width = parent_rect.right - parent_rect.left;
    let parent_height = parent_rect.bottom - parent_rect.top;
    if width <= 0 || width > 320 || width >= parent_width / 2 {
        return false;
    }
    if height <= 0 || height < parent_height / 2 {
        return false;
    }
    if rect.bottom <= parent_rect.top || rect.top >= parent_rect.bottom {
        return false;
    }

    let class_name = window_class_name(hwnd);
    !matches!(class_name.as_str(), "TaskbarWidgetWindow")
}

fn window_class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let length = unsafe { GetClassNameW(hwnd, &mut buffer) } as usize;
    String::from_utf16_lossy(&buffer[..length])
}

fn horizontal_overlap(left: i32, width: i32, rect: &RECT) -> bool {
    let right = left + width;
    left < rect.right && right > rect.left
}

fn rect_has_area(rect: &RECT) -> bool {
    rect.right > rect.left && rect.bottom > rect.top
}

fn enabled_group_count() -> usize {
    let config = settings_bridge::current_config();
    usize::from(config.monitoring.codex_enabled) + usize::from(config.monitoring.claude_enabled)
}

pub fn log_layout(result: &TaskbarLayoutResult) {
    if !result.attempted {
        win32::debug_log("[taskbar-loop] taskbar layout skipped: missing parent or target window");
        return;
    }

    win32::debug_log(&format!(
        "[taskbar-loop] taskbar layout result moved={} anchor={} parent_rect={} parent_client_rect={} anchor_rect={} module_rect={} move_args=({}, {}, {}, {})",
        result.moved,
        win32::format_hwnd(result.anchor),
        win32::format_rect(&result.parent_rect),
        win32::format_rect(&result.parent_client_rect),
        win32::format_rect(&result.anchor_rect),
        win32::format_rect(&result.module_rect),
        result.x,
        result.y,
        result.width,
        result.height
    ));
}

pub fn refresh_visibility(hwnd: HWND, layout: &TaskbarLayoutResult, config: &DebugLoopConfig) {
    if !layout.attempted || !is_valid_window(hwnd) {
        if win32::runtime_log_enabled() {
            win32::runtime_debug_log(&format!(
                "{} refresh skipped attempted={} hwnd_valid={}",
                win32::LIVE_DEBUG_PREFIX,
                layout.attempted,
                is_valid_window(hwnd)
            ));
        }
        return;
    }

    if win32::runtime_log_enabled() {
        win32::runtime_debug_log(&format!(
            "{} refresh start mode={} hwnd={} module_rect={}",
            win32::LIVE_DEBUG_PREFIX,
            config.refresh_mode.as_str(),
            win32::format_hwnd(hwnd),
            win32::format_rect(&layout.module_rect)
        ));
    }

    match config.refresh_mode {
        RefreshMode::None => {}
        RefreshMode::Redraw => {
            for attempt in 1..=3 {
                let moved = unsafe {
                    MoveWindow(hwnd, layout.x, layout.y, layout.width, layout.height, true).is_ok()
                };
                let invalidated = unsafe { InvalidateRect(hwnd, None, true).as_bool() };
                let updated = unsafe { UpdateWindow(hwnd).as_bool() };
                win32::debug_log(&format!(
                    "[taskbar-loop] refresh redraw attempt={} moved={} invalidated={} updated={}",
                    attempt, moved, invalidated, updated
                ));
                if win32::runtime_log_enabled() {
                    win32::runtime_debug_log(&format!(
                        "{} refresh redraw attempt={} moved={} invalidated={} updated={}",
                        win32::LIVE_DEBUG_PREFIX,
                        attempt,
                        moved,
                        invalidated,
                        updated
                    ));
                }
                if attempt < 3 {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
        RefreshMode::TopmostPulse => {
            for attempt in 1..=3 {
                let positioned = unsafe {
                    SetWindowPos(
                        hwnd,
                        HWND_TOP,
                        layout.x,
                        layout.y,
                        layout.width,
                        layout.height,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    )
                    .is_ok()
                };
                let invalidated = unsafe { InvalidateRect(hwnd, None, true).as_bool() };
                let updated = unsafe { UpdateWindow(hwnd).as_bool() };
                win32::debug_log(&format!(
                    "[taskbar-loop] refresh topmost_pulse attempt={} positioned={} invalidated={} updated={}",
                    attempt, positioned, invalidated, updated
                ));
                if win32::runtime_log_enabled() {
                    win32::runtime_debug_log(&format!(
                        "{} refresh topmost_pulse attempt={} positioned={} invalidated={} updated={}",
                        win32::LIVE_DEBUG_PREFIX,
                        attempt,
                        positioned,
                        invalidated,
                        updated
                    ));
                }
                if attempt < 3 {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }
}

pub fn write_diagnostics(
    path: Option<OsString>,
    hwnd: HWND,
    config: &DebugLoopConfig,
    probe: &TaskbarProbe,
    attach: &TaskbarAttachResult,
    layout: &TaskbarLayoutResult,
) {
    let Some(path) = path else {
        return;
    };

    let path = PathBuf::from(path);
    let payload = format!(
        concat!(
            "{{\n",
            "  \"config\": {{ \"mode\": \"{mode}\", \"parent\": \"{parent}\", \"anchor\": \"{anchor}\", \"coord_mode\": \"{coord}\", \"style_mode\": \"{style_mode}\", \"refresh_mode\": \"{refresh_mode}\", \"layered\": \"{layered}\" }},\n",
            "  \"probe\": {{ \"shell_tray\": \"{shell}\", \"host_parent\": \"{host_parent}\", \"candidate_parent\": \"{candidate_parent}\", \"position_anchor\": \"{position_anchor}\", \"rebar\": \"{rebar}\", \"task_switch\": \"{task_switch}\", \"composition_bridge\": \"{composition}\" }},\n",
            "  \"attach\": {{ \"attempted\": {attach_attempted}, \"success\": {attach_success}, \"api_ok\": {attach_api_ok}, \"last_error\": {attach_last_error}, \"parent_relation_verified\": {parent_relation_verified}, \"previous_parent\": \"{previous_parent}\", \"current_parent\": \"{current_parent}\", \"style_before\": \"0x{style_before:X}\", \"style_after\": \"0x{style_after:X}\", \"ex_style_before\": \"0x{ex_style_before:X}\", \"ex_style_after\": \"0x{ex_style_after:X}\", \"layered_attempted\": {layered_attempted}, \"layered_ok\": {layered_ok}, \"layered_last_error\": {layered_last_error}, \"top_level_style_cleared\": {top_level_style_cleared}, \"residual_top_level_style_bits\": \"0x{residual_top_level_style_bits:X}\" }},\n",
            "  \"layout\": {{ \"attempted\": {layout_attempted}, \"moved\": {layout_moved}, \"x\": {x}, \"y\": {y}, \"width\": {width}, \"height\": {height}, \"parent_rect\": \"{parent_rect}\", \"parent_client_rect\": \"{parent_client_rect}\", \"anchor_rect\": \"{anchor_rect}\", \"module_rect\": \"{module_rect}\" }},\n",
            "  \"window\": {{ \"hwnd\": \"{hwnd}\", \"system_dpi\": {system_dpi}, \"window_dpi\": {window_dpi}, \"window_scale\": {window_scale:.2}, \"awareness\": \"{awareness}\" }}\n",
            "}}\n"
        ),
        mode = if config.uses_env_overrides {
            "env_override"
        } else {
            "runtime_default"
        },
        parent = config.parent_strategy.as_str(),
        anchor = config.anchor_strategy.as_str(),
        coord = config.coordinate_mode.as_str(),
        style_mode = config.style_mode.as_str(),
        refresh_mode = config.refresh_mode.as_str(),
        layered = config.layered_mode.as_str(),
        shell = win32::format_hwnd(probe.shell_tray),
        host_parent = win32::format_hwnd(probe.host_parent),
        candidate_parent = win32::format_hwnd(probe.host_parent),
        position_anchor = win32::format_hwnd(probe.position_anchor),
        rebar = win32::format_hwnd(probe.rebar),
        task_switch = win32::format_hwnd(probe.task_switch),
        composition = win32::format_hwnd(probe.composition_bridge),
        attach_attempted = attach.attempted,
        attach_success = attach.set_parent_succeeded,
        attach_api_ok = attach.set_parent_api_ok,
        attach_last_error = attach.set_parent_last_error,
        parent_relation_verified = attach.parent_relation_verified,
        previous_parent = win32::format_hwnd(attach.previous_parent),
        current_parent = win32::format_hwnd(attach.current_parent),
        style_before = attach.style_before,
        style_after = attach.style_after,
        ex_style_before = attach.ex_style_before,
        ex_style_after = attach.ex_style_after,
        layered_attempted = attach.layered_attempted,
        layered_ok = attach.layered_ok,
        layered_last_error = attach.layered_last_error,
        top_level_style_cleared = attach.top_level_style_cleared,
        residual_top_level_style_bits = attach.residual_top_level_style_bits,
        layout_attempted = layout.attempted,
        layout_moved = layout.moved,
        x = layout.x,
        y = layout.y,
        width = layout.width,
        height = layout.height,
        parent_rect = win32::format_rect(&layout.parent_rect),
        parent_client_rect = win32::format_rect(&layout.parent_client_rect),
        anchor_rect = win32::format_rect(&layout.anchor_rect),
        module_rect = win32::format_rect(&layout.module_rect),
        hwnd = win32::format_hwnd(hwnd),
        system_dpi = win32::system_dpi(),
        window_dpi = win32::window_dpi(hwnd),
        window_scale = win32::dpi_scale(win32::window_dpi(hwnd)),
        awareness = win32::window_dpi_awareness(hwnd),
    );

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(error) = fs::write(&path, payload) {
        win32::debug_log(&format!(
            "[taskbar-loop] failed to write diagnostics to {}: {error}",
            path.display()
        ));
    }
}

fn find_top_level_window(class_name: PCWSTR) -> HWND {
    unsafe { FindWindowW(class_name, None).unwrap_or_default() }
}

fn find_child_window(parent: HWND, class_name: PCWSTR) -> HWND {
    if !is_valid_window(parent) {
        return HWND::default();
    }

    unsafe { FindWindowExW(parent, HWND::default(), class_name, None).unwrap_or_default() }
}

fn first_valid_window(primary: HWND, fallback: HWND) -> HWND {
    if is_valid_window(primary) {
        primary
    } else {
        fallback
    }
}

fn is_valid_window(hwnd: HWND) -> bool {
    !hwnd.0.is_null()
}

fn choose_parent(
    strategy: ParentStrategy,
    shell_tray: HWND,
    rebar: HWND,
    task_switch: HWND,
    composition_bridge: HWND,
) -> HWND {
    match strategy {
        ParentStrategy::None => HWND::default(),
        ParentStrategy::ShellTray => shell_tray,
        ParentStrategy::Rebar => first_valid_window(rebar, shell_tray),
        ParentStrategy::TaskSwitch => {
            first_valid_window(task_switch, first_valid_window(rebar, shell_tray))
        }
        ParentStrategy::CompositionBridge => first_valid_window(composition_bridge, shell_tray),
    }
}

fn choose_anchor(
    strategy: AnchorStrategy,
    tray_notify: HWND,
    task_switch: HWND,
    start_button: HWND,
    shell_tray: HWND,
) -> HWND {
    match strategy {
        AnchorStrategy::TrayNotify => {
            first_valid_window(tray_notify, first_valid_window(start_button, shell_tray))
        }
        AnchorStrategy::TaskSwitch => {
            first_valid_window(task_switch, first_valid_window(start_button, shell_tray))
        }
        AnchorStrategy::Start => first_valid_window(start_button, shell_tray),
        AnchorStrategy::ShellTray => shell_tray,
    }
}

fn client_rect_for_window(hwnd: HWND) -> Option<RECT> {
    if !is_valid_window(hwnd) {
        return None;
    }

    let mut rect = RECT::default();
    let result = unsafe { GetClientRect(hwnd, &mut rect) };
    if result.is_ok() { Some(rect) } else { None }
}

fn top_level_style_bits(style: isize) -> u32 {
    let mask = WS_POPUP.0
        | WS_CAPTION.0
        | WS_SYSMENU.0
        | WS_MINIMIZEBOX.0
        | WS_MAXIMIZEBOX.0
        | WS_THICKFRAME.0
        | WS_DLGFRAME.0
        | WS_BORDER.0;

    (style as u32) & mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_placement_shifts_left_when_small_peer_overlaps() {
        let parent_rect = RECT {
            left: 0,
            top: 0,
            right: 500,
            bottom: 40,
        };
        let occupied = [RECT {
            left: 372,
            top: 0,
            right: 432,
            bottom: 40,
        }];

        let left = adjust_for_occupied_widgets(
            412,
            WidgetPlacement::Right,
            80,
            8,
            &parent_rect,
            &occupied,
        );

        assert_eq!(left, 284);
    }

    #[test]
    fn left_placement_shifts_right_when_small_peer_overlaps() {
        let parent_rect = RECT {
            left: 0,
            top: 0,
            right: 500,
            bottom: 40,
        };
        let occupied = [RECT {
            left: 56,
            top: 0,
            right: 116,
            bottom: 40,
        }];

        let left = adjust_for_occupied_widgets(
            48,
            WidgetPlacement::Left,
            80,
            8,
            &parent_rect,
            &occupied,
        );

        assert_eq!(left, 124);
    }
}
