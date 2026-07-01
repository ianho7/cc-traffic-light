# Hook 集成追问记录、ADR 与术语表

## 追问结论摘要

本次设计围绕一个很窄的 MVP 目标反复追问：让 Claude Code 和 Codex 的 hook 事件，能够被当前 Rust 任务栏组件直接消费。

### Question: 目标是 hook 监控，还是完整的 agent 存在性检测？

Recommended answer: 先做 hook 监控。

Reasoning: 用户要的是红绿灯组件的逻辑依赖，而这个依赖本质上是状态，不是“进程是否存在”。进程检测只能回答 agent 在不在，不能回答它是 `waiting`、`working`、`done` 还是 `error`。

Decision: 先实现 hook 状态，进程检测延期。

### Question: 这个仓库是否应该复制参考项目的 Electron 架构？

Recommended answer: 不应该。

Reasoning: 当前仓库是 Rust Win32 PoC。复制 Electron 的 main/preload/renderer 结构会额外引入一层与当前目标无关的运行时。

Decision: 实现保留在 Rust 项目内部。

### Question: hooks 是否应该直接写到任务栏进程里？

Recommended answer: MVP 阶段不应该。

Reasoning: hooks 是短生命周期命令。直接 IPC 要求任务栏进程已经运行，还会带来新的生命周期失败模式。本地文件则不依赖 widget 是否已打开。

Decision: 使用本地 JSON 状态文件。

### Question: hook 接收器该是脚本，还是 Rust binary？

Recommended answer: Rust binary。

Reasoning: 仓库已经以 Rust 构建为主。Rust hook CLI 能和 widget 共用类型，也避免新增 Node/PowerShell 作为实现依赖。

Decision: 增加第二个 binary target 作为 hook 回调入口。

### Question: 每个 agent 只保留一份状态够不够？

Recommended answer: 对现在的担忧来说不够；MVP UI 可以只展示聚合状态，但状态存储应提前支持多 task。

Reasoning: 多个 Claude Code 和多个 Codex 任务同时运行时，`agent -> status` 会互相覆盖。Rust hook CLI 能持续接收独立 hook 进程，但必须用稳定 `TaskKey` 和跨进程锁来保证状态不会丢。

Decision: `codex` 和 `claude` 都支持多个 `TaskStatus`；第一版 UI 只消费 `global_summary`。

### Question: UI 应该先渲染一个聚合灯，还是两个 agent 灯？

Recommended answer: 先渲染一个聚合灯。

Reasoning: 当前 widget 是一个很小的单块自绘模块。按-agent布局会立刻扩大 UI 范围。状态 schema 仍然可以保留 per-agent 数据，给以后扩展使用。

Decision: 状态文件同时保存 `global_summary` 和按-agent summary。MVP UI 先使用 `global_summary`，聚合优先级为 `error` > `waiting` > `working` > `done` > `idle`。

### Question: Rust 是否能正确并持续接收多个 Claude/Codex 任务状态？

Recommended answer: 能，但前提是设计成 process-safe 的短命令接收模型。

Reasoning: 每次 hook 触发都会启动一个 Rust CLI 进程。Rust 不是持续订阅方，持续性来自“每次 hook 都更新同一个状态存储”。多任务正确性的关键不是 Rust 能不能运行，而是 `TaskKey`、锁、原子写入和过期清理是否设计到位。

Decision: 使用 task-aware 状态文件、跨进程锁、latest snapshot 和 summary 聚合。

### Question: TaskKey 到底如何生成？

Recommended answer: 使用 `agent_name + "_" + session_id`。

Reasoning: 这个规则足够稳定，也足够直接。`cwd` 不适合作为自动 fallback，因为同一目录里可以同时跑多个 Claude Code 或 Codex 任务。

Decision: `TaskKey` 固定为类似 `claude_123`、`codex_546` 的格式。缺少 `session_id` 时进入 `_unknown` 诊断 task，不静默合并到 cwd。

### Question: 任务栏渲染是否能支持图片、图片+文字、背景文字？

Recommended answer: 文本、背景色、简单形状可以直接支持；背景文字也可以用现有 GDI 路径支持；图片和图片+文字需要单独探针验证后再进主线。

Reasoning: 当前代码使用 `FillRect` 和 `DrawTextW`，这条路径天然适合文本和色块。图片渲染需要 bitmap 加载、缩放、透明通道和 GDI 资源释放，风险高于文字渲染。

