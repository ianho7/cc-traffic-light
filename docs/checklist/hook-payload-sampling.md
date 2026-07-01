# Hook Payload 采样记录

日期：2026-06-30

## 结论

本轮最初未接入真实 Claude Code / Codex hook 环境，先走人工 payload 路径。2026-07-01 已补充 Codex lifecycle hooks 真实采样。

- 真实 Codex lifecycle payload shape：已采样通过，见 [codex-lifecycle-hooks-validation.md](/D:/project/cc-traffic-light/docs/checklist/codex-lifecycle-hooks-validation.md)。
- 真实 Claude Code payload shape：待采样。
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

## Accepted Limitation

真实 Codex lifecycle hooks 已确认在 `PreToolUse`、`PostToolUse`、`Stop` 中携带 `session_id`、`turn_id`、`hook_event_name`、`cwd` 和 `model`。尚未观察到独立 event order 字段，当前实现使用 `received_at` 作为排序兜底。

Claude Code payload 是否在所有 hook 类型都携带 `session_id` 和事件顺序字段，仍需后续用真实环境确认。本轮 parser 已优先支持常见字段；缺失 `session_id` 时会写入 `<agent>_unknown` 诊断 task，且不参与正常 summary。

## Codex 当前接入结论

截至 2026-07-01，已区分 Codex `notify` 和 Codex lifecycle hooks：

- [codex-notify-probe.md](/D:/project/cc-traffic-light/docs/checklist/codex-notify-probe.md) 已证明当前本机 `notify` 实测为空 stdin，不能作为多任务状态主路径。
- `notify` 只能作为低保真兼容通知或现有通知链路排障入口。
- 官方 Codex lifecycle hooks 是 Codex 主状态来源；本机实测已提供结构化 JSON。
- 本机 hooks 已提供 `session_id`、`hook_event_name`、`cwd`、`model` 和 turn 级 `turn_id`。
