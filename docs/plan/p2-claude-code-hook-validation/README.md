# P2: Claude Code Hook 验证

## 目标

验证真实 Claude Code hook payload，并确认它能够写入与 Codex 相同的共享状态模型。预期结果是：确认 Claude Code 的身份字段提取、事件映射和示例配置，不再只依赖人工构造 payload。

范围内：payload shape 采样、状态写入验证、配置示例。范围外：Codex 全局安装、任务栏 UI 重构，以及 Claude 专属安装器交互。

## 背景与上下文

当前 Rust hook CLI 已经支持 `claude` 作为 `AgentId`，并且仓库中已有 `examples.claude-hooks.json`。但这个仓库还没有采样过真实 Claude Code payload。Codex lifecycle hooks 已经确认可用；Claude Code 也需要达到同样的证据等级。

## 当前状态分析

相关文件：

- `taskbar-widget/examples.claude-hooks.json`：当前只是示例。
- `taskbar-widget/src/bin/taskbar_widget_hook.rs`：支持 `claude <HookName>`。
- `taskbar-widget/src/hook_rules.rs`：支持常见 hook 名称和候选字段路径。
- `docs/checklist/hook-payload-sampling.md`：当前仍标记 Claude Code payload 待采样。

当前限制：如果 Claude payload 缺少 `session_id`，状态会写入 `claude_unknown`，并且不会参与正常 summary。这不满足多任务监控目标。

## 方案建议

先对 Claude Code 做一次与 Codex 类似的 shape-only 采样，再验证真实状态写入。字段提取尽量保持通用，只有在真实证据出现后，才加 Claude 专属字段。

## 备选方案

- 假设 Claude Code 与已知 schema 一致：速度快，但对多任务正确性风险太高。
- 只用人工 debug CLI 验证 Claude：适合 UI 测试，但不代表真实接入。
- 现在就加入大量字段猜测：会增加误判和隐私风险。

## 实施计划

### Phase 1: 准备 Shape-Only 采样器

- 目标：在不保存原值的前提下采集真实 Claude Code payload shape。
- 文件：如有必要新增脚本，或者直接用现有 `taskbar_widget_hook.exe sample`
- 任务：配置 Claude Code hooks，让关键事件先走 sample/dump 模式。
- 预期结果：产出只含 hook event 和 identity 候选字段的 shape 日志。

### Phase 2: 分析身份字段

- 目标：确认 Claude 的 task key 策略。
- 文件：`docs/checklist/hook-payload-sampling.md`、`taskbar-widget/src/hook_rules.rs`
- 任务：验证是否存在 `session_id` 或等价字段；只有在真实证据出现后才加入字段候选。
- 预期结果：Claude Code 能形成 `claude_<session_id>`，或至少形成有文档说明的 fallback 路径。

### Phase 3: 切换到状态写入

- 目标：验证真实 Claude Code 能更新共享状态。
- 文件：Claude hook 配置、`state.json`
- 任务：把 hooks 指向 `taskbar_widget_hook.exe claude <HookName>`，触发真实事件，检查 state 和 widget 重绘。
- 预期结果：`agents.claude.summary` 和 `global_summary` 都能正确更新。

## 验证策略

- shape-only 采样记录中不能包含原始 prompt 或代码
- `taskbar_widget_hook.exe list` 中出现 `claude_<session_id>`
- 多 agent 状态测试：一个 Codex task 和一个 Claude task 能并存
- 缺失 session 的 fallback 仍然不会污染正常 summary

## 风险与缓解

- Risk: Claude Code payload 字段名与现有候选不一致。Mitigation: 先采样，再精确补字段。
- Risk: Claude hook 配置结构与示例不一致。Mitigation: 以真实 Claude Code 配置为准，而不是只看示例文件。
- Risk: waiting/error 语义与 Codex 不同。Mitigation: 在真实 payload 证据出来前，保持保守映射。

## 待确认问题

- 最终产品应该把 Claude Code hooks 安装到哪里？
- Claude Code 是否也有类似 Codex 的 trust/approval 流程？

## 推荐下一步

先对 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Stop`、失败或错误事件做一次 Claude Code shape-only 采样。
