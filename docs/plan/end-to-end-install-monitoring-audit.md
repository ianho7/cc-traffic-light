# 安装后监控链路全面审查与补齐计划

## Objective

全面审查 CC Traffic Light 从 release 构建、安装、hook 配置、Agent 事件、共享状态，到任务栏 widget 展示的完整链路，并补齐会阻断“安装后可用”的缺口。

预期结果：

~~~text
用户安装
  → 文件和启动项正确
  → Codex / Claude 事件进入状态链路
  → detector 得到正确状态
  → widget 稳定挂载
  → Working / NeedsAttention / Completed / Error 灯效正确
  → 设置页和托盘能解释失败并提供恢复路径
~~~

范围包括安装器、Codex hooks、Claude 当前支持等级、hook CLI、state.json、detector、Win32 taskbar、闪烁动画、Tauri 设置页、首次运行引导和清洁环境验收。

范围不包括绕过 Agent trust、企业 managed hooks、多显示器和云端会话。

---

## Background and Context

当前架构：

~~~text
Codex / Claude Code
  → command hook 或进程检测
  → taskbar_widget_hook.exe
  → %APPDATA%\CcTrafficLight\state.json
  → taskbar-widget 每秒轮询
  → detector → widget_effects / widget_render
  → Win32 taskbar surface
~~~

关键事实：

1. Claude Code 参考文档规定：有 args 是 exec form，直接启动可执行文件；无 args 是 shell form，Windows 默认走 Git Bash，Git Bash 不存在时才可能使用 PowerShell。
2. 更新的 claude-code-hooks-retrospective.md 是当前 Claude command hook 的决策依据：当前 Windows 环境中 shell form 存在路径和 stdin 问题，exec form 也曾静默不执行，因此不能把 Claude command hooks 当成生产保证。
3. 较早的 claude-code-hooks-investigation.md 记录过一次 exec form 成功写入，但属于历史实验，不能推翻更新复盘中的稳定性结论。
4. Codex 有 commandWindows 字段，Claude Code 没有等价字段；两者不能只替换 agent 名称后直接复用配置。
5. Host 验收必须先构建 Tauri settings，再单独构建 taskbar-widget；不能使用 cargo build --workspace 代替。

---

## Current State Analysis

### 已有能力

- taskbar_widget_hook.rs 支持 codex|claude HookName、sample、list、set、clear。
- agent_state.rs 已有 session task key、命名 mutex、原子写入、摘要、TTL 和 stale 清理。
- detector.rs 已实现 state-file 优先和 process fallback。
- hook_rules.rs 已实现 Working、Waiting、Done、Error 映射。
- widget_effects.rs 已实现 Working / NeedsAttention / Error 闪烁和 Completed 保持。
- main.rs 已有每秒轮询、80ms 动画 timer、taskbar attach 重试和 Win32 fallback。
- install-codex-hooks.ps1 已有 dry-run、apply、backup、restore、merge。
- 当前 installer.iss 已包含三个 exe，包含 taskbar_widget_hook.exe；旧计划中“安装器缺少 hook exe”的描述已经过时。

### 真实缺口

#### Release 与安装

- scripts/pack-all.ps1 没有显式构建并验收 taskbar_widget_hook.exe。
- 安装器只部署 Codex hooks，没有 Claude 的正式部署或明确降级说明。
- PowerShell 部署使用隐藏、异步运行，失败反馈不明确。
- 清洁安装、重复安装、升级、卸载和 restore 缺少完整证据。
- 脚本、示例和文档仍需统一稳定安装路径，排除 debug target 路径。

#### Codex hooks

- 需要验证 release 路径、用户已有 hooks 的结构化 merge、幂等性、backup/restore 和 trust。
- 需要验证 hook CLI 不存在、state 文件不可写、JSON 损坏、PowerShell 策略拒绝时的诊断。

#### Claude Code

