use std::sync::OnceLock;

use crate::{settings_bridge, win32};
use taskbar_widget::{
    app_config::{AppConfig, SettingsPage},
    ui_state::AppStatusSnapshot,
};
use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreatePen, CreateSolidBrush, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_TOP,
            DeleteObject, DrawTextW, EndPaint, FillRect, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID,
            SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
        },
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
            HideCaret, IDC_ARROW, LoadCursorW, RegisterClassW, SW_HIDE, SW_SHOW,
            SetForegroundWindow, ShowWindow, WINDOW_EX_STYLE, WM_CLOSE, WM_LBUTTONUP, WM_NCDESTROY,
            WM_PAINT, WNDCLASSW, WS_CAPTION, WS_CLIPCHILDREN, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
            WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
        },
    },
    core::{Error, Result, w},
};

const SETTINGS_CLASS_NAME: windows::core::PCWSTR = w!("CcTrafficLightSettingsWindow");
const SETTINGS_TITLE: windows::core::PCWSTR = w!("CC Traffic Light Settings");
const SETTINGS_WIDTH: i32 = 720;
const SETTINGS_HEIGHT: i32 = 460;
static SETTINGS_WINDOW_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

const NAV_ITEMS: [(SettingsPage, &str); 6] = [
    (SettingsPage::Overview, "OVERVIEW"),
    (SettingsPage::General, "GENERAL"),
    (SettingsPage::Monitoring, "MONITORING"),
    (SettingsPage::Appearance, "APPEARANCE"),
    (SettingsPage::Diagnostics, "DIAGNOSTICS"),
    (SettingsPage::About, "ABOUT"),
];

#[derive(Clone, Copy)]
struct LayoutRects {
    nav_rect: RECT,
    content_rect: RECT,
    top_rect: RECT,
    codex_rect: RECT,
    claude_rect: RECT,
    general_rows: [RECT; 2],
    diagnostics_card: RECT,
    diagnostics_refresh_button: RECT,
}

pub fn create_window(
    hinstance: HINSTANCE,
    snapshot: AppStatusSnapshot,
    config: AppConfig,
) -> Result<HWND> {
    register_window_class(hinstance)?;
    settings_bridge::initialize(snapshot, config);

    let hwnd = unsafe {
        CreateWindowExW(
            window_ex_style(),
            SETTINGS_CLASS_NAME,
            SETTINGS_TITLE,
            window_style(),
            180,
            120,
            SETTINGS_WIDTH,
            SETTINGS_HEIGHT,
            HWND::default(),
            None,
            hinstance,
            None,
        )
    }?;

    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
    settings_bridge::register_settings_window(hwnd);

    Ok(hwnd)
}

pub fn show_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

pub fn hide_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn update_snapshot(hwnd: HWND, snapshot: AppStatusSnapshot) {
    settings_bridge::update_snapshot(snapshot);

    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
    }
}

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_PAINT => paint_window(hwnd),
        WM_LBUTTONUP => {
            handle_left_click(hwnd, lparam);
            LRESULT(0)
        }
        WM_CLOSE => {
            if settings_bridge::current_config().general.close_to_tray {
                hide_window(hwnd);
            } else {
                unsafe {
                    let _ = DestroyWindow(hwnd);
                };
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            settings_bridge::unregister_settings_window(hwnd);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, _wparam, lparam) },
    }
}

fn register_window_class(hinstance: HINSTANCE) -> Result<()> {
    if SETTINGS_WINDOW_CLASS_REGISTERED.get().is_some() {
        return Ok(());
    }

    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: hinstance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        lpszClassName: SETTINGS_CLASS_NAME,
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        let last_error = win32::last_error_code();
        if last_error != 1410 {
            return Err(Error::from_win32());
        }
    }

    let _ = SETTINGS_WINDOW_CLASS_REGISTERED.set(());

    Ok(())
}

