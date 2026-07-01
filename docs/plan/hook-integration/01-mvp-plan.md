# Claude Code / Codex Hook 监控 MVP 计划

## 目标

为当前这个 Rust Win11 任务栏 PoC 实现一条最小但可用的 hook 监控闭环。

当前直接问题：`taskbar-widget` 现在只能绘制固定文本，还没有运行时信号来表示 Claude Code 或 Codex 处于 `working`、`waiting`、`done`、`idle` 还是 `error`。

最小可用结果：Codex 和 Claude Code 的外部 hook 命令能够更新本地状态文件，`taskbar-widget` 读取该文件后绘制一个简单的红绿灯状态。

MVP 范围内：

- Phase 0 脱敏采样：确认真实 Codex / Claude Code hook payload 字段。
- 为 `codex` 和 `claude` 定义共享的 `AgentId` 与 `AgentState`。
- 状态文件提前支持多个并发 task，并同时保存 `global_summary` 和按-agent summary。
- 在本地 app data 目录下维护一个由 Win32 named mutex 保护、原子写入的 JSON 状态文件，并支持环境变量覆盖路径用于验证。
- 提供一个 hook CLI 入口，从 `stdin` 读取 JSON 并把 hook 名映射到状态；同时提供 debug 用 `set/clear/list`。
- 在现有 Win32 消息循环里增加定时轮询，状态变化时触发 `invalidate/repaint`。
- 提供 Codex TOML 和 Claude Code JSON 的示例 hook 配置片段。

MVP 范围外：

- hook 安装器，或自动修改用户配置文件。
- 迁移 `working-light-agent` 的 Electron 风格架构。
- 进程检测、多 workspace UI、多任务列表展示、声音、菜单、设置界面、托盘界面或 dashboard。
- 与本计划无关的 Win11 任务栏可见性深入修复，除非它直接阻塞状态重绘验证。

主验证信号：多个不同 task 的人工 hook 命令修改 `state.json` 后，正在运行的组件能在一个轮询周期内重绘到预期聚合颜色/状态。

## MVP 范围边界

### 必须有

Requirement: 定义 `AgentId = codex | claude` 和 `AgentState = idle | working | done | waiting | error`。  
MVP Justification: 任务栏组件需要一个稳定的状态依赖面，才能映射红/绿/黄/灰显示。

Requirement: 为每个 agent 保存多个 task 的最新状态，并保留一个聚合状态。  
MVP Justification: 多个 Claude Code/Codex 任务可能同时运行；MVP 可以不展示列表，但状态模型不能只用 `agent -> status` 覆盖掉并发任务。

Requirement: 状态更新必须使用跨进程互斥锁 + 原子写入。  
MVP Justification: hook 进程可能并发触发；单纯 rename 只能避免半写入，不能避免两个进程 read-modify-write 时丢更新。

Requirement: 先做真实 payload 脱敏采样。  
MVP Justification: `session_id` 和事件顺序字段属于外部 hook 契约，不能只靠推断；采样只记录字段结构，不保存 prompt、代码或命令原文。

Requirement: 增加一个 Rust hook CLI，调用格式为 `taskbar-widget-hook <codex|claude> <HookName>`。  
MVP Justification: 当前仓库本身就是 Rust 项目；用 Rust 接收 hook 可以避免为了复现参考实现而额外引入 Node。

Requirement: 将常见生命周期 hook 映射到状态，并对 `Stop` 单独处理。  
MVP Justification: `Stop` 既可能表示完成，也可能表示等待用户输入；如果一律映射成 `done`，红绿灯就会误导。

Requirement: 每个 task 状态更新必须带事件顺序判断。  
MVP Justification: 多个 hook 进程可能乱序写入；旧 `Stop` 不能覆盖更新鲜的 `working` 或 `waiting`。

Requirement: 定义 task 生命周期和 stale 策略。  
MVP Justification: 没有 TTL 时 `done/error/working/waiting` 会长期挂住；清理过快又会丢失用户可见反馈。

Requirement: 给组件加入轮询与重绘逻辑。  
MVP Justification: 红绿灯组件必须真正消费 hook 状态，而不只是生成一个文件。

Requirement: 提供 debug CLI、手工验证命令和示例 hook 配置。  
MVP Justification: 真实 hook 配置在仓库外部；示例和人工命令是最快的验证路径。

### 明确不要有

Excluded Item: 自动安装到 Claude Code 或 Codex 配置中。  
Reason for Exclusion: 这会修改用户仓库外文件，不是证明 hook 到 widget 闭环所必需的。

Excluded Item: 长驻 hook daemon。  
Reason for Exclusion: hook 系统已经能直接执行命令；在没有证据证明不够用之前，引入 daemon 只会增加生命周期和 IPC 复杂度。

Excluded Item: 进程检测。  
Reason for Exclusion: 当前直接任务是 hook 监控。基于进程的显示控制可以等组件确实需要隐藏 inactive agent 时再加。

