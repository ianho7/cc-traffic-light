use std::cell::RefCell;

use crate::{settings_bridge, win32};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::{Brush, CloseRequestResponse, Color, ComponentHandle, PlatformError};
use taskbar_widget::{
    app_config::{AppConfig, IndicatorStyle, SettingsPage, UiTheme, WidgetSize, config_file_path},
    i18n::Localizer,
    ui_state::{AppStatusSnapshot, SourceVisualState},
};

slint::include_modules!();

thread_local! {
    static SLINT_SETTINGS_HOST: RefCell<Option<SlintSettingsHost>> = const { RefCell::new(None) };
}

pub fn initialize(snapshot: &AppStatusSnapshot, config: &AppConfig) -> Result<(), String> {
    runtime_log("initialize requested");
    SlintSettingsHost::create(snapshot, config)
        .map(|host| {
            SLINT_SETTINGS_HOST.with(|slot| {
                *slot.borrow_mut() = Some(host);
            });
            runtime_log("initialize succeeded");
        })
        .map_err(|error| error.to_string())
}

pub fn is_available() -> bool {
    SLINT_SETTINGS_HOST.with(|slot| slot.borrow().is_some())
}

pub fn show(snapshot: &AppStatusSnapshot, config: &AppConfig) -> bool {
    SLINT_SETTINGS_HOST.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(host) = slot.as_mut() else {
            runtime_log("show skipped because host missing");
            return false;
        };
        host.apply_state(snapshot, config);
        let shown = host.window.show().is_ok();
        if shown {
            schedule_foreground_activation(host.window.as_weak());
        }
        runtime_log(&format!("show result={shown}"));
        shown
    })
}

pub fn update(snapshot: &AppStatusSnapshot, config: &AppConfig) -> bool {
    SLINT_SETTINGS_HOST.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(host) = slot.as_mut() else {
            return false;
        };
        host.apply_state(snapshot, config);
        true
    })
}

#[allow(dead_code)]
pub fn hide() -> bool {
    SLINT_SETTINGS_HOST.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|host| host.window.hide().is_ok())
    })
}

pub fn shutdown() {
    SLINT_SETTINGS_HOST.with(|slot| {
        if let Some(host) = slot.borrow_mut().take() {
            let _ = host.window.hide();
        }
    });
    let _ = slint::quit_event_loop();
}

struct SlintSettingsHost {
    window: SettingsWindow,
}

impl SlintSettingsHost {
    fn create(snapshot: &AppStatusSnapshot, config: &AppConfig) -> Result<Self, PlatformError> {
        let window = SettingsWindow::new()?;
        window
            .window()
            .on_close_requested(|| CloseRequestResponse::HideWindow);
        bind_callbacks(&window);
        let host = Self { window };
        host.apply_state(snapshot, config);
        Ok(host)
    }

    fn apply_state(&self, snapshot: &AppStatusSnapshot, config: &AppConfig) {
        apply_state(&self.window, snapshot, config);
    }
}

