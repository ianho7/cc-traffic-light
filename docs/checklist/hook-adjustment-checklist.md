# Hook 接入调整执行 Checklist

## Checklist Objective

目标是把 [03-hook-adjustment-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/03-hook-adjustment-plan.md) 转成可执行任务，修正当前 hook 接入中的 4 个风险点：

- Codex 不能把 `notify` 当作正式 lifecycle hooks；notify 已验证为低保真兼容入口，正式 Codex 状态主路径需要验证 lifecycle hooks。
- 当前 `taskbar_widget_hook.rs` 中的状态推断逻辑需要拆成独立 `hook_rules.rs`。
- 需要补齐 Electron 参考实现中的 `SubagentStop -> done`。
- 文档需要明确当前 Rust MVP 显示的是“最近 hook 状态聚合”，不是 agent 在线检测结果。

范围：

- 仅调整 hook 接入策略、规则层结构和相关文档。
- 不自动修改 `C:\Users\admin\.codex\config.toml`。
- 不引入 Electron、Node、daemon、HTTP server、IPC 或进程检测。
- 不改 taskbar probing、`SetParent`、positioning 或 Win11 可见性策略。

## Runtime Loop Spec

### Goal

- 交付一组可验证的 hook 调整任务：Codex notify 探针计划清晰、规则层可独立测试、`SubagentStop` 语义补齐、文档不再误导用户直接套用未确认 Codex hooks 配置。
- 进度证据来自 checklist 勾选、代码 diff、`cargo check`、人工 hook CLI 验证、采样/探针记录和 reflection。

### State

- Source of truth: 本 checklist、[03-hook-adjustment-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/03-hook-adjustment-plan.md)、[hook-integration-checklist.md](/D:/project/cc-traffic-light/docs/checklist/hook-integration-checklist.md)。
- Persistent loop state: 每个完成任务的原因、验证结果、失败分类和下一步决定，保存到 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: 命令输出摘要、临时状态目录、脱敏 notify shape、人工 hook 验证结果、构建结果。
- Discardable state: 一次性终端输出、未保存的临时 JSON、无关探索结果。

### Planner

- 默认先执行不依赖外部配置的 Phase B/C，再执行需要用户确认的 Codex notify 探针。
- 每个 phase 开始前确认上一 phase verifier 已通过。
- 如果下一步需要修改用户外部配置，停止并请求用户确认。
- 如果 Codex notify 不提供足够上下文，记录 accepted limitation，不继续强行推断 session 状态；下一步转向 lifecycle hooks 验证。

### Actor

- 允许动作：读取计划和源码、用 `apply_patch` 小步编辑仓库文件、运行 `cargo check`、运行人工 hook CLI 验证、更新 docs/checklist/reflections。
- 高风险动作：修改 `C:\Users\admin\.codex\config.toml`、替换现有 `notify`、删除状态文件、引入 daemon 或进程检测。
- 高风险动作必须停下让用户确认。

### Observer

- 记录每次规则调整前后的人工 hook 输出。
- 记录 notify 探针是否拿到 argv、stdin、JSON shape、session/thread/turn 相关字段。
- 观察和推断分开写：先记录实际输出，再判断是否能作为状态来源。

### Verifier

- Verifier order:
- 1. `cargo check`。
- 2. 人工 hook CLI 验证：working、waiting、error、Stop waiting、SubagentStop、缺失 session、乱序。
- 3. Codex notify wrapper 探针验证，仅在用户确认外部配置操作后执行。
- 4. 文档检查：不再暗示 Codex 多事件 hooks 格式已确认。

### Failure Semantics

- Transient failure: 命令偶发失败，可重试 1 次。
- Code failure: 编译错误或人工 hook 断言失败，先回到最小相关 task 修复。
- Strategy failure: Codex notify 没有 JSON/stdin/session 信息，不继续扩大推断，记录为低保真 notify。
- Environment failure: 无法修改或观察外部 Codex 配置时，停止为 blocked。
- Policy failure: 需要自动修改用户配置、保存完整 payload、替换原 notify 且不转发时，停止并请求用户确认。

### Exit Conditions

- Success exit: Phase B/C/D 通过验证；Phase A 已完成探针文档，真实外部配置操作由用户确认后另行执行。
- Blocked exit: Codex notify 或正式 hooks 配置无法确认，且继续需要用户提供真实配置/运行环境。
- Budget exit: 同一 phase 连续 3 次无新证据失败。
- Risk exit: 下一步会修改外部配置或引入非 MVP 架构。
- Human takeover exit: 需要用户决定是否允许临时包装 Codex `notify`。

### Policy