- 当前不能承诺 command hooks 稳定可用；examples.claude-hooks.json 应标注实验性质。
- 若继续追求高置信度，应单独验证 HTTP hook；在此之前保持进程检测降级。
- 不能把“配置文件存在”显示成“Claude hook 已就绪”。

#### 状态检测与设置页

- settings_bridge.rs 当前对 Codex 和 Claude 都读取 .codex/hooks.json，Claude 路径不正确。
- 仅用 active_task_count > 0 判断 Active，无法表示“已收到过 hook、当前只是空闲”。
- MonitoringPage.tsx 的 Claude snapshot key 与 Rust 侧 claude 存在 claude_code / claude 不一致风险。
- 设置页部署按钮目前只调用 Codex 部署函数。
- 启动通知只检查 Codex，且提示去重没有持久化。

#### Widget 与文档

- 单元测试和 debug CLI 不能替代真实 Windows 最终桌面画面验收。
- 需要验证无 state、损坏 state、延迟事件、任务完成回落、任务栏重启、挂载失败重试和双 Agent 同时运行。
- 旧计划、investigation、retrospective 的时间线和结论需要明确标注。

---

## Proposed Solution

采用证据门控流程：

1. 先统一当前事实和发布契约。
2. 先闭环 Codex，因为它是当前已验证的高置信度路径。
3. Claude command hooks 保持实验性；默认使用进程检测并显示 Degraded。
4. 若必须获得 Claude 的高置信度状态，优先研究 HTTP hook，而不是继续重复 shell/exec 路径试错。
5. 把 hook 状态分为 NotConfigured、ConfiguredUnverified、Active、DegradedProcessOnly、Error。
6. 所有 release artifact 必须显式构建、列出并验证。

---

## Alternatives Considered

### 立即为 Claude 自动写入 command hooks

- 优点：改动小，形式类似 Codex。
- 缺点：与当前 retrospective 的 Windows 实测冲突，可能静默失败。
- 结论：不作为当前生产方案，只作为受控实验。

### Claude 永久只做进程检测

- 优点：稳定、安装简单。
- 缺点：无法区分 Working、NeedsAttention 和 Completed。
- 结论：当前安全 fallback，是否永久采用取决于 HTTP hook 验证。

### Claude 改用 HTTP hook

- 优点：绕开 Git Bash、路径转义和 stdin 管道问题。
- 缺点：需要本地 endpoint、端口、安全、启动顺序和配置管理。
- 结论：作为高置信度接入的优先研究方向，但必须真实验收后再产品化。

---

## Implementation Plan

### Phase 0: 建立审查基线

- Goal: 统一事实、假设和待验证项。
- Files: docs/plan、docs/checklist、两个 Claude handoff。
- Tasks:
  - 标记 investigation 为历史实验记录。
  - 以 retrospective 作为当前 Claude command hook 决策依据。
  - 更新旧计划中“installer 缺少 hook exe”等过时条目。
  - 建立证据表：配置存在、hook 触发、stdin 有效、state 写入、widget 可见。
- Expected Result: 后续实现不再把历史一次成功当作稳定性承诺。

### Phase 1: Release artifact 与安装器

- Goal: 证明安装包完整、路径稳定、可重复安装和回滚。
- Files: scripts/pack-all.ps1、installer.iss、install-codex-hooks.ps1、taskbar_widget_hook.rs。
- Tasks:
  - 显式构建并检查三个 release exe。
  - 验证 --version 自检。
  - 固定 {app}\taskbar_widget_hook.exe 路径契约。
  - 测试无配置、有配置、重复安装、升级、卸载和 restore。
  - 记录 PowerShell 部署失败，避免静默成功。
- Expected Result: 安装包 artifact、配置路径和启动行为可验证。

### Phase 2: Codex 全局 hook 闭环

