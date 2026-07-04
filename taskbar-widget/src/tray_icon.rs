use crate::win32;
use std::sync::OnceLock;
use taskbar_widget::{app_config::AppConfig, i18n::Localizer, ui_state::AppStatusSnapshot};
use windows::{
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, POINT, WPARAM},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateDIBSection, DIB_RGB_COLORS, DeleteObject,
            HBITMAP, HDC,
        },
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreateIconIndirect, CreatePopupMenu, DestroyIcon, DestroyMenu,
                GetCursorPos, ICONINFO, MF_STRING, SetForegroundWindow, TPM_LEFTALIGN,
                TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP, WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP,
            },
        },
    },
    core::{PCWSTR, Result},
};

pub const TRAY_ICON_UID: u32 = 1;
pub const TRAY_CALLBACK_MESSAGE: u32 = WM_APP + 1;
pub const TRAY_CMD_OPEN_SETTINGS: u16 = 1001;
pub const TRAY_CMD_REFRESH: u16 = 1002;
pub const TRAY_CMD_EXIT: u16 = 1003;
const TRAY_ICON_SIZE: usize = 16;
const TRAY_ICON_PADDING: i32 = 3;

static TRAY_ICON_CACHE: OnceLock<TrayIconCache> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayAction {
    OpenSettings,
    Refresh,
    Exit,
}

pub fn add_tray_icon(hwnd: HWND, config: &AppConfig) -> Result<()> {
    let mut data = notify_icon_data(hwnd, config);
    unsafe {
        Shell_NotifyIconW(NIM_ADD, &mut data).ok()?;
    }
    Ok(())
}

pub fn remove_tray_icon(hwnd: HWND) {
    let mut data = notify_icon_data(hwnd, &AppConfig::default_v1());
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &mut data);
    }
}

pub fn sync_tray_state(hwnd: HWND, snapshot: &AppStatusSnapshot, config: &AppConfig) {
    let mut data = notify_icon_data(hwnd, config);
    data.hIcon = tray_icon_handle_for_overall(snapshot.overall_state.as_str());
    let localizer = Localizer::for_config(config);
    copy_tooltip(&mut data.szTip, &localizer.tray_tooltip(snapshot));
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &mut data);
    }
}

pub fn handle_callback(
    hwnd: HWND,
    _wparam: WPARAM,
    lparam: LPARAM,
    config: &AppConfig,
) -> Option<TrayAction> {
    match lparam.0 as u32 {
        WM_LBUTTONUP => Some(TrayAction::OpenSettings),
        WM_RBUTTONUP => {
            show_tray_menu(hwnd, config);
            None
        }
        _ => None,
    }
}

fn show_tray_menu(hwnd: HWND, config: &AppConfig) {
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return;
    };
    let localizer = Localizer::for_config(config);
    let open_settings = wide_null(&localizer.text("tray.menu.open_settings"));
    let refresh_detection = wide_null(&localizer.text("tray.menu.refresh_detection"));
    let exit = wide_null(&localizer.text("tray.menu.exit"));
    unsafe {
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            usize::from(TRAY_CMD_OPEN_SETTINGS),
            PCWSTR(open_settings.as_ptr()),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            usize::from(TRAY_CMD_REFRESH),
            PCWSTR(refresh_detection.as_ptr()),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            usize::from(TRAY_CMD_EXIT),
            PCWSTR(exit.as_ptr()),
        );
    }

    let mut point = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
    }

    unsafe {
        let _ = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            None,
        );
        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
            hwnd,
            WM_COMMAND,
            WPARAM(0),
            LPARAM(0),
        );
    }

    unsafe {
        let _ = DestroyMenu(menu);
    }
}

fn notify_icon_data(hwnd: HWND, config: &AppConfig) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: TRAY_CALLBACK_MESSAGE,
        hIcon: tray_icon_handle_for_overall("idle"),
        ..Default::default()
    };
    copy_tooltip(
        &mut data.szTip,
        &Localizer::for_config(config).text("app.name"),
    );
    data
}