Excluded Item: 多 workspace UI 或 session 列表展示。  
Reason for Exclusion: 状态层需要提前记录 task，但第一版任务栏 UI 不需要展示完整列表。

Excluded Item: 设置 UI、菜单、音效、动画或 dashboard。  
Reason for Exclusion: 这些都不能验证核心状态依赖是否成立。

Excluded Item: 保存完整 hook payload。  
Reason for Exclusion: payload 可能包含 prompt、代码、路径和命令参数；MVP 只保存脱敏摘要和必要状态字段。

### 延后到 MVP 之后

Deferred Item: `codex` 和 `claude` 的进程检测。  
Why Deferred: 它控制的是显示存在性，不是 hook 状态转移本身是否正确。  
Signal to Reconsider: 任务栏 UI 需要在没有匹配进程时隐藏对应 agent。

Deferred Item: 多 workspace UI、session 列表和任务详情面板。  
Why Deferred: 状态层先记录 task，UI 先只显示聚合结果。  
Signal to Reconsider: 用户需要直接查看每个 Claude/Codex 任务的独立状态。

Deferred Item: 外部配置安装器。  
Why Deferred: 在事件映射仍在验证时，手工片段更安全。  
Signal to Reconsider: hook schema 已经稳定，且重复手工配置开始频繁出错。

Deferred Item: 更丰富的按-agent渲染。  
Why Deferred: 一个聚合红绿灯就足以证明集成可用。  
Signal to Reconsider: MVP 跑通后，用户需要一眼区分 Codex 和 Claude。

## 背景与上下文

当前已知行为：

- `taskbar-widget/src/main.rs` 负责创建 Win32 窗口、绘制固定文本并运行消息循环。
- `taskbar-widget/src/taskbar.rs` 负责探测和挂接 Win11 任务栏，并输出诊断信息。
- `taskbar-widget/src/win32.rs` 提供 Win32 辅助函数。
- `taskbar-widget/Cargo.toml` 目前只依赖 `windows` crate。

来自 `D:/project/working-light-agent/docs/implementation-guide.md` 的参考行为：

- hook 回调是短生命周期命令执行，不是持久订阅流。
- hook 载荷通过 `stdin` 以 JSON 形式传入。
- 本地状态文件是跨进程交接面。
- `Stop` 需要特殊处理，因为它可能表示完成，也可能表示等待用户输入。

当前假设：

- Codex 和 Claude Code 都支持执行 command hook，并通过 `stdin` 传 JSON。
- 真实 payload 大概率包含 `session_id`，但必须通过 Phase 0 采样确认字段名和嵌套位置。
- 在真实 hook 接入前，先用人工构造的 `stdin` 载荷做 MVP 验证是可接受的。
- 为了 JSON 解析和写入，引入 `serde` 与 `serde_json` 是合理且必要的。

## 当前状态分析

相关文件：

- `taskbar-widget/src/main.rs`：增加 timer 设置、读取当前 agent 状态，并把固定 `TASKBAR WIDGET` 绘制改为红绿灯输出。
- `taskbar-widget/src/taskbar.rs`：除非重绘定位受影响，否则不改任务栏探测策略。
- `taskbar-widget/src/win32.rs`：尽量复用现有日志辅助，只在确有需要时加很小的辅助函数。
- `taskbar-widget/Cargo.toml`：加入 JSON 序列化依赖，并在需要时定义第二个 binary。
- `docs/plan/hook-integration/`：存放本次集成计划文档。

影响 MVP 的已知限制：

- 当前任务栏真实可见性仍在诊断中。如果组件暂时不可见，就通过日志或窗口捕获验证重绘，不要把这份 hook 计划扩展成新的 taskbar-host 研究。
- 目前没有正式测试套件。MVP 验证以 `cargo check` 和确定性的人工 hook 命令为主。

依赖或外部系统：

- Claude Code 的 hook 配置是仓库外 JSON。
- Codex 的 hook 配置是仓库外 TOML。
- Windows app data 路径解析依赖本机 OS 行为。

## MVP 决策门

这是否解决直接问题？是。它补上了任务栏红绿灯当前缺失的状态依赖。

是否可以去掉其中某项仍然验证 MVP？不行。hook CLI、状态文件和 widget 轮询都不能省。进程检测、安装器和复杂 UI 可以省。

是否存在可复用的现有项目模式，避免重新设计？有。保持当前 Rust 模块拆分，只新增一个小型 state/hook 模块，而不是导入 Electron 架构。

计划里是否有为未来假设做准备的内容？多 task 状态记录现在需要纳入；进程检测、session 列表 UI、配置安装器和设置界面仍属于未来项，应延期。

新依赖是否真的必要？`serde` 和 `serde_json` 对可靠的 JSON 解析/写入是必要的。除此之外没有其它依赖被 MVP 证明需要。

最简单可接受实现：

- 一个 task-aware JSON 状态文件，包含 `tasks`、`global_summary` 和 `agents.*.summary`。
- 一个 Rust hook 命令。
- 一个 debug CLI：`set/clear/list`。
- 一个任务栏窗口轮询 timer。
- 一个聚合后的渲染状态。