- Goal: 把 Codex 从脚本可用验收到安装后可重复工作。
- Files: install-codex-hooks.ps1、settings_bridge.rs、MonitoringPage.tsx、tauri_ipc.rs。
- Tasks:
  - 解析 JSON 验证 managed entries，不用字符串 contains 作为唯一判断。
  - 验证 apply / backup / restore / idempotency / preserve-user-entries。
  - 验证稳定路径、升级后的 trust 和 hook CLI 版本。
  - 设计 NotConfigured / ConfiguredUnverified / Active / Error 状态。
  - 设置页显示最近事件、时间、路径和错误，并保留重新部署入口。
- Expected Result: Codex 安装、trust、状态写入、UI 展示和恢复形成闭环。

### Phase 3: Claude 证据门与方案决策

- Goal: 决定 Claude 保持进程 fallback，还是进入 HTTP hook prototype。
- Files: 两份本地 Claude 参考文档、claude-lifecycle-hook-dump.ps1、examples.claude-hooks.json、相关 handoff。
- Tasks:
  - 用最小 marker、dump、direct state write 分别验证触发、stdin 和写入。
  - 固定 Claude Code 版本、Windows、Git Bash、配置层级和 trust 状态。
  - 一次只改变一个变量，避免再次混合路径、shell form 和 payload 解析。
  - 如果 command hook 仍不稳定，正式记录 DegradedProcessOnly，停止重复试错。
  - 如需高置信度，设计 HTTP hook prototype，验证 POST body、session_id、事件顺序、widget 未启动和端口冲突。
- Expected Result: Claude 得到明确支持等级和下一步。

### Phase 4: 共享状态与 detector

- Goal: 确保有效事件不会被状态模型、并发或 stale 逻辑错误展示。
- Files: taskbar_widget_hook.rs、agent_state.rs、hook_rules.rs、detector.rs、shared UI state。
- Tasks:
  - 测试 session_id、缺失 session_id、事件顺序、同 session 并发和双 Agent。
  - 测试 UTF-8、BOM、UTF-16、空 stdin、非法 JSON、损坏 state 恢复。
  - 验证 Working / Waiting / Done / Error TTL、stale 和 overall priority。
  - 验证 process fallback 不覆盖高优先级 state-file observation。
- Expected Result: 状态文件与 detector 有 fixture、单元和诊断证据。

### Phase 5: Widget 挂载与闪烁

- Goal: 证明真实 Windows 最终桌面能显示状态。
- Files: main.rs、taskbar.rs、widget_effects.rs、widget_render.rs、任务栏 checklist。
- Tasks:
  - 验证 Idle、两个 source group 和配置开关的可见性。
  - 注入 Working / Waiting / Done / Error，验证绿慢闪、黄快闪、红慢闪、完成常亮和回落。
  - 测试任务栏重启、attach 失败、重试恢复和窗口隐藏。
  - 区分日志、PrintWindow 和最终桌面可见性证据。
- Expected Result: 每个用户可见状态都有真实桌面证据。

### Phase 6: Settings、托盘和首次运行 UX

- Goal: 用户能理解未安装、未 trust、hook 不可用和进程降级，并能恢复。
- Files: settings_bridge.rs、tray_icon.rs、main.rs、MonitoringPage.tsx、types、IPC、i18n。
- Tasks:
  - 修正 Claude 配置路径和 claude / claude_code key。
  - 分别显示 Codex 与 Claude 的配置、验证、Active、降级和错误。
  - 启动通知覆盖两类 Agent，但进行持久化去重。
  - 增加重新检测和重试。
  - Claude 若为进程检测，文案明确“不能判断工作状态”。
- Expected Result: UI 不把“配置存在”误报为“监控已工作”。

### Phase 7: 清洁环境、升级和发布验收

- Goal: 用最终用户路径验证产品。
- Files: installer.iss、pack-all.ps1、生命周期脚本、新增安装验收 checklist。
- Tasks:
  - 清洁用户环境安装并记录文件、注册表、配置和状态路径。
  - 验证无 Agent、只有 Codex、只有 Claude、两者同时运行。
  - 验证已有 hooks、损坏配置、PowerShell 拒绝、未 trust、state 不可写。
  - 验证升级不丢用户配置、不改变 hook 路径契约。
  - 验证卸载后的 hook 残留、restore 和自启动清理。