- 不保存完整 payload。
- 不自动修改 Codex 或 Claude Code 外部配置。
- 不引入进程检测作为当前 MVP 的显示条件。
- 不改 taskbar host 相关逻辑。
- notify wrapper 必须保留原 notify 转发行为，且安装前必须经过用户确认。

## Pre-Implementation Checks

- [x] HCA-PRE-01 阅读 [03-hook-adjustment-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/03-hook-adjustment-plan.md)，确认调整范围。
- [x] HCA-PRE-02 阅读 `taskbar-widget/src/bin/taskbar_widget_hook.rs`，确认当前 hook rules 混在 CLI 中的函数。
- [x] HCA-PRE-03 阅读 `taskbar-widget/src/agent_state.rs`，确认状态 schema 本轮不需要修改。
- [x] HCA-PRE-04 确认 `taskbar-widget/src/main.rs`、`taskbar-widget/src/taskbar.rs` 不属于本轮调整目标。
- [x] HCA-PRE-05 确认当前验证命令至少包含 `cargo check` 和人工 hook CLI 调用。
- [x] HCA-PRE-06 确认本轮不自动修改 `C:\Users\admin\.codex\config.toml`。

## Implementation Checklist

### Phase A: Codex Notify 探针方案

- [x] HCA-A-GATE 确认用户已理解当前实测对象是 `notify`，不是正式 lifecycle hooks。（用户已按 notify 探针路径执行测试；后续应验证 lifecycle hooks）
- [x] HCA-A-01 新增 `docs/checklist/codex-notify-probe.md`，说明 notify 探针目标、风险、停机条件和人工步骤。
- [x] HCA-A-02 在探针文档中记录当前原始 `notify` 命令的转发要求，不要求用户直接覆盖。
- [x] HCA-A-03 设计 wrapper 输入记录格式：只记录 argv shape、stdin shape、是否 JSON、字段路径，不记录完整值。
- [x] HCA-A-04 设计 wrapper 转发规则：原命令路径和参数必须逐字保留，wrapper 失败时不吞掉原 notify。
- [x] HCA-A-05 明确 notify 探针的判定：若无 session/thread/turn 信息，则仅作为低保真 `turn-ended`，不作为 task 状态主来源。
- [x] HCA-A-06 记录需要用户确认的外部操作：临时改 `notify` 或使用 profile/config override。

### Phase B: Hook Rules 拆分

- [x] HCA-B-GATE 确认 Phase A 不要求先修改用户外部配置，Phase B 可独立执行。
- [x] HCA-B-01 新增 `taskbar-widget/src/hook_rules.rs`。
- [x] HCA-B-02 将 hook name 到 `AgentState` 的映射从 `taskbar_widget_hook.rs` 移入 `hook_rules.rs`。
- [x] HCA-B-03 将 `Stop` waiting heuristic 移入 `hook_rules.rs`。
- [x] HCA-B-04 将 payload 字段提取函数移入 `hook_rules.rs`，包括 hook name、session id、event order 和文本提取。
- [x] HCA-B-05 保持 `taskbar_widget_hook.rs` 只负责 argv/stdin decode、调用规则层、写状态、输出错误和兼容成功输出。
- [x] HCA-B-06 确认拆分不改变 state schema、不改变状态文件路径、不改变 Codex `{}` / Claude 静默成功行为。

### Phase C: Electron 语义对齐

- [x] HCA-C-GATE 确认规则层拆分后 `cargo check` 通过。
- [x] HCA-C-01 在规则层补充 `SubagentStop -> done`。
- [x] HCA-C-02 明确未知 hook 的处理策略：保守映射为 `working`，并保留 hook name 到 task message/diagnostics。
- [x] HCA-C-03 保持 `Stop` 规则：previous state 为 `waiting` 时继续保持 `waiting`。
- [x] HCA-C-04 等真实 payload 证据出现后再扩充 waiting pattern，不提前无限添加关键词。
- [x] HCA-C-05 增加人工验证命令，覆盖 `SubagentStop`、未知 hook、Stop waiting。

### Phase D: 文档同步

- [x] HCA-D-GATE 确认 Phase B/C 的行为验证已记录。
- [x] HCA-D-01 更新 [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)，标记 Codex notify 已验证为低保真，正式 hooks 是主路径。
- [x] HCA-D-02 更新 `taskbar-widget/examples.codex-hooks.toml`，使用官方 lifecycle hooks inline TOML 结构。
- [x] HCA-D-03 更新 `taskbar-widget/README.md`，明确当前显示的是最近 hook 状态聚合，不代表 agent 在线检测。
- [x] HCA-D-04 更新 [hook-integration-checklist.md](/D:/project/cc-traffic-light/docs/checklist/hook-integration-checklist.md)，关联本调整 checklist 或标记被本 checklist 覆盖的后续任务。
- [x] HCA-D-05 如执行 notify 探针，更新 `docs/checklist/codex-notify-probe.md` 的实际观察结果。（真实 Codex notify 无 stdin、无 session/thread/turn/event_order）