Keep: Phase 0 脱敏采样、hook CLI、debug CLI、task-aware 状态文件、Win32 named mutex、原子写入、状态映射、widget 轮询、示例配置。

Remove: daemon、安装器、dashboard、外部服务、通用插件架构。

Defer: 进程检测、session 列表 UI、按-agent布局、声音、偏好设置。

Simplify: 先渲染一个聚合状态；每个 task 的 JSON snapshot 保留，后续如有需要再扩展显示。

## 建议的 MVP 方案

MVP 只在 `taskbar-widget` 内增加一个很小的本地状态子系统。hook 命令按 task 更新状态文件，运行中的组件每秒轮询一次，并基于所有活跃 task 重绘一个简单聚合状态指示。

### Decision: 使用 task-aware JSON 文件作为交接面

Choice: 在本地 `state.json` 中保存 `HookMonitorState`，包含 `tasks`、`global_summary` 和 `agents.codex/claude.summary`。

MVP Justification: hooks 是短生命周期命令，而组件是长生命周期进程。文件是最小且可靠的跨进程桥接方式；`tasks` 避免并发任务互相覆盖，`global_summary` 让 MVP UI 仍然保持简单，按-agent summary 给后续分列展示预留稳定 schema。

Sufficient evidence used: 参考实现采用同样模型，并明确把 hook 回调和 UI 轮询解耦。

Simpler Alternative Considered: 只把状态打印到 `stdout`。

Why Not More Complex: 在文件轮询被证明不够之前，named pipe、socket、shared memory 或 daemon 都没有必要。

### Decision: 用 Rust 实现 Hook 接收器

Choice: 增加第二个 Rust binary，例如 `src/bin/taskbar_widget_hook.rs`。

MVP Justification: 当前仓库是纯 Rust；用 Rust 接 hook 可以避免引入 Node 运行时假设。

Simpler Alternative Considered: 用 PowerShell 脚本改 JSON。

Why Not More Complex: PowerShell 做原子写入和类型化状态规则都更脆弱。

### Decision: 每个 Task 保留最新状态

Choice: 持久化 `tasks: Record<TaskKey, TaskStatus>`，每项包含 `agent`、`task_id`、`state`、`updated_at`、`event_order`、可选 `workspace` 和 `message`。

MVP Justification: 多个任务同时运行时，`agent -> status` 会覆盖；`task -> status` 可以持续记录多个任务，任务栏第一版只读取聚合结果。

Simpler Alternative Considered: 只存一个聚合状态，或只存 `codex`/`claude` 两个状态。

Why Not More Complex: 不记录完整事件历史，不做任务详情 UI，只保留最新 task snapshot 和聚合结果。

### Decision: 用 `agent_name + "_" + session_id` 生成 TaskKey

Choice: 正常路径使用 `TaskKey = agent_name + "_" + session_id`，例如 `claude_123`、`codex_546`。如果 payload 缺少 `session_id`，不使用 `cwd` 自动合并任务，而是写入 `agent_name + "_unknown"` 并记录 `task_id_source = missing_session_id` 诊断。

MVP Justification: Rust 可以持续接收多个 hook 进程，但要正确区分任务，必须有稳定 key。`cwd` 不能唯一代表任务，同目录并发会被错误合并；`agent_name + session_id` 是多任务状态的主键。

Simpler Alternative Considered: 全部按 agent 聚合。

Why Not More Complex: 不引入外部任务注册服务；如果上游确实不提供 `session_id`，先暴露诊断，再决定是否让 hook 配置显式传入。

### Decision: 用跨进程锁保护状态更新

Choice: Windows MVP 优先使用 Win32 `CreateMutexW` named mutex。写入前获取 mutex，完成 read-modify-write 后释放；写入仍采用临时文件 + rename。

MVP Justification: 多任务并发 hook 会让多个进程同时更新状态。锁负责防止丢更新，rename 负责防止半写入。

Simpler Alternative Considered: 只使用原子 rename。

Why Not More Complex: 暂不引入 daemon 或数据库；Win32 named mutex 语义明确，lock file 只作为 fallback。

### Decision: 使用事件顺序防止旧 Hook 覆盖新状态

Choice: 优先使用 hook payload 自带的时间或序号字段生成 `event_order`；缺失时使用 hook CLI 接收时间。更新 task 时，只接受 `event_order >= current.event_order` 的事件。旧事件只写诊断，不覆盖状态。

MVP Justification: 跨进程锁只能保证写入互斥，不能保证业务事件顺序。没有顺序判断时，晚到的旧 `Stop` 可能把新的 `working/waiting` 覆盖成 `done`。

Simpler Alternative Considered: 只用锁保证顺序。

Why Not More Complex: 不引入事件队列或 daemon；只在 latest snapshot 更新时做一次顺序比较。

### Decision: 轮询而不是文件监听

Choice: 使用 Win32 timer 定期读取状态文件。

MVP Justification: 现有应用已经有 Win32 消息循环；轮询实现简单、可调试，也避免 watcher 的平台边界问题。

