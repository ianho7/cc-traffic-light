# Taskbar Widget

`taskbar-widget` 是一个面向当前主力 Win11 机器的 Rust / Win32 技术验证项目。它的目标不是做完整产品，而是验证并稳定实现这样一条路径：

- Rust
- Win32 API
- 任务栏窗口探测
- `SetParent`
- layered surface
- 任务栏内自定义内容显示

当前仓库已经验证通过：程序启动后可以在 Win11 任务栏右侧稳定显示一个固定文本模块 `TASKBAR WIDGET`，关闭进程后模块消失，再次启动可以复现。

## What It Proves

这个项目当前真正证明的不是“普通 GDI 窗口能画到任务栏附近”，而是：

> Win11 任务栏自定义内容要稳定进入最终桌面合成画面，不能只依赖普通 child HWND + GDI 绘制；当前机器上的稳定路径是 `popup_parented + WS_EX_LAYERED + SetLayeredWindowAttributes(alpha=255, LWA_ALPHA)`。

也就是说：

- `SetParent` 成功，不等于用户肉眼可见。
- `MoveWindow` 成功，不等于用户肉眼可见。
- `PrintWindow` 里能看到内容，不等于用户肉眼可见。
- 真正决定能否显示在 Win11 任务栏最终画面中的关键变量，是窗口是否拥有可被 DWM / taskbar composition 呈现的 layered surface。

## Current Stable Path

当前默认运行路径：

1. 启动早期设置 per-monitor DPI awareness。
2. 创建 `WS_POPUP + WS_EX_TOOLWINDOW` 的 Win32 窗口。
3. 探测 `Shell_TrayWnd` 和 `TrayNotifyWnd`。
4. 在 `SetParent` 之前给窗口加 `WS_EX_LAYERED`。
5. 调用 `SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)`。
6. 调用 `SetParent(hwnd, Shell_TrayWnd)`。
7. 以 `TrayNotifyWnd` 左侧为锚点，通过 `MoveWindow` 放到任务栏右侧区域。
8. 进入消息循环，使用 GDI 绘制固定文本 `TASKBAR WIDGET`。

当前默认配置等价于：

```powershell
TASKBAR_MVP_PARENT=shell
TASKBAR_MVP_ANCHOR=tray_notify
TASKBAR_MVP_COORD_MODE=rect_delta
TASKBAR_MVP_STYLE_MODE=popup_parented
TASKBAR_MVP_LAYERED=opaque
TASKBAR_MVP_REFRESH_MODE=none
```

## Project Layout

```text
taskbar-widget/
  Cargo.toml
  README.md
  examples.codex-hooks.toml
  examples.claude-hooks.json
  scripts/
    diagnose-taskbar-loop.ps1
  src/
    agent_state.rs
    hook_rules.rs
    lib.rs
    main.rs
    taskbar.rs
    win32.rs
    bin/
      taskbar_widget_hook.rs
```

职责划分：

- `src/main.rs`
  负责进程启动、DPI 初始化、窗口类注册、窗口创建、状态轮询、绘制和消息循环。
- `src/agent_state.rs`
  负责 hook 状态 schema、Win32 named mutex、原子 JSON 写入、TTL/stale 和 summary 聚合。
- `src/hook_rules.rs`
  负责 hook name 映射、Stop waiting heuristic、payload 字段提取和 payload shape 采样。
- `src/bin/taskbar_widget_hook.rs`
  负责 hook CLI、stdin/argv 解码、调用规则层、状态写入和 debug `set/clear/list`。
- `src/taskbar.rs`
  负责任务栏探测、attach 策略、layered 设置、定位和诊断 JSON 输出。
- `src/win32.rs`
  提供日志、DPI、HWND/RECT 格式化等小型 Win32 helper。
- `scripts/diagnose-taskbar-loop.ps1`
  跑诊断矩阵，收集 parent / anchor / coord / render / visibility 证据。

## Build And Run

在 `taskbar-widget/` 下执行：

