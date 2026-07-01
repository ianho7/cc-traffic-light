# Hook Payload 采样记录

日期：2026-06-30

## 结论

本轮最初未接入真实 Claude Code / Codex hook 环境，先走人工 payload 路径。2026-07-01 已补充 Codex lifecycle hooks 真实采样。

- 真实 Codex lifecycle payload shape：已采样通过，见 [codex-lifecycle-hooks-validation.md](/D:/project/cc-traffic-light/docs/checklist/codex-lifecycle-hooks-validation.md)。
- 真实 Claude Code payload shape：已完成 `UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Notification`、`PostToolUseFailure`、`Stop` 六类首批采样。
- 当前实现：已提供 `sample` 模式，只输出字段结构和值类型，不保存完整 payload。
- 继续条件：用人工 payload 完成状态模型、hook CLI、debug CLI、widget 轮询和验证文档。

## 已实现采样模式

命令：

```powershell
cd D:\project\cc-traffic-light\taskbar-widget
'{"hook_event_name":"UserPromptSubmit","session_id":"123","event_order":100}' | .\target\debug\taskbar_widget_hook.exe sample
```

输出只包含：

- `hook_name_candidates`
- `session_id_candidates`
- `turn_id_candidates`
- `event_order_candidates`
- `shape`

所有字段值都会替换为 `<redacted>`，不会保存 prompt、代码、命令参数、完整路径或完整 payload。

## 当前候选字段

- Hook name: `hook_event_name`、`hookName`、`eventName`、`hook_name`
- Session id: `session_id`、`sessionId`、`sessionID`
- Turn id: `turn_id`、`turnId`、`turnID`
- Event order: `event_order`、`eventOrder`、`timestamp`、`created_at`、`createdAt`、`time`

## Claude Code 当前采样结论

截至 2026-07-01 20:39，已获得真实 Claude Code `UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Notification`、`PostToolUseFailure`、`Stop` 六类 shape-only sample。

已确认字段：

- `hook_event_name`
- `session_id`
- `prompt_id`
- `cwd`
- `permission_mode`
- `transcript_path`

按事件补充确认：

- `UserPromptSubmit`：存在 `prompt`
- `PreToolUse`：存在 `tool_name`、`tool_input`、`tool_use_id`、`effort.level`
- `PostToolUse`：存在 `tool_response`、`duration_ms`，说明 Claude Code 在成功后会把结构化工具结果回传给 hook
- `Notification`：存在 `message`、`notification_type`
- `PostToolUseFailure`：存在 `error`、`is_interrupt`、`duration_ms`
- `Stop`：存在 `last_assistant_message`、`background_tasks`、`session_crons`、`stop_hook_active`

当前未观察到：

- `turn_id`
- `event_order`
- `timestamp` / `created_at` 一类独立排序字段

当前结论：

- Claude Code 已满足 `hook_event_name + session_id` 的最小状态写入前提。
- 当前 parser 中的 `hook_event_name` 和 `session_id` 候选字段可直接命中真实 payload。
- 排序暂时仍应继续使用 `received_at` 兜底。
- `Notification` 已可作为等待态信号，`PostToolUseFailure` 已可作为失败态信号的真实证据来源。
- `Stop` 已确认在结束态也携带 `session_id` 和 `hook_event_name`。
- 当前 shape-only 阶段的首批关键事件证据已经闭环；后续主要工作转为真实状态写入验证。

## Accepted Limitation

真实 Codex lifecycle hooks 已确认在 `PreToolUse`、`PostToolUse`、`Stop` 中携带 `session_id`、`turn_id`、`hook_event_name`、`cwd` 和 `model`。尚未观察到独立 event order 字段，当前实现使用 `received_at` 作为排序兜底。

尚未观察到独立 `turn_id` 或 `event_order` 字段，因此当前仍继续使用 `received_at` 作为排序兜底。本轮 parser 已优先支持常见字段；缺失 `session_id` 时会写入 `<agent>_unknown` 诊断 task，且不参与正常 summary。

## Codex 当前接入结论

截至 2026-07-01，已区分 Codex `notify` 和 Codex lifecycle hooks：

- [codex-notify-probe.md](/D:/project/cc-traffic-light/docs/checklist/codex-notify-probe.md) 已证明当前本机 `notify` 实测为空 stdin，不能作为多任务状态主路径。
- `notify` 只能作为低保真兼容通知或现有通知链路排障入口。
- 官方 Codex lifecycle hooks 是 Codex 主状态来源；本机实测已提供结构化 JSON。
- 本机 hooks 已提供 `session_id`、`hook_event_name`、`cwd`、`model` 和 turn 级 `turn_id`。
