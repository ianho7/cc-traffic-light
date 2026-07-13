# Stop 语义与 README 对齐 — 2026-07-13

- Decision: `Stop` / `SubagentStop` 始终表示本轮结束并映射为已完成；错误只由显式 `StopFailure`、`PostToolUseFailure`、`ToolUseFailure` 表示。
- Rationale: Codex 的 Stop 是 turn-scope 停止点；Claude Code 的 Stop 在主 agent 完成响应时触发，而 API 错误使用独立的 StopFailure。最终消息中的失败字样不是可靠的失败状态机输入。
- Change: 删除 README 中“带明确失败语义的 Stop”映射错误的表述，并记录不从自由文本推断错误。
