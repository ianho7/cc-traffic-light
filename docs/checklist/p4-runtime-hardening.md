# P4 Runtime Hardening Checklist

日期：2026-07-02

## Checklist Objective

目标是把 [p4-runtime-hardening/README.md](/D:/project/cc-traffic-light/docs/plan/p4-runtime-hardening/README.md) 转成可执行 checklist，并把 hook 共享状态链路的长期运行验证收敛成有预算、有夹具、有退出条件的 hardening 闭环。

目标结果：

- 建立可重复执行的压力测试夹具，覆盖 Codex / Claude 并发会话、高频事件、stale 清理、损坏状态恢复和 hook 失败场景。
- 建立 hook CLI、状态文件大小、widget 轮询与重绘行为的基线数据和 pass / fail 阈值。
- 验证 `agent_state.rs` 在损坏、竞争和异常输入下仍能保持安全恢复与可解释 summary。
- 为 P1 安装脚本补足 apply / restore 回滚验证标准，确保不会污染用户原有 hook 配置。

范围：

- 仅覆盖 `taskbar-widget/src/agent_state.rs`、`taskbar-widget/src/main.rs`、`taskbar-widget/src/bin/taskbar_widget_hook.rs`、P1 安装脚本和 `taskbar-widget/scripts/` 下的压力 / 验证脚本。
- 仅覆盖文件桥接 + named mutex 这条现有 MVP 路径的运行时验证与最小硬化。
- 允许补充最小 diagnostics 字段、验证脚本、fixture 和文档，以支持性能与恢复行为留证。

非目标：

- 不把文件桥接替换为 HTTP、IPC、服务进程或其他新架构。
- 不新增任务栏 UI 功能、多显示器支持、云端同步或通用插件化设计。
- 不在没有压力证据前提前切到 compact JSON、缩短轮询周期或重写状态 schema。
- 不把 P4 扩展成“所有 agent 生命周期都完全精确排序”的平台级方案。

## Loop Engineering Spec

### Goal

- 交付一个能支撑长期运行信心的 P4 hardening 包：压力夹具、基线数据、恢复验证、回滚验证和文档化结论。
- 进度证据来自：新增脚本 / fixture / checklist 文档、`cargo fmt -- --check`、`cargo check`、压力脚本输出、状态文件样本、恢复场景记录、安装回滚记录。
- 完成证据不是“代码更稳了”，而是“预算、验证结果、已知限制和下一步决策都有独立证据支撑”。

### State

- Source of truth: 本 checklist、[p4-runtime-hardening/README.md](/D:/project/cc-traffic-light/docs/plan/p4-runtime-hardening/README.md)、[agent_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/agent_state.rs)、[main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)、[taskbar_widget_hook.rs](/D:/project/cc-traffic-light/taskbar-widget/src/bin/taskbar_widget_hook.rs)、P1 安装脚本与相关文档。
- Persistent loop state: 当前 phase / task、当前性能预算、最新一次压力结果、最新一次恢复验证结论、最新一次安装回滚结论，写入 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: 压力脚本输出、状态文件大小样本、`state.json` / backup 文件、损坏前后摘要、人工长时间运行观察、安装脚本 apply / restore 日志。
- Discardable state: 一次性命令全文、未形成结论的临时压力参数、被否决的预算猜测。

### Planner

- 默认选择“当前依赖满足且最能改变验证状态的最小任务”。
- 固定顺序为：先定义预算与夹具，再测量基线，再验证恢复，再验证安装回滚，最后做文档收口。
- 若压力结果没有越界，优先记录证据而不是继续优化。
- 若验证暴露问题，先补最小 hardening 或更精确 diagnostics，再重跑最小相关验证，不扩大到架构替换。

### Actor

- 允许动作：读取源码与文档，编辑 checklist / 脚本 / 最小 Rust 代码，运行 `cargo fmt -- --check`、`cargo check`、压力脚本、状态注入脚本和安装脚本验证。
- 中风险动作：构造损坏状态文件、制造 mutex 竞争、长时间运行 widget、在 fixture 配置上执行安装 / 恢复。
- 非默认动作：修改真实用户全局 hook 配置、删除用户原配置、切换状态存储架构、引入常驻后台服务。

### Observer

