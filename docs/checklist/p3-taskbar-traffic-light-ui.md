# P3 Taskbar Traffic Light UI Checklist

日期：2026-07-01

## Checklist Objective

目标是把 [p3-taskbar-traffic-light-ui/README.md](/D:/project/cc-traffic-light/docs/plan/p3-taskbar-traffic-light-ui/README.md) 转成可执行 checklist，并在不破坏当前稳定 Win11 attach/render 路径的前提下，把任务栏 widget 从文字为主演进为最小红绿灯状态指示器。

目标结果：

- 定义 `idle`、`working`、`waiting`、`done`、`error` 的稳定视觉契约。
- 在 `taskbar-widget/src/main.rs` 中用 GDI 实现最小状态指示器。
- 保持当前窗口尺寸、任务栏挂载路径和 layered 可见性策略不变。
- 通过人工状态矩阵和真实 hook 事件验证 UI 可读性与状态链路。

范围：

- 仅覆盖现有 Win32 widget 的 GDI 绘制修改、状态到视觉样式映射、紧凑文本策略和人工可见性验证。
- 仅覆盖 `taskbar-widget/src/main.rs` 中与 paint state / paint style / paint window 直接相关的代码路径。
- 可在必要时读取 `taskbar-widget/src/agent_state.rs` 以确认当前 `global_summary` 的字段与状态来源。

非目标：

- 不修改 `taskbar-widget/src/taskbar.rs` 中的 probing、`SetParent`、定位或锚点策略，除非出现阻塞证据。
- 不引入 D2D、DirectComposition、图片资源加载、多显示器支持、Explorer 重启恢复、设置界面或通用主题系统。
- 不在 P3 提前展开按 agent 分列渲染。
- 不把 UI 改造扩展成新的状态 schema 或新的 hook 事件模型。

## Loop Engineering Spec

### Goal

- 交付一个可见、可辨认、可由现有共享状态驱动的任务栏红绿灯 UI。
- 进度证据来自：视觉契约文档化、`main.rs` diff、`cargo fmt -- --check`、`cargo check`、人工状态矩阵验证记录、真实 hook smoke test 记录。
- 完成证据不是“代码看起来更漂亮”，而是“状态变化在当前任务栏尺寸下能被稳定区分，且未破坏现有 Win11 attach/render 路径”。

### State

- Source of truth: 本 checklist、[p3-taskbar-traffic-light-ui/README.md](/D:/project/cc-traffic-light/docs/plan/p3-taskbar-traffic-light-ui/README.md)、[main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)、[agent_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/agent_state.rs)。
- Persistent loop state: 当前 phase / task id、已确认的状态视觉映射、当前 UI 布局假设、最近一次人工可见性结论、最近一次真实 hook 验证结论，写入 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: 编译结果、任务栏截图或人工观察摘要、状态矩阵执行记录、`taskbar_widget_hook.exe` 的最小验证输出。
- Discardable state: 一次性调色尝试、被推翻的布局猜测、没有形成结论的临时颜色组合。

### Planner

- 默认选择“当前 phase 中依赖已满足且最能改变验证状态的最小任务”。
- 固定顺序为：先锁定视觉契约，再做最小 GDI 改造，再做人工状态矩阵验证，最后做真实 hook smoke test 和文档回写。
- 若 UI 可见性与状态可读性冲突，优先保留可见性，再缩减文本或装饰复杂度。
- 若需要动 `taskbar.rs` 才能让 UI 可见，先判定为异常并回到观测，不把 UI 问题误扩展为宿主路径重构。

### Actor

- 允许动作：读取 `main.rs` / `agent_state.rs` / 文档，编辑 checklist 与 `main.rs`，运行 `cargo fmt -- --check`、`cargo check`、`cargo run`，使用 `taskbar_widget_hook.exe set` 触发状态。
- 中风险动作：在真实桌面会话中运行 widget 并做人工观察；修改 GDI 绘制逻辑导致暂时性可见性回退。
- 非默认动作：修改 `taskbar.rs`、改变窗口尺寸、改变 layered / parent 路径、增加新的资源文件或图片依赖。

### Observer