Simpler Alternative Considered: 只在启动时读取一次。

Why Not More Complex: 对 1 秒级别的 UI 更新来说，没有必要引入 file watcher 或 async runtime。

### Decision: Stale 不参与主 Summary

Choice: 生命周期采用保守保留：`done` 保留 10 分钟，`error` 保留 30 分钟，`waiting` 24 小时无新事件后标记 stale，`working` 30 分钟无新事件后标记 stale。stale task 不参与 `global_summary.state` 和按-agent `summary.state`，但设置 `has_stale = true` 并保留诊断计数。

MVP Justification: 卡住的 `working/waiting` 不应长期锁住任务栏主灯；同时 stale 是重要诊断信息，不能静默删除。

Simpler Alternative Considered: stale 继续参与主状态，或立刻清理。

Why Not More Complex: 不做完整任务历史，只保留 latest snapshot 和诊断字段。

### Decision: Hook CLI 失败必须可见

Choice: 状态写入失败返回非 0；stderr 输出短错误；Codex 成功路径继续输出 `{}`。非法 JSON、非法 agent、锁超时、写入失败都不能静默成功。

MVP Justification: hook 状态丢失如果静默，会让任务栏显示错误状态且难以排查。失败可见比“看起来没事”更重要。

Simpler Alternative Considered: 永远返回 0，避免影响 Claude/Codex。

Why Not More Complex: 不做复杂恢复，只保证失败可诊断。

### Decision: 不保存 Hook Payload 原文

Choice: 状态文件只保存 `agent`、`session_id`、`TaskKey`、hook name、状态、短 message、event_order、诊断字段和脱敏 payload shape。完整 payload 永不保存。

MVP Justification: payload 可能包含用户 prompt、代码片段、路径、命令参数和环境信息。状态灯不需要这些内容。

Simpler Alternative Considered: debug 模式保存完整 payload。

Why Not More Complex: 脱敏 shape 足够用于字段契约排障。

### Decision: 状态文件损坏时先备份再恢复

Choice: 读取到损坏 JSON 时，将原文件 rename 为 `state.corrupt.<timestamp>.json`，再创建默认状态，并记录诊断。

MVP Justification: 直接覆盖会丢失排障证据；直接失败会让状态链路停住。

Simpler Alternative Considered: 直接覆盖成默认状态，或 CLI 失败退出。

Why Not More Complex: 备份 + 默认状态是最小可恢复路径。

### Decision: 先渲染聚合状态

Choice: 绘制一个由两个 agent 状态推导出的单一红绿灯状态。

MVP Justification: 这已经足以证明任务栏组件可以依赖 hook 状态，而无需立刻重设计布局。

Suggested precedence: `error` > `waiting` > `working` > `done` > `idle`。

Why Not More Complex: 按-agent列展示、标签和响应式布局应当等到底层状态信号被证明确实稳定后再做。

### Decision: 第一版渲染使用 GDI 文本和基础图形，图片能力单独验证

Choice: MVP 继续使用 GDI 绘制背景、文字和简单形状；图片、图片+文字、水印式背景文字列入 Phase 5 探针验证。

MVP Justification: 当前代码已经是 GDI 绘制路径，文本和背景色可以直接支持；图片渲染涉及 bitmap 加载、透明通道、缩放和资源生命周期，应该独立验证后再进入主线。

Simpler Alternative Considered: 立刻引入 GDI+ 或 WIC。

Why Not More Complex: 任务栏红绿灯状态验证不依赖图片。先证明状态闭环，再评估图片渲染成本。

### Decision: 增加 Debug 状态 CLI

Choice: 同一个 hook binary 或相邻 binary 提供 `set <task_key> <state>`、`clear <task_key>`、`list`。该能力只用于调试，仍走 named mutex、schema、summary 和原子写入。

MVP Justification: 真实 hook 接入前，需要验证 widget 渲染、summary 聚合和状态清理；hook 不稳定时也需要手工清理卡住 task。

Simpler Alternative Considered: 只接受真实 hook。

Why Not More Complex: 不做用户设置 UI，不做长期管理工具。

## 考虑过的替代方案

Description: 复制完整的 `working-light-agent` Electron 架构。  
Advantages: 已经具备 hooks、进程检测、renderer state 和 preferences。  
Disadvantages: 与当前 Rust Win32 仓库不匹配，还会引入无关的运行时层。  
MVP Fit: 差。  
Reason Not Selected: 当前需求是给任务栏组件补上 hook 状态依赖，不是迁移整套桌面应用。

Description: 只做进程检测。  
Advantages: 不需要 hook 配置。  
Disadvantages: 无法区分 `working`、`waiting`、`done` 或 `error`。  
MVP Fit: 差。  
Reason Not Selected: 用户明确要求的是 hook 监控。

Description: 为 hooks 增加本地 HTTP server。  
Advantages: 更容易用浏览器工具观察。  
Disadvantages: 需要 daemon、端口管理、生命周期处理和安全决策。  
MVP Fit: 过大。  
Reason Not Selected: command hook + 文件状态已经足够。