- 每次动作后先记录原始观察，再给出解释：例如“状态文件达到 310 KB”与“需要改成 compact JSON”必须分开。
- 对性能验证至少记录：样本数、并发度、总耗时、单次耗时分布、状态文件大小、summary 是否变化、是否触发重绘。
- 对恢复验证至少记录：输入损坏类型、预期保护、实际恢复路径、summary 是否仍有效、是否生成 backup 或忽略异常事件。

### Verifier

- Verifier order:
- 1. 文档 / 代码审查：确认当前桥接架构、轮询频率、mutex timeout 和恢复逻辑。
- 2. `cargo fmt -- --check`。
- 3. `cargo check`。
- 4. 压力夹具 dry run。
- 5. 基线性能测量。
- 6. 损坏 / 竞争 / stale / 异常输入恢复验证。
- 7. P1 安装回滚验证。
- 8. 长时间运行人工观察。
- Actor 不能自证完成；必须有命令结果、状态文件证据或回滚记录作为独立证据。

### Failure Semantics

- Transient failure: 一次性构建失败、桌面轮询抖动、偶发文件锁竞争，可重试 1 次。
- Fixture failure: 压力脚本无法稳定复现目标场景，先修脚本或补夹具，不直接推导运行时结论。
- Budget failure: 压测超预算但缺少定位信息，先增加最小 diagnostics，再决定是否优化。
- Recovery failure: 损坏恢复导致 summary 无效、旧事件污染状态或 backup 缺失，必须回到代码修复。
- Policy failure: 下一步需要触碰真实用户配置或超出 P4 范围的架构改造，立即停止并升级为 handoff / 人工决策。

### Exit Conditions

- Success exit: Completion Criteria 满足，且预算、验证结果、已知限制和推荐下一步已写回文档。
- Blocked exit: 下一步必须依赖 P1 尚未存在的安装脚本、真实多会话环境或用户许可，且当前轮已无更多本地动作。
- Budget exit: 连续多轮只在微调压力参数或日志格式，没有新增验证证据。
- Risk exit: 为满足目标需要改动状态架构、轮询模型或用户全局配置。
- Human takeover exit: 是否接受某个预算、是否批准真实用户配置回滚测试，需要用户明确取舍。

### Policy

- 优先保留现有文件桥接 + named mutex 设计。
- 只有在预算被真实证据打穿时才做最小 hardening。
- fixture 用户配置优先于真实用户配置。
- 性能日志与 diagnostics 默认脱敏、简短、可复现。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、P4 README、相关源码和最近一次验证结果。
2. Choose：选择一个能直接改变验证状态的最小 task。
3. Act：做最小夹具补充、最小代码硬化或最小验证动作。
4. Observe：记录原始压力 / 恢复 / 回滚证据。
5. Verify：运行当前 task 对应的最小 verifier。
6. Reflect：完成、阻塞或跳过都生成 reflection。
7. Decide：继续下一 task、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 phase 仍有未完成 task，且上一轮新增了结构化证据。
- 失败已分类，且仍在 retry / replan 预算内。

停止条件：

- Completion Criteria 满足。
- 下一步必须扩大到 P4 非目标。
- 连续多轮没有新增证据，只剩重复压测或重复观察。

## Pre-Implementation Checks

- [ ] PRH-PRE-01 阅读 [p4-runtime-hardening/README.md](/D:/project/cc-traffic-light/docs/plan/p4-runtime-hardening/README.md)，确认本轮目标是运行时验证与最小硬化，不是架构替换。
- [ ] PRH-PRE-02 阅读 [agent_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/agent_state.rs)，确认当前 mutex、原子写入、TTL / stale 和损坏恢复路径。
- [ ] PRH-PRE-03 阅读 [main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)，确认当前 1000 ms 轮询和 summary 变化时重绘逻辑。
- [ ] PRH-PRE-04 阅读 [taskbar_widget_hook.rs](/D:/project/cc-traffic-light/taskbar-widget/src/bin/taskbar_widget_hook.rs)，确认现有 hook CLI 输入、状态写入和可复用的 diagnostics 入口。
- [ ] PRH-PRE-05 确认 P1 安装脚本当前是否已存在；若不存在，把真实 apply / restore 验证标记为后置依赖。
- [ ] PRH-PRE-06 确认验证命令至少包含 `cargo fmt -- --check`、`cargo check`、压力脚本和长时间运行观察路径。