fn bind_callbacks(window: &SettingsWindow) {
    let weak = window.as_weak();
    window.on_select_page({
        let weak = weak.clone();
        move |page| {
            runtime_log(&format!("callback select_page page={page}"));
            let result = settings_bridge::update_config(|config| {
                config.diagnostics.last_opened_page = page_from_index(page);
            });
            apply_backend_result(&weak, result);
        }
    });
    window.on_toggle_autostart({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_autostart");
            apply_backend_result(&weak, settings_bridge::toggle_autostart_setting());
        }
    });
    window.on_toggle_start_minimized({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_start_minimized");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.general.start_minimized_to_tray =
                        !config.general.start_minimized_to_tray;
                }),
            );
        }
    });
    window.on_toggle_close_to_tray({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_close_to_tray");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.general.close_to_tray = !config.general.close_to_tray;
                }),
            );
        }
    });
    window.on_cycle_language({
        let weak = weak.clone();
        move || {
            runtime_log("callback cycle_language");
            apply_backend_result(&weak, settings_bridge::cycle_language_setting());
        }
    });
    window.on_request_refresh({
        let weak = weak.clone();
        move || {
            runtime_log("callback request_refresh");
            apply_backend_result(&weak, settings_bridge::request_manual_refresh_command());
        }
    });
    window.on_toggle_monitoring_codex({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_monitoring_codex");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.monitoring.codex_enabled = !config.monitoring.codex_enabled;
                }),
            );
        }
    });
    window.on_toggle_monitoring_claude({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_monitoring_claude");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.monitoring.claude_enabled = !config.monitoring.claude_enabled;
                }),
            );
        }
    });
    window.on_cycle_indicator_style({
        let weak = weak.clone();
        move || {
            runtime_log("callback cycle_indicator_style");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.appearance.indicator_style = match config.appearance.indicator_style {
                        IndicatorStyle::Classic => IndicatorStyle::Minimal,
                        IndicatorStyle::Minimal => IndicatorStyle::Classic,
                    };
                }),
            );
        }
    });
    window.on_cycle_widget_size({
        let weak = weak.clone();
        move || {
            runtime_log("callback cycle_widget_size");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.appearance.widget_size = match config.appearance.widget_size {
                        WidgetSize::Compact => WidgetSize::Standard,
                        WidgetSize::Standard => WidgetSize::Compact,
                    };
                }),
            );
        }
    });
    window.on_cycle_ui_theme({
        let weak = weak.clone();
        move || {
            runtime_log("callback cycle_ui_theme");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.appearance.ui_theme = match config.appearance.ui_theme {
                        UiTheme::Light => UiTheme::Dark,
                        UiTheme::Dark => UiTheme::Light,
                    };
                }),
            );
        }
    });
    window.on_toggle_show_labels({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_show_labels");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.appearance.show_labels = !config.appearance.show_labels;
                }),
            );
        }
    });
    window.on_toggle_reduced_motion({
        let weak = weak.clone();
        move || {
            runtime_log("callback toggle_reduced_motion");
            apply_backend_result(
                &weak,
                settings_bridge::update_config(|config| {
                    config.appearance.reduced_motion = !config.appearance.reduced_motion;
                }),
            );
        }
    });
}

fn apply_backend_result(weak: &slint::Weak<SettingsWindow>, result: Result<AppConfig, String>) {
    let config = match result {
        Ok(config) => config,
        Err(error) => {
            runtime_log(&format!("callback backend error={error}"));
            return;
        }
    };
    if let Some(window) = weak.upgrade() {
        let snapshot = settings_bridge::current_snapshot();
        apply_state(&window, &snapshot, &config);
        runtime_log("callback state reapplied");
    }
}