Decision: MVP 主线使用文字/色块；新增渲染能力探针记录图片可行性。

### Question: 性能是否需要进入计划？

Recommended answer: 需要，而且应该用预算约束而不是泛泛而谈。

Reasoning: hook CLI 会被 Claude/Codex 的工作流同步调用，如果一次 hook 慢，就会拖慢 agent；widget 轮询如果频繁重绘，会影响任务栏体验。

Decision: 加入 hook 执行耗时、状态文件大小、轮询频率、重绘条件和多任务清理策略。

### Question: hook 配置是否应该自动安装？

Recommended answer: 不应该。

Reasoning: 自动安装会修改仓库外的用户配置文件。在事件映射尚未稳定前，手工样例更安全。

Decision: 只提供 example config。

### Question: 事件乱序时如何避免旧状态覆盖新状态？

Recommended answer: 优先使用 payload 自带时间/序号，缺失时使用 hook CLI 接收时间。

Reasoning: 跨进程锁只能保证同一时间只有一个写入者，不能保证业务事件按顺序到达。必须给每个事件一个 `event_order`，旧事件不能覆盖更新 task snapshot。

Decision: `TaskStatus` 保存 `event_order` 和 `event_order_source`；更新时只接受不旧于当前记录的事件。旧 `Stop` 不能覆盖新的 `working/waiting`。

### Question: 缺少 `session_id` 时如何处理？

Recommended answer: 写入 `_unknown` 诊断 task，但不参与正常 summary 聚合。

Reasoning: 一般预期上游会提供 `session_id`，但计划不能把外部契约视为无条件保证。缺失时如果参与主 summary，会污染红绿灯；如果静默按 `cwd` 合并，会把同目录多任务混在一起。

Decision: 缺少 `session_id` 的事件进入 `agent_unknown` 诊断 task，记录原因，不参与 `global_summary.state` 或按-agent `summary.state`。

### Question: Hook CLI 写状态失败时如何退出？

Recommended answer: 返回非 0。

Reasoning: 状态写入失败如果静默成功，任务栏会展示错误状态且难以排查。

Decision: 非法 JSON、非法 agent、锁超时、写入失败均返回非 0，并通过 stderr 输出短错误。成功路径保持 Codex/Claude 兼容输出。

### Question: 是否允许保存完整 payload？

Recommended answer: 不允许。

Reasoning: payload 可能包含 prompt、代码、路径、命令参数和环境信息。

Decision: 状态文件只保存脱敏摘要、payload shape 和必要状态字段。

### Question: 锁实现优先选什么？

Recommended answer: Windows MVP 使用 Win32 named mutex。

Reasoning: 当前目标平台是 Win11，`CreateMutexW` 的跨进程语义比 lock file 更明确。

Decision: 首选 Win32 named mutex；lock file 只作为 fallback 或未来跨平台方案。

### Question: 状态文件损坏时如何恢复？

Recommended answer: 先备份损坏文件，再创建默认状态。

Reasoning: 直接覆盖会丢失排障证据，直接失败会中断状态链路。

Decision: 损坏文件 rename 为 `state.corrupt.<timestamp>.json`，随后创建默认状态并记录诊断。

### Question: Stale 是否参与 summary 主状态？

Recommended answer: 不参与主状态，但保留诊断字段。

Reasoning: stale `working/waiting` 继续参与主状态会让任务栏长期显示错误信号；直接删除又会丢失重要排障线索。

Decision: stale task 不参与 `global_summary.state` 或按-agent `summary.state`，但设置 `has_stale`、`stale_task_count`。

### Question: 是否需要真实 payload 结构采样？

Recommended answer: 需要，作为 Phase 0。

Reasoning: `session_id`、事件时间/序号、hook name 字段都属于外部契约。先采样可以减少实现返工。

Decision: 增加脱敏 payload shape 采样，不保存完整 payload。

### Question: 是否需要手工状态覆盖 CLI？

Recommended answer: 需要，只作为 debug 能力。

Reasoning: 真实 hook 未接好时，需要验证 widget、summary 和 stale 逻辑；状态卡住时也需要手工清理。

Decision: 提供 `set/clear/list`，所有命令走同一套 named mutex、schema、summary 和原子写入逻辑。

## ADR-001: 使用基于文件的状态交接

Status: MVP 阶段接受。

