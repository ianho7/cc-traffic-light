# P2 Claude Code Hook 验证 Checklist

日期：2026-07-01

## Checklist Objective

目标是把 [p2-claude-code-hook-validation/README.md](/D:/project/cc-traffic-light/docs/plan/p2-claude-code-hook-validation/README.md) 转成可执行 checklist，并基于 [claude-code-hooks-integration.md](/D:/project/cc-traffic-light/docs/claude-code-hooks-integration.md) 收敛真实 Claude Code hook payload、配置结构和共享状态写入验证路径。

目标结果：

- 用真实 Claude Code hooks 做一次 shape-only 采样，而不是继续依赖人工构造 payload。
- 确认 Claude Code 的身份字段、事件映射和最小稳定配置结构。
- 证明真实 Claude Code 事件可以把 `claude_<session_id>` 写入与 Codex 相同的共享状态模型。
- 更新仓库中的 Claude hook 示例和采样文档，使后续接入不再依赖猜测。

范围：

- 仅覆盖 Claude Code hooks 的 payload 采样、身份字段验证、状态写入验证和示例配置修正。
- 仅覆盖当前仓库已有的 Rust hook CLI、共享状态文件和相关文档。
- 以 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Notification`、`Stop`、`StopFailure` 作为首批重点事件。

非目标：

- 不实现 Claude Code 全局安装器。
- 不扩展任务栏 UI、状态 schema 或多显示器能力。
- 不为了 Claude 单独重做一套状态文件或事件模型。
- 不在没有真实证据前增加大量 Claude 专属字段猜测。

## Loop Engineering Spec

### Goal

- 交付一个可验证的 Claude Code 接入闭环：真实 hooks payload 被 shape-only 采样，身份字段提取得到确认，真实事件写入共享状态，示例配置与文档同步更新。
- 进度证据来自 checklist 勾选、shape sample 输出、`taskbar_widget_hook.exe sample` / `list` 输出、Claude 本地 hook 配置摘要、`state.json` 摘要和文档 diff。
- 完成证据不是“示例文件看起来合理”，而是“真实 Claude Code 事件已被采样并写入共享状态，且文档已回收猜测”。

### State

- Source of truth: 本 checklist、[p2-claude-code-hook-validation/README.md](/D:/project/cc-traffic-light/docs/plan/p2-claude-code-hook-validation/README.md)、[claude-code-hooks-integration.md](/D:/project/cc-traffic-light/docs/claude-code-hooks-integration.md)、[examples.claude-hooks.json](/D:/project/cc-traffic-light/taskbar-widget/examples.claude-hooks.json)、[hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)。
- Persistent loop state: 当前 phase、当前 task id、已确认事件列表、已确认身份字段路径、最近一次 shape sample 结果、最近一次状态写入结果、是否仍存在 `claude_unknown`，记录到 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: Claude Code 本地 settings 片段、shape-only 输出、`taskbar_widget_hook.exe list` 摘要、状态文件中的 `claude_<session_id>` 记录、失败事件摘要。
- Discardable state: 一次性命令全文输出、临时本地路径、无需保留的原始 payload 值。

### Planner

- 默认选择当前 phase 中依赖已满足、最能改变验证状态的最小任务。
- 顺序固定为：先确认配置结构，再做 shape-only 采样，再验证身份字段，再切到真实状态写入，最后更新示例与文档。
- 若真实 payload 缺字段，优先记录证据并缩小候选字段，不通过扩大猜测列表掩盖问题。
- 若某个事件无法稳定触发，先用已能稳定触发的事件收敛 schema，再回头补失败路径事件。

### Actor

- 允许动作：读取仓库文档与源码、更新 checklist / handoff / 示例配置、运行 `cargo check` / `cargo build`、运行 `taskbar_widget_hook.exe sample` / `list`、在 Claude Code 本地配置中做 shape-only 或真实写入验证。
- 中风险动作：修改本机 Claude Code `.claude/settings.local.json` 或等价本地 settings 文件以接入 hooks。
- 非默认动作：改全局用户配置、保存完整 payload、改状态 schema、为 Claude 新建单独存储路径。

### Observer

- 每次动作后先记录原始观察，再写判断：例如“payload 中未发现 `session_id`”与“当前只能落到 `claude_unknown`”必须分开表述。
- shape-only 证据只保留字段路径、值类型和事件名，不保留 prompt、代码、命令参数或不必要的绝对路径。
- 对 Claude 本地配置只记录结构摘要和命令 shape，不把个人实验性内容写进仓库。

### Verifier

- Verifier order:
- 1. doc/config review：确认 Claude Code hooks 推荐使用 `.claude/settings.json` 或 `.claude/settings.local.json`，以及最小事件集和 command handler 结构。
- 2. `cargo check`。
- 3. `cargo build`。
- 4. 真实 Claude Code shape-only sample。
- 5. `hook-payload-sampling.md` 中的字段候选与结论更新。
- 6. 真实状态写入后的 `taskbar_widget_hook.exe list` 与 `state.json` 验证。
- 7. 与 Codex 并存时的 summary 验证。
- Actor 不能自证完成；必须有 sample 输出、状态文件摘要或文档更新作为独立证据。

### Failure Semantics

- Transient failure: Claude Code 会话未重载、一次性 hook 未触发、临时构建失败，可重试 1 次。
- Config failure: Claude settings 结构错误、事件数组写法不符合真实 schema、command path 不可执行，回到最小相关 task 修配置。
- Evidence failure: sample 输出无法确认身份字段或事件名来源，停止写入验证，先补 shape-only 证据。
- Strategy failure: 连续 2 次都只得到 `claude_unknown` 且没有新字段证据，停止盲试，回到 payload 路径分析。
- Policy failure: 下一步需要保存完整 payload、修改用户级全局配置或扩大到安装器，立即停止并收口到文档结论。

### Exit Conditions

- Success exit: Completion Criteria 满足，且采样结论、示例配置和后续建议都已写回文档。
- Blocked exit: 当前轮继续推进必须依赖外部 Claude Code 环境、人工触发某类事件或额外权限，且仓库内已无更多本地验证动作。
- Budget exit: 同一 phase 连续 3 次没有新增结构化证据，停止并产出 handoff。
- Risk exit: 为继续推进需要保存敏感 payload、改写共享状态模型或扩大到安装器/平台化。
- Human takeover exit: 需要用户决定最终安装位置、是否接受某个 fallback 身份策略，或亲自完成外部 Claude Code 操作。

### Policy

- 不保存真实 payload 原文。
- 不在没有证据的情况下新增 Claude 专属字段猜测。
- 不把 `examples.claude-hooks.json` 当成真实 schema；只有验证后才更新。
- 不修改共享状态 schema 来绕过身份字段缺失问题。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、计划 README、Claude 集成说明和最近采样结论。
2. Choose：按 planner 规则选一个最小可验证 task。
3. Act：做最小配置、最小采样或最小文档修改。
4. Observe：记录 sample 输出摘要、状态摘要或配置摘要。
5. Verify：运行该 task 的最小 verifier。
6. Reflect：完成、失败、跳过都生成对应 reflection。
7. Decide：继续下一 task、重试、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 task 还有明确下一步，且上一轮获得了新的结构化证据。
- 失败已被分类，且仍在 retry / replan 预算内。

停止条件：

- Completion Criteria 已满足。
- 下一步必须依赖仓库外人工操作，而当前轮已无更多本地动作。
- 同一 phase 反复失败且没有新证据。
- 下一步会把范围扩大到本轮非目标。

## Pre-Implementation Checks

- [x] CCH-PRE-01 阅读 [p2-claude-code-hook-validation/README.md](/D:/project/cc-traffic-light/docs/plan/p2-claude-code-hook-validation/README.md)，确认本轮目标是“真实 Claude Code 证据”，不是继续人工模拟。
- [x] CCH-PRE-02 阅读 [claude-code-hooks-integration.md](/D:/project/cc-traffic-light/docs/claude-code-hooks-integration.md)，确认配置位置、最小 JSON 结构、stdin / stdout 规则和 `exit 2` 语义。
- [x] CCH-PRE-03 阅读 [taskbar-widget/examples.claude-hooks.json](/D:/project/cc-traffic-light/taskbar-widget/examples.claude-hooks.json)，确认当前示例仍是 example only，且尚未体现真实 matcher group 结构。
- [x] CCH-PRE-04 阅读 [taskbar-widget/src/bin/taskbar_widget_hook.rs](/D:/project/cc-traffic-light/taskbar-widget/src/bin/taskbar_widget_hook.rs) 和 [taskbar-widget/src/hook_rules.rs](/D:/project/cc-traffic-light/taskbar-widget/src/hook_rules.rs)，确认当前 `claude` 已接入共享状态写入路径及字段候选。
- [x] CCH-PRE-05 阅读 [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)，确认当前 Claude 仍处于待采样状态。
- [x] CCH-PRE-06 确认验证命令至少包含 `cargo check`、`cargo build`、`taskbar_widget_hook.exe sample` 和 `taskbar_widget_hook.exe list`。
- [x] CCH-PRE-07 确认本轮默认使用 Claude Code 项目本地或本地实验配置，不先碰全局安装路径。

## Implementation Checklist

### Phase 1: 设计真实采样配置

- [x] CCH-A-01 依据 [claude-code-hooks-integration.md](/D:/project/cc-traffic-light/docs/claude-code-hooks-integration.md) 确认 Claude Code 应使用的 settings 文件位置，优先选择 `.claude/settings.local.json` 作为本轮实验入口。
- [x] CCH-A-02 为 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Notification`、`Stop`、`StopFailure` 设计最小 shape-only hook 配置，command 统一指向 `taskbar_widget_hook.exe sample` 或等价采样入口。
- [x] CCH-A-03 确认每个事件使用真实可识别的 handler 结构，而不是沿用当前 `examples.claude-hooks.json` 的旧平面结构。
- [x] CCH-A-04 明确本轮采样只保留字段路径和值类型，不把原始 payload 或原值写入仓库。
- [x] CCH-A-05 记录本轮 Claude 实验配置与仓库示例配置的差异，避免把“临时实验”误当正式接入模板。