## Validation Checklist

- [x] HCA-VAL-01 运行 `cargo check`，期望无编译错误。
- [x] HCA-VAL-02 人工触发 `UserPromptSubmit`，期望状态仍为 `working`。
- [x] HCA-VAL-03 人工触发 `PermissionRequest`，期望状态仍为 `waiting`。
- [x] HCA-VAL-04 人工触发 `StopFailure`，期望状态仍为 `error`。
- [x] HCA-VAL-05 人工触发 previous `waiting` 后的 `Stop`，期望保持 `waiting`。
- [x] HCA-VAL-06 人工触发 `SubagentStop`，期望状态为 `done`。
- [x] HCA-VAL-07 人工触发未知 hook，期望状态为 `working`，并记录 hook name。
- [x] HCA-VAL-08 构造缺失 `session_id` 的 payload，期望仍进入 `<agent>_unknown` 且不参与正常 summary。
- [x] HCA-VAL-09 构造乱序事件，期望旧事件不覆盖新状态。
- [x] HCA-VAL-10 验证 Codex 成功输出 `{}`、Claude 成功静默行为未回归。
- [x] HCA-VAL-11 若用户确认执行 notify 探针，验证 wrapper 是否拿到 argv/stdin shape，并确认原 notify 被转发。（拿到 argv shape；stdin 为空；原 notify attempted=true 但 exit_code=1）
- [x] HCA-VAL-12 若 notify 无 session/thread/turn 信息，记录为低保真来源，不进入 task 状态主路径。

## Documentation Checklist

- [x] HCA-DOC-01 新增或更新 Codex notify 探针文档。
- [x] HCA-DOC-02 更新 Codex hooks 示例的风险说明。
- [x] HCA-DOC-03 更新 README 的“状态聚合不是在线检测”说明。
- [x] HCA-DOC-04 更新 payload 采样文档中的 Codex 当前限制。
- [x] HCA-DOC-05 每个完成的 task 生成 reflection。

## Cleanup Checklist

- [x] HCA-CLN-01 确认没有保存完整 payload、prompt、代码、命令参数或完整路径。
- [x] HCA-CLN-02 确认没有自动修改 `C:\Users\admin\.codex\config.toml`。
- [x] HCA-CLN-03 确认没有引入 Electron、Node、daemon、HTTP server 或 IPC。
- [x] HCA-CLN-04 确认没有修改 taskbar probing、`SetParent`、positioning。
- [x] HCA-CLN-05 确认 wrapper 文档中保留原 notify 转发要求。
- [x] HCA-CLN-06 确认错误信息短而可诊断，不泄露 payload 原文。

## Completion Criteria

- `hook_rules.rs` 已独立承载 hook 状态推断逻辑，CLI 只做输入/输出和状态写入编排。
- `SubagentStop -> done` 已验证。
- 原有人工 hook 行为没有回归。
- Codex notify 探针有明确文档和用户确认边界。
- 文档不再暗示当前 Codex hooks TOML 格式已被真实确认。
- `cargo check` 通过。
- 所有已完成 checklist item 均有 reflection。
- 未执行的外部配置动作被明确标记为 blocked 或 awaiting user confirmation。

## Reflection / Task Summary Generation

每完成一个 checklist item，自动生成：

```text
docs/reflections/task-<task-id>-<timestamp>.md
```

模板：

```markdown
- Task: <task name>
- Encountered Problem: <problem description>
- Thought Process: <how problem was analyzed>
- Options Considered: <list of solutions considered>
- Chosen Solution: <final decision>
- Rationale: <reason for choosing this solution>
```

规则：

- task id 必须对应本 checklist，例如 `HCA-B-01`。
- 涉及外部 Codex 配置的任务必须记录是否经过用户确认。
- 涉及 payload/notify 的任务必须记录隐私边界。
- 涉及规则拆分的任务必须记录回归验证结果。
- blocked 或 skipped 的任务也要生成 reflection，说明阻塞原因和下一步条件。

## Goal Usage Recommendation

这项调整适合继续作为当前 hook 集成 goal 的子目标，不需要单独创建新的长期 goal。

Continue condition:

- 当前阶段有可验证任务，且不需要修改外部用户配置。

Blocked condition:

- 如果下一步要再次修改 Codex `notify`，必须先获得用户确认。
- Codex notify 探针没有提供足够上下文；下一步应验证正式 lifecycle hooks 配置。