fn paint_window(hwnd: HWND) -> LRESULT {
    let snapshot = settings_bridge::current_snapshot();
    let config = current_config();

    let mut paint = PAINTSTRUCT::default();
    let mut client_rect = RECT::default();

    unsafe {
        let hdc = BeginPaint(hwnd, &mut paint);
        let _ = HideCaret(hwnd);
        let _ = GetClientRect(hwnd, &mut client_rect);

        let layout = compute_layout(client_rect);
        let background = CreateSolidBrush(rgb(8, 8, 10));
        let panel = CreateSolidBrush(rgb(18, 18, 22));
        let card = CreateSolidBrush(rgb(26, 26, 30));

        let _ = FillRect(hdc, &client_rect, background);
        let _ = FillRect(hdc, &layout.nav_rect, panel);
        let _ = FillRect(hdc, &layout.content_rect, panel);

        let _ = SetBkMode(hdc, TRANSPARENT);

        draw_text_line(
            hdc,
            "CC TRAFFIC LIGHT",
            RECT {
                left: layout.nav_rect.left + 16,
                top: layout.nav_rect.top + 18,
                right: layout.nav_rect.right - 16,
                bottom: layout.nav_rect.top + 40,
            },
            rgb(246, 246, 246),
        );

        for (index, (page, label)) in NAV_ITEMS.iter().enumerate() {
            let top = if index == 0 {
                layout.nav_rect.top + 78
            } else {
                layout.nav_rect.top + 112 + ((index - 1) as i32 * 28)
            };
            let color = if *page == config.diagnostics.last_opened_page {
                rgb(255, 255, 255)
            } else {
                rgb(146, 146, 152)
            };
            draw_text_line(
                hdc,
                label,
                RECT {
                    left: layout.nav_rect.left + 16,
                    top,
                    right: layout.nav_rect.right - 16,
                    bottom: top + 20,
                },
                color,
            );
        }

        match config.diagnostics.last_opened_page {
            SettingsPage::Overview => {
                let _ = FillRect(hdc, &layout.top_rect, card);
                let _ = FillRect(hdc, &layout.codex_rect, card);
                let _ = FillRect(hdc, &layout.claude_rect, card);
                paint_overview(hdc, &snapshot, &layout);
            }
            SettingsPage::General => {
                let card_rect = RECT {
                    left: layout.content_rect.left + 20,
                    top: layout.content_rect.top + 20,
                    right: layout.content_rect.right - 20,
                    bottom: layout.content_rect.bottom - 20,
                };
                let _ = FillRect(hdc, &card_rect, card);
                paint_general(hdc, &config, &layout);
            }
            SettingsPage::Diagnostics => {
                let _ = FillRect(hdc, &layout.diagnostics_card, card);
                paint_diagnostics(hdc, &snapshot, &layout);
            }
            SettingsPage::Monitoring | SettingsPage::Appearance | SettingsPage::About => {}
        }

        let _ = DeleteObject(card);
        let _ = DeleteObject(panel);
        let _ = DeleteObject(background);
        let _ = EndPaint(hwnd, &paint);
    }

    LRESULT(0)
}

fn paint_overview(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &AppStatusSnapshot,
    layout: &LayoutRects,
) {
    draw_text_line(
        hdc,
        "SYSTEM OVERVIEW",
        RECT {
            left: layout.top_rect.left + 16,
            top: layout.top_rect.top + 16,
            right: layout.top_rect.right - 16,
            bottom: layout.top_rect.top + 36,
        },
        rgb(244, 244, 244),
    );
    draw_text_line(
        hdc,
        &format!(
            "Overall: {}",
            snapshot.overall_state.as_str().to_uppercase()
        ),
        RECT {
            left: layout.top_rect.left + 16,
            top: layout.top_rect.top + 48,
            right: layout.top_rect.right - 16,
            bottom: layout.top_rect.top + 68,
        },
        rgb(220, 220, 220),
    );
    draw_text_line(
        hdc,
        &format!(
            "Widget: {}",
            snapshot.widget_mount_state.as_str().to_uppercase()
        ),
        RECT {
            left: layout.top_rect.left + 16,
            top: layout.top_rect.top + 72,
            right: layout.top_rect.right - 16,
            bottom: layout.top_rect.top + 92,
        },
        rgb(168, 168, 172),
    );
    draw_text_line(
        hdc,
        &format!(
            "Last Refresh: {}",
            format_timestamp(snapshot.last_detection_refresh_at)
        ),
        RECT {
            left: layout.top_rect.left + 260,
            top: layout.top_rect.top + 72,
            right: layout.top_rect.right - 16,
            bottom: layout.top_rect.top + 92,
        },
        rgb(168, 168, 172),
    );

    paint_source_card(hdc, snapshot, "codex", layout.codex_rect, "CHATGPT");
    paint_source_card(hdc, snapshot, "claude", layout.claude_rect, "CLAUDE");
}