Description: 只提供手工 CLI 改状态。  
Advantages: 最简单。  
Disadvantages: 不能验证 Claude/Codex hook 集成本身。  
MVP Fit: 不完整。  
Reason Not Selected: 它可以作为调试辅助，但不能作为 MVP 核心。

## 实施计划

### Phase 0: 真实 Payload 脱敏采样

Goal: 确认 Codex 和 Claude Code 真实 hook payload 的字段契约。

Files:

- `taskbar-widget/src/bin/taskbar_widget_hook.rs`
- `docs/checklist/hook-payload-sampling.md`

Tasks:

- 增加 `sample` 或环境变量采样模式，只输出脱敏 payload shape。
- 验证不同 hook 是否都包含 `session_id`。
- 验证是否存在 payload 时间或序号字段。
- 验证 Codex 成功输出和 Claude 静默输出的兼容边界。
- 不保存 prompt、代码、命令参数或完整路径。

Expected Result: 得到一份字段清单，确认 Phase 1/2 的 `TaskKey` 和 `event_order` 解析规则。

MVP Check:

- Why necessary: 上游 hook payload 是外部契约，不应只靠参考文档猜字段。
- Intentionally not included: 长期日志采集、完整 payload 归档、自动安装。

### Phase 1: 共享 Hook 状态模型

Goal: 增加类型化状态定义和 JSON 存储辅助。

Files:

- `taskbar-widget/src/agent_state.rs`
- `taskbar-widget/Cargo.toml`

Tasks:

- 增加 `AgentId`、`AgentState`、`AgentStatus` 和 `HookMonitorState`。
- 增加 `TaskKey`、`TaskStatus`、`HookSummary`，状态文件中同时保存 `tasks`、`global_summary` 和按-agent summary。
- 在 `TaskStatus` 中加入 `event_order` 和 `event_order_source`，用于拒绝旧事件覆盖。
- 在 `HookSummary` 中加入 `has_stale`、`stale_task_count`、`active_task_count` 和 `highest_priority_task`。
- 增加默认状态创建逻辑。
- 增加带 `TASKBAR_WIDGET_STATE_HOME` 覆盖的 app data 路径解析。
- 增加 Win32 named mutex 和原子 JSON 读写辅助。
- 增加对缺失/损坏文件的处理：损坏文件先备份为 `state.corrupt.<timestamp>.json`，再创建默认状态并记录日志。
- 增加 conservative TTL 和 stale 标记逻辑。

Expected Result: Rust 代码可以稳定读取和写入合法状态文件。

MVP Check:

- Why necessary: hook CLI 和 widget 需要共享同一份状态契约，并且不能在多任务并发时互相覆盖。
- Intentionally not included: preferences、完整 history、schema migration 或任务详情 UI。

Evidence Used: 参考实现使用版本化状态对象；当前计划在此基础上提前加入 task 维度，避免后续多任务扩展重做 schema。

Output Format: JSON 文件。

Read-Only vs Mutating Behavior: store helper 只修改本地状态文件。

### Phase 2: Hook CLI

Goal: 产出一个可供 Codex 和 Claude Code 调用的 command hook 目标。

Files:

- `taskbar-widget/src/bin/taskbar_widget_hook.rs`
- `taskbar-widget/src/hook_rules.rs`
- `taskbar-widget/src/agent_state.rs`

Tasks:

- 解析参数 `<codex|claude> <HookName>`。
- 从 `stdin` 读取完整字符串。
- 把 `stdin` 解析成 JSON object；对人工验证允许空输入视作 `{}`。
- 缺失 `session_id` 时写入 `_unknown` 诊断 task，但不参与正常 summary 聚合。
- 从参数和常见 payload 字段中解析最终 hook name。
- 从 payload 中提取 `session_id`，生成 `agent_name + "_" + session_id` 形式的稳定 `TaskKey`。
- 从 payload 中提取事件时间/序号；缺失时使用 hook CLI 接收时间作为 `event_order`。
- 把生命周期 hooks 映射成状态。
- 对 `Stop` 单独判断：结合 previous state 和 waiting-like 文本。
- 比较 `event_order`，拒绝旧事件覆盖新状态。
- 用时间戳、`event_order` 和 message 写回 task 状态，并刷新 `global_summary` 和按-agent summary。
- 写入失败、锁超时、非法 JSON 或非法 agent 时返回非 0，并在 stderr 输出短错误。
- 为兼容 Codex 成功路径输出 `{}`；Claude 成功路径保持静默。

Expected Result: 不启动 widget 也能通过人工命令更新 `state.json`。

MVP Check:

- Why necessary: 这是对两个 hook 系统的直接集成点。
- Intentionally not included: 安装器、shell wrapper、config auto-patching、daemon mode。

Evidence Used: 参考实现明确记录了 command hook 调用方式和 `stdin` JSON 载荷。

Filtering Before Reasoning: 只使用 hook name 和部分文本字段，不保存完整 payload。

