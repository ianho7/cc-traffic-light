# 安装后监控链路全面审查 Checklist

## Checklist Objective

逐项验收 CC Traffic Light 从 release 构建、安装、Codex / Claude hook、共享状态、detector、Win32 taskbar，到设置页和最终闪烁效果的完整流程。

目标：

- Codex 的全局 hook 安装、trust、事件写入、状态展示和恢复闭环。
- Claude 的当前支持等级有证据；command hook 不稳定时明确降级为进程检测。
- 安装器、升级、卸载、用户配置保护和首次运行引导可重复验证。
- Working、NeedsAttention、Completed、Error 的最终桌面效果有人工证据。

非目标：绕过 trust、企业 managed hooks、多显示器、云端会话和远程控制会话。

---

## Closed-Loop Execution Contract

状态标记：x=completed，~=partial，空白=pending；正文带 blocked 表示环境或人工阻塞。

每一轮只推进一个最小未完成 task，流程固定为：

1. 读取当前代码、checklist 状态和上一次证据。
2. 记录假设、预期验证信号和风险。
3. 做一次小范围、可回滚的修改或实验。
4. 运行 focused verifier，保存原始输出、日志、截图或 state JSON 路径。
5. verifier 通过后才勾选 task，并生成 reflection。
6. 根据证据选择继续、重规划、阻塞或结束。

失败规则：

- 瞬态/环境失败最多重试一次。
- 逻辑失败必须更新假设或补验证，不能盲改。
- 同一原因连续两次失败后停止重复尝试，转为重新规划或人工接管。
- 缺少 Claude 环境、trust、版本或产品决策时，标记 Blocked，不伪造完成。

状态与退出：

- Complete：task 验收证据存在。
- Blocked：需要外部环境、人工 trust 或产品决策。
- Risk stop：下一步会修改真实用户配置或执行高风险操作。

---

## Pre-Implementation Checks

- [x] AUD-PRE-01 确认当前工作区已有变更属于用户，避免覆盖未完成实现。
- [x] AUD-PRE-02 阅读 AGENTS.md、本 checklist 和 docs/plan/end-to-end-install-monitoring-audit.md。
- [x] AUD-PRE-03 阅读最新 Claude retrospective，并将较早 investigation 标记为历史证据。
- [~] AUD-PRE-04 记录 Windows、Claude Code、Codex、Git Bash、PowerShell 和 Rust 版本。（Windows 11 build 26100.8655、Claude Code 2.1.204、PowerShell 5.1.26100.8655、Rust/Cargo 1.93.0、Git 2.38.1、Codex CLI 0.144.2；2026-07-13 已在 `[windows] sandbox = "unelevated"` 下通过默认 launcher 的只读 sandbox probe。Git Bash 入口、pnpm/Node 与 ISCC 仍由用户接管，待结果回填。）
- [x] AUD-PRE-05 准备临时用户配置、临时状态目录和清洁安装目录。
- [x] AUD-PRE-06 确认基础验证命令和发布构建顺序。

## Phase 0: 基线与文档纠偏

- [x] AUD-0-01 更新旧计划中“installer 缺少 hook exe”等过时描述。
- [x] AUD-0-02 记录 Claude 官方文档规范与 retrospective 实测的差异。
- [x] AUD-0-03 建立证据表：配置存在、hook 触发、stdin 有效、state 写入、detector 更新、桌面可见。
- [x] AUD-0-04 将 examples.claude-hooks.json 标记为实验配置。
- [x] AUD-0-05 为每个待验证项指定环境、命令、证据位置和停止条件。
- [ ] AUD-0-GATE Phase 0 证据表完整，后续不再依赖矛盾历史结论。

### Phase 0 evidence matrix

