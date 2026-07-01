# P0 Codex 状态写入验证 Checklist

日期：2026-07-01

## Checklist Objective

目标是把 [p0-codex-state-write/README.md](/D:/project/cc-traffic-light/docs/plan/p0-codex-state-write/README.md) 转成可执行 checklist，完成项目级 Codex lifecycle hooks 从 dump 模式切到真实状态写入的验证闭环。

目标结果：

- 项目 `.codex/hooks.json` 指向 `taskbar_widget_hook.exe codex <HookName>`。
- 真实 Codex 会话会把 `codex_<session_id>` 写入 `state.json`。
- 正在运行的 taskbar widget 会响应 `global_summary` 变化并重绘。

范围：

- 仅验证当前项目 `D:\project\cc-traffic-light` 的项目级 hook 配置。
- 仅覆盖 Codex 生命周期事件到共享状态文件、再到 widget 的链路。

非目标：

- 不做用户级全局安装。
- 不做 Claude Code 接入。
- 不扩展更丰富的 UI 展示、经典任务栏兼容、多显示器或安装器流程。

## Loop Engineering Spec

### Goal

- 交付一个可验证的项目级闭环：真实 Codex lifecycle hooks 触发 `taskbar_widget_hook.exe`，共享状态文件出现 `codex_<session_id>`，运行中的 widget 响应 `global_summary` 更新。
- 进度证据来自 checklist 勾选、`.codex/hooks.json` diff、`cargo check` / `cargo build` 结果、`taskbar_widget_hook.exe list` 输出、`state.json` 摘要和人工 widget 观察。
- 完成证据不是“hook 配好了”，而是状态写入和 widget 重绘都被独立验证。

### State

- Source of truth: 本 checklist、[p0-codex-state-write/README.md](/D:/project/cc-traffic-light/docs/plan/p0-codex-state-write/README.md)、现有 Codex lifecycle payload 验证文档。
- Persistent loop state: 当前 phase、当前 task id、最近一次验证结果、失败分类、下一步假设、是否已完成 `/hooks` trust，记录到 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: `.codex/hooks.json` 当前内容、构建结果摘要、`taskbar_widget_hook.exe list` 输出摘要、`state.json` 关键字段、人工 widget 观察结论。
- Discardable state: 一次性终端输出、未保存的临时观察、无需长期保留的瞬时状态文件内容。

### Planner

- 默认选择当前 phase 中编号最小、依赖已满足、能改变验证状态的 task。
- 优先顺序是：先证明 hook CLI 可执行，再切换项目 hooks，再验证真实状态写入，最后做 widget 重绘观察。
- 若某一步失败，先按失败类型分类，再决定重试、修配置、回到 dump 采样，还是停在 blocked。
- 不在每轮重新发明计划；继续以本 checklist 为执行序列，只有出现新证据时才局部 replan。

### Actor

- 允许动作：读取文档和源码、用 `apply_patch` 小步更新仓库文件、运行 `cargo check` / `cargo build`、运行 `taskbar_widget_hook.exe` debug 命令、更新 checklist/handoff/reflections。
- 中风险动作：修改 `.codex/hooks.json`、触发项目 hooks reload、依赖人工 `/hooks` review/trust 的验证。
- 非默认动作：用户级全局 hooks 安装、与本轮无关的 UI 扩展、修改状态 schema 以掩盖验证失败。

### Observer

- 每次动作后先记录原始观察，再写判断：例如“`tasks` 中出现 `codex_unknown`”与“session 提取失败”必须分开表述。
- 原始证据应尽量落到文档或 reflection：命令是否成功、hook 是否 trusted、`session_id_source` 是 `payload` 还是 `unknown`、widget 是否真的变化。
- UI 观察必须区分“状态文件已变”与“界面已重绘”，避免把底层成功误判为端到端成功。

### Verifier

- Verifier order:
- 1. focused check：`taskbar_widget_hook.exe sample` / `list` 或对应最小 CLI 检查。
- 2. `cargo check`。
- 3. `cargo build`。
- 4. 真实 Codex `/hooks` 加载与 trust 验证。
- 5. 真实事件后的 `taskbar_widget_hook.exe list` 与 `state.json` 验证。
- 6. widget 人工重绘验证。
- Actor 不能自证完成；必须有命令输出、状态文件或人工观察结论作为独立证据。

### Failure Semantics