## Implementation Checklist

### Phase 1: 定义压力测试夹具与预算

- [ ] PRH-A-01 定义 P4 默认压力场景，至少覆盖 10 个 session、100 条混合 Codex / Claude 事件、stale 输入、损坏状态文件和 hook 失败事件。
- [ ] PRH-A-02 在 `taskbar-widget/scripts/` 设计可重复执行的压力脚本入口，明确输入参数、输出目录和样本记录格式。
- [ ] PRH-A-03 明确 baseline budget：hook CLI 耗时、状态文件大小、widget 重绘行为、mutex wait timeout 和允许的已知波动。
- [ ] PRH-A-04 定义每类场景的 pass / fail 标准，避免“跑过一次看起来没问题”式结论。
- [ ] PRH-A-05 如需 fixture 文件，定义 fixture 状态文件、损坏样本和安装脚本用户配置样本的存放位置与命名规则。

### Phase 2: 测量 Hook 与轮询开销

- [ ] PRH-B-01 用压力夹具测量大量 hook CLI 调用的耗时基线，区分冷启动波动与稳定执行耗时。
- [ ] PRH-B-02 测量不同任务数量下状态文件大小，确认 pretty JSON 是否仍在预算内。
- [ ] PRH-B-03 验证 `main.rs` 在 summary 未变化时不会触发无意义重绘，并记录观察方式。
- [ ] PRH-B-04 如当前 diagnostics 不足以判定开销来源，补最小字段或日志，再重跑最小相关场景。
- [ ] PRH-B-05 把基线结果写回 checklist 或配套验证文档，形成可复测阈值。

### Phase 3: 验证恢复与异常输入行为

- [ ] PRH-C-01 构造损坏状态文件，验证是否能触发备份、重建或安全降级，而不是直接污染 summary。
- [ ] PRH-C-02 构造 mutex 竞争场景，验证超时、等待和失败路径是否符合当前设计。
- [ ] PRH-C-03 构造缺失 `session_id`、旧 `received_at` 或 stale event order 场景，验证是否被正确忽略或隔离。
- [ ] PRH-C-04 构造 hook 失败场景，验证不会把失败写成误导性的完成态，也不会留下不可解释的脏状态。
- [ ] PRH-C-05 若恢复验证暴露缺口，只做最小硬化修复，并为修复补对应回归验证。

### Phase 4: 验证安装与回滚路径

- [ ] PRH-D-01 确认 P1 安装脚本的 apply / restore 接口、输入文件和预期副作用。
- [ ] PRH-D-02 在 fixture 用户配置上验证 apply / restore，确认原 hooks 能完整保留与恢复。
- [ ] PRH-D-03 记录安装脚本对混合 Codex / Claude 配置的处理边界，避免 hardening 文档只覆盖单一 agent。
- [ ] PRH-D-04 若用户许可且环境满足，再在真实用户配置上执行一次受控 apply / restore 验证；否则明确记录 blocker。
- [ ] PRH-D-05 为安装回滚留下可复核证据，避免只凭人工口头确认。

### Phase 5: 长时间运行观察与文档收口

- [ ] PRH-E-01 长时间运行 widget 并记录 summary、轮询、重绘和 stale 清理的人类观察结论。
- [ ] PRH-E-02 汇总 P4 预算是否被打穿；若没有，明确记录“不优化也是结论”的依据。
- [ ] PRH-E-03 若 P4 结论改变后续路线，更新 `docs/handoff/` 说明是否进入安装落地或进一步性能优化。
- [ ] PRH-E-04 为每个完成、阻塞或跳过的 task 生成 reflection。
- [ ] PRH-E-05 明确列出剩余可接受限制，例如继续使用 `received_at` 兜底或继续保留 pretty JSON。

## Validation Checklist