fn apply_state(window: &SettingsWindow, snapshot: &AppStatusSnapshot, config: &AppConfig) {
    let localizer = Localizer::for_config(config);
    Theme::get(window).set_dark_mode(config.appearance.ui_theme == UiTheme::Dark);

    window.set_window_title(localizer.text("settings.window_title").into());
    window.set_badge_text(localizer.text("settings.badge").into());
    window.set_page_title(localizer.text("settings.page_title").into());
    window.set_page_subtitle(localizer.text("settings.page_subtitle").into());
    window.set_nav_overview_text(localizer.text("settings.nav.overview").into());
    window.set_nav_general_text(localizer.text("settings.nav.general").into());
    window.set_nav_monitoring_text(localizer.text("settings.nav.monitoring").into());
    window.set_nav_appearance_text(localizer.text("settings.nav.appearance").into());
    window.set_nav_diagnostics_text(localizer.text("settings.nav.diagnostics").into());
    window.set_nav_about_text(localizer.text("settings.nav.about").into());
    window.set_selected_page(page_to_index(config.diagnostics.last_opened_page));

    window.set_hero_label(uppercase_display_label(&localizer.text("settings.hero_label")).into());
    window
        .set_widget_label(uppercase_display_label(&localizer.text("settings.widget_label")).into());
    window.set_codex_label(uppercase_display_label(&localizer.text("settings.codex_label")).into());
    window
        .set_claude_label(uppercase_display_label(&localizer.text("settings.claude_label")).into());

    window.set_hero_value(localizer.state_label(snapshot.overall_state).into());
    window.set_hero_detail(localizer.status_detail(snapshot).into());
    window.set_hero_meta(
        format_timestamp_line(
            &localizer.text("settings.last_refresh"),
            snapshot.last_detection_refresh_at,
            &localizer.text("detail.pending"),
        )
        .into(),
    );
    window.set_widget_value(
        localizer
            .widget_mount_label(snapshot.widget_mount_state)
            .into(),
    );
    window.set_widget_detail(
        format_timestamp_line(
            &localizer.text("settings.last_attach"),
            snapshot.last_widget_attach_at,
            &localizer.text("detail.pending"),
        )
        .into(),
    );
    window.set_hero_status_color(visual_state_brush(snapshot.overall_state));
    window.set_widget_status_color(widget_mount_brush(snapshot.widget_mount_state));
    window.set_hero_status_hollow(visual_state_hollow(snapshot.overall_state));
    window.set_widget_status_hollow(widget_mount_hollow(snapshot.widget_mount_state));

    let codex_source = snapshot.sources.get("codex");
    window.set_codex_value(
        codex_source
            .map(|source| localizer.state_label(source.state))
            .unwrap_or_else(|| localizer.state_label(SourceVisualState::Undiscovered))
            .into(),
    );
    window.set_codex_detail(
        codex_source
            .map(|source| format_source_detail(&localizer, source))
            .unwrap_or_else(|| fallback_source_detail(&localizer, "codex"))
            .into(),
    );
    window.set_codex_status_color(
        codex_source
            .map(|source| visual_state_brush(source.state))
            .unwrap_or_else(|| visual_state_brush(SourceVisualState::Undiscovered)),
    );
    window.set_codex_status_hollow(
        codex_source
            .map(|source| visual_state_hollow(source.state))
            .unwrap_or_else(|| visual_state_hollow(SourceVisualState::Undiscovered)),
    );

    let claude_source = snapshot.sources.get("claude");
    window.set_claude_value(
        claude_source
            .map(|source| localizer.state_label(source.state))
            .unwrap_or_else(|| localizer.state_label(SourceVisualState::Undiscovered))
            .into(),
    );
    window.set_claude_detail(
        claude_source
            .map(|source| format_source_detail(&localizer, source))
            .unwrap_or_else(|| fallback_source_detail(&localizer, "claude"))
            .into(),
    );
    window.set_claude_status_color(
        claude_source
            .map(|source| visual_state_brush(source.state))
            .unwrap_or_else(|| visual_state_brush(SourceVisualState::Undiscovered)),
    );
    window.set_claude_status_hollow(
        claude_source
            .map(|source| visual_state_hollow(source.state))
            .unwrap_or_else(|| visual_state_hollow(SourceVisualState::Undiscovered)),
    );

    window.set_general_title(localizer.text("settings.general_title").into());
    window.set_autostart_label(localizer.text("settings.general_autostart").into());
    window.set_autostart_value(
        localizer
            .bool_label(config.general.autostart_enabled)
            .into(),
    );
    window.set_start_minimized_label(localizer.text("settings.general_start_minimized").into());
    window.set_start_minimized_value(
        localizer
            .bool_label(config.general.start_minimized_to_tray)
            .into(),
    );
    window.set_close_to_tray_label(localizer.text("settings.general_close_to_tray").into());
    window.set_close_to_tray_value(localizer.bool_label(config.general.close_to_tray).into());
    window.set_language_label(localizer.text("settings.general_language").into());
    window.set_language_value(
        localizer
            .language_label(config.localization.language)
            .into(),
    );

    window.set_monitoring_title(localizer.text("settings.monitoring_title").into());
    window.set_monitoring_hint(localizer.text("settings.monitoring_hint").into());
    window.set_monitoring_codex_label(localizer.text("settings.monitoring_codex").into());
    window.set_monitoring_codex_value(localizer.bool_label(config.monitoring.codex_enabled).into());
    window.set_monitoring_claude_label(localizer.text("settings.monitoring_claude").into());
    window.set_monitoring_claude_value(
        localizer
            .bool_label(config.monitoring.claude_enabled)
            .into(),
    );

    window.set_appearance_title(localizer.text("settings.appearance_title").into());
    window.set_appearance_hint(localizer.text("settings.appearance_hint").into());
    window.set_dark_mode(config.appearance.ui_theme == UiTheme::Dark);
    window.set_ui_theme_label(localizer.text("settings.appearance_ui_theme").into());
    window.set_ui_theme_value(localizer.ui_theme_label(config.appearance.ui_theme).into());
    window.set_indicator_style_label(localizer.text("settings.appearance_indicator_style").into());
    window.set_indicator_style_value(
        localizer
            .indicator_style_label(config.appearance.indicator_style)
            .into(),
    );
    window.set_widget_size_label(localizer.text("settings.appearance_widget_size").into());
    window.set_widget_size_value(
        localizer
            .widget_size_label(config.appearance.widget_size)
            .into(),
    );
    window.set_show_labels_label(localizer.text("settings.appearance_show_labels").into());
    window.set_show_labels_value(localizer.bool_label(config.appearance.show_labels).into());
    window.set_reduced_motion_label(localizer.text("settings.appearance_reduced_motion").into());
    window.set_reduced_motion_value(
        localizer
            .bool_label(config.appearance.reduced_motion)
            .into(),
    );

    window.set_diagnostics_title(localizer.text("settings.diagnostics_title").into());
    window.set_diagnostics_hint(localizer.text("settings.diagnostics_hint").into());
    window.set_diagnostics_button_text(localizer.text("settings.diagnostics_refresh").into());
    window.set_diagnostics_primary(
        format_timestamp_value(
            snapshot.last_detection_refresh_at,
            &localizer.text("detail.pending"),
        )
        .into(),
    );
    window.set_diagnostics_secondary(
        format_diagnostics_error(&localizer, snapshot.last_error_summary.as_deref()).into(),
    );
    window
        .set_diagnostics_codex(format_source_diagnostic_line(&localizer, snapshot, "codex").into());
    window.set_diagnostics_claude(
        format_source_diagnostic_line(&localizer, snapshot, "claude").into(),
    );

    window.set_about_title(localizer.text("settings.about_title").into());
    window.set_about_description(localizer.text("settings.about_description").into());
    window.set_about_product_value(localizer.text("app.name").into());
    window.set_about_version_label(localizer.text("settings.about_version").into());
    window.set_about_version_value(env!("CARGO_PKG_VERSION").into());
    window.set_about_runtime_label(localizer.text("settings.about_runtime").into());
    window.set_about_runtime_value(localizer.text("settings.about_runtime_value").into());
    window.set_about_config_label(localizer.text("settings.about_config_path").into());
    window.set_about_config_value(config_file_path().display().to_string().into());
    window.set_about_language_mode_label(localizer.text("settings.about_language_mode").into());
    window.set_about_language_mode_value(
        localizer
            .language_label(config.localization.language)
            .into(),
    );
}