```powershell
cargo check
cargo build
cargo run
```

`cargo run` 的预期结果：

- Win11 任务栏右侧出现 `TASKBAR WIDGET`
- 没有普通桌面浮窗
- 关闭进程后内容消失

如果只做构建校验：

```powershell
cargo fmt -- --check
cargo check
```

## Hook State Integration

当前已增加 Claude Code / Codex hook 状态接收器：

```powershell
cargo build
```

生成的 hook CLI：

```text
target\debug\taskbar_widget_hook.exe
```

默认状态文件：

```text
%APPDATA%\CcTrafficLight\state.json
```

验证时建议用隔离目录：

```powershell
$env:TASKBAR_WIDGET_STATE_HOME = Join-Path $env:TEMP "cc-traffic-light-hook-test"
```

运行期诊断日志可额外写到文件：

```powershell
$env:TASKBAR_MVP_RUNTIME_LOG_FILE = Join-Path $env:TEMP "cc-traffic-light-runtime.log"
```

人工 hook 示例：

```powershell
'{"hook_event_name":"UserPromptSubmit","session_id":"123","event_order":100}' | .\target\debug\taskbar_widget_hook.exe codex UserPromptSubmit
'{"hook_event_name":"PermissionRequest","session_id":"546","event_order":200}' | .\target\debug\taskbar_widget_hook.exe claude PermissionRequest
.\target\debug\taskbar_widget_hook.exe list
```

debug CLI：

```powershell
.\target\debug\taskbar_widget_hook.exe set codex_123 working
.\target\debug\taskbar_widget_hook.exe clear codex_123
.\target\debug\taskbar_widget_hook.exe list
```

采样模式只输出 payload shape，不保存完整 payload：

```powershell
'{"hook_event_name":"UserPromptSubmit","session_id":"123"}' | .\target\debug\taskbar_widget_hook.exe sample
```

状态 schema 提前支持多个 task：`tasks`、`global_summary`、`agents.codex.summary`、`agents.claude.summary`。MVP widget 只消费 `global_summary`，并通过 1000ms Win32 timer 在状态变化时重绘。

当前显示的是“最近 hook 状态聚合”，不是 agent 在线检测结果。Rust MVP 不做进程检测；如果没有新的 hook 事件，显示状态只代表状态文件里的最新 task snapshot 和 TTL/stale 聚合。

示例配置片段：

- [examples.codex-hooks.toml](/D:/project/cc-traffic-light/taskbar-widget/examples.codex-hooks.toml)
- [examples.claude-hooks.json](/D:/project/cc-traffic-light/taskbar-widget/examples.claude-hooks.json)

注意：本项目不自动修改用户外部 Claude Code / Codex 配置。Codex `notify` 已按 [codex-notify-probe.md](/D:/project/cc-traffic-light/docs/checklist/codex-notify-probe.md) 验证为低保真兼容通知，不进入主状态路径。Codex 主状态来源应验证正式 lifecycle hooks，见 [codex-lifecycle-hooks-validation.md](/D:/project/cc-traffic-light/docs/checklist/codex-lifecycle-hooks-validation.md)；`examples.codex-hooks.toml` 使用官方 inline TOML 结构，但仍需要在本机完成 hook trust 和真实 payload 采样。

当前项目级 `.codex/hooks.json` 默认已切到 `taskbar_widget_hook.exe codex <HookName>` 的真实状态写入路径；原 `scripts/codex-lifecycle-hook-dump.ps1` 仍保留，供后续重新采样真实 payload shape。

## Diagnostics

聚焦诊断脚本：

```powershell
.\scripts\diagnose-taskbar-loop.ps1 -SkipBuild -Parents shell -Anchors tray_notify -CoordModes rect_delta
```

widget 生命周期 / live redraw 诊断脚本：

```powershell
.\scripts\diagnose-widget-liveness.ps1 -LoopType baseline -SkipBuild
.\scripts\diagnose-widget-liveness.ps1 -LoopType fixture_replay -SkipBuild
```