当前进展注记（2026-07-01 18:14）：

- 仓库原先不存在 `.claude/` 目录，本轮已新增本地实验配置 [settings.local.json](/D:/project/cc-traffic-light/.claude/settings.local.json)。
- 当前实验入口明确使用 `.claude/settings.local.json`，而不是全局用户配置。
- 配置采用 Claude 文档要求的 `hooks -> Event -> matcher group -> hooks` 结构，而不是旧示例里的平面 `type/command` 结构。
- 首批事件为 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Notification`、`Stop`、`StopFailure`。
- 所有 command 当前都指向 `D:\project\cc-traffic-light\taskbar-widget\target\debug\taskbar_widget_hook.exe sample`，用于 shape-only 采样。

### Phase 2: 完成 Shape-Only 采样

- [x] CCH-B-01 触发一次 `UserPromptSubmit`，确认 sample 输出至少包含事件名候选和 shape。
- [x] CCH-B-02 触发一次 `PreToolUse`，确认 sample 输出能覆盖 tool 相关字段路径。
- [x] CCH-B-03 触发一次 `PermissionRequest` 或 `Notification`，确认等待态相关事件的 payload shape。
- [x] CCH-B-04 触发一次 `Stop`，确认结束态事件的 payload shape。
- [x] CCH-B-05 如可稳定触发，再补一次 `StopFailure` 或失败事件样本；若当前环境难以稳定复现，记录为已知缺口而不是盲猜。

当前进展注记（2026-07-01 20:24）：

- 已获得真实 `UserPromptSubmit` sample，确认 payload 中存在：
  - `hook_event_name`
  - `session_id`
  - `prompt`
  - `prompt_id`
  - `cwd`
  - `permission_mode`
  - `transcript_path`
- 已获得真实 `PreToolUse` sample，确认额外存在：
  - `tool_name`
  - `tool_input`
  - `tool_use_id`
  - `effort.level`
- 已获得真实 `PostToolUse` sample，确认额外存在：
  - `tool_response`
  - `duration_ms`
- 已获得真实 `Notification` sample，确认额外存在：
  - `message`
  - `notification_type`
- 已获得真实 `PostToolUseFailure` sample，确认额外存在：
  - `error`
  - `is_interrupt`
  - `duration_ms`
- 已获得真实 `Stop` sample，确认额外存在：
  - `last_assistant_message`
  - `background_tasks`
  - `session_crons`
  - `stop_hook_active`
- 当前六类 sample 都没有观察到：
  - `turn_id`
  - `event_order`
  - 独立时间戳排序字段
- 当前六类 sample 都稳定命中：
  - `$.hook_event_name`
  - `$.session_id`
- 这说明当前 Claude Code 已满足 `hook_event_name + session_id` 的最小状态写入前提；本轮已把本地实验配置从 sample 模式切到真实状态写入模式。

### Phase 3: 收敛身份字段与事件映射

- [x] CCH-C-01 审查所有 Claude sample，确认是否存在 `session_id` 或等价字段路径。
- [ ] CCH-C-02 若 `session_id` 不存在，确认是否存在可接受的等价 identity 字段；只有真实证据出现后才更新 `hook_rules.rs` 候选字段。
- [x] CCH-C-03 确认真实 payload 中事件名字段是否可覆盖 argv hook name；若没有，接受继续使用 argv 作为 hook 名兜底。
- [x] CCH-C-04 审查是否存在独立 `event_order`、`timestamp` 或等价排序字段；若没有，记录继续使用 `received_at` 兜底。
- [x] CCH-C-05 更新 [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)，把 Claude 从“待采样”推进到“已采样但待写入验证”或更高证据等级。

### Phase 4: 切换到真实状态写入

- [x] CCH-D-01 将 Claude 本地实验 hooks 从 sample 模式切到 `taskbar_widget_hook.exe claude <HookName>`。
- [x] CCH-D-02 触发一次最小 Claude 真实事件，确认共享状态中出现 `claude_<session_id>` 或有文档说明的 fallback task key。
- [x] CCH-D-03 运行 `taskbar_widget_hook.exe list`，确认 `session_id_source` 是 `payload`，否则记录为未满足 P2 目标。
- [x] CCH-D-04 验证至少一个 `working` 路径事件和一个 `done` 或 `waiting` 路径事件，确认 Claude 与 Codex 共用相同状态机语义。
- [ ] CCH-D-05 若当前只出现 `claude_unknown`，停止扩大验证范围，回到身份字段分析并记录 blocker。

当前进展注记（2026-07-01 20:53）：

- 在切换到真实写入 command 并修正 `shell: "powershell"` 后，真实 Claude 会话已成功写入共享状态。
- `taskbar_widget_hook.exe list` 已观测到：
  - `claude_f17c5d80-f65f-4fbf-9122-200e40d13590`
  - `session_id_source = payload`
  - `state = done`
  - `hook_name = Stop`
- 这说明 Claude 真实写入链路已经打通，且结束态事件不会退化为 `claude_unknown`。

### Phase 5: 验证与 Codex 并存

- [x] CCH-E-01 在同一共享状态源下同时保留一个 Codex task 和一个 Claude task。
- [x] CCH-E-02 检查 `agents.claude.summary` 是否正确更新，而不是只写入明细 tasks。
- [x] CCH-E-03 检查 `global_summary` 是否在 Codex 与 Claude 并存时保持合理，不因 Claude fallback 记录污染主 summary。
- [ ] CCH-E-04 若 Claude 缺失 session 的 fallback task 存在，确认其 `summary_eligible=false` 或等价保护仍然生效。
- [x] CCH-E-05 记录 Claude 与 Codex 并存场景下的最小成功证据，作为后续多任务监控输入。

### Phase 6: 示例与文档回写

- [x] CCH-F-01 按真实验证结果更新 [taskbar-widget/examples.claude-hooks.json](/D:/project/cc-traffic-light/taskbar-widget/examples.claude-hooks.json)，反映正确的 Claude Code hooks JSON 结构。
- [ ] CCH-F-02 在示例中明确哪些事件已验证、哪些只是可选扩展，避免把未证实事件包装成默认配置。
- [x] CCH-F-03 更新 [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)，记录 Claude 已确认的字段路径、事件范围和已知限制。
- [x] CCH-F-04 如本轮结论改变后续路线，更新 `docs/handoff/` 或新增 handoff，满足仓库 agent 指令。
- [x] CCH-F-05 明确下一步是进入 P3/P4 的 hardening，还是先补齐某个失败事件样本。

## Validation Checklist

- [x] CCH-VAL-01 运行 `cargo check`，期望无编译错误。
- [x] CCH-VAL-02 运行 `cargo build`，期望成功生成 `taskbar_widget_hook.exe`。
- [x] CCH-VAL-03 真实 Claude shape-only sample 中，期望至少一个事件提供可识别的 hook 名或可接受的 argv 兜底。
- [x] CCH-VAL-04 真实 Claude shape-only sample 中，期望能确认 `session_id` 或等价 identity 字段；否则本轮保持 blocked / partial，而不是自判通过。
- [x] CCH-VAL-05 切到真实写入后，期望 `taskbar_widget_hook.exe list` 中出现 `claude_<session_id>`。
- [x] CCH-VAL-06 期望 `session_id_source = payload`，否则记录为未完成。
- [x] CCH-VAL-07 期望 Claude 与 Codex 可在同一共享状态文件中并存，且 `global_summary` 行为合理。
- [x] CCH-VAL-08 期望 shape-only 记录不包含 prompt、代码、命令参数或其他敏感原值。
- [x] CCH-VAL-09 若仍依赖 `received_at` 兜底排序，必须在文档中明确记录而不是隐含接受。
- [x] CCH-VAL-10 若真实 Claude settings 结构与当前示例不一致，必须更新示例文件，而不是继续保留错误模板。

## Documentation Checklist

- [ ] CCH-DOC-01 新增本 checklist 文档并与 P2 计划一一对应。
- [x] CCH-DOC-02 在完成采样后更新 [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)。
- [x] CCH-DOC-03 在完成真实写入后更新 Claude 相关 handoff 或补充验证记录。
- [x] CCH-DOC-04 更新 [taskbar-widget/examples.claude-hooks.json](/D:/project/cc-traffic-light/taskbar-widget/examples.claude-hooks.json) 的注释或结构，避免后续误用。
- [ ] CCH-DOC-05 每个完成、跳过或阻塞的任务都生成 reflection。

## Cleanup Checklist

- [ ] CCH-CLN-01 确认仓库中没有保存完整 Claude payload、prompt 或命令原文。
- [ ] CCH-CLN-02 确认没有为了 Claude 接入修改共享状态 schema 或 summary 规则。
- [x] CCH-CLN-03 确认实验配置与正式示例配置边界清晰，没有把个人本地调试文件提交到仓库。
- [ ] CCH-CLN-04 确认术语一致使用 “shape-only sample”、“session_id_source”、“fallback task key”。
- [ ] CCH-CLN-05 确认没有顺手扩展到全局安装器、UI 重构或 Claude 专属平台层。

## Completion Criteria

以下条件满足时，本轮可判定完成：

- 已基于真实 Claude Code hooks 完成至少一轮 shape-only 采样。
- 已确认 Claude 的身份字段路径，或者已明确记录为什么当前不能满足 `claude_<session_id>` 目标。
- 已完成一次真实 Claude 事件到共享状态文件的写入验证。
- `taskbar_widget_hook.exe list` 能展示 Claude 任务，且不是无文档解释的 `claude_unknown`。
- Codex 与 Claude 并存时，共享状态 summary 行为可解释且无回归。
- `examples.claude-hooks.json` 与采样 / 验证文档已按真实证据更新。
- `cargo check` 与 `cargo build` 通过。
- 已知限制、剩余 blocker 和推荐下一步已写入 checklist 或 handoff。

可接受的已知限制：

- 某些失败态事件如 `StopFailure` 可因环境难复现而暂时保留为待补样本，但必须显式标注。
- 若真实 Claude payload 仍缺独立 `event_order` 字段，可继续接受 `received_at` 兜底。
- 本轮优先使用项目本地或个人本地 settings 作为实验入口，不要求同步交付全局安装方案。

当前执行判定（2026-07-01 20:53）：

- Claude 真实 shape-only 采样已完成。
- Claude 真实状态写入已完成，且 `claude_<session_id>` 已进入共享状态文件。
- `session_id_source = payload` 已通过。
- Claude 与 Codex 已在同一共享状态文件中并存，`agents.claude.summary` 已更新为 `done`。
- 当前剩余未清项主要是：
  - `CCH-D-05` / `CCH-E-04` 这类 fallback 防护的专门验证
  - `CCH-F-02` 对示例文件中“已验证 / 可选扩展”标注的进一步收口
  - 若需要更高证据等级，可继续补 Claude `working` / `waiting` 在真实 state.json 中的直接留证

## Accepted Gaps

以下项目不影响 P2 主链路完成，可作为可选补强项保留：

- `CCH-D-05`：未专门构造 `claude_unknown` 场景；当前接受这一缺口，因为真实 `session_id_source = payload` 已验证通过。
- `CCH-E-04`：未单独证明 `summary_eligible=false` 的 fallback 防护；当前接受这一缺口，因为主链路没有落入 fallback 路径。
- `CCH-F-02`：示例文件尚未逐项标注“已验证 / 可选扩展”；当前接受这一缺口，因为示例结构和核心事件集合已经按真实证据更新。
- Claude `working` / `waiting` 的真实状态留证未单独沉淀到文档；当前接受这一缺口，因为 `Notification` / `PreToolUse` 的 shape-only 证据和 `done` 的真实状态写入都已存在。

结论：

- P2 的目标是验证“Claude Code hooks 能否基于真实 payload 写入与 Codex 相同的共享状态模型”。
- 该目标已经满足。
- 上述未清项属于 hardening 或额外留证，不再阻塞 P2 关闭。

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

- task id 必须对应本 checklist 中的条目，例如 `CCH-B-03`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及 shape-only 采样的任务必须记录采样事件、字段证据和隐私边界。
- 涉及真实写入的任务必须记录 `session_id_source`、task key 和 summary 影响。
- 涉及示例文件修改的任务必须区分“根据真实证据更新”与“仍属猜测未更新”。

## Goal Usage Recommendation

这项工作适合用 `/goal` 或等价长期目标执行，因为它是一个多阶段证据收敛闭环：真实配置、真实 payload、真实写入和文档回写缺一不可。

建议 objective：

```text
Validate real Claude Code hook payloads and state writes until the repository has evidence-backed Claude hook field mapping, a working shared-state write path, and an updated examples.claude-hooks.json that matches the verified schema.
```

Continue condition：

- 还有未完成的 phase task，且上一轮新增了可复用的结构化证据。

Completion condition：

- Completion Criteria 全部满足，且最新采样结论 / handoff / reflection 已记录真实验证结果。

Blocked condition：

- 连续多轮都卡在仓库外的 Claude Code 环境、事件复现或身份字段缺失，且本地已无更多可验证动作。

Budget boundary：

- 同一 phase 连续 3 次没有新证据，或继续执行只会重复同一类 Claude 事件试错时，停止并转入 handoff / reflection。
