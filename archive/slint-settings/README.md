# Slint Settings Archive

日期：2026-07-04

本目录保存已经从主链路退场的 Slint settings 实现，供迁移回溯和视觉参考使用。

- `taskbar-widget-settings_slint.rs`：旧 Slint host 与回调绑定实现。
- `taskbar-widget-settings.slint`：旧 Slint settings UI 结构和样式 token。

当前仓库状态：

- 默认 settings 主入口是 `taskbar-settings-tauri.exe`。
- `taskbar-widget` 宿主不再编译或初始化 Slint settings。
- 若 Tauri settings 无法打开，当前仅回退到 `taskbar-widget/src/settings_window.rs` 的 Win32 极限 fallback。