fn paint_source_card(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &AppStatusSnapshot,
    key: &str,
    rect: RECT,
    title: &str,
) {
    let source = snapshot.sources.get(key);
    let state = source
        .map(|source| source.state.as_str().to_uppercase())
        .unwrap_or_else(|| "UNDISCOVERED".to_string());
    let confidence = source
        .map(|source| source.confidence.as_str().to_uppercase())
        .unwrap_or_else(|| "DEGRADED".to_string());
    let method = source
        .map(|source| source.method.as_str().to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    draw_text_line(
        hdc,
        title,
        RECT {
            left: rect.left + 16,
            top: rect.top + 16,
            right: rect.right - 16,
            bottom: rect.top + 36,
        },
        rgb(244, 244, 244),
    );
    draw_text_line(
        hdc,
        &format!("State: {state}"),
        RECT {
            left: rect.left + 16,
            top: rect.top + 52,
            right: rect.right - 16,
            bottom: rect.top + 72,
        },
        rgb(220, 220, 220),
    );
    draw_text_line(
        hdc,
        &format!("Method: {method}"),
        RECT {
            left: rect.left + 16,
            top: rect.top + 78,
            right: rect.right - 16,
            bottom: rect.top + 98,
        },
        rgb(160, 160, 164),
    );
    draw_text_line(
        hdc,
        &format!("Confidence: {confidence}"),
        RECT {
            left: rect.left + 16,
            top: rect.top + 104,
            right: rect.right - 16,
            bottom: rect.top + 124,
        },
        rgb(160, 160, 164),
    );
}

fn paint_general(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    config: &AppConfig,
    layout: &LayoutRects,
) {
    draw_text_line(
        hdc,
        "GENERAL",
        RECT {
            left: layout.content_rect.left + 36,
            top: layout.content_rect.top + 36,
            right: layout.content_rect.right - 36,
            bottom: layout.content_rect.top + 56,
        },
        rgb(244, 244, 244),
    );
    draw_text_line(
        hdc,
        "Startup and tray behavior",
        RECT {
            left: layout.content_rect.left + 36,
            top: layout.content_rect.top + 62,
            right: layout.content_rect.right - 36,
            bottom: layout.content_rect.top + 82,
        },
        rgb(150, 150, 156),
    );

    paint_toggle_row(
        hdc,
        layout.general_rows[0],
        config.general.autostart_enabled,
        "Enable autostart",
        "Current user only. Starts quietly and restores tray/widget.",
    );
    paint_toggle_row(
        hdc,
        layout.general_rows[1],
        config.general.close_to_tray,
        "Close window to tray",
        "Keep the process running when the settings window is closed.",
    );
}

fn paint_toggle_row(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: RECT,
    enabled: bool,
    title: &str,
    description: &str,
) {
    let accent = if enabled {
        rgb(244, 244, 244)
    } else {
        rgb(96, 96, 104)
    };
    let checkbox = RECT {
        left: rect.left,
        top: rect.top + 4,
        right: rect.left + 18,
        bottom: rect.top + 22,
    };

    draw_checkbox(hdc, checkbox, enabled, accent);
    draw_text_line(
        hdc,
        title,
        RECT {
            left: rect.left + 30,
            top: rect.top,
            right: rect.right,
            bottom: rect.top + 22,
        },
        rgb(228, 228, 232),
    );
    draw_text_line(
        hdc,
        description,
        RECT {
            left: rect.left + 30,
            top: rect.top + 24,
            right: rect.right,
            bottom: rect.top + 44,
        },
        rgb(140, 140, 148),
    );
}

fn draw_checkbox(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: RECT,
    enabled: bool,
    accent: COLORREF,
) {
    unsafe {
        let pen = CreatePen(PS_SOLID, 1, accent);
        let fill = CreateSolidBrush(if enabled {
            rgb(235, 235, 235)
        } else {
            rgb(24, 24, 26)
        });
        let old_pen = SelectObject(hdc, pen);
        let old_brush = SelectObject(hdc, fill);
        let _ = windows::Win32::Graphics::Gdi::Rectangle(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
        );

        if enabled {
            let _ = MoveToEx(
                hdc,
                rect.left + 4,
                rect.top + 10,
                Some(&mut POINT::default()),
            );
            let _ = LineTo(hdc, rect.left + 8, rect.bottom - 4);
            let _ = LineTo(hdc, rect.right - 4, rect.top + 4);
        }

        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(fill);
        let _ = DeleteObject(pen);
    }
}

fn paint_diagnostics(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &AppStatusSnapshot,
    layout: &LayoutRects,
) {
    draw_text_line(
        hdc,
        "DIAGNOSTICS",
        RECT {
            left: layout.diagnostics_card.left + 16,
            top: layout.diagnostics_card.top + 16,
            right: layout.diagnostics_card.right - 16,
            bottom: layout.diagnostics_card.top + 36,
        },
        rgb(244, 244, 244),
    );
    draw_text_line(
        hdc,
        "Read-only detector evidence with manual refresh.",
        RECT {
            left: layout.diagnostics_card.left + 16,
            top: layout.diagnostics_card.top + 42,
            right: layout.diagnostics_card.right - 160,
            bottom: layout.diagnostics_card.top + 62,
        },
        rgb(146, 146, 152),
    );
    draw_button(
        hdc,
        layout.diagnostics_refresh_button,
        "REFRESH NOW",
        rgb(230, 230, 234),
        rgb(32, 32, 36),
    );

    draw_text_line(
        hdc,
        &format!(
            "Widget Mount: {}",
            snapshot.widget_mount_state.as_str().to_uppercase()
        ),
        RECT {
            left: layout.diagnostics_card.left + 16,
            top: layout.diagnostics_card.top + 84,
            right: layout.diagnostics_card.right - 16,
            bottom: layout.diagnostics_card.top + 104,
        },
        rgb(218, 218, 222),
    );
    draw_text_line(
        hdc,
        &format!(
            "Last Detection Refresh: {}",
            format_timestamp(snapshot.last_detection_refresh_at)
        ),
        RECT {
            left: layout.diagnostics_card.left + 16,
            top: layout.diagnostics_card.top + 108,
            right: layout.diagnostics_card.right - 16,
            bottom: layout.diagnostics_card.top + 128,
        },
        rgb(168, 168, 172),
    );
    draw_text_line(
        hdc,
        &format!(
            "Last Error: {}",
            snapshot
                .last_error_summary
                .clone()
                .unwrap_or_else(|| "none".to_string())
        ),
        RECT {
            left: layout.diagnostics_card.left + 16,
            top: layout.diagnostics_card.top + 132,
            right: layout.diagnostics_card.right - 16,
            bottom: layout.diagnostics_card.top + 152,
        },
        rgb(168, 168, 172),
    );

    paint_diagnostics_source(
        hdc,
        snapshot,
        "codex",
        "CHATGPT",
        layout.diagnostics_card.left + 16,
        layout.diagnostics_card.top + 182,
    );
    paint_diagnostics_source(
        hdc,
        snapshot,
        "claude",
        "CLAUDE",
        layout.diagnostics_card.left + 16,
        layout.diagnostics_card.top + 286,
    );
}

fn paint_diagnostics_source(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &AppStatusSnapshot,
    key: &str,
    label: &str,
    left: i32,
    top: i32,
) {
    let source = snapshot.sources.get(key);
    let state = source
        .map(|value| value.state.as_str().to_uppercase())
        .unwrap_or_else(|| "UNDISCOVERED".to_string());
    let method = source
        .map(|value| value.method.as_str().to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let confidence = source
        .map(|value| value.confidence.as_str().to_uppercase())
        .unwrap_or_else(|| "DEGRADED".to_string());
    let updated = source
        .map(|value| value.updated_at)
        .filter(|value| *value > 0)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let message = source
        .and_then(|value| value.message.clone())
        .unwrap_or_else(|| "none".to_string());

    draw_text_line(
        hdc,
        label,
        RECT {
            left,
            top,
            right: left + 120,
            bottom: top + 20,
        },
        rgb(236, 236, 240),
    );
    draw_text_line(
        hdc,
        &format!("State: {state}"),
        RECT {
            left,
            top: top + 24,
            right: left + 520,
            bottom: top + 44,
        },
        rgb(214, 214, 220),
    );
    draw_text_line(
        hdc,
        &format!("Method: {method}"),
        RECT {
            left,
            top: top + 46,
            right: left + 520,
            bottom: top + 66,
        },
        rgb(156, 156, 162),
    );
    draw_text_line(
        hdc,
        &format!("Confidence: {confidence}"),
        RECT {
            left: left + 180,
            top: top + 46,
            right: left + 520,
            bottom: top + 66,
        },
        rgb(156, 156, 162),
    );
    draw_text_line(
        hdc,
        &format!("Updated: {updated}"),
        RECT {
            left,
            top: top + 68,
            right: left + 520,
            bottom: top + 88,
        },
        rgb(156, 156, 162),
    );
    draw_text_line(
        hdc,
        &format!("Message: {message}"),
        RECT {
            left,
            top: top + 90,
            right: left + 520,
            bottom: top + 110,
        },
        rgb(156, 156, 162),
    );
}

fn handle_left_click(hwnd: HWND, lparam: LPARAM) {
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
    let point = POINT { x, y };

    let mut client_rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut client_rect);
    }
    let layout = compute_layout(client_rect);

    if let Some(page) = nav_hit_test(point, &layout) {
        mutate_config(hwnd, |config| {
            config.diagnostics.last_opened_page = page;
        });
        return;
    }

    if current_config().diagnostics.last_opened_page == SettingsPage::General {
        for (index, row) in layout.general_rows.iter().enumerate() {
            if rect_contains(*row, point) {
                if index == 0 {
                    toggle_autostart(hwnd);
                } else {
                    mutate_config(hwnd, |config| match index {
                        1 => config.general.close_to_tray = !config.general.close_to_tray,
                        _ => {}
                    });
                }
                return;
            }
        }
    }

    if current_config().diagnostics.last_opened_page == SettingsPage::Diagnostics
        && rect_contains(layout.diagnostics_refresh_button, point)
    {
        request_manual_refresh();
    }
}