Context: Codex 和 Claude Code 的 hooks 都是独立执行的命令回调，而任务栏组件是一个长生命周期进程。

Decision: hook 回调写入一个版本化 JSON 状态文件，widget 轮询读取该文件。

Consequences:

- 跨进程通信路径简单。
- 容易做人工构造测试。
- 即使 widget 晚于 hook 事件启动，也能读取已有状态。
- 需要原子写入和容错读取。
- 不是实时推送，但对 MVP 来说 1 秒轮询足够。

## ADR-001A: 状态文件提前支持多 Task

Status: MVP 阶段接受。

Context: 用户明确关心多个 Claude Code 和多个 Codex 任务同时运行时的状态记录能力。

Decision: 状态文件保存 `tasks`、`global_summary` 和按-agent summary。`tasks` 记录每个 task 的 latest snapshot，summary 供任务栏 MVP 聚合显示和后续按-agent扩展。

Consequences:

- 后续可以扩展任务列表 UI。
- 第一版 UI 仍然保持简单。
- 需要 `TaskKey` 提取规则和 stale task 清理。
- 状态文件可能增长，需要设置大小和 TTL 边界。

## ADR-001B: 用跨进程锁保护状态更新

Status: MVP 阶段接受。

Context: 多个 hook 进程可能同时读写 `state.json`。仅靠原子 rename 不能防止 read-modify-write 丢更新。

Decision: 写状态前获取 Win32 named mutex，完成读取、更新、写入、rename 后释放锁。

Consequences:

- 多任务 hook 更新不会互相覆盖。
- Hook CLI 需要处理锁等待超时，并在失败时返回非 0。
- 如果锁实现不稳定，fallback 是 append-only event log。

## ADR-001C: 用事件顺序保护 TaskStatus

Status: MVP 阶段接受。

Context: 多个 hook 进程可能乱序执行。锁解决并发写入，不解决事件语义顺序。

Decision: 每个 hook 事件生成 `event_order`。优先读取 payload 时间/序号字段，缺失时使用 hook CLI 接收时间。更新时只接受 `event_order >= current.event_order`。

Consequences:

- 晚到的旧 `Stop` 不会覆盖新 `working/waiting`。
- 需要记录 `event_order_source`，便于判断顺序可信度。
- 如果真实 payload 不提供顺序字段，MVP 退回接收时间，但诊断里会明确标记。

## ADR-001D: Stale Task 不参与主 Summary

Status: MVP 阶段接受。

Context: 长时间无新事件的 task 可能是进程退出、hook 丢失或任务真的停住。

Decision: 使用保守生命周期：`done` 10 分钟后可清理，`error` 30 分钟后可清理，`waiting` 24 小时后标记 stale，`working` 30 分钟后标记 stale。stale 不参与主状态，但保留诊断字段。

Consequences:

- 主红绿灯不会被旧 task 长期污染。
- 诊断信息仍能提示存在 stale task。
- 需要 summary 计算区分 active 和 stale。

## ADR-001E: 状态文件损坏时备份恢复

Status: MVP 阶段接受。

Context: 状态文件是跨进程共享数据，可能因异常退出、磁盘问题或 bug 损坏。

Decision: 损坏文件 rename 为 `state.corrupt.<timestamp>.json`，再创建默认状态。

Consequences:

- 状态链路可以恢复。
- 保留排障证据。
- 需要避免无限生成 corrupt 文件，后续可加数量上限。

## ADR-001F: 不保存完整 Payload

Status: MVP 阶段接受。

Context: hook payload 可能含敏感内容。

Decision: 只保存脱敏 payload shape 和必要状态字段，不保存完整 payload。

Consequences:

- 降低隐私和泄露风险。
- 排障依赖字段结构而非原文。
- 如需更多信息，只扩展脱敏白名单。

## ADR-001G: Debug CLI 走同一状态链路

Status: MVP 阶段接受。

Context: 真实 hooks 接入前仍需要验证任务栏绘制、summary 聚合和状态清理。

Decision: 提供 `set/clear/list` 调试命令，且必须走同一套 named mutex、schema、summary 和原子写入逻辑。

Consequences:

- 可以不依赖真实 Claude/Codex 会话验证 UI。
- 可以手工清理卡住的 task。
- 该能力不作为用户功能暴露。

## ADR-002: 用 Rust 实现 Hook 接收器

Status: MVP 阶段接受。

Context: 当前仓库是 Rust Win32 项目，没有 Node/Electron 运行时。