Context Boundary: 一次只处理一个 hook 事件载荷；状态文件可同时保留多个 task 的 latest snapshot。

Human Review Point: 用户手工审阅生成的 Codex/Claude 配置片段后再安装。

### Phase 2A: Debug 状态 CLI

Goal: 提供不依赖真实 hook 的调试入口。

Files:

- `taskbar-widget/src/bin/taskbar_widget_hook.rs`
- `taskbar-widget/src/agent_state.rs`

Tasks:

- 增加 `set <task_key> <state>`。
- 增加 `clear <task_key>`。
- 增加 `list`。
- 所有 debug 命令走同一套 named mutex、schema、summary 和原子写入逻辑。

Expected Result: 可以不启动 Claude/Codex，直接验证 widget 渲染、summary 和 stale 逻辑。

MVP Check:

- Why necessary: 真实 hook 不稳定时仍能验证状态链路和手工清理卡住 task。
- Intentionally not included: 用户设置 UI、长期任务管理器。

### Phase 3: 组件轮询与绘制

Goal: 让当前任务栏组件真正消费 hook 状态。

Files:

- `taskbar-widget/src/main.rs`
- `taskbar-widget/src/agent_state.rs`

Tasks:

- 在窗口创建完成后调用 `SetTimer`。
- 处理 `WM_TIMER`：读取状态、在 `global_summary` 变化时触发 `invalidate`。
- 把固定 `MODULE_TEXT` 绘制替换成与状态相关的文本和颜色。
- 使用多 task 聚合优先级：`error`、`waiting`、`working`、`done`、`idle`。
- stale task 不参与主状态，但 `has_stale` 可用于诊断日志。
- 为诊断记录状态切换日志。

Expected Result: 运行中的组件在人工 hook 调用后切换颜色/文本。

MVP Check:

- Why necessary: 任务栏红绿灯必须真正依赖 hook 状态。
- Intentionally not included: 按-agent列、动画、自定义字体、声音、设置或右键菜单。

Output Format: 现有 Win32 自绘窗口。

Read-Only vs Mutating Behavior: widget 只读状态文件；只有 hook CLI 负责写入。

### Phase 4: 文档和验证样例

Goal: 提供接入真实 hooks 所需的最小操作说明。

Files:

- `taskbar-widget/examples.codex-hooks.toml`
- `taskbar-widget/examples.claude-hooks.json`
- `taskbar-widget/README.md`
- `docs/checklist/hook-integration-validation.md`

Tasks:

- 增加指向已构建 hook binary 路径的示例配置片段。
- 增加 `UserPromptSubmit`、`PermissionRequest`、`Stop` 和 `StopFailure` 的人工验证命令。
- 记录 `TASKBAR_WIDGET_STATE_HOME` 的隔离测试用法。
- 记录 debug CLI 的 `set/clear/list` 用法。
- 给出期望状态文件样例。

Expected Result: 开发者无需猜命令格式即可完成验证。

MVP Check:

- Why necessary: hook 配置在仓库外部，所以需要样例来支撑安全的手工接入。
- Intentionally not included: 自动安装或全局配置修改。

### Phase 5: 渲染能力探针

Goal: 明确任务栏窗口渲染图片、图片+文字、背景文字的可行边界。

Files:

- `taskbar-widget/src/main.rs`
- `taskbar-widget/src/render_probe.rs`
- `docs/checklist/hook-rendering-capability.md`

Tasks:

- 验证 GDI 文本、背景色、简单形状的稳定绘制。
- 验证背景文字：先用 `DrawTextW` 低对比度绘制，再叠加前景状态文本。
- 验证 bitmap 图片加载和缩放：优先用 Win32/GDI 路径，不直接进入主 UI。
- 记录透明 PNG、DPI 缩放、资源释放、闪烁和任务栏裁剪表现。

Expected Result: 得到一份明确结论：MVP 主线使用哪种绘制能力，哪些图片能力需要延后。

MVP Check:

- Why necessary: 用户关心任务栏最终表现形态，必须尽早验证能力边界。
- Intentionally not included: 复杂皮肤系统、动态主题、图片资源管理器。

## 验证策略

最小命令：

```powershell
cd D:\project\cc-traffic-light\taskbar-widget
cargo check
cargo build
```

人工 hook 验证：

```powershell
$env:TASKBAR_WIDGET_STATE_HOME = "$PWD\target\hook-state-test"
'{"hook_event_name":"UserPromptSubmit","message":"start"}' | target\debug\taskbar_widget_hook.exe codex UserPromptSubmit
Get-Content "$env:TASKBAR_WIDGET_STATE_HOME\state.json"
'{"hook_event_name":"PermissionRequest","message":"approval required"}' | target\debug\taskbar_widget_hook.exe claude PermissionRequest
Get-Content "$env:TASKBAR_WIDGET_STATE_HOME\state.json"
'{"hook_event_name":"Stop","message":"Build finished successfully."}' | target\debug\taskbar_widget_hook.exe codex Stop
Get-Content "$env:TASKBAR_WIDGET_STATE_HOME\state.json"
```