fn nav_hit_test(point: POINT, layout: &LayoutRects) -> Option<SettingsPage> {
    for (index, (page, _label)) in NAV_ITEMS.iter().enumerate() {
        let top = if index == 0 {
            layout.nav_rect.top + 78
        } else {
            layout.nav_rect.top + 112 + ((index - 1) as i32 * 28)
        };
        let rect = RECT {
            left: layout.nav_rect.left + 12,
            top: top - 2,
            right: layout.nav_rect.right - 12,
            bottom: top + 20,
        };
        if rect_contains(rect, point) {
            return Some(*page);
        }
    }
    None
}

fn mutate_config<F>(hwnd: HWND, mutate: F)
where
    F: FnOnce(&mut AppConfig),
{
    let _ = update_config(mutate);
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
    }
}

fn toggle_autostart(hwnd: HWND) {
    if settings_bridge::toggle_autostart_setting().is_err() {
        return;
    }
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
    }
}

pub fn current_config() -> AppConfig {
    settings_bridge::current_config()
}

pub fn update_config<F>(mutate: F) -> std::result::Result<AppConfig, String>
where
    F: FnOnce(&mut AppConfig),
{
    settings_bridge::update_config(mutate)
}

pub fn request_manual_refresh_command() -> std::result::Result<AppConfig, String> {
    settings_bridge::request_manual_refresh_command()
}

