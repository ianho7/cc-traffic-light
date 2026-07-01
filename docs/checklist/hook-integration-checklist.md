# Claude Code / Codex Hook 集成执行 Checklist

## Checklist Objective

目标是在 `taskbar-widget` 中实现 Claude Code 和 Codex hook 状态监控闭环，为任务栏红绿灯组件提供可靠的状态依赖。

调整说明：2026-07-01 起，Codex 正式接入策略受 [hook-adjustment-checklist.md](/D:/project/cc-traffic-light/docs/checklist/hook-adjustment-checklist.md) 覆盖。Codex `notify` 已验证为低保真兼容入口；正式 Codex 主状态路径应按 [codex-lifecycle-hooks-validation.md](/D:/project/cc-traffic-light/docs/checklist/codex-lifecycle-hooks-validation.md) 验证 lifecycle hooks。

范围：

- 采样真实 hook payload 的脱敏字段结构。
- 实现 task-aware 状态模型，支持多个并发 task。
- 使用 Win32 named mutex + 原子 JSON 写入保护状态文件。
- 实现 hook CLI 和 debug CLI。
- 让任务栏组件读取 `global_summary` 并重绘。
- 提供示例配置、验证命令和渲染能力探针。

非目标：

- 不自动修改用户 Claude Code / Codex 配置。
- 不做 session 列表 UI、settings UI、托盘菜单、音效或 dashboard。
- 不做进程检测。
- 不保存完整 hook payload。
- 不把图片渲染能力放入 hook MVP 主路径。

## Loop Engineering Spec

### Goal

- 交付一个可验证的 hook 状态闭环：真实或人工 hook 事件进入 Rust hook CLI，更新 task-aware 状态文件，任务栏组件读取 `global_summary` 并重绘。
- 进度证据来自 checklist item 完成状态、相关文件 diff、验证命令输出、采样记录和 task reflection。
- 完成证据是 `cargo check`、`cargo build`、人工 hook 验证、多 task 验证、乱序验证、stale 验证和 widget 重绘验证全部通过。

### State

- Source of truth: 本 checklist、`docs/plan/hook-integration/01-mvp-plan.md`、`docs/plan/hook-integration/02-grill-decisions-and-adr.md`。
- Persistent loop state: 当前 phase、当前 task id、触碰文件、最近验证命令、失败原因、下一步假设，记录到对应 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: 命令输出摘要、生成的采样文档、状态 JSON 样例、验证文档、构建结果。
- Discardable state: 临时终端输出、一次性采样原始 payload、无关探索笔记。

### Planner

- 默认选择当前 phase 中编号最小、依赖已满足、能改变验证状态的任务。
- Phase 0 是 gate：没有确认真实 payload shape 前，不进入完整 hook 解析实现。
- 每完成一个 phase，先运行该 phase 的最小 verifier，再决定是否进入下一 phase。
- 出现重复失败时不继续盲改；必须更新 hypothesis，并在 reflection 中记录策略变化。

### Actor

- 允许动作：读取源码和文档、用 `apply_patch` 做小步编辑、运行 `cargo check` / `cargo build`、执行人工 hook CLI 验证、更新 docs/checklist/reflections。
- 高风险动作：修改用户全局 Claude Code / Codex 配置、删除状态文件、重构 taskbar probing、引入 daemon/database/HTTP server。
- 高风险动作不属于本 checklist 的默认执行范围。

### Observer

- 每次动作后记录观察：文件 diff、构建结果、状态文件内容摘要、失败类型。
- 观察和判断分开写：先记录“发生了什么”，再写“这意味着什么”。
- 采样模式只能记录脱敏 payload shape，不能保存 prompt、代码、命令参数或完整路径。

### Verifier

- Verifier order:
- 1. 当前 task 的 focused check。
- 2. `cargo check`。
- 3. `cargo build`。
- 4. 人工 hook CLI 验证。
- 5. widget 运行时重绘验证。
- 6. 渲染能力探针验证。
- Actor 不能自证完成；必须有命令输出、状态文件、文档或 UI 观察作为证据。

### Failure Semantics