- 每次动作后先记录原始观察，再写解释：例如“圆点被裁切”与“控件太小不适合纯圆点方案”必须分开。
- 对人工验证至少记录：状态名、是否一眼可辨认、是否和背景形成足够对比、是否影响任务栏布局。
- 若真实 hook 验证失败，记录失败来自状态写入、轮询延迟、绘制映射还是可见性退化。

### Verifier

- Verifier order:
- 1. code review：确认仅动 `main.rs` 及必要文档，未触碰 `taskbar.rs`。
- 2. `cargo fmt -- --check`。
- 3. `cargo check`。
- 4. 状态矩阵人工验证：`idle`、`working`、`waiting`、`done`、`error`、`stale`。
- 5. 真实 hook smoke test：确认真实 lifecycle event 能在 1000 ms 内驱动 UI 变化。
- Actor 不能自证完成；必须有命令结果和人工可见性结论作为独立证据。

### Failure Semantics

- Transient failure: 一次性构建失败、桌面会话偶发未刷新、状态写入未及时轮询，可重试 1 次。
- Layout failure: UI 元素在当前尺寸下被裁切、重叠或失去可辨识度，必须回到视觉契约或简化布局。
- Visibility failure: 修改绘制后 widget 在任务栏中不明显或不可见，立即缩小改动面，不把问题扩散到宿主逻辑。
- Scope failure: 为解决读不清问题开始引入图片、D2D、按 agent 分列或主题系统，立即停止并回收。
- Evidence failure: 没有形成状态矩阵或真实 hook 的独立验证证据，不得自判完成。

### Exit Conditions

- Success exit: Completion Criteria 满足，且视觉映射、实现、验证和文档回写都完成。
- Blocked exit: 当前 UI 是否可接受取决于用户产品选择，或桌面环境限制导致无法形成本地可见性结论。
- Budget exit: 连续多轮仅在微调颜色/尺寸而没有新增验证证据，停止并产出 handoff。
- Risk exit: 下一步需要超出 P3 范围去改宿主挂载路径或引入新渲染栈。
- Human takeover exit: 需要用户在“纯圆点”与“圆点 + 文本”之间做产品取舍，且本地证据无法自行收敛。

### Policy

- 默认不改 `taskbar-widget/src/taskbar.rs`。
- 默认不改窗口尺寸、父子关系、layered 模式、锚点或定位。
- 第一版只用 GDI 和高对比固定颜色。
- 先优化“任务栏上一眼可见”，再考虑更丰富信息密度。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、P3 README、`main.rs` 当前绘制路径和最近验证记录。
2. Choose：选择一个能直接改变验证状态的最小 task。
3. Act：做最小文档决策、最小绘制改造或最小状态注入。
4. Observe：记录命令结果和人工可见性观察。
5. Verify：运行该 task 对应的最小 verifier。
6. Reflect：完成、阻塞或跳过都生成 reflection。
7. Decide：继续下一 task、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 phase 还有未完成 task，且上一轮拿到了新的结构化证据。
- 失败已被分类，且仍在 retry / replan 预算内。

停止条件：

- Completion Criteria 满足。
- 下一步必须扩大到 P3 非目标。
- 连续多轮只剩低收益视觉试错，没有新的验证信号。

## Pre-Implementation Checks

- [x] TLU-PRE-01 阅读 [p3-taskbar-traffic-light-ui/README.md](/D:/project/cc-traffic-light/docs/plan/p3-taskbar-traffic-light-ui/README.md)，确认 P3 目标是 UI 演进，不是宿主挂载路径重构。
- [x] TLU-PRE-02 阅读 [main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)，定位当前 `paint_window`、`paint_style` 和状态文本渲染路径。
- [x] TLU-PRE-03 阅读 [agent_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/agent_state.rs)，确认当前仅消费 `global_summary`，避免误做按 agent 分列。
- [x] TLU-PRE-04 检查是否已有文档或 handoff 对 P3 视觉方向做出明确结论；若没有，先在本 checklist 中记录默认决策。
- [x] TLU-PRE-05 确认验证命令至少包含 `cargo fmt -- --check`、`cargo check`、`cargo run` 和 `taskbar_widget_hook.exe set`。
- [x] TLU-PRE-06 确认本轮不修改 `taskbar.rs`、窗口尺寸和 Win11 attach/render 路径，除非后续证据强制要求。