fn request_manual_refresh() {
    let _ = request_manual_refresh_command();
}

fn compute_layout(client_rect: RECT) -> LayoutRects {
    let nav_rect = RECT {
        left: 20,
        top: 20,
        right: 180,
        bottom: client_rect.bottom - 20,
    };
    let content_rect = RECT {
        left: 196,
        top: 20,
        right: client_rect.right - 20,
        bottom: client_rect.bottom - 20,
    };
    let top_rect = RECT {
        left: content_rect.left + 20,
        top: content_rect.top + 20,
        right: content_rect.right - 20,
        bottom: content_rect.top + 108,
    };
    let codex_rect = RECT {
        left: content_rect.left + 20,
        top: top_rect.bottom + 16,
        right: content_rect.left + 20 + ((content_rect.right - content_rect.left - 52) / 2),
        bottom: top_rect.bottom + 150,
    };
    let claude_rect = RECT {
        left: codex_rect.right + 12,
        top: codex_rect.top,
        right: content_rect.right - 20,
        bottom: codex_rect.bottom,
    };
    let card_left = content_rect.left + 36;
    let card_right = content_rect.right - 36;
    let general_rows = [
        RECT {
            left: card_left,
            top: content_rect.top + 112,
            right: card_right,
            bottom: content_rect.top + 160,
        },
        RECT {
            left: card_left,
            top: content_rect.top + 176,
            right: card_right,
            bottom: content_rect.top + 224,
        },
    ];
    let diagnostics_card = RECT {
        left: content_rect.left + 20,
        top: content_rect.top + 20,
        right: content_rect.right - 20,
        bottom: content_rect.bottom - 20,
    };
    let diagnostics_refresh_button = RECT {
        left: diagnostics_card.right - 142,
        top: diagnostics_card.top + 16,
        right: diagnostics_card.right - 16,
        bottom: diagnostics_card.top + 44,
    };

    LayoutRects {
        nav_rect,
        content_rect,
        top_rect,
        codex_rect,
        claude_rect,
        general_rows,
        diagnostics_card,
        diagnostics_refresh_button,
    }
}