- Transient failure: 命令偶发失败或文件短暂锁冲突，可重试 1 次。
- Code failure: 编译错误、测试失败、状态 JSON 错误，先定位最小相关 task，再修复。
- Strategy failure: 同一逻辑失败重复 2 次，停止当前策略并重读计划/ADR 后重新拆分。
- Environment failure: 缺少真实 hook 环境、权限不足、GUI 不可见，记录 blocked evidence，并切换到可验证的人工路径。
- Policy failure: 需要修改外部用户配置或执行破坏性操作时停止并请求用户确认。

### Exit Conditions

- Success exit: Completion Criteria 全部满足，且相关 reflection 已生成。
- Blocked exit: Phase 0 无法获得真实 payload，或验证必须依赖用户外部配置/桌面观察。
- Budget exit: 同一 phase 出现 3 次无新证据的失败，停止并生成 handoff/reflection。
- Risk exit: 下一步需要越过非目标范围，例如自动改外部 hook 配置、引入 daemon 或重构 taskbar host。
- Human takeover exit: 产品语义无法由代码推断，例如真实 payload 没有 `session_id` 且需要用户决定 fallback。

### Policy

- 不保存完整 hook payload。
- 不自动修改用户 Claude Code / Codex 配置。
- 不执行破坏性文件操作，除非用户明确要求。
- 不将图片渲染探针并入 hook MVP 热路径。
- 不在 hook 状态实现中顺手修改 taskbar probing、SetParent 或 Win11 visibility strategy。

## Runtime Loop Protocol

每轮执行必须遵循：

1. Inspect: 读取当前 task、相关文件、上轮 reflection 和最新验证结果。
2. Choose: 按 planner 规则选择一个最小可验证 task。
3. Act: 做最小编辑或验证动作。
4. Observe: 记录原始证据路径或输出摘要。
5. Verify: 运行该 task 的最小 verifier。
6. Reflect: 完成或失败都生成 task reflection。
7. Decide: 继续下一 task、重试、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 task 有明确下一步，并且上轮产生了新证据。
- verifier 失败但失败类型已被分类，且未超过 retry/replan 限制。

停止条件：

- Completion Criteria 满足。
- 继续需要外部输入或权限。
- 同一 phase 反复失败且没有新证据。
- 下一步会扩大 MVP 范围。

## Pre-Implementation Checks

- [x] HC-PRE-01 确认 `taskbar-widget/src/main.rs`、`taskbar-widget/src/taskbar.rs`、`taskbar-widget/src/win32.rs` 当前职责边界。
- [x] HC-PRE-02 阅读 [01-mvp-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/01-mvp-plan.md) 和 [02-grill-decisions-and-adr.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/02-grill-decisions-and-adr.md)。
- [x] HC-PRE-03 确认 `taskbar-widget/Cargo.toml` 需要新增的依赖只包含 MVP 必需项，例如 `serde`、`serde_json`。
- [x] HC-PRE-04 确认 Win32 named mutex 需要的 `windows` crate feature 是否已启用，不足时只补最小 feature。
- [x] HC-PRE-05 确认验证命令为 `cargo check`、`cargo build`，以及人工 hook CLI 调用。
- [x] HC-PRE-06 确认不在本轮修改 taskbar probing、SetParent 策略或 Win11 可见性诊断逻辑。

## Implementation Checklist

### Loop Gate: Phase Entry Rules

- [x] HC-GATE-00 每个 phase 开始前确认上一 phase 的 verifier 和 reflection 已完成。
- [x] HC-GATE-01 每个 phase 开始前记录当前 loop state：当前 phase、当前 task、最近验证、剩余范围。
- [x] HC-GATE-02 每个 phase 开始前确认没有未解决的 policy/risk exit 条件。

### Phase 0: 真实 Payload 脱敏采样

- [x] HC-P0-01 新增 hook CLI 的 `sample` 模式或采样环境变量，只输出 payload shape，不保存完整 payload。
- [x] HC-P0-02 在采样输出中识别 hook name 字段候选，例如 `hook_event_name`、`hookName`、`eventName`。
- [ ] HC-P0-03 在采样输出中识别 `session_id` 字段位置，确认 Codex 和 Claude Code 不同 hook 类型是否都携带该字段。
- [ ] HC-P0-04 在采样输出中识别事件时间或序号字段，作为 `event_order` 优先来源。
- [x] HC-P0-05 验证 Codex 成功路径输出 `{}`，Claude 成功路径保持静默。
- [x] HC-P0-06 记录采样结论到 `docs/checklist/hook-payload-sampling.md`。