- Transient failure: 偶发命令失败、文件暂时被占用，可重试 1 次。
- Code/config failure: 构建失败、hook 路径错误、`.codex/hooks.json` 语法错误，回到最小相关 task 修复。
- Strategy failure: 同一验证目标连续 2 次没有产生新证据，停止盲试，回到 dump 证据或文档重审。
- Environment failure: `/hooks` 无法 trust、Codex 会话未重载、widget 无法在当前桌面环境观察，记录 blocked evidence。
- Policy failure: 下一步若需要扩展到用户级配置、破坏性操作或非本轮范围，停止并等待用户决定。

### Exit Conditions

- Success exit: Completion Criteria 满足，且 handoff/reflection 已记录真实验证结论。
- Blocked exit: 继续推进必须依赖用户执行 `/hooks`、桌面人工观察或外部环境变化，且当前轮已无更多本地证据可收集。
- Budget exit: 同一 phase 连续 3 次没有新证据的失败，停止并产出 handoff。
- Risk exit: 下一步需要跨出项目级验证范围，例如改用户级全局 hooks 或扩展 schema/架构掩盖问题。
- Human takeover exit: 需要用户判断产品语义，例如 `PostToolUse` 失败究竟映射 `error` 还是继续 `working`。

### Policy

- 不保存完整 payload、prompt、命令参数或无必要的本地路径。
- 不自动执行 `/hooks` trust；该步骤只记录为人工 gate。
- 不把用户级全局安装、Claude Code、UI 扩展或 runtime hardening 混入本轮。
- 不用改状态 schema 或 widget 行为来绕过真实 hook 验证失败。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、上轮 reflection、当前 `.codex/hooks.json` 和最近验证结果。
2. Choose：按 planner 规则选一个最小可验证 task。
3. Act：做最小编辑或验证动作。
4. Observe：记录命令结果、状态摘要或 UI 观察。
5. Verify：运行该 task 的最小 verifier。
6. Reflect：完成、失败、跳过都生成对应 reflection。
7. Decide：继续下一 task、重试、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 task 有明确下一步，且上一轮带来了新证据。
- 失败已被分类，且还在对应 retry / replan 预算内。

停止条件：

- Completion Criteria 已满足。
- 下一步需要外部输入、人工 trust 或桌面观察而当前轮无法继续。
- 同一 phase 反复失败且没有新证据。
- 下一步会扩大本轮范围。

## Pre-Implementation Checks

- [ ] CSW-PRE-01 阅读 [p0-codex-state-write/README.md](/D:/project/cc-traffic-light/docs/plan/p0-codex-state-write/README.md)，确认本轮只做项目级验证。
- [ ] CSW-PRE-02 阅读 `.codex/hooks.json`，确认当前仍指向 dump 脚本而不是真实状态写入 CLI。
- [ ] CSW-PRE-03 阅读 `taskbar-widget/src/bin/taskbar_widget_hook.rs`，确认真实 hook CLI 入口与 argv 约定。
- [ ] CSW-PRE-04 阅读 `taskbar-widget/src/hook_rules.rs` 与 `taskbar-widget/src/agent_state.rs`，确认 `session_id`、`received_at` 和 summary 相关行为。
- [ ] CSW-PRE-05 确认验证命令包含 `cargo check`、`cargo build` 和 `taskbar_widget_hook.exe list`。
- [ ] CSW-PRE-06 确认本轮需要人工执行 Codex `/hooks` review/trust，且该动作不自动化。
- [ ] CSW-PRE-07 确认 `state.json` 目标路径与读取方式，避免验证时误看旧文件。

## Implementation Checklist

### Phase 1: Hook 二进制预检

- [x] CSW-A-01 在 `taskbar-widget/` 运行 `cargo check`，确认当前代码可通过快速编译检查。
- [x] CSW-A-02 在 `taskbar-widget/` 运行 `cargo build`，生成 `target/debug/taskbar_widget_hook.exe`。
- [x] CSW-A-03 运行 `taskbar_widget_hook.exe sample` 或等价最小命令，确认二进制可启动。
- [x] CSW-A-04 运行 `taskbar_widget_hook.exe list`，确认 CLI 能读取当前状态文件且输出未崩溃。
- [x] CSW-A-05 记录如果 `list` 为空或只出现 `codex_unknown` 时的基线观察，作为切换前对照。

### Phase 2: 切换项目级 Hooks