fn rect_contains(rect: RECT, point: POINT) -> bool {
    point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
}

fn draw_text_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    mut rect: RECT,
    color: COLORREF,
) {
    let mut wide = win32::wide_text(text);
    unsafe {
        let _ = SetTextColor(hdc, color);
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
}

fn draw_button(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: RECT,
    label: &str,
    border: COLORREF,
    fill: COLORREF,
) {
    unsafe {
        let pen = CreatePen(PS_SOLID, 1, border);
        let brush = CreateSolidBrush(fill);
        let old_pen = SelectObject(hdc, pen);
        let old_brush = SelectObject(hdc, brush);
        let _ = windows::Win32::Graphics::Gdi::Rectangle(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
        );
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(brush);
        let _ = DeleteObject(pen);
    }

    draw_text_line(
        hdc,
        label,
        RECT {
            left: rect.left + 14,
            top: rect.top + 7,
            right: rect.right - 10,
            bottom: rect.bottom - 7,
        },
        border,
    );
}

fn format_timestamp(value: Option<u64>) -> String {
    value
        .map(|timestamp| timestamp.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    win32::rgb(red, green, blue)
}

fn window_style() -> windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE {
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE | WS_CLIPCHILDREN
}

fn window_ex_style() -> WINDOW_EX_STYLE {
    WS_EX_APPWINDOW | WS_EX_TOOLWINDOW
}