### Phase 1: 共享状态模型与持久化

- [x] HC-P1-GATE 确认 Phase 0 已记录真实 payload shape；如果没有真实环境，记录 blocked reason 并使用人工 payload 继续最小实现。
- [x] HC-P1-01 新增 `taskbar-widget/src/agent_state.rs`，定义 `AgentId`、`AgentState`、`TaskKey`、`TaskStatus`、`HookSummary`、`HookMonitorState`。
- [x] HC-P1-02 将状态 schema 设计为包含 `tasks`、`global_summary`、`agents.codex.summary`、`agents.claude.summary`。
- [x] HC-P1-03 在 `TaskStatus` 中加入 `event_order` 和 `event_order_source`，用于拒绝旧事件覆盖。
- [x] HC-P1-04 在 `HookSummary` 中加入 `state`、`active_task_count`、`has_stale`、`stale_task_count`、`highest_priority_task`、`updated_at`。
- [x] HC-P1-05 实现 `TaskKey = agent_name + "_" + session_id`；缺失 `session_id` 时进入 `_unknown` 诊断 task。
- [x] HC-P1-06 实现 `%APPDATA%\CcTrafficLight\state.json` 默认路径和 `TASKBAR_WIDGET_STATE_HOME` 测试覆盖路径。
- [x] HC-P1-07 使用 Win32 named mutex 保护 read-modify-write；锁超时必须返回可诊断错误。
- [x] HC-P1-08 实现临时文件 + rename 的原子 JSON 写入。
- [x] HC-P1-09 实现状态文件损坏恢复：rename 为 `state.corrupt.<timestamp>.json` 后创建默认状态。
- [x] HC-P1-10 实现 conservative TTL：`done` 10 分钟可清理，`error` 30 分钟可清理，`waiting` 24 小时 stale，`working` 30 分钟 stale。
- [x] HC-P1-11 实现 summary 聚合规则：`error > waiting > working > done > idle`，stale task 不参与主状态但设置诊断字段。

### Phase 2: Hook CLI

- [x] HC-P2-GATE 确认 Phase 1 状态模型、named mutex、原子写入和 summary 聚合已有 focused verifier。
- [x] HC-P2-01 新增 `taskbar-widget/src/bin/taskbar_widget_hook.rs`，支持 `<codex|claude> <HookName>` 调用格式。
- [x] HC-P2-02 从 `stdin` 读取完整 JSON；空输入仅允许人工验证路径视作 `{}`。
- [x] HC-P2-03 校验 agent id，非法 agent 返回非 0 且不写状态。
- [x] HC-P2-04 解析 hook name，优先使用 argv，其次使用采样确认的 payload 字段。
- [x] HC-P2-05 解析 `session_id` 并生成 `TaskKey`；缺失时写入 `_unknown` 诊断 task 且不参与正常 summary。
- [x] HC-P2-06 解析 payload 时间或序号生成 `event_order`；缺失时使用 hook CLI 接收时间。
- [x] HC-P2-07 实现 hook 到状态映射：`UserPromptSubmit`/`PreToolUse` 为 `working`，`PermissionRequest`/`Notification` 为 `waiting`，`StopFailure` 为 `error`。
- [x] HC-P2-08 实现 `Stop` 特殊处理：previous state 为 `waiting` 时保持 `waiting`；文本像等待用户输入时设为 `waiting`；否则设为 `done`。
- [x] HC-P2-09 实现事件顺序保护：只接受 `event_order >= current.event_order` 的事件，旧事件只写诊断。
- [x] HC-P2-10 写入失败、锁超时、非法 JSON、非法 agent 均返回非 0，并在 stderr 输出短错误。
- [x] HC-P2-11 确保状态文件不保存完整 payload，只保存脱敏摘要、hook name、短 message 和必要状态字段。

### Phase 2A: Debug 状态 CLI

- [x] HC-P2A-GATE 确认 Phase 2 的正常 hook 路径和失败退出语义已验证。
- [x] HC-P2A-01 增加 `set <task_key> <state>`，用于手工设置 task 状态。
- [x] HC-P2A-02 增加 `clear <task_key>`，用于手工清理卡住 task。
- [x] HC-P2A-03 增加 `list`，输出当前 tasks、`global_summary` 和按-agent summary。
- [x] HC-P2A-04 所有 debug 命令必须走同一套 named mutex、schema、summary 和原子写入逻辑。
- [x] HC-P2A-05 debug CLI 明确标记为调试能力，不作为用户功能或设置界面。