Decision: 增加一个 Rust hook CLI binary，解析 hook 参数和 `stdin`，并更新状态。

Consequences:

- CLI 和 widget 可以共享同一份模型。
- 需要引入 `serde` 和 `serde_json`。
- hook 配置里的命令路径必须指向构建出的 binary。
- 避免复制参考项目的运行时架构。

## ADR-003: 外部 Hook 配置保持手工接入

Status: MVP 阶段接受。

Context: Codex 和 Claude Code 配置文件都在仓库外，且通常是用户级文件。

Decision: 提供 example config 和验证命令，但不自动编辑外部配置。

Consequences:

- 第一版接入更安全。
- 手工步骤略多。
- 等真实事件行为稳定后，再考虑自动化安装。

## ADR-004: 从 Win32 消息循环里轮询状态

Status: MVP 阶段接受。

Context: widget 已经有自己的 Win32 message loop 和 paint handler。

Decision: 使用 Win32 timer 定期加载 hook 状态，并在状态变化时 `invalidate` 窗口。

Consequences:

- 不需要 async runtime 或 file watcher。
- timer 行为可以通过日志观察。
- UI 不是即时更新，但延迟被 timer 周期明确限制。

## ADR-005: 渲染能力分层验证

Status: MVP 阶段接受。

Context: 当前渲染路径是 GDI 文本和背景色。用户关心图片、图片+文字、背景文字能力。

Decision: MVP 主线继续使用 GDI 文本、色块和简单形状。背景文字可以作为同一路径验证。图片渲染进入独立探针，验证 bitmap 加载、缩放、透明度、DPI 和资源释放。

Consequences:

- 状态闭环不会被图片资源问题阻塞。
- 能尽早得到图片能力边界。
- 如果图片成本过高，第一版仍可稳定使用文字/色块表达状态。

## ADR-006: 明确性能预算

Status: MVP 阶段接受。

Context: Hook CLI 位于 Claude/Codex 工作流路径上，任务栏绘制位于常驻 UI 路径上。

Decision: 为 hook 执行时间、状态文件大小、轮询频率、重绘条件和 stale task 清理设置预算。

Consequences:

- 后续实现有明确性能验收标准。
- 多任务支持不会无限扩大状态文件。
- 图片渲染必须先通过探针，不进入热路径。

## 术语表

Agent：一个被监控的 coding assistant 进程族。当前 MVP 只包含 `codex` 和 `claude`。

Agent state：归一化后的显示状态，包括 `idle`、`working`、`done`、`waiting`、`error`。

Task：一个可独立跟踪的 Claude Code 或 Codex 工作单元，由 `agent_name + "_" + session_id` 标识。

TaskKey：状态文件中识别 task 的稳定 key，格式为 `agent_name + "_" + session_id`，例如 `claude_123`、`codex_546`。

TaskStatus：某个 task 的最新状态快照，包括 agent、task_id、state、updated_at、event_order、event_order_source 和 message。

Event order：用于判断 hook 事件新旧的顺序值，优先来自 payload 时间/序号，缺失时来自 hook CLI 接收时间。

Aggregate state：由多个 agent state 推导出的单一显示状态，供第一版红绿灯 UI 使用。

Summary：由多个 `TaskStatus` 聚合出来的当前显示状态，供 MVP 任务栏 UI 直接读取。

Global summary：从所有 active task 聚合出来的全局状态，MVP UI 默认读取它。

Agent summary：按 `codex` 或 `claude` 分组聚合出来的状态，供后续按-agent展示使用。

Stale task：超过生命周期阈值且长期无新事件的 task，不参与主 summary 状态，但会进入诊断字段。

Hook：由 Codex 或 Claude Code 触发的生命周期回调，以命令形式执行，并通过 `stdin` 传递事件数据。

Hook receiver：由 hook 配置调用的本地 Rust CLI，用于更新状态文件。

State file：保存 `tasks`、`global_summary` 和按-agent summary 的 JSON 文件。

Payload shape：脱敏后的 payload 字段结构，只用于确认字段契约，不包含 prompt、代码或命令原文。

Atomic write：先写临时文件，再 rename 覆盖目标文件，避免读取到半写入内容。

Waiting heuristic：用于判断 `Stop` 事件实际上是不是“等待用户输入”的一组小规则。

Synthetic hook：人为手工调用、用于模拟真实 hook 事件的测试命令。