fn copy_tooltip(target: &mut [u16], value: &str) {
    let wide = win32::wide_text(value);
    for (index, code_unit) in wide
        .into_iter()
        .take(target.len().saturating_sub(1))
        .enumerate()
    {
        target[index] = code_unit;
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    let mut wide = win32::wide_text(value);
    wide.push(0);
    wide
}

fn tray_icon_handle_for_overall(overall: &str) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    let cache = TRAY_ICON_CACHE.get_or_init(build_tray_icon_cache);
    match overall {
        "idle" => cache.idle(),
        "working" => cache.working(),
        "completed" => cache.completed(),
        "needs_attention" => cache.needs_attention(),
        "error" => cache.error(),
        _ => cache.idle(),
    }
}

fn build_tray_icon_cache() -> TrayIconCache {
    TrayIconCache {
        idle: create_status_icon(rgb(82, 214, 113)).0 as isize,
        working: create_status_icon(rgb(82, 214, 113)).0 as isize,
        completed: create_status_icon(rgb(255, 210, 76)).0 as isize,
        needs_attention: create_status_icon(rgb(255, 210, 76)).0 as isize,
        error: create_status_icon(rgb(255, 108, 96)).0 as isize,
    }
}

fn create_status_icon(color: u32) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    let mut pixels = vec![0u32; TRAY_ICON_SIZE * TRAY_ICON_SIZE];
    let center = (TRAY_ICON_SIZE as i32) / 2;
    let radius = center - TRAY_ICON_PADDING;
    let radius_squared = radius * radius;

    for y in 0..TRAY_ICON_SIZE as i32 {
        for x in 0..TRAY_ICON_SIZE as i32 {
            let dx = x - center;
            let dy = y - center;
            if (dx * dx) + (dy * dy) <= radius_squared {
                let index = (y as usize * TRAY_ICON_SIZE) + x as usize;
                pixels[index] = 0xFF00_0000 | color;
            }
        }
    }

    unsafe {
        let color_bitmap = create_argb_bitmap(&pixels);
        let mask_bitmap = create_mask_bitmap();
        let icon = CreateIconIndirect(&ICONINFO {
            fIcon: BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask_bitmap,
            hbmColor: color_bitmap,
        })
        .unwrap_or_default();
        let _ = DeleteObject(color_bitmap);
        let _ = DeleteObject(mask_bitmap);
        icon
    }
}

unsafe fn create_argb_bitmap(pixels: &[u32]) -> HBITMAP {
    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: TRAY_ICON_SIZE as i32,
        biHeight: -(TRAY_ICON_SIZE as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };

    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe {
        CreateDIBSection(HDC::default(), &info, DIB_RGB_COLORS, &mut bits, None, 0)
            .unwrap_or_default()
    };
    if !bits.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits.cast::<u32>(), pixels.len());
        }
    }
    bitmap
}

unsafe fn create_mask_bitmap() -> HBITMAP {
    let pixels = vec![0u32; TRAY_ICON_SIZE * TRAY_ICON_SIZE];
    unsafe { create_argb_bitmap(&pixels) }
}

fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    u32::from(blue) | (u32::from(green) << 8) | (u32::from(red) << 16)
}

struct TrayIconCache {
    idle: isize,
    working: isize,
    completed: isize,
    needs_attention: isize,
    error: isize,
}

impl TrayIconCache {
    fn idle(&self) -> windows::Win32::UI::WindowsAndMessaging::HICON {
        windows::Win32::UI::WindowsAndMessaging::HICON(self.idle as *mut _)
    }

    fn working(&self) -> windows::Win32::UI::WindowsAndMessaging::HICON {
        windows::Win32::UI::WindowsAndMessaging::HICON(self.working as *mut _)
    }

    fn completed(&self) -> windows::Win32::UI::WindowsAndMessaging::HICON {
        windows::Win32::UI::WindowsAndMessaging::HICON(self.completed as *mut _)
    }

    fn needs_attention(&self) -> windows::Win32::UI::WindowsAndMessaging::HICON {
        windows::Win32::UI::WindowsAndMessaging::HICON(self.needs_attention as *mut _)
    }

    fn error(&self) -> windows::Win32::UI::WindowsAndMessaging::HICON {
        windows::Win32::UI::WindowsAndMessaging::HICON(self.error as *mut _)
    }
}

impl Drop for TrayIconCache {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.idle());
            if self.working != self.idle {
                let _ = DestroyIcon(self.working());
            }
            let _ = DestroyIcon(self.completed());
            let _ = DestroyIcon(self.needs_attention());
            let _ = DestroyIcon(self.error());
        }
    }
}