### Phase 3: 任务栏组件轮询与绘制

- [x] HC-P3-GATE 确认 debug CLI 可构造 `global_summary`，否则不要开始 widget 消费逻辑。
- [x] HC-P3-01 在 `taskbar-widget/src/main.rs` 中添加 Win32 timer，默认 1000ms 轮询。
- [x] HC-P3-02 在 `WM_TIMER` 中读取状态文件，但只在 `global_summary` 变化时触发 `InvalidateRect`。
- [x] HC-P3-03 确保 `WM_PAINT` 不做磁盘 IO，只使用内存中的 latest summary。
- [x] HC-P3-04 将固定 `MODULE_TEXT` 绘制替换为基于 `global_summary.state` 的文本和颜色。
- [x] HC-P3-05 stale task 不参与主状态，但在诊断日志中记录 `has_stale` 和 `stale_task_count`。
- [x] HC-P3-06 保持 taskbar probing、parent strategy、positioning 逻辑不变。

### Phase 4: 文档、样例与验证脚本

- [x] HC-P4-GATE 确认 hook CLI、debug CLI 和 widget 消费路径的当前限制已记录。
- [x] HC-P4-01 新增 `taskbar-widget/examples.codex-hooks.toml`，包含真实 hook 调用示例。
- [x] HC-P4-02 新增 `taskbar-widget/examples.claude-hooks.json`，包含真实 hook 调用示例。
- [x] HC-P4-03 更新 `taskbar-widget/README.md`，记录 hook CLI、debug CLI、状态路径和验证命令。
- [x] HC-P4-04 新增 `docs/checklist/hook-integration-validation.md`，记录人工验证步骤。
- [x] HC-P4-05 文档明确不自动修改用户外部配置。
- [x] HC-P4-06 文档明确不保存完整 payload。

### Phase 5: 渲染能力探针

- [ ] HC-P5-GATE 确认 hook 状态闭环已可验证；图片探针不能阻塞 hook MVP 完成。
- [ ] HC-P5-01 新增 `taskbar-widget/src/render_probe.rs` 或等价隔离代码路径。
- [ ] HC-P5-02 验证 GDI 文本、背景色和简单形状的稳定绘制。
- [ ] HC-P5-03 验证背景文字：低对比度 `DrawTextW` 后叠加前景状态文本。
- [ ] HC-P5-04 验证 bitmap 图片加载和缩放，优先 Win32/GDI 路径，不进入主 UI 热路径。
- [ ] HC-P5-05 记录透明 PNG、DPI 缩放、资源释放、闪烁和任务栏裁剪结果。
- [ ] HC-P5-06 将结论保存到 `docs/checklist/hook-rendering-capability.md`。

## Validation Checklist

- [x] HC-LOOP-VAL-01 每轮验证前记录当前 hypothesis 和预期结果。
- [x] HC-LOOP-VAL-02 每轮验证后记录实际结果、失败分类和下一步决定。
- [x] HC-VAL-01 运行 `cargo check`，期望无编译错误。
- [x] HC-VAL-02 运行 `cargo build`，期望生成主程序和 hook CLI。
- [x] HC-VAL-03 使用 sample 模式验证 payload shape 输出不包含 prompt、代码、命令参数或完整路径。
- [x] HC-VAL-04 人工触发 `UserPromptSubmit`，期望目标 task 进入 `working`。
- [x] HC-VAL-05 人工触发 `PermissionRequest`，期望目标 task 进入 `waiting`。
- [x] HC-VAL-06 人工触发 `StopFailure`，期望目标 task 进入 `error`。
- [x] HC-VAL-07 人工触发 `Stop`，期望非等待场景进入 `done`，等待场景保持或进入 `waiting`。
- [x] HC-VAL-08 构造两个不同 `session_id`，期望生成不同 `TaskKey` 且不互相覆盖。
- [x] HC-VAL-09 构造缺失 `session_id` 的 payload，期望进入 `_unknown` 诊断 task 且不参与正常 summary。
- [x] HC-VAL-10 构造乱序事件，期望旧事件不覆盖新状态。
- [x] HC-VAL-11 构造 stale task，期望不参与 `global_summary.state`，但 `has_stale = true`。
- [x] HC-VAL-12 人工损坏 `state.json`，期望生成 `state.corrupt.<timestamp>.json` 并恢复默认状态。
- [x] HC-VAL-13 并发触发 20-50 次 hook，期望无 JSON 损坏、无明显延迟。
- [x] HC-VAL-14 构造 10 个并发 task，期望状态文件小于 64KB 且 summary 正确。
- [x] HC-VAL-15 使用 debug CLI `set/clear/list`，期望状态、summary 和重绘链路正确。
- [ ] HC-VAL-16 启动 widget 后触发状态变更，期望一个 timer 周期内重绘。
- [ ] HC-VAL-17 状态未变化时，期望不连续触发无意义重绘。