- [x] CSW-B-01 备查当前 `.codex/hooks.json` 的 dump 命令 shape，确保后续还能回到采样模式。
- [x] CSW-B-02 将 `.codex/hooks.json` 中每个目标 lifecycle event 的 command 更新为 `D:\project\cc-traffic-light\taskbar-widget\target\debug\taskbar_widget_hook.exe codex <HookName>`。
- [x] CSW-B-03 复核 Windows 命令路径、参数顺序和 hook name 大小写，避免因为配置错误导致完全不触发。
- [x] CSW-B-04 确认切换后没有顺手删除 dump 脚本，保留后续再次采样能力。
- [x] CSW-B-05 记录这次变更会触发 Codex 重新 review/trust hooks，作为后续人工步骤前置条件。

### Phase 3: 真实 Codex 状态写入验证

当前进展注记（2026-07-01 13:45）：

- 已用手工 payload 通过 `taskbar_widget_hook.exe codex <HookName>` 验证 `%APPDATA%\CcTrafficLight\state.json` 写入路径。
- 已验证 `PreToolUse -> working`、`PreToolUse -> Stop -> done`、`PermissionRequest -> Stop -> waiting` 的本地状态迁移。
- 手工验证完成后，临时 `manual-*` tasks 已清理，当前状态文件回到空任务基线。
- 之后已观察到真实 Codex 线程写入共享状态：
  - `codex_019f1c28-451c-7f43-9f25-7240f6e161fc`
  - `codex_019f1c2c-c77b-7e92-bea7-3750ccc31dff`
  - `codex_019f1c2f-701a-7212-ac11-9dce86f8de67`
- 当前 `taskbar_widget_hook.exe list` 已能读到真实 `codex_<session_id>` tasks，且 `session_id_source = payload`，状态覆盖 `working` 与 `done`。
- `event_order_source` 目前仍来自 `received_at`，本轮继续接受该兜底。
- 早先“后台子线程不能替代前台 trust”的结论仍保留为一次失败尝试记录，但不再代表最终结果；随后真实线程事件已经证明项目级 hooks 确实在执行。
- 仍待补的是更干净的 `/hooks` UI 侧证据、显式 `PreToolUse` + `PostToolUse` 同轮采样，以及“真实事件驱动下 widget 持续可见刷新”的稳定前后截图。

- [ ] CSW-C-01 重新打开或刷新当前仓库的 Codex 会话，让项目 `.codex/hooks.json` 重新加载。
- [ ] CSW-C-02 在 Codex 中运行 `/hooks`，确认项目级 hooks 已出现且 source 正确。
- [ ] CSW-C-03 完成一次 hooks review/trust，确认非 managed command hooks 已被允许执行。
- [x] CSW-C-04 触发一个最小用户 prompt，确认 `UserPromptSubmit` 或后续事件会真正触发 hook。
- [ ] CSW-C-05 触发一个最小只读 tool call，覆盖 `PreToolUse` 与 `PostToolUse`。
- [x] CSW-C-06 在事件触发后运行 `taskbar_widget_hook.exe list`，确认 `tasks` 中出现 `codex_<session_id>`。
- [x] CSW-C-07 检查状态记录中的 `session_id_source` 是否为 `payload`，而不是 fallback `unknown`。
- [x] CSW-C-08 检查状态迁移是否至少覆盖 `working`，并在会话结束后观察到 `done` 或设计允许的保守状态。
- [ ] CSW-C-09 如果 `Stop` 前一状态为 `waiting`，确认当前保守保持 `waiting` 的既有设计没有回归。
- [x] CSW-C-10 记录若未观察到独立 `event_order` 字段，本轮继续接受 `received_at` 兜底排序。

### Phase 4: Widget 重绘验证

当前进展注记（2026-07-01 13:45）：

- 已在隔离状态目录 `taskbar-widget/target/p0-redraw-verify-20260701-2/state` 下运行 widget。
- 已通过 debug CLI 驱动 `IDLE -> WORKING 1 -> WAITING 1 -> DONE 1`，并保存任务栏截图：
  - `before.png`
  - `working.png`
  - `waiting.png`
  - `done.png`
- `widget.stdout.log` 中记录了 `[hook-state] summary changed state=working|waiting|done`。
- 模块区域像素差异显著：
  - `idle -> working` mean delta `69.06`
  - `working -> waiting` mean delta `97.33`
  - `waiting -> done` mean delta `65.26`
- 这证明 widget 会响应共享状态变化并重绘；仍待补的是“真实 Codex 事件”而不是 debug CLI 驱动。
- 另外已在真实 `%APPDATA%` 状态源下抓到 widget 可见渲染：
  - [before.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-fast/before.png) 显示 `WAITING 3`
  - [after.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-fast/after.png) 模块区域为空白