- Expected Result: 形成可复跑的 release gate，明确 Codex 与 Claude 的支持等级。

---

## Validation Strategy

### Static and unit validation

~~~powershell
cargo fmt --all -- --check
cargo check -p taskbar-widget --offline
cargo test --workspace --offline
pnpm build
~~~

应补充 hooks merge / restore、配置路径、source key、payload 编码、state 并发、stale、TTL、detector 优先级和 widget effects 测试。

### Release artifact validation

~~~powershell
cargo build -p taskbar-settings-tauri --release --offline
cargo build -p taskbar-widget --bin taskbar_widget_hook --release --offline
cargo build -p taskbar-widget --release --offline
~~~

必须检查：

- target\release\taskbar-widget.exe
- target\release\taskbar-settings-tauri.exe
- target\release\taskbar_widget_hook.exe

### Hook and desktop validation

- Codex：dry-run → apply → 真实 prompt/tool → 检查 codex_<session_id> → restore。
- Claude command：仅实验，分别记录触发、stdin、状态写入；任何不稳定都保持 Degraded。
- Claude HTTP：若进入实现，测试 endpoint 未启动、端口冲突、POST 失败、重复事件、真实 session_id 和多 session。
- Windows：观察最终桌面画面，而不是只看日志、PrintWindow 或诊断截图。

### Failure cases

- hook CLI 缺失或版本不匹配。
- PowerShell 被策略阻止。
- Git Bash 存在导致 shell form 路径失败。
- Claude command hook 静默失败。
- 用户未执行 trust。
- 用户 hooks JSON 非法或有重复 managed entries。
- state.json 不可写、损坏或被占用。
- taskbar attach 失败或窗口被系统隐藏。

---

## Risks and Mitigations

| Risk | Impact | Mitigation | Fallback |
|---|---|---|---|
| Claude command hooks 静默失败 | Claude 状态错误或长期 idle | 不作为默认承诺，保留 Degraded，使用证据门 | 进程检测或 HTTP hook |
| 用户未 trust hooks | 无状态事件 | 安装器、托盘、设置页多入口引导 | process fallback |
| 安装脚本静默失败 | 安装完成但未部署 | 记录结果、安装后校验、设置页重试 | 手动修复 |
| 用户 hooks 被破坏 | 用户工作流中断 | 原子备份、结构化 merge、restore fixture | 回滚备份 |
| hook 路径随升级变化 | trust 失效 | 固定 {app} 路径并重新校验 | 重新部署 |
| state 并发或损坏 | 状态丢失 | mutex、原子写入、损坏备份、fixture | 回落 idle 并显示诊断 |
| 诊断成功但桌面不可见 | 用户看不到 widget | 最终桌面画面作为 release gate | tray-only + recovery |
| 历史文档矛盾 | 重复错误方向 | Phase 0 统一时间线和当前决策 | 以最新实测 handoff 为准 |

---

## Open Questions

1. 发布前是否必须让 Claude 达到高置信度 hook 监控，还是接受进程检测降级？
2. Claude HTTP hook 是否需要同样的 trust/review 流程？需要当前版本实测。
3. 安装器是否允许安装后弹出交互式 trust 指引？
4. hook 状态是否扩展为独立的 HookDiagnosticsDto，还是先扩充现有 HookStatusDto？
5. 是否需要支持旧版本配置和 state 文件迁移？

---

## Recommended Next Step

先执行 Phase 0 + Phase 1 的只读审查和 release artifact 验证：

1. 更新旧计划中已经过时的状态描述。
2. 修改或验证 scripts/pack-all.ps1，显式构建并检查三个 release exe。
3. 在临时用户目录运行 Codex hooks 的 dry-run / apply / restore fixture。
4. 固化 Claude 当前不能承诺 command hook 的产品决策。

完成后再决定 Claude 进入 HTTP hook prototype，还是正式保留进程检测；不要在决策未确定前把 Claude command hook 自动写入正式安装器。