| Evidence stage | Current evidence | Status |
|---|---|---|
| 配置存在 | Codex fixture apply 输出；Claude 配置层级已由官方文档确认 | partial |
| hook 触发 | Codex 用户级 hooks 已 trust，项目级 debug hooks 已禁用；`unelevated` 下非交互式 sandbox probe 通过，但该模式不触发生命周期 hooks；Claude command hook 有 retrospective 失败证据 | partial |
| stdin 有效 | release hook CLI 隔离 payload 写入成功 | verified for direct CLI |
| state 写入 | codex_audit-session-01，session_id_source=payload | verified for direct CLI |
| detector 更新 | Rust 单测覆盖部分逻辑，真实 host 状态轮询尚未完成 | partial |
| 桌面可见 | 尚未完成本轮真实 Windows taskbar 手工验收 | missing |

### Pending verification runbook

| Pending group | Required environment | Command / evidence | Stop condition |
|---|---|---|---|
| Codex trust and lifecycle | Interactive Codex CLI session with the installed release directory | Run `/hooks`, review and trust the CC Traffic Light command hooks shown as non-managed/new or changed, perform one read-only task, then capture `taskbar_widget_hook.exe list`, the explicitly configured state file, runtime log and screenshot | Stop after two trust/trigger attempts if no state change; do not modify unrelated user hooks manually |
| Claude command-hook evidence | Claude Code 2.1.204, Windows, fixed terminal, Git Bash and user-level `.claude/settings.json` | Use the marker/dump-only wrapper; test one variable at a time across shell form, exec form and explicit PowerShell; preserve shape-only stdin/log evidence | Stop after the same failure reproduces twice; keep product status `ProcessOnly` unless a stable trigger is proved |
| Installer | Inno Setup `ISCC.exe`, clean temporary user profile and install directory | Build `installer.iss`, install/upgrade/uninstall, inspect three executables, Run/Registry entries, hooks backup and restore | Stop if ISCC is unavailable or installer output is stale; do not use the old `dist/installer` binary as evidence |
| Widget desktop | Interactive Windows Explorer/taskbar session | Inject isolated state fixtures, capture Working/Waiting/Error/Done screenshots, restart Explorer and record runtime log | Stop on taskbar attach failure after the configured retry window; preserve tray-only diagnostics |
| Frontend release asset | Working Node/pnpm toolchain with Volta directory permission | Run `pnpm build`, record `taskbar-settings-tauri/dist` timestamp and build output | Stop after one environment/permission failure; do not alter global Volta installation in this audit |

## Phase 1: Release Artifact 与安装器

- [x] AUD-1-01 修改 scripts/pack-all.ps1，显式构建 release taskbar_widget_hook.exe。
- [x] AUD-1-02 显式检查三个 release artifact：
  - target\release\taskbar-widget.exe
  - target\release\taskbar-settings-tauri.exe
  - target\release\taskbar_widget_hook.exe
- [x] AUD-1-03 验证 taskbar_widget_hook.exe --version 与发布版本一致。
- [x] AUD-1-04 静态审核 installer.iss 的 Files、Run、Registry、Tasks 和卸载行为。
- [x] AUD-1-05 静态确认 installer 将 `{app}\taskbar_widget_hook.exe` 显式传给 Codex 脚本，脚本默认优先解析自身安装目录；开发 README/debug fixture 路径保留为开发用途。
- [ ] AUD-1-06 在临时安装目录验证三个 exe、脚本和资源均到位。（用户接管 ISCC 构建；待提供新安装器和临时安装证据）
- [ ] AUD-1-07 验证安装后的自动启动项指向当前安装目录。
- [x] AUD-1-08 验证重复安装不会破坏用户 hooks 或产生重复 managed entry。
- [ ] AUD-1-09 验证升级后的配置、备份、路径和 trust 行为。
- [ ] AUD-1-10 验证卸载后的自启动清理、程序文件和 hook 条目处理。
- [ ] AUD-1-GATE release artifact、安装、升级和卸载证据齐全。

## Phase 2: Codex 全局 Hook 闭环

