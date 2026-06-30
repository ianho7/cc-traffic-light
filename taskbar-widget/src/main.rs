mod taskbar;
mod win32;

use std::env;
use std::mem::size_of;

use taskbar::{AppState, DebugLoopConfig, probe_taskbar};
use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject,
            DrawTextW, EndPaint, FillRect, PAINTSTRUCT, SetBkMode, SetTextColor, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GetClientRect, GetMessageW, IDC_ARROW, LoadCursorW, MSG, PostQuitMessage,
            RegisterClassW, SW_SHOW, SetCursor, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_DESTROY, WM_PAINT, WM_SETCURSOR, WNDCLASSW, WS_EX_TOOLWINDOW,
            WS_POPUP,
        },
    },
    core::{Error, Result, w},
};

const WINDOW_CLASS_NAME: windows::core::PCWSTR = w!("TaskbarWidgetWindow");
const WINDOW_TITLE: windows::core::PCWSTR = w!("Taskbar Widget");
const WINDOW_WIDTH: i32 = 160;
const WINDOW_HEIGHT: i32 = 48;
const MODULE_TEXT: &str = "TASKBAR WIDGET";

fn main() -> Result<()> {
    win32::debug_log("phase 1 bootstrap start");
    win32::enable_per_monitor_dpi_awareness();
    let debug_config = DebugLoopConfig::from_env();
    taskbar::log_debug_config(&debug_config);

    let hmodule = unsafe { GetModuleHandleW(None) }?;
    let hinstance = HINSTANCE(hmodule.0);
    register_window_class(hinstance)?;
    let probe = probe_taskbar(&debug_config);
    taskbar::log_probe(&probe);

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

    let attach = taskbar::attach_to_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_attach(&attach);
    let layout = taskbar::position_in_taskbar(hwnd, &probe, &debug_config);
    taskbar::log_layout(&layout);
    let state = AppState::from_runtime(hwnd, &probe, &attach, &layout);
    taskbar::log_state(&state);

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
    taskbar::refresh_visibility(hwnd, &layout, &debug_config);
    win32::log_window_dpi("main_window", hwnd);
    win32::debug_log("window created and shown");
    taskbar::write_diagnostics(
        env::var_os("TASKBAR_MVP_DIAG_FILE"),
        hwnd,
        &debug_config,
        &probe,
        &attach,
        &layout,
    );

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

    loop {
        let status = unsafe { GetMessageW(&mut message, None, 0, 0) };
        match status.0 {
            -1 => {
                win32::debug_log(&format!(
                    "GetMessageW failed: last_error={}",
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
        WM_PAINT => paint_window(hwnd),
        WM_SETCURSOR => {
            unsafe {
                let _ = SetCursor(LoadCursorW(None, IDC_ARROW).unwrap_or_default());
            }
            LRESULT(1)
        }
        WM_DESTROY => {
            win32::debug_log("WM_DESTROY received");
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn paint_window(hwnd: HWND) -> LRESULT {
    let mut paint = PAINTSTRUCT::default();
    let mut client_rect = RECT::default();
    let mut text = win32::wide_text(MODULE_TEXT);

    unsafe {
        let hdc = BeginPaint(hwnd, &mut paint);
        let _ = GetClientRect(hwnd, &mut client_rect);

        let background_brush = CreateSolidBrush(COLORREF(0x00181818));
        let _ = FillRect(hdc, &client_rect, background_brush);
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, COLORREF(0x00F4F4F4));
        let _ = DrawTextW(
            hdc,
            &mut text,
            &mut client_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        let _ = DeleteObject(background_brush);
        let _ = EndPaint(hwnd, &paint);
    }

    LRESULT(0)
}