fn uppercase_display_label(text: &str) -> String {
    if text.is_ascii() {
        text.to_ascii_uppercase()
    } else {
        text.to_string()
    }
}

fn format_diagnostics_error(localizer: &Localizer, error: Option<&str>) -> String {
    match error {
        Some(error) if !error.is_empty() => error.to_string(),
        _ => localizer.text("detail.pending"),
    }
}

fn format_source_diagnostic_line(
    localizer: &Localizer,
    snapshot: &AppStatusSnapshot,
    key: &str,
) -> String {
    let fallback_label = match key {
        "codex" => localizer.text("source.codex"),
        "claude" => localizer.text("source.claude"),
        _ => key.to_string(),
    };
    snapshot
        .sources
        .get(key)
        .map(|source| {
            format!(
                "{} | {}",
                localizer.source_label(source.source_id),
                format_source_detail(localizer, source)
            )
        })
        .unwrap_or_else(|| format!("{fallback_label} | {}", localizer.text("detail.pending")))
}

fn fallback_source_detail(localizer: &Localizer, key: &str) -> String {
    let fallback_label = match key {
        "codex" => localizer.text("source.codex"),
        "claude" => localizer.text("source.claude"),
        _ => key.to_string(),
    };
    format!("{fallback_label} | {}", localizer.text("detail.pending"))
}

fn format_source_detail(
    localizer: &Localizer,
    source: &taskbar_widget::ui_state::SourceStatus,
) -> String {
    let mut parts = vec![
        format!(
            "{} {}",
            localizer.text("detail.method"),
            localizer.method_label(source.method)
        ),
        format!(
            "{} {}",
            localizer.text("detail.confidence"),
            localizer.confidence_label(source.confidence)
        ),
        format!(
            "{} {}",
            localizer.text("detail.updated"),
            format_timestamp_value(Some(source.updated_at), &localizer.text("detail.pending"))
        ),
    ];

    if let Some(message) = &source.message {
        parts.push(compact_identifier(message));
    }

    parts.join(" | ")
}

fn format_timestamp_line(prefix: &str, value: Option<u64>, pending: &str) -> String {
    format!("{prefix}: {}", format_timestamp_value(value, pending))
}