## Implementation Checklist

### Phase 1: 锁定视觉契约

- [x] TLU-A-01 为 `idle`、`working`、`waiting`、`done`、`error` 定义颜色、主形态、可选文字和对比要求。
- [x] TLU-A-02 明确 P3 默认方案为“圆点或 pill + 紧凑文本”还是“纯圆点”，并把理由写回本 checklist。
- [x] TLU-A-03 定义 `stale` 的视觉处理方式，优先选择最小附加标记而不是新增整套状态。
- [x] TLU-A-04 确认所有状态都能在当前窗口尺寸内容纳，不依赖扩大 widget 尺寸。
- [x] TLU-A-05 明确 fallback 规则：当文本放不下或对比不足时，保留状态指示器，削减次要文本。

当前决策（2026-07-02）：

- 采用“左侧高对比圆点 + 右侧紧凑标签”的默认方案，不做纯圆点版本。
- 状态标签收敛为 `IDLE`、`RUN`、`WAIT`、`DONE`、`ERR`，只有 `active_task_count > 0` 时才追加计数。
- `stale` 不单独占用主布局，改为右上角轻量告警点，并在标签尾部追加 `!` 作为兜底文本信号。
- 保留按状态变化的背景色，但把状态灯做得比背景更亮，优先保证任务栏上一眼可见。
- 文本一旦放不下，优先截断标签，不扩大 widget，也不恢复到完整长文本。

### Phase 2: 实现最小 GDI 状态指示器

- [x] TLU-B-01 在 [main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs) 中提炼状态到视觉样式的映射，避免把颜色和布局常量散落在 paint 逻辑里。
- [x] TLU-B-02 重构 `paint_style` 或等价路径，使其能绘制圆点或 pill，而不是仅渲染大段状态文本。
- [x] TLU-B-03 保留当前背景色 fallback 逻辑，避免因 UI 改造破坏任务栏可见性。
- [x] TLU-B-04 若保留紧凑文本，只显示最小必要信息，例如单词缩写、计数或 stale 标记，避免恢复成“文字为主”的旧样式。
- [x] TLU-B-05 为非显然的绘制决策补最小注释，说明为何该布局适合当前任务栏约束。

当前实现（2026-07-02）：

- `paint_style` 已收敛为 `PaintStyle` 结构，统一返回标签、背景色、前景色、状态灯颜色和 stale 告警色。
- `paint_window` 已改为自定义 GDI 布局：整块底色、左侧圆形状态灯、右侧左对齐紧凑文本。
- 文本使用 `DT_END_ELLIPSIS`，在当前窗口宽度下优先保住状态灯与整体布局。
- `summary.has_stale` 为真时，会额外绘制右上角轻量告警点。

### Phase 3: 做人工状态矩阵验证

- [x] TLU-C-01 用 `taskbar_widget_hook.exe set codex_123 idle` 验证 `idle` 的颜色、形态和可读性。
- [x] TLU-C-02 用 `taskbar_widget_hook.exe set codex_123 working` 验证 `working` 的颜色、形态和可读性。
- [x] TLU-C-03 用 `taskbar_widget_hook.exe set codex_123 waiting` 验证 `waiting` 的颜色、形态和可读性。
- [x] TLU-C-04 用 `taskbar_widget_hook.exe set codex_123 done` 验证 `done` 的颜色、形态和可读性。
- [x] TLU-C-05 用 `taskbar_widget_hook.exe set codex_123 error` 验证 `error` 的颜色、形态和可读性。
- [ ] TLU-C-06 构造 `stale` 场景并验证 stale 标记不会压过主状态。
- [x] TLU-C-07 记录状态矩阵结果：哪些状态一眼可辨、哪些状态仍需调色或简化。

当前观察（隔离共享状态，2026-07-02）：