## Documentation Checklist

- [x] HC-DOC-01 更新 [hook-integration README](/D:/project/cc-traffic-light/docs/plan/hook-integration/README.md)，关联本 checklist。
- [x] HC-DOC-02 更新 `taskbar-widget/README.md`，记录 hook 集成使用方式。
- [x] HC-DOC-03 添加 Codex 和 Claude Code 示例配置文件。
- [x] HC-DOC-04 添加 payload 采样记录文档。
- [x] HC-DOC-05 添加 hook 集成验证文档。
- [ ] HC-DOC-06 添加渲染能力探针结果文档。

## Cleanup Checklist

- [x] HC-CLN-01 移除临时采样输出和实验日志。
- [x] HC-CLN-02 确认没有保存完整 hook payload。
- [x] HC-CLN-03 确认没有自动修改用户外部 Claude Code / Codex 配置。
- [x] HC-CLN-04 确认错误信息短而可诊断，不泄露 payload 原文。
- [x] HC-CLN-05 确认新增文件命名与现有 `docs/checklist/` 风格一致。
- [x] HC-CLN-06 确认没有顺手修改 taskbar probing、SetParent、positioning 等无关逻辑。

## Completion Criteria

- Loop state 显示所有 phase gate 已通过，或未通过项已有明确 accepted limitation。
- Phase 0 已确认真实 payload 字段，且采样不保存敏感原文。
- Hook CLI 能按 `agent_name + "_" + session_id` 更新多个 task。
- 状态文件使用 Win32 named mutex + 原子写入，损坏时可备份恢复。
- `global_summary` 和按-agent summary 计算正确，stale 不参与主状态。
- Debug CLI `set/clear/list` 可用于验证和清理状态。
- Widget 读取 `global_summary` 并在状态变化时重绘。
- `cargo check` 和 `cargo build` 通过。
- 示例配置、验证文档和 README 链接已更新。

## Reflection / Task Summary Generation

每完成一个 checklist item，自动在 `docs/reflections/` 生成一份反思文档：

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

- task id 必须对应 checklist item，例如 `HC-P1-07`。
- 记录真实遇到的问题和取舍；如果任务很顺利，也要说明验证依据。
- 涉及隐私、锁、事件顺序、stale、summary 的任务必须写清楚选择理由。
- 涉及失败重试或 replan 的任务必须写清楚失败分类、尝试次数和策略变化。
- 每个 phase gate 必须生成 reflection，记录是否进入下一 phase 的证据。
- 反思文档应使用当前日期时间戳，保存到 `docs/reflections/`。

## Goal Usage Recommendation

这项工作适合使用 `/goal` 或等价的长期执行目标，因为它满足多轮实现、跨阶段验证、状态需要持久化、失败需要分类恢复等条件。

建议 goal objective：

```text
Implement the Claude Code / Codex hook monitoring MVP for taskbar-widget until payload sampling, task-aware state, hook CLI, debug CLI, widget summary rendering, validation docs, and required checks are complete.
```

Goal completion condition:

- Completion Criteria 全部满足。
- 最新 reflection 记录最终验证证据。
- 没有未分类失败或未记录风险。

Goal blocked condition:

- 连续 3 轮卡在同一外部依赖，例如无法获得真实 payload 或无法观察 Win11 widget。
- 下一步需要用户确认外部配置修改、权限提升或范围扩张。