fn format_timestamp_value(value: Option<u64>, pending: &str) -> String {
    let Some(timestamp_ms) = value.filter(|value| *value > 0) else {
        return pending.to_string();
    };

    let total_seconds = timestamp_ms / 1_000;
    let seconds = (total_seconds % 60) as u32;
    let total_minutes = total_seconds / 60;
    let minutes = (total_minutes % 60) as u32;
    let total_hours = total_minutes / 60;
    let hours = (total_hours % 24) as u32;
    let days = total_hours / 24;
    let (year, month, day) = civil_from_days(days as i64);

    format!("{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02} UTC")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u32, d as u32)
}

fn compact_identifier(value: &str) -> String {
    if value.len() <= 28 || !value.contains('_') {
        return value.to_string();
    }

    let start = &value[..16];
    let end = &value[value.len().saturating_sub(6)..];
    format!("{start}...{end}")
}

fn visual_state_brush(state: SourceVisualState) -> Brush {
    match state {
        SourceVisualState::Idle => brush_rgb(0x9a, 0x9a, 0x9a),
        SourceVisualState::Working => brush_rgb(0x3b, 0xa5, 0x5d),
        SourceVisualState::Attention => brush_rgb(0xd6, 0xa4, 0x39),
        SourceVisualState::Blocking | SourceVisualState::Untrusted => brush_rgb(0xd6, 0x4a, 0x3a),
        SourceVisualState::Undiscovered => brush_rgb(0x9a, 0x9a, 0x9a),
    }
}

fn visual_state_hollow(state: SourceVisualState) -> bool {
    matches!(
        state,
        SourceVisualState::Undiscovered | SourceVisualState::Idle
    )
}

fn widget_mount_brush(state: taskbar_widget::ui_state::WidgetMountState) -> Brush {
    match state {
        taskbar_widget::ui_state::WidgetMountState::Attached => brush_rgb(0x3b, 0xa5, 0x5d),
        taskbar_widget::ui_state::WidgetMountState::TrayOnly => brush_rgb(0x9a, 0x9a, 0x9a),
        taskbar_widget::ui_state::WidgetMountState::Retrying => brush_rgb(0xd6, 0xa4, 0x39),
    }
}

fn widget_mount_hollow(state: taskbar_widget::ui_state::WidgetMountState) -> bool {
    matches!(state, taskbar_widget::ui_state::WidgetMountState::TrayOnly)
}

fn brush_rgb(red: u8, green: u8, blue: u8) -> Brush {
    Color::from_rgb_u8(red, green, blue).into()
}

fn page_to_index(page: SettingsPage) -> i32 {
    match page {
        SettingsPage::Overview => 0,
        SettingsPage::General => 1,
        SettingsPage::Monitoring => 2,
        SettingsPage::Appearance => 3,
        SettingsPage::Diagnostics => 4,
        SettingsPage::About => 5,
    }
}

fn page_from_index(index: i32) -> SettingsPage {
    match index {
        1 => SettingsPage::General,
        2 => SettingsPage::Monitoring,
        3 => SettingsPage::Appearance,
        4 => SettingsPage::Diagnostics,
        5 => SettingsPage::About,
        _ => SettingsPage::Overview,
    }
}

fn runtime_log(message: &str) {
    win32::runtime_debug_log(&format!("[slint-settings] {message}"));
}

fn schedule_foreground_activation(weak: slint::Weak<SettingsWindow>) {
    let invoke_result = slint::invoke_from_event_loop(move || {
        let Some(window) = weak.upgrade() else {
            runtime_log("foreground activation skipped because window dropped");
            return;
        };
        match slint_hwnd(&window) {
            Some(hwnd) => unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                let focused =
                    windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd).as_bool();
                runtime_log(&format!(
                    "foreground activation hwnd={} focused={focused}",
                    win32::format_hwnd(hwnd)
                ));
            },
            None => {
                runtime_log("foreground activation could not resolve native hwnd");
            }
        }
    });

    if let Err(error) = invoke_result {
        runtime_log(&format!("foreground activation scheduling failed: {error}"));
    }
}

fn slint_hwnd(window: &SettingsWindow) -> Option<windows::Win32::Foundation::HWND> {
    let handle = window.window().window_handle();
    let Ok(raw) = handle.window_handle() else {
        return None;
    };

    match raw.as_raw() {
        RawWindowHandle::Win32(win32_handle) => Some(windows::Win32::Foundation::HWND(
            win32_handle.hwnd.get() as *mut std::ffi::c_void,
        )),
        _ => None,
    }
}