- 结合 `widget.pid.txt` 与后续进程检查，`after.png` 的空白更像是那次 widget 进程已退出，而不是一个干净的“3 -> 4”可见刷新证据。
- 因此，本轮可以确认“真实状态源可被 widget 读取并显示”，但“真实事件驱动下稳定、持续可见的前后刷新”仍是 partial。
- 之后又做了两轮更短时序的真实验证：
  - [before-screen.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-live-20260701-164239/before-screen.png) 抓到 `WAITING 2`
  - 同轮真实线程事件后，`taskbar_widget_hook.exe list` 证实 `active_task_count = 3`，但 [after-screen.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-live-20260701-164239/after-screen.png) 仍为空白
  - 新一轮 [before-screen.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-live-20260701-164536/before-screen.png) 抓到 `WAITING 3`
  - 再触发一个此前不在 `tasks` 表中的真实线程后，`taskbar_widget_hook.exe list` 证实 `active_task_count = 4`，但 [after-screen.png](/D:/project/cc-traffic-light/taskbar-widget/target/p0-redraw-real-live-20260701-164536/after-screen.png) 仍为空白
- `widget.pid.txt` 记录的 PID 在 after 阶段再次查不到对应进程，说明当前 blocker 已收紧为“widget 运行期稳定性/持续存活”，而不是 hooks 或共享状态没有变化。
- 新增诊断 harness 后，isolated feedback loop 已给出更强反证：
  - [baseline report](/D:/project/cc-traffic-light/taskbar-widget/target/diagnose-widget-liveness/baseline-20260701-171354/report.json) 证明空状态下同一实例可稳定存活 5 秒，且截图持续非空
  - [fixture replay report](/D:/project/cc-traffic-light/taskbar-widget/target/diagnose-widget-liveness/fixture_replay-20260701-171416/report.json) 证明 isolated 状态变化不会打掉实例；同一实例稳定显示 `WAITING 2 -> WAITING 3 -> WAITING 4`
  - 这把“状态变化本身导致实例退出”从高概率降到了较低概率
- 新的运行期日志还暴露了一个实现问题：`HookSummary.updated_at` 之前会让每次 timer tick 都被误判成 summary changed。当前已改成只按显示相关字段触发 redraw，baseline / fixture replay 都已验证通过。
- 当前剩余 blocker 不再是“任意状态变化就会让 widget 死掉”，而是“真实 live loop 里的状态变化还没有在采样窗口内被稳定观测到”，需要继续查真实线程触发时序、桌面 capture 上下文，或真实 taskbar host 的特殊行为。

- [x] CSW-D-01 运行 taskbar widget，确保它在验证期间持续读取共享状态。
- [ ] CSW-D-02 触发一次真实 Codex 状态变化，观察 widget 是否在 1000 ms 轮询窗口内更新。
- [x] CSW-D-03 记录 widget 的可见文字、颜色或摘要变化，区分“状态文件已变”与“界面已重绘”。
- [x] CSW-D-04 若 widget 未更新，先回查 `state.json` 与 `taskbar_widget_hook.exe list`，再决定是否进入 UI 侧排障。
- [x] CSW-D-05 记录本轮人工观察结论，明确是否已证明端到端链路成立。

### Phase 5: 文档与收尾

- [x] CSW-E-01 更新相关 checklist 或 handoff，记录真实 Codex 状态写入是否验证通过。
- [x] CSW-E-02 如结论或下一步建议发生变化，更新 `docs/handoff/` 或新增 handoff，满足仓库 agent 指令。
- [x] CSW-E-03 记录已知限制：当前 payload 无独立 `event_order` 字段、`PostToolUse` 失败映射仍待确认。
- [x] CSW-E-04 记录若需回退到 dump 模式时的操作方式，避免后续排障时重复探索。

## Validation Checklist

- [x] CSW-VAL-01 运行 `cargo check`，期望无编译错误。
- [x] CSW-VAL-02 运行 `cargo build`，期望成功生成 `taskbar_widget_hook.exe`。
- [x] CSW-VAL-03 运行 `taskbar_widget_hook.exe list`，期望命令成功且能展示当前状态摘要。
- [ ] CSW-VAL-04 通过真实 Codex `/hooks` 界面确认项目 hooks 已加载并 trusted。
- [x] CSW-VAL-05 触发一个最小只读 tool call 后，期望 `codex_<session_id>` 出现在状态列表中。
- [x] CSW-VAL-06 检查 `state.json`，期望存在与当前会话对应的 task 记录，且 `session_id_source = payload`。
- [x] CSW-VAL-07 检查 summary，期望会随真实事件在 `working`、`waiting`、`done` 等允许状态间变化。
- [x] CSW-VAL-08 人工观察 widget，期望无需重启即可反映新的 summary。
- [ ] CSW-VAL-09 若 hook 未触发，优先排查 trust、命令路径和 `.codex/hooks.json` 语法，而不是直接修改状态 schema。
- [ ] CSW-VAL-10 若只出现 `codex_unknown`，记录为失败并回到 payload/session 采样证据排查。