- 初始状态为 `idle`，黑/灰色，无数字。
- 执行 `set codex_123 idle` 后无视觉变化；该结果符合预期，因为当前已经是 `idle`。
- 执行 `set codex_123 working` 后显示 `run`，蓝色，`1`。
- 执行 `set codex_123 waiting` 后显示 `wait`，黄色，`1`。
- 执行 `set codex_123 done` 后显示 `done`，绿色，`1`。
- 执行 `set codex_123 error` 后显示 `err`，红色，`1`。

当前结论：

- 在清理旧记录并隔离共享状态后，`global_summary` 能正确反映单任务状态。
- P3 当前的“状态灯 + 紧凑标签”映射已通过 `idle / working / waiting / done / error` 主链路人工状态矩阵验证。
- 当前没有观察到需要继续调色或简化布局的问题。

### Phase 4: 用真实 hook 做 smoke test

- [x] TLU-D-01 运行 widget 并触发真实 Codex lifecycle events，确认 `working` 与 `done` 至少能自动切换。
- [ ] TLU-D-02 若当前环境具备 Claude 状态输入，确认共享状态中的其他 agent 不会把 UI 推回不可解释状态。
- [x] TLU-D-03 验证真实事件到 UI 更新的响应时间不超过 1000 ms，或明确记录当前轮询延迟结论。
- [x] TLU-D-04 确认真实事件驱动路径不要求修改 `taskbar.rs` 或状态 schema。
- [x] TLU-D-05 若真实 hook 结果与人工 `set` 不一致，先定位是状态映射问题还是轮询/绘制问题，再决定是否 replan。

当前观察（真实 Codex hook，2026-07-02）：

- Prompt 1（成功只读）观测到 `run -> done`。
- Prompt 2（读取不存在文件）观测到 `run -> done`，随后对话文本明确说明文件不存在。
- Prompt 3（删除不存在文件）观测到 `run -> wait -> 允许执行 -> wait`，之后没有回到明确的 `done` 或 `error` 终态。

当前结论：

- 真实 hook 已证明 UI 可以被真实对话自动驱动进入 `working`，且至少一种成功路径会回到 `done`。
- 失败路径当前没有稳定落到 `error`，而是被错误归并到了 `done`。
- 需要用户批准的路径当前可以进入 `waiting`，但批准后没有稳定退出 `waiting` 回到终态。
- 这说明当前主要问题不在绘制层，而在真实 hook 事件到 `working / waiting / done / error` 的状态映射。

复测结果（hook 规则修正后，2026-07-02）：

- Prompt 1：`run -> done`
- Prompt 2：`run -> error`
- waiting 验证：`run -> wait -> error`

复测结论：

- 成功路径现在稳定落到 `done`。
- 失败路径现在稳定落到 `error`。
- 需要批准的路径可以进入 `waiting`，并在后续退出到明确终态，不再卡死在 `waiting`。
- 这说明真实 hook 状态映射问题已修正，P3 的真实 hook smoke test 主链路通过。

### Phase 5: 文档与收口

- [ ] TLU-E-01 回写本 checklist 中的默认视觉决策和实际验证结果，避免后续重复讨论同一 UI 取舍。
- [ ] TLU-E-02 若 P3 结论改变后续路线，更新 `docs/handoff/` 说明推荐下一步是 P4 hardening 还是继续 UI 调整。
- [ ] TLU-E-03 为每个完成、阻塞或跳过的 task 生成 reflection。
- [ ] TLU-E-04 确认仓库文档明确写出 P3 仍只消费 `global_summary`，未提前进入按 agent 布局。

## Validation Checklist

- [x] TLU-VAL-01 运行 `cargo fmt -- --check`，期望格式检查通过。
- [x] TLU-VAL-02 运行 `cargo check`，期望无编译错误。
- [x] TLU-VAL-03 运行 `cargo run`，期望 widget 正常启动且没有回退到明显错误的普通窗口行为。
- [x] TLU-VAL-04 人工状态矩阵中，`idle`、`working`、`waiting`、`done`、`error` 都应一眼可区分。
- [ ] TLU-VAL-05 `stale` 标记应可见，但不能盖过主状态。
- [x] TLU-VAL-06 UI 变化不应要求更改窗口尺寸、父子关系、layered 模式、锚点或定位。
- [x] TLU-VAL-07 真实 hook smoke test 中，至少一个开始态和一个结束态能自动驱动 UI 更新。
- [x] TLU-VAL-08 若保留文本，文本长度必须受控，不能回退成以完整状态词为主的旧 UI。
- [ ] TLU-VAL-09 若某个状态在当前尺寸下仍不可读，必须记录证据并回到视觉契约，而不是直接扩大范围。