- [x] AUD-2-01 建立无既有配置的 hooks fixture。
- [x] AUD-2-02 验证 dry-run 不修改 fixture。
- [x] AUD-2-03 验证 apply 写入正确事件和稳定 commandWindows 路径。
- [x] AUD-2-04 验证已有用户 hooks 被保留。
- [x] AUD-2-05 验证重复 apply 幂等。
- [x] AUD-2-06 验证 restore 能恢复原文件或删除原本不存在的文件。
- [x] AUD-2-07 验证非法 JSON、空文件、目录不存在和只读文件时均有明确结果；只读 `hooks.json` 会被拒绝且保持原文件与属性不变。
- [x] AUD-2-08 在交互式 Codex 会话中触发 lifecycle events。2026-07-13 真实会话依次完成 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop`，全部 exit 0；state 使用真实 payload session `019f5a5a-cee5-7462-ba3a-78ce6171141f`，最终 `Stop -> done`。根因是 Windows runner 可执行 `cmd.exe` 但不能直接启动本地 hook EXE；正式部署改为 `commandWindows -> cmd.exe /d /s /c call <user-data>\codex-taskbar-widget-hook.cmd`，wrapper 再转发至安装目录 EXE。项目级 debug hooks 保持禁用。诊断代码和 marker 清理、重新构建并替换 release hook 后，用户再次重启 Codex 执行只读任务：无需重新 trust，且不再出现任何 `hook exited with code 1`。
- [x] AUD-2-09 检查 state.json 出现 codex_<session_id>，并记录 session_id_source。
- [x] AUD-2-10 验证 Working、Waiting、Completed 和 Error 状态。
- [x] AUD-2-11 纯函数已验证配置存在但无近期事件返回 ConfiguredUnverified；真实新鲜 Codex lifecycle 事件后，Settings 的“立即刷新检测”在 30 秒内显示 `Codex：✅ 已就绪`。活动窗口为 5 分钟；超过窗口回落为 ConfiguredUnverified 属预期行为。
- [x] AUD-2-12 已验证 hook CLI 缺失、state 损坏和 state 不可写诊断。2026-07-13 将 `TASKBAR_WIDGET_STATE_HOME` 指向普通文件后，正式安装 hook 以 exit code 1 失败并输出 `当文件已存在时，无法创建该文件。 (os error 183)`；fixture 未被修改，证据根目录：`C:\Users\admin\AppData\Local\Temp\cc-traffic-light-audit-2-12-20260713-142630`。
- [ ] AUD-2-GATE Codex 从安装、trust、真实事件、状态文件到 UI 闭环。

## Phase 3: Claude Code 证据门

- [ ] AUD-3-01 固定 Claude 版本、Windows、Git Bash、启动终端和配置层级。
- [ ] AUD-3-02 用最小 marker 单独验证 hook 是否触发。
- [ ] AUD-3-03 用 dump-only wrapper 保存 stdin JSON 或 shape-only 数据。
- [ ] AUD-3-04 单独验证 shell form 的路径、stdin 和退出码。
- [ ] AUD-3-05 单独验证 exec form，command 仅指向真实 exe，args 仅放参数。
- [ ] AUD-3-06 单独验证 shell=powershell，并记录与 Git Bash 的差异。
- [ ] AUD-3-07 验证 SessionStart、UserPromptSubmit、PreToolUse、PostToolUse、Notification、Stop 和失败事件。
- [ ] AUD-3-08 验证 session_id、hook_event_name、event order 和编码。
- [ ] AUD-3-09 同一原因连续两次复现后停止路径变体试错。
- [ ] AUD-3-10 command hook 仍不稳定时，将产品状态定为 DegradedProcessOnly。
- [ ] AUD-3-11 若必须高置信度，建立 HTTP hook prototype 并验证 POST、失败响应、端口冲突和 endpoint 未启动。
- [ ] AUD-3-12 明确 Claude 是否需要 trust/review，以及用户级 settings.json 是否适合作为部署目标。
- [ ] AUD-3-13 更新 Claude 示例、handoff 和支持矩阵。
- [ ] AUD-3-GATE Claude 得到明确支持等级：Active、实验性或 DegradedProcessOnly。

## Phase 4: Hook CLI、状态模型与 Detector

- [x] AUD-4-01 测试 Claude 有效和缺失 session_id；Codex 对应 fixture 已在 AUD-2-09 覆盖。
- [x] AUD-4-02 测试同一 session 的乱序事件。
- [x] AUD-4-03 测试多个 session 和两个 Agent 并发写入。
- [x] AUD-4-04 测试 UTF-8、UTF-8 BOM、UTF-16 LE BOM、空输入和非法 JSON 解析路径。
- [x] AUD-4-05 测试 state 损坏恢复、corrupt backup 和成功写入后的临时文件清理。
- [x] AUD-4-06 验证 Working / Waiting stale 和 Done / Error TTL。
- [x] AUD-4-07 验证 Error > NeedsAttention > Working > Completed > Idle 的优先级。
- [x] AUD-4-08 验证 state-file observation 优先于 process fallback。
- [x] AUD-4-09 修正 Codex / Claude 配置路径解析，并通过 host 编译验证。
- [x] AUD-4-10 已区分最近收到 hook 与当前 active_task_count=0；DTO 现明确区分 ConfiguredUnverified、Active、ProcessOnly 和 Error。
- [x] AUD-4-GATE 状态、并发、异常输入、过期和 detector 优先级有自动化或 fixture 证据。

## Phase 5: Widget 挂载与闪烁

- [x] AUD-5-01 无 state.json 启动 widget，确认 Idle 和任务栏可见（2026-07-13：已安装 release host 的 Settings 诊断显示 `attached`，Done 保留期结束后 state 和 widget 均回落 `idle`）。
- [x] AUD-5-02 注入 Working，确认左侧绿灯慢闪（2026-07-13：用户桌面确认）。
- [x] AUD-5-03 注入 Waiting / NeedsAttention，确认中间黄灯快闪（2026-07-13：用户桌面确认）。
- [x] AUD-5-04 注入 Error，确认右侧红灯慢闪（2026-07-13：用户桌面确认）。
- [x] AUD-5-05 注入 Done，确认完成态常亮并回落 Idle（2026-07-13：用户确认绿灯常亮；60 秒保留后 `taskbar_widget_hook list` 的 tasks 为空且用户确认 widget idle）。
- [x] AUD-5-06 同时运行 Codex 和 Claude，确认 source group、overall priority 和标签（2026-07-13：state 同时保留 Codex=working、Claude=waiting；用户确认两个 source group 分别为绿慢闪、黄快闪）。
- [x] AUD-5-07 reduced motion 已实现：Appearance 有持久化控件，Working/Waiting/Error 在 reduced-motion 下保持各自绿/黄/红常亮；用户接受不保存该模式的桌面截图/录屏，已验证 UI persistence、config reload 和 focused renderer test。
- [x] AUD-5-08 测试 attach 失败、tray-only、重试和恢复：`TASKBAR_MVP_PARENT=none` 证明 tray-only；Explorer recovery 修复后日志记录 tray pending→retry registered→attach，且 host_count=1。
- [x] AUD-5-09 重启 Explorer/taskbar 后不重启 host 即自动恢复。证据：`%TEMP%\\cc-traffic-light-explorer-recovery-20260713-163739.log`，包含 `WM_NCDESTROY`、new HWND、tray retry 和 attach；旧失败基线保留在 `...1783929848740.log`。
- [~] AUD-5-10 保存最终桌面截图或录屏、runtime log 和 state JSON（2026-07-13：已保留 state JSON 和 Explorer-restart runtime log；最终桌面截图或录屏尚未保存）。
- [ ] AUD-5-GATE 每个用户可见状态都有最终桌面证据。

## Phase 6: Settings、托盘与首次运行 UX

- [x] AUD-6-01 修正 Claude 配置路径和 claude / claude_code source key。
- [x] AUD-6-02 统一 Codex / Claude hook status 的 DTO、Rust bridge 和 React 类型。
- [x] AUD-6-03 设置页支持 Active、ConfiguredUnverified、NotInstalled、ProcessOnly 和 Error 状态。
- [x] AUD-6-04 设置页显示最近事件时间、来源、可信度、错误、组件挂载状态和配置路径。
- [x] AUD-6-05 Settings lifecycle 通过：修复初始 HWND 可见性 race 后，debug 与 root `target\\release` verifier 均通过 spawn/reuse/close/reopen/kill-recover；已安装 Settings 真实执行重新部署和刷新并显示 `hooks deployed successfully`、`检测刷新已请求`。
- [x] AUD-6-06 Claude 降级时明确“只能检测进程，不能判断工作状态”。
- [x] AUD-6-07 启动通知按 Codex / Claude 各自状态给出 trust、未安装、进程降级或损坏诊断。
- [x] AUD-6-08 持久化启动通知 fingerprint；状态恢复为双 Active 时清除 fingerprint，下一次异常仍可提醒。
- [x] AUD-6-09 设置页增加立即刷新检测入口，并在失败时展示错误；托盘已有刷新入口。
- [x] AUD-6-10 Tauri settings 启动失败同时写入 runtime log，并通过托盘通知说明已切换 Win32 fallback。
- [x] AUD-6-GATE 已安装 Settings 显示当前状态、最近事件、可信度、组件挂载和最近错误；Claude ProcessOnly、未 trust、state 不可写、attach failure 均有可定位 UX 或日志。

## Phase 7: 清洁环境与最终验收

- [ ] AUD-7-01 在没有既有 Agent 配置的临时用户环境运行安装器。
- [ ] AUD-7-02 记录安装目录、三个 exe、脚本、注册表、自启动、配置和状态路径。
- [ ] AUD-7-03 验证只有 Codex 时的安装、trust、Working、Completed 和 Idle。
- [ ] AUD-7-04 验证只有 Claude 时的实际支持等级和 UI 文案。
- [ ] AUD-7-05 验证 Codex 与 Claude 同时运行不会互相覆盖状态。
- [ ] AUD-7-06 验证已有 hooks、非法配置、PowerShell 拒绝、未 trust 和 state 不可写。
- [ ] AUD-7-07 验证升级不丢用户配置、不改变 hook 路径契约。
- [ ] AUD-7-08 验证卸载后的自启动、hook 条目、restore 和残留文件。
- [ ] AUD-7-09 生成最终 release gate 报告。
- [ ] AUD-7-GATE 清洁安装和异常路径均有证据，支持等级没有 UI 误报。

---

## Validation Checklist

- [ ] AUD-VAL-01 cargo fmt --all -- --check 通过。（blocked：工作区已有多文件 rustfmt 漂移，不能覆盖用户未完成改动）
- [x] AUD-VAL-02 cargo check -p taskbar-widget --offline 通过。
- [x] AUD-VAL-03 cargo test --workspace --offline 通过。
- [ ] AUD-VAL-04 pnpm build 通过。（用户接管 Node/pnpm 环境；待提供构建输出和 dist 产物时间戳）
- [x] AUD-VAL-05 cargo build -p taskbar-settings-tauri --release --offline 通过。
- [x] AUD-VAL-06 cargo build -p taskbar-widget --bin taskbar_widget_hook --release --offline 通过。
- [x] AUD-VAL-07 cargo build -p taskbar-widget --release --offline 通过。
- [x] AUD-VAL-08 `pack-all.ps1 -ValidateOnly` 在缺少任一 release artifact 时失败退出；failure injection 后已恢复 artifact。
- [x] AUD-VAL-09 hooks fixture 覆盖 dry-run / apply / restore / idempotency / preserve-user-entries。
- [x] AUD-VAL-10 payload fixture 覆盖 session、事件、编码、空输入、非法 JSON 和乱序事件。
- [x] AUD-VAL-11 state fixture 覆盖并发、损坏恢复、TTL、stale 和 overall priority。
- [x] AUD-VAL-12 detector fixture 覆盖 state-file 与 process fallback 优先级。
- [ ] AUD-VAL-13 Windows 手工验证最终桌面可见性和闪烁。
- [ ] AUD-VAL-14 Windows 手工验证 taskbar recovery 和 tray-only。
- [ ] AUD-VAL-15 失败路径均有用户文案或可定位日志。

## Documentation Checklist

- [x] AUD-DOC-01 更新 docs/plan/post-install-monitoring-readiness.md 的过时状态。
- [x] AUD-DOC-02 更新旧 checklist 或添加迁移说明。
- [x] AUD-DOC-03 更新 Claude command、HTTP、process fallback 支持矩阵。
- [x] AUD-DOC-04 更新 README 的安装、trust、状态和限制。
- [x] AUD-DOC-05 为每个完成、跳过或阻塞 task 生成 reflection。
- [ ] AUD-DOC-06 保存命令原始输出、截图、日志和 state 样本路径。

## Cleanup Checklist

- [ ] AUD-CLN-01 删除临时用户配置、marker、dump、state 和实验 endpoint。
- [ ] AUD-CLN-02 确认没有提交 debug target、真实用户路径、临时日志或 secrets。
- [x] AUD-CLN-03 静态确认实验 Claude 配置没有被安装器默认引用；installer 只部署 host、settings、hook CLI 和 Codex 部署脚本。
- [ ] AUD-CLN-04 移除未使用诊断代码、重复配置和失效链接。
- [x] AUD-CLN-05 canonical source key 统一为 `claude`；legacy `claude_code` 仅在输入兼容层保留，旧未验证状态文案已修正。

## Completion Criteria

- [ ] 所有 Phase gate 均通过，或明确记录 Blocked / Accepted limitation。
- [ ] Codex 清洁安装到真实事件和任务栏展示闭环。
- [ ] Claude 支持等级由当前环境证据决定，UI 和文档没有过度承诺。
- [ ] 三个 release artifact 被显式构建和验收。
- [ ] 自动化测试、Windows 手工验证和安装器验证均有原始证据。
- [ ] 用户已有 hooks、升级、卸载和失败路径不会被静默破坏。
- [ ] 每个完成 task 都有对应 reflection。

可接受限制：

- Claude 的产品默认仍为 ProcessOnly；当前 Windows/Claude Code 2.1.204 仅验证 project-level `shell: powershell` 的 opt-in Active 路径，其他 shell/form 不是发布承诺。
- trust 仍需用户在 Agent 中确认，不能由安装器绕过。
- 多显示器和云端会话不属于本轮验收。

## Reflection / Task Summary Generation

每完成、跳过或阻塞一个 task，都生成：

docs/reflections/task-<task-id>-<timestamp>.md

模板：

~~~markdown
- Task: <task id and name>
- Encountered Problem: <what happened, or none>
- Thought Process: <how evidence was evaluated>
- Options Considered: <alternatives>
- Chosen Solution: <decision>
- Rationale: <why>
- Evidence: <commands, logs, screenshots, or files>
- Status: completed | skipped | blocked
~~~

Reflection 必须区分代码问题、环境问题、权限问题、产品决策问题和验证方法问题。

## Recommended Execution Order

1. AUD-PRE-01 至 AUD-PRE-06。
2. Phase 0 并通过 AUD-0-GATE。
3. Phase 1，并先完成 AUD-1-GATE。
4. Phase 2，形成 Codex 可发布基线。
5. Phase 3，决定 Claude 支持等级；若需要 HTTP，再另开 prototype loop。
6. Phase 4 至 Phase 6，修复确定性 bug 并闭环 UX。
7. Phase 7 和 AUD-VAL 全量验收。
8. 完成 AUD-DOC 和 AUD-CLN 后，生成最终 release gate 报告。

同一原因连续两次失败后，不继续重复修改；记录 reflection，更新假设并重新规划或转人工。
