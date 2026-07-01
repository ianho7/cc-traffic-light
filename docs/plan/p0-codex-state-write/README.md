# P0: Codex 状态写入验证

## 目标

把已经验证通过的项目级 Codex lifecycle hooks，从只做 shape dump 的模式切换到通过 `taskbar_widget_hook.exe` 写入真实状态。预期结果是：真实 Codex 对话会把 `codex_<session_id>` 写入 `state.json`，并驱动正在运行的任务栏 widget 根据 `global_summary` 更新。

范围内：只做项目级验证，使用 `D:\project\cc-traffic-light\.codex\hooks.json`。范围外：用户级全局安装、安装器交互、Claude Code，以及更丰富的 UI 展示。

## 背景与上下文

Codex lifecycle hooks 已在 2026-07-01 验证通过。真实 payload 中，`PreToolUse`、`PostToolUse`、`Stop` 都包含 `session_id`、`turn_id`、`hook_event_name`、`cwd`、`model`。当前 `.codex/hooks.json` 仍指向 `taskbar-widget/scripts/codex-lifecycle-hook-dump.ps1`，所以还不会写共享状态文件。

相关文件：

- `.codex/hooks.json`：当前项目级 hook 配置。
- `taskbar-widget/src/bin/taskbar_widget_hook.rs`：hook CLI 入口。
- `taskbar-widget/src/hook_rules.rs`：hook 到状态的映射规则。
- `taskbar-widget/src/agent_state.rs`：task 模型、状态文件、mutex 和 summary。
- `docs/checklist/codex-lifecycle-hooks-validation.md`：真实 Codex payload 证据。

## 当前状态分析

当前状态写入路径已经支持多任务，task key 采用 `agent_name + session_id`，例如 `codex_123`。如果缺少 `session_id`，会回退为 `codex_unknown`，并从正常 summary 中排除。由于 Codex 真实 payload 已有 `session_id`，主任务身份路径可用。

当前已知限制：Codex payload 没有独立的 `event_order` 字段，CLI 目前会用 `received_at` 兜底。对 MVP 来说可以接受，但需要保留在诊断里。

## 方案建议

更新 `.codex/hooks.json`，让每个 lifecycle event 运行：

```text
D:\project\cc-traffic-light\taskbar-widget\target\debug\taskbar_widget_hook.exe codex <HookName>
```

保留 dump 脚本供后续再次采样使用，但不再把它作为当前项目级 hook 的默认命令。改完配置后，通过 `/hooks` 重新 trust 更新后的 hooks，触发真实 Codex 事件，再验证 `state.json` 和 widget 重绘。

## 备选方案

- 继续使用 dump 模式：更安全，但无法证明真实产品链路。
- 直接安装全局 hooks：更接近最终目标，但在本地状态写入未验证前风险过高。
- 写一个同时 dump 和写状态的 wrapper：后续可考虑，但当前会增加复杂度，也更容易保存超出必要的信息。

## 实施计划

### Phase 1: 构建 Hook 二进制

- 目标：确保目标 hook 可执行文件存在。
- 文件：`taskbar-widget/target/debug/taskbar_widget_hook.exe`
- 任务：运行 `cargo build`，并保留 `cargo check` 作为快速校验。
- 预期结果：hook 可执行文件存在，且能运行 `sample`、`list` 和真实 hook 命令。

### Phase 2: 切换项目级 Hooks

- 目标：把 dump 脚本命令替换为真实状态写入 hook CLI。
- 文件：`.codex/hooks.json`
- 任务：把每个事件的 command 更新为 `taskbar_widget_hook.exe codex <HookName>`。
- 预期结果：Codex 会因为 hook 定义变化，要求重新 review/trust 一次。

### Phase 3: 真实 Codex 验证

- 目标：证明真实 lifecycle events 能写状态。
- 文件：`%APPDATA%\CcTrafficLight\state.json`
- 任务：重新启动或刷新当前 repo 的 Codex，会话中通过 `/hooks` trust，触发一个 prompt 和一个简单 tool call，再运行 `taskbar_widget_hook.exe list`。
- 预期结果：`tasks` 中出现 `codex_<session_id>`，`session_id_source = payload`，summary 能经历 `working` 和 `done` 等状态变化。

### Phase 4: Widget 重绘验证

- 目标：确认任务栏 widget 会响应真实状态文件变化。
- 文件：`taskbar-widget/src/main.rs`
- 任务：运行 widget，触发 Codex 事件，观察状态文字或颜色变化。
- 预期结果：widget 无需重启，会在 1000 ms 轮询间隔内更新。

## 验证策略

- `cargo fmt -- --check`
- `cargo check`
- `cargo build`
- `taskbar_widget_hook.exe list`
- 人工执行 Codex `/hooks` trust 检查
- 人工观察任务栏重绘

失败场景：

- hook 未 trust，导致完全不触发。
- hook command 路径错误。
- 状态文件写入因权限失败。
- `Stop` 在 previous `waiting` 后仍保持 `waiting`，这是当前设计行为。

## 风险与缓解

- Risk: 修改 `.codex/hooks.json` 后需要重新 trust。Mitigation: 明确记录并接受一次 `/hooks` trust。
- Risk: 并发事件下 `received_at` 排序不够稳。Mitigation: 先用快速 tool event 做验证；如果后续暴露问题，再在 P4 考虑增加每 session 单调序列。
- Risk: widget 没有重绘。Mitigation: 先验证 `state.json`，再排查 `main.rs` 轮询逻辑。

## 待确认问题

- `PostToolUse` 的失败场景，是否应该根据真实 payload 字段映射到 `error`？当前仍保守映射为 `working`。

## 推荐下一步

先运行 `cargo build`，然后把 `.codex/hooks.json` 从 dump 命令切到 `taskbar_widget_hook.exe codex <HookName>`，验证一个真实 Codex 会话。
