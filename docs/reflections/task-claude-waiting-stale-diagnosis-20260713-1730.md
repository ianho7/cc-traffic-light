# Claude waiting 灯组未回落诊断 — 2026-07-13

- Symptom: Claude Code 验证结束较久后，任务栏 Claude 灯组仍显示黄灯快闪。
- Evidence: `%APPDATA%\CcTrafficLight\state.json` 保留 `claude_5ab3572f-f26e-4f71-b4a5-78aa31b721a4`，其 `hook_name` 为 `Notification`、状态为 `waiting`、`summary_eligible=true`；Claude summary 因此仍为 waiting。
- Diagnosis: `hook_rules::infer_state` 将 `Notification` 映射为 `Waiting`，而 `agent_state.rs` 将 `WAITING_STALE_MS` 固定为 24 小时。约 25 分钟不会标 stale，轮询会持续呈现 waiting/黄灯快闪。
- Scope: 这是当前实现的状态语义结果，不是 widget 渲染线程停滞。
- Fix: 用户确认按 README 语义处理后，`Notification` 改映射为 `Idle`，`PermissionRequest` 继续映射为 `Waiting`。先新增会失败的 `notification_does_not_create_a_waiting_state` 回归测试，再修复；focused tests（4/4）、host tests（20/20）和 release build 通过。
- Deployment: 已安装 host 已替换为新 release（SHA-256 与 `target\\release\\taskbar-widget.exe` 一致）并重新启动；精确清除旧 `claude_5ab…` 遗留任务后，默认 state 的 Claude summary 为 `idle`、active_task_count 为 0。