## Documentation Checklist

- [x] CSW-DOC-01 新增本 checklist 文档并与 P0 计划一一对应。
- [x] CSW-DOC-02 在执行完成后更新相关 handoff，写明 pass、partial 或 fail 结论。
- [x] CSW-DOC-03 如项目默认 hook 已切到真实写入，补充保留 dump 脚本用途的说明。
- [ ] CSW-DOC-04 如验证暴露新的状态映射问题，记录到计划或后续 P4 hardening 输入。
- [ ] CSW-DOC-05 每个完成、跳过或阻塞的任务都生成 reflection。

## Cleanup Checklist

- [ ] CSW-CLN-01 确认没有删除 dump 脚本或其再采样能力。
- [ ] CSW-CLN-02 确认没有扩展到用户级全局 hooks 安装。
- [ ] CSW-CLN-03 确认没有引入与本轮无关的 UI 功能或架构改动。
- [ ] CSW-CLN-04 确认没有提交临时日志、截图或本地状态文件。
- [ ] CSW-CLN-05 确认命令路径、文档描述和 task 命名保持一致。
- [ ] CSW-CLN-06 确认错误记录保留诊断价值，但不额外保存不必要的 payload 内容。

## Completion Criteria

以下条件满足时，本轮可判定完成：

- 项目 `.codex/hooks.json` 已从 dump 模式切到真实状态写入命令。
- 真实 Codex 会话能生成 `codex_<session_id>`，且 `session_id_source = payload`。
- `taskbar_widget_hook.exe list` 与 `state.json` 都能证明共享状态已被真实事件更新。
- 正在运行的 widget 已观察到重绘或摘要变化。
- `cargo check` 与 `cargo build` 通过。
- 关键人工验证结果已写入 handoff 或相关文档。
- 已知限制被明确记录，且没有擅自扩展到用户级安装或更大范围功能。

可接受的已知限制：

- 继续使用 `received_at` 作为缺失 `event_order` 时的排序兜底。
- `PostToolUse` 失败映射仍可保持待后续 payload 证据再收紧。

当前执行判定（2026-07-01 13:45）：

- “真实 Codex 会话写入 `codex_<session_id>` 且 `session_id_source = payload`”已通过。
- “共享状态 summary 会被真实事件推进到 `working` / `done`”已通过。
- “widget 在真实 `%APPDATA%` 状态源下可见显示 summary”已通过。
- “widget 在真实事件驱动下持续可见并完成一轮稳定前后刷新截图”仍是 partial；当前更具体的 blocker 是实例在 after 采样阶段反复退出或不可见，因此本 checklist 还未完全清零。

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

- task id 必须对应本 checklist 中的条目，例如 `CSW-B-02`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及 `/hooks` trust 的任务必须记录人工步骤是否完成。
- 涉及 `state.json` 观察的任务必须区分“文件已更新”和“widget 已响应”。
- 涉及已知限制接受的任务必须记录为什么当前仍接受该限制。

## Goal Usage Recommendation

这项工作适合用 `/goal` 或等价长期目标执行，因为它具备多阶段验证、人工 gate、状态持久化和明确 blocked/complete 语义。

建议 objective：

```text
Validate project-scoped Codex lifecycle hook state writing end-to-end until .codex/hooks.json uses taskbar_widget_hook.exe, real sessions write codex_<session_id> into state.json, and the running widget reflects the updated summary.
```

Continue condition：

- 还有未完成的 phase task，且上一轮产生了新的可执行证据。

Completion condition：

- Completion Criteria 全部满足，且最新 handoff / reflection 已记录真实验证结果。

Blocked condition：

- 连续多轮都卡在 `/hooks` trust、会话重载或桌面观察这类外部依赖，且本地已无更多可验证动作。

Budget boundary：

- 同一 phase 连续 3 次没有新证据，或继续执行只会重复已有人工步骤时，停止并转入 handoff / reflection。
