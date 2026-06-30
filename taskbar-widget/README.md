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
  Cargo.lock
  README.md
  scripts/
    diagnose-taskbar-loop.ps1
  src/
    main.rs
    taskbar.rs
    win32.rs
```

职责划分：

- `src/main.rs`
  负责进程启动、DPI 初始化、窗口类注册、窗口创建、绘制和消息循环。
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

## Diagnostics

聚焦诊断脚本：

```powershell
.\scripts\diagnose-taskbar-loop.ps1 -SkipBuild -Parents shell -Anchors tray_notify -CoordModes rect_delta
```

诊断输出会写到：

```text
taskbar-widget/target/diagnose-taskbar-loop/
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

当前核心依赖只有一个：

```toml
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_System_LibraryLoader",
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

## Limitations

当前项目仍然是窄范围验证，不是通用兼容层：

- 仅支持当前主力 Win11 路径
- 仅支持主任务栏
- 仅显示固定文本 `TASKBAR WIDGET`
- 不处理 Explorer 重启恢复
- 不处理多显示器
- 不处理透明背景融合
- 不处理 D2D / DirectComposition 渲染
- 不处理主题、设置、插件系统

## Related Docs

如果要继续维护或扩展，优先阅读：

- [../docs/handoff/2026-06-30-1420.md](../docs/handoff/2026-06-30-1420.md)
- [../docs/checklist/win11-taskbar-runtime-map.md](../docs/checklist/win11-taskbar-runtime-map.md)
- [../docs/checklist/win11-taskbar-widget-checklist.md](../docs/checklist/win11-taskbar-widget-checklist.md)
- [../docs/checklist/win11-taskbar-visibility-replan-checklist.md](../docs/checklist/win11-taskbar-visibility-replan-checklist.md)

## Deferred Next Steps

只有在当前路径继续稳定的前提下，才值得考虑：

- 动态内容刷新
- 更自然的尺寸、间距和对齐
- `LWA_COLORKEY` 背景融合
- 多显示器
- Explorer 重启恢复
- 不同 Win11 / Win10 结构差异
