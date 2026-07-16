# 2026-07-15 设置窗口托盘行为交接

## 已完成

- 配置 schema 升级至 7，移除无实际启动行为的 `general.start_minimized_to_tray`。
- 旧配置仍可读取；下一次 schema 迁移或保存会写出 schema 7，因此旧字段会被清理。`general.close_to_tray` 保留，默认值仍为 `true`。
- Tauri 设置窗现在在 bootstrap 与保存设置后同步 `close_to_tray`：开启时关闭请求会被拦截并隐藏窗口，关闭时允许设置进程退出。
- Win32 fallback 设置窗采用同一配置：开启时隐藏，关闭时销毁。再次打开时 host 会检测失效 HWND，重新创建 fallback 窗口并重新绑定当前配置和快照。
- 通用页和 fallback 布局已删除“启动时最小化到托盘”选项，只显示实际有效的“关闭窗口时仅缩到托盘”。

## 验证

- `cargo test -p shared-core --offline`：通过（包含旧字段迁移后不再写出的测试）。
- `cargo check -p taskbar-settings-tauri --offline`：通过。
- `cargo check -p taskbar-widget --offline`：通过。
- `pnpm build:frontend`：通过。
- `taskbar-widget/scripts/validate-tauri-settings-lifecycle.ps1` 现在使用临时 APPDATA 隔离配置；默认的 `close_to_tray=true` 隐藏、复用与恢复路径已通过。脚本的外部 `PostMessage(WM_CLOSE)` 对 OFF 路径未能稳定模拟真实标题栏关闭，需以手工关闭窗口完成该项桌面验收。

## 手工桌面验收建议

1. 保持开关 ON，关闭设置窗后从托盘“打开设置”：应恢复同一 settings 进程。
2. 关闭开关，关闭设置窗后从托盘“打开设置”：旧 settings 进程应退出，随后启动新进程。
3. 在 Tauri 不可用时重复上述两步，确认 Win32 fallback 关闭后可再次打开。

## 边界

主程序托盘菜单的“退出”语义未变：它仍会退出主程序，并清理其管理的 Tauri 设置进程；本次开关仅控制设置窗口的关闭按钮行为。

## 追加：登录时启动状态修复

- 诊断发现安装器未创建 `CcTrafficLight` Run 项时，注册表实际状态是 OFF，但旧 `config.json` 仍可能保存 `autostart_enabled: true`。
- host 启动初期会读取注册表修正内存值，但 `initialize_paint_state` 的后续磁盘刷新曾把旧值重新覆盖，导致 Settings 显示 ON。
- 现在启动同步和 `settings_bridge::refresh_config_from_disk` 都以注册表为事实来源；检测到差异会写回配置。`autostart::tests::registry_state_overrides_a_stale_persisted_flag` 覆盖该回归场景。

## 追加：任务栏图标避让

- 右侧 widget 的避让计算原已存在，但仅在启动、手动刷新和恢复时运行；任务栏新增应用图标不会自动触发重排。
- `WidgetRuntimeState` 现在保存任务栏布局指纹（父窗口、锚点和候选占位矩形）。监控轮询每秒只比较指纹；发生变化才调用现有定位逻辑，因此任务管理器等图标出现/消失会自动避让，静止时不会重复移动或重绘 widget。