手工组件验证：

```powershell
$env:TASKBAR_WIDGET_STATE_HOME = "$PWD\target\hook-state-test"
cargo run
```

期望结果：

- `UserPromptSubmit` 把目标 agent 设为 `working`。
- `PermissionRequest` 把目标 agent 设为 `waiting`。
- `StopFailure` 把目标 agent 设为 `error`。
- `Stop` 把目标 agent 设为 `done`，除非 previous state 已经是 `waiting`，或文本看起来像是在等待用户输入。
- 组件会在一个 timer 周期内按聚合优先级完成重绘。
- 多个不同 `session_id` 的 hook 会生成不同 `TaskKey`，不会互相覆盖；`global_summary` 会反映所有 active task 的最高优先级状态。
- 旧事件不会覆盖更新鲜状态；尤其是旧 `Stop` 不能覆盖新的 `UserPromptSubmit` 或 `PermissionRequest`。
- 缺失 `session_id` 的事件进入 `_unknown` 诊断 task，且不参与正常 summary 聚合。
- stale task 不参与主状态，但会设置 `has_stale`。
- debug CLI 的 `set/clear/list` 可以验证 summary 和重绘，不依赖真实 hook。

关键失败场景：

- 非法 agent id 以非 0 退出，且不写状态。
- 非法 JSON 以非 0 退出，且不破坏已有状态。
- 锁超时或状态写入失败以非 0 退出，并输出短错误。
- 状态文件缺失时自动创建默认状态。
- 状态文件损坏时先备份为 `state.corrupt.<timestamp>.json`，再通过受控回退路径恢复。

当前不需要测试的内容：

- 真实全局 Claude/Codex 配置修改。
- 多个并发 session 的 UI 列表展示。
- 进程检测。
- 长时间运行稳定性。
- classic taskbar 或多显示器行为。

## 性能预算

MVP 的性能目标不是做高频实时监控，而是保证 hook 命令不拖慢 Claude/Codex，widget 轮询不造成可感知负担。

预算：

- Hook CLI 单次执行目标：正常路径小于 100ms，超过 300ms 需要记录诊断。
- 状态文件大小目标：MVP 保持在 64KB 以内；超过后需要清理 stale task 或截断 message。
- Widget 轮询频率：默认 1000ms；状态未变化时不触发重绘。
- 绘制耗时目标：`WM_PAINT` 内只做内存中状态读取和 GDI 绘制，不做磁盘 IO。
- 多任务保留策略：只保存 latest snapshot；`done` 10 分钟后可清理，`error` 30 分钟后可清理，`waiting` 24 小时无新事件后标记 stale，`working` 30 分钟无新事件后标记 stale。stale 不参与主 summary，但保留 `has_stale` 诊断。

性能验证：

- 人工连续触发 20-50 次 hook，确认没有 JSON 损坏或明显延迟。
- 构造 10 个并发 task，确认状态文件大小和聚合计算稳定。
- 构造同一 task 的乱序事件，确认旧事件只记录诊断，不覆盖新状态。
- 在 widget 运行时观察日志，确认未变化状态不会连续重绘。
- 图片渲染探针单独记录资源加载和绘制耗时，不进入 MVP 主循环。
- 采样模式确认不保存完整 payload。

## 风险与缓解

Risk: 实际 Codex 或 Claude Code hook payload 字段和示例不同。  
Impact: hook name 或 waiting 文本可能被误分类。  
Likelihood: 中。  
MVP Mitigation: Phase 0 先做脱敏 payload shape 采样；同时从 `argv` 和常见 payload 字段解析 hook name，并在 `message` 中保存最终 hook name。  
Fallback Plan: 如果真实 payload 缺字段，先进入诊断路径，再调整解析规则。

Risk: `Stop` waiting 启发式出现误判。  
Impact: 需要用户输入时显示成绿色，或完成时显示成红/黄。  
Likelihood: 中。  
MVP Mitigation: 如果 previous state 已经是 `waiting` 则优先保持；并使用一份小规模中英文 waiting pattern 列表。  
Fallback Plan: 观察真实 payload 后再补充 pattern。

Risk: 状态文件写入竞争。  
Impact: 更新丢失或文件损坏。  
Likelihood: 低到中。  
MVP Mitigation: 使用 Win32 named mutex 保护 read-modify-write，并使用临时文件 + rename 的原子写入。  
Fallback Plan: 如果锁实现复杂或不稳定，退回 append-only event log，并由 widget 聚合最新状态。

Risk: hook 事件乱序到达，导致旧状态覆盖新状态。  
Impact: 例如旧 `Stop` 晚到，把新的 `working` 覆盖成 `done`。  
Likelihood: 中。  
MVP Mitigation: 每个事件计算 `event_order`，只允许新事件覆盖旧 task snapshot。  
Fallback Plan: 如果 payload 没有可信顺序字段，统一使用 hook CLI 接收时间并记录 `event_order_source = received_at`。

