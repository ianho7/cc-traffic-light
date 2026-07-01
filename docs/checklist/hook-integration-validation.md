# Hook 集成验证记录

日期：2026-06-30

## 构建验证

```powershell
cd D:\project\cc-traffic-light\taskbar-widget
cargo fmt -- --check
cargo check
cargo build
```

结果：

- `cargo fmt -- --check` 通过。
- `cargo check` 通过。
- `cargo build` 通过，生成 `target\debug\taskbar-widget.exe` 和 `target\debug\taskbar_widget_hook.exe`。

环境说明：普通沙箱下 `cargo build` 写 `target\debug\.cargo-lock` 被拒绝，本轮通过已审批的 `cargo build` 提升权限完成构建。

## 人工 Hook 验证

所有验证都使用临时状态目录：

```powershell
$env:TASKBAR_WIDGET_STATE_HOME = Join-Path $env:TEMP "cc-traffic-light-hook-<run-id>"
```

已验证：

- `UserPromptSubmit` 写入 `working`。
- `PermissionRequest` 写入 `waiting`。
- `StopFailure` 写入 `error`。
- `Stop` 在 previous state 为 `waiting` 时保持 `waiting`。
- Codex 成功输出 `{}`。
- Claude 成功路径保持静默。
- 非法 agent 和非法 JSON 返回非 0。
- 缺失 `session_id` 写入 `codex_unknown`，且 `summary_eligible=false`。
- 乱序事件不会覆盖更新状态，并写入 `diagnostics.last_ignored_event`。
- 20 个并发 hook 进程写入同一状态文件时，JSON 未损坏，状态文件约 9KB。
- stale `working` task 不参与 `global_summary.state`，但设置 `has_stale=true`。
- 损坏 `state.json` 后，下一次写入会备份为 `state.corrupt.<timestamp>.json` 并恢复默认状态。
- debug CLI `set/clear/list` 走同一状态 schema、summary 和写入路径。

## Widget 验证状态

代码已实现：

- `SetTimer` 每 1000ms 轮询状态文件。
- `WM_TIMER` 读取 `global_summary`，仅变化时 `InvalidateRect`。
- `WM_PAINT` 不做磁盘 IO，只使用内存中的 latest summary。
- 绘制颜色和文本由 `global_summary.state` 决定。

待人工确认：

- 启动 `cargo run` 后，任务栏组件是否在一个 timer 周期内肉眼切换状态。
- 状态未变化时是否没有连续无意义重绘。

这两项需要 Windows 桌面可见性观察，本轮未自动执行。