诊断输出会写到：

```text
taskbar-widget/target/diagnose-taskbar-loop/
```

以及：

```text
taskbar-widget/target/diagnose-widget-liveness/
```

注意：

- `PrintWindow`、child capture、窗口树日志、诊断 JSON 都只是辅助证据。
- 最终成功标准仍然是用户肉眼可见。

## Key Runtime Facts

当前主路径的已知有效事实：

- 父窗口：`Shell_TrayWnd`
- 锚点窗口：`TrayNotifyWnd`
- 坐标模式：`rect_delta`
- 样式模式：`popup_parented`
- layered 模式：`opaque`
- 固定模块宽度：`160`
- 模块高度：任务栏 client height

当前默认路径下，诊断 JSON 可能显示：

```json
"success": true,
"api_ok": true,
"parent_relation_verified": false,
"layered_ok": true
```

这是预期行为。默认 `popup_parented` 路径保留 `WS_POPUP` 语义，因此不能简单把 `GetParent(hwnd) == Shell_TrayWnd` 当成唯一成功标准。对当前稳定路径，应结合以下信号判断：

- `api_ok=true`
- `layered_ok=true`
- 位置正确
- 人工可见

## Dependencies

当前核心依赖：

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_Security",
  "Win32_System_LibraryLoader",
  "Win32_System_Threading",
  "Win32_UI_HiDpi",
  "Win32_UI_WindowsAndMessaging",
] }
```

当前使用到的核心 API 包括：

- `RegisterClassW`
- `CreateWindowExW`
- `FindWindowW`
- `FindWindowExW`
- `SetParent`
- `SetWindowLongPtrW`
- `SetLayeredWindowAttributes`
- `MoveWindow`
- `GetWindowRect`
- `BeginPaint`
- `DrawTextW`
- `GetMessageW`
- `DispatchMessageW`
- `CreateMutexW`
- `WaitForSingleObject`
- `SetTimer`
- `InvalidateRect`

## Limitations

当前项目仍然是窄范围验证，不是通用兼容层：

- 仅支持当前主力 Win11 路径
- 仅支持主任务栏
- MVP 只显示聚合后的 hook 状态，不显示 task 列表
- 不处理 Explorer 重启恢复
- 不处理多显示器
- 不处理透明背景融合
- 不处理 D2D / DirectComposition 渲染
- 不处理主题、设置、插件系统
- 不自动安装或修改 Claude Code / Codex 外部配置
- 不保存完整 hook payload

## Related Docs

如果要继续维护或扩展，优先阅读：

- [../docs/handoff/2026-06-30-1420.md](../docs/handoff/2026-06-30-1420.md)
- [../docs/checklist/win11-taskbar-runtime-map.md](../docs/checklist/win11-taskbar-runtime-map.md)
- [../docs/checklist/win11-taskbar-widget-checklist.md](../docs/checklist/win11-taskbar-widget-checklist.md)
- [../docs/checklist/win11-taskbar-visibility-replan-checklist.md](../docs/checklist/win11-taskbar-visibility-replan-checklist.md)
- [../docs/checklist/hook-integration-checklist.md](../docs/checklist/hook-integration-checklist.md)
- [../docs/checklist/hook-integration-validation.md](../docs/checklist/hook-integration-validation.md)
- [../docs/checklist/hook-adjustment-checklist.md](../docs/checklist/hook-adjustment-checklist.md)
- [../docs/checklist/codex-notify-probe.md](../docs/checklist/codex-notify-probe.md)
- [../docs/checklist/codex-lifecycle-hooks-validation.md](../docs/checklist/codex-lifecycle-hooks-validation.md)

## Deferred Next Steps

只有在当前路径继续稳定的前提下，才值得考虑：

- 动态内容刷新
- 更自然的尺寸、间距和对齐
- `LWA_COLORKEY` 背景融合
- 多显示器
- Explorer 重启恢复
- 不同 Win11 / Win10 结构差异