当前判定（2026-07-02）：

- `TLU-VAL-07` 已通过。
- 真实 hook 复测已覆盖：
  - 成功路径：`run -> done`
  - 失败路径：`run -> error`
  - waiting 路径：`run -> wait -> error`
- 当前剩余未补证据主要是 `stale` 可见性，而不是 hook 映射或 UI 主链路问题。

## Documentation Checklist

- [x] TLU-DOC-01 新增本 checklist 文档并与 P3 计划一一对应。
- [x] TLU-DOC-02 在 checklist 中记录最终采用的是“纯圆点”还是“圆点/pill + 紧凑文本”。
- [ ] TLU-DOC-03 如 P3 结果改变了后续建议，更新 `docs/handoff/`。
- [ ] TLU-DOC-04 每个完成、跳过或阻塞的 task 都生成 reflection。
- [x] TLU-DOC-05 如需保留已知限制，明确写在 checklist 或 handoff，而不是只留在会话上下文里。

## Cleanup Checklist

- [x] TLU-CLN-01 确认没有顺手修改 `taskbar.rs` 或其他非必要宿主逻辑。
- [x] TLU-CLN-02 确认没有引入图片资源、额外依赖或新的渲染栈。
- [ ] TLU-CLN-03 确认没有留下实验性日志、截图路径或临时调色代码。
- [x] TLU-CLN-04 确认状态命名与现有共享状态模型一致。
- [x] TLU-CLN-05 确认注释和文档描述都聚焦当前 MVP，不扩展到多显示器、主题系统或按 agent 分列。

## Completion Criteria

以下条件满足时，P3 可判定完成：

- 已明确并文档化 `idle`、`working`、`waiting`、`done`、`error` 的视觉契约。
- `taskbar-widget/src/main.rs` 已实现最小红绿灯状态指示器。
- 当前任务栏窗口尺寸、attach 路径和 layered 可见性策略未被破坏。
- 人工状态矩阵至少覆盖 `idle`、`working`、`waiting`、`done`、`error`、`stale`。
- 真实 hook smoke test 证明至少一条真实生命周期链路能驱动 UI 变化。
- `cargo fmt -- --check`、`cargo check` 通过。
- 已知限制、剩余 UI 缺口和推荐下一步已写回 checklist 或 handoff。

可接受的已知限制：

- 第一版可继续只消费 `global_summary`，不提前做按 agent 分列。
- 若任务栏尺寸过小，可接受仅保留极简文本或仅保留状态图形，但必须有验证证据。
- 若 `stale` 只能以轻量附加标记表达，可接受不为其分配完整独立布局。

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

- task id 必须对应本 checklist 中的条目，例如 `TLU-B-02`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及人工状态矩阵的任务必须记录状态名、可见性结论和是否需要回调色。
- 涉及真实 hook smoke test 的任务必须记录真实事件、响应时间结论和是否仍依赖 `global_summary`。

## Goal Usage Recommendation

P3 适合在真正开始编码时用 `/goal` 或等价长期目标执行，因为它是一个多阶段验证闭环：视觉契约、最小实现、人工矩阵验证和真实 hook 验证缺一不可。

建议 objective：

```text
Implement and validate a minimal GDI traffic-light UI for the Win11 taskbar widget so that the existing global summary can drive clearly distinguishable idle, working, waiting, done, and error states without changing the stable taskbar attach path.
```

Continue condition：

- 还有未完成的 phase task，且上一轮拿到了新的代码或验证证据。

Completion condition：

- Completion Criteria 全部满足，且最新验证结果已回写 checklist / handoff / reflection。

Blocked condition：

- 连续多轮都卡在 UI 可读性与当前任务栏尺寸冲突，且下一步需要产品决策或范围外重构。

Budget boundary：

- 同一 phase 连续 3 次只有低收益视觉微调而没有新增验证证据时，停止并转入 handoff。