Risk: `session_id` 缺失或不稳定，导致多个任务被错误合并。  
Impact: 多任务状态互相覆盖，聚合结果不可信。  
Likelihood: 中。  
MVP Mitigation: 正常路径只用 `agent_name + "_" + session_id`；缺失时进入 `_unknown` 诊断 task，不参与正常 summary，并明确打诊断日志。  
Fallback Plan: 如果真实 payload 缺失 `session_id`，在 hook 配置中允许显式传入 session id。

Risk: 完整 payload 泄露敏感信息。  
Impact: 状态文件可能包含 prompt、代码片段、路径或命令参数。  
Likelihood: 中。  
MVP Mitigation: 永不保存完整 payload，只保存脱敏 shape 和短 message。  
Fallback Plan: 如需排障，临时扩大脱敏字段白名单，而不是保存原文。

Risk: stale task 长期污染红绿灯状态。  
Impact: 一个卡住的 `working/waiting` 可能长期占据主状态。  
Likelihood: 中。  
MVP Mitigation: stale 不参与主 summary，只设置 `has_stale` 和计数。  
Fallback Plan: 使用 debug CLI `clear` 手工清理指定 task。

Risk: 图片渲染引入额外 CPU/GDI 资源压力。  
Impact: 任务栏组件闪烁、卡顿或资源泄漏。  
Likelihood: 中。  
MVP Mitigation: MVP 主线只做文本/形状；图片进入独立探针并记录耗时和资源释放。  
Fallback Plan: 图片能力延期，只保留文字和色块。

Risk: 当前 Win11 任务栏可见性问题掩盖 UI 验证。  
Impact: hook 状态本身是通的，但用户看不到。  
Likelihood: 已知当前风险。  
MVP Mitigation: 通过日志和 repaint 调用验证状态变化，不要把 hook 正确性和 host 可见性混在一起。  
Fallback Plan: 继续沿用现有任务栏可见性诊断路径单独处理。

## 过度设计警戒清单

- 不要引入 Electron、Node 或浏览器 UI。
- 不要创建通用插件系统。
- 不要自动修改用户的 Codex 或 Claude Code 配置文件。
- 不要增加 daemon、HTTP server、WebSocket、数据库或 async runtime。
- 不要在 MVP 中做 session 列表 UI、workspace 管理、嵌套任务模型或完整任务历史。
- 可以在状态文件中记录 task latest snapshot，但不要做 task 详情 UI。
- 不要保存完整 hook payload。
- 不要在实现 hook 状态时顺手重构 taskbar probing。
- 在 hook 状态渲染跑通前，不要加进程检测。
- 不要加声音、动画、菜单、偏好设置或 settings UI。
- 不要借这个计划扩展到 classic taskbar 或多显示器支持。

## 已确认问题

Question: MVP 应该先渲染一个聚合灯，还是两个按-agent灯？  
Why It Matters: 这会直接影响绘制和布局复杂度。  
Decision: 状态文件同时保存 `global_summary` 和按-agent summary；MVP UI 先使用 `global_summary`。

Question: 多任务 task key 应该如何生成？  
Why It Matters: key 不稳定会导致状态覆盖或任务泄漏。  
Decision: 使用 `agent_name + "_" + session_id`，例如 `claude_123`、`codex_546`；缺失 `session_id` 时进入 `_unknown` 并记录诊断，不用 `cwd` 自动合并。

Question: 事件乱序时如何决定是否覆盖状态？  
Why It Matters: hook 进程并发执行，写入锁不等于业务顺序。  
Decision: 优先使用 payload 自带时间/序号；缺失时使用 hook CLI 接收时间。只接受不旧于当前 `event_order` 的事件。

Question: Stale task 是否参与主 summary？  
Why It Matters: 卡住的 task 可能长期污染任务栏主状态。  
Decision: 不参与主状态，但保留 `has_stale`、`stale_task_count` 和诊断日志。

Question: 是否需要 debug 状态 CLI？  
Why It Matters: 真实 hook 接入前仍需验证 widget 和 summary，hook 异常时也需要手工清理。  
Decision: 需要 `set/clear/list`，只作为 debug 能力。

Question: 图片渲染是否属于第一版 UI 必须能力？  
Why It Matters: 图片会引入资源加载、缩放、透明度和性能验证成本。  
Decision: 不作为第一版红绿灯必须能力；先做 Phase 5 探针。

Question: Windows 生产状态文件应该放在哪里？  
Why It Matters: hook 命令和 widget 必须使用同一路径。  
Decision: 使用 `%APPDATA%\CcTrafficLight\state.json`，测试时用 `TASKBAR_WIDGET_STATE_HOME` 覆盖目录。

Question: 真实 hook 配置安装是否要自动化？  
Why It Matters: 这会修改外部工具的用户配置。  
Decision: 不自动化，只提供样例。

## 推荐下一步

先执行 Phase 0：增加脱敏 payload shape 采样模式，确认真实 Codex 和 Claude Code hook payload 中的 `session_id`、hook name、事件时间/序号字段，再进入状态模型实现。