- [ ] PRH-VAL-01 运行 `cargo fmt -- --check`，期望格式检查通过。
- [ ] PRH-VAL-02 运行 `cargo check`，期望无编译错误。
- [ ] PRH-VAL-03 压力脚本可重复生成同类场景，且输出结果可复核。
- [ ] PRH-VAL-04 hook CLI 稳定执行耗时应满足或明确对比既定预算。
- [ ] PRH-VAL-05 正常使用下状态文件大小应满足或明确对比既定预算。
- [ ] PRH-VAL-06 summary 未变化时不应触发额外重绘；若当前只能人工验证，必须记录观察方法和限制。
- [ ] PRH-VAL-07 损坏状态文件后，summary 仍应有效，且恢复路径可解释。
- [ ] PRH-VAL-08 stale / 旧事件不应污染当前 summary。
- [ ] PRH-VAL-09 hook 失败不应被误判为成功完成，也不应留下无法恢复的脏状态。
- [ ] PRH-VAL-10 P1 安装脚本的 apply / restore 不应丢失用户原有 hooks；若未执行真实用户验证，必须明确写出 blocker。
- [ ] PRH-VAL-11 长时间运行观察应至少说明：是否出现明显 CPU / I/O 异常、summary 抖动或脏状态累积。

## Documentation Checklist

- [ ] PRH-DOC-01 新增本 checklist 文档并与 P4 计划一一对应。
- [ ] PRH-DOC-02 在 checklist 或配套验证文档中记录性能预算、样本规模和 pass / fail 标准。
- [ ] PRH-DOC-03 在 checklist 或 handoff 中记录恢复行为、已知风险和接受理由。
- [ ] PRH-DOC-04 如 P1 安装脚本验证有结论，更新相关安装文档或 handoff。
- [ ] PRH-DOC-05 每个完成、跳过或阻塞的 task 都生成 reflection。

## Cleanup Checklist

- [ ] PRH-CLN-01 移除实验性压力输出、临时损坏文件和无用日志，只保留需要的 fixture 与文档证据。
- [ ] PRH-CLN-02 确认没有因为压测而顺手引入架构升级、额外后台服务或状态 schema 扩张。
- [ ] PRH-CLN-03 确认 diagnostics 不泄露 prompt、payload 原文或本地敏感路径。
- [ ] PRH-CLN-04 确认脚本、fixture、文档命名和术语一致使用 budget / stale / recovery / rollback。
- [ ] PRH-CLN-05 确认真实用户配置只在获得许可时才被触碰，并有明确恢复路径记录。

## Completion Criteria

以下条件满足时，P4 可判定完成：

- 已有可重复执行的压力测试夹具，覆盖并发会话、高频事件、stale、损坏恢复和 hook 失败。
- 已有 hook CLI、状态文件大小、widget 轮询 / 重绘行为的基线数据与预算结论。
- 已验证 `agent_state.rs` 在损坏、竞争和异常输入下的恢复行为可解释。
- 已完成或明确阻塞 P1 安装脚本的 apply / restore 验证。
- `cargo fmt -- --check` 与 `cargo check` 通过。
- 已知限制、未完成验证、推荐下一步已写回 checklist 或 handoff。

可接受的已知限制：

- 在没有证据打穿预算前，继续保留 pretty JSON。
- 在没有独立 `event_order` 字段前，继续接受 `received_at` 兜底排序。
- 真实用户配置回滚验证可在获得许可前保持 blocker，但必须先完成 fixture 级验证与文档说明。

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

- task id 必须对应本 checklist 中的条目，例如 `PRH-B-03`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及压力验证的任务必须记录样本规模、预算和结果摘要。
- 涉及恢复验证的任务必须记录损坏类型、恢复路径和 summary 结论。
- 涉及安装回滚的任务必须记录 apply / restore 范围、是否使用 fixture、以及是否触碰真实用户配置。

## Goal Usage Recommendation

P4 适合在真正开始执行 hardening 时用 `/goal` 或等价长期目标推进，因为它是一个多阶段闭环：夹具、基线、恢复、回滚和文档证据必须串起来。

建议 objective：

```text
Harden and validate the shared-state hook runtime until repeatable stress fixtures, performance budgets, recovery behavior, and install rollback evidence show the taskbar traffic light can run for long periods without corrupting state or causing obvious overhead.
```

Continue condition：

- 还有未完成的 phase task，且上一轮拿到了新的压力、恢复或回滚证据。

Completion condition：

- Completion Criteria 全部满足，且最新验证结果已回写 checklist / handoff / reflection。

Blocked condition：

- 连续多轮都卡在 P1 安装脚本缺失、真实用户配置权限不足或仓库外环境依赖，且本地已无更多可验证动作。

Budget boundary：

- 同一 phase 连续 3 次没有新增结构化证据，或继续执行只是在重复压测同一场景时，停止并转入 handoff。
