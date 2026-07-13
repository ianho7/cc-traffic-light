# Codex / Claude Code 监控卸载与重新安装计划

## Objective

为 Settings 的「数据源 → 监控配置」增加按来源独立管理 Codex 与 Claude Code 监控的能力：卸载本软件注入的 hook、重新安装当前版本 hook、清理对应历史状态，并在产品卸载时提供默认不勾选的 hook 清理选项。

范围包括用户级 `%USERPROFILE%\.codex\hooks.json` 与 `%USERPROFILE%\.claude\settings.json`、host IPC、Tauri Settings、共享状态和安装器。范围外包括卸载 Codex/Claude Code 本体、修改项目级配置、企业 managed policy，以及变更 hook 事件集合与状态语义。

## Background and Context

运行时事实链路为 `Codex / Claude Code hook → taskbar_widget_hook.exe → %APPDATA%\CcTrafficLight\state.json → host snapshot → widget / Tauri`。Codex 与 Claude 的用户级 hook 已由 PowerShell 安装脚本管理，Settings 通过命名管道请求 host 执行脚本。

已确认的产品决定：

- 卸载只删除带 `CcTrafficLight Codex` / `CcTrafficLight Claude` 管理标记的 hook；保留其他配置和 hook，且不从备份整体恢复文件。
- 重新安装先执行上述安全卸载，再写入当前安装目录的 hook。
- 卸载后只清除同一来源的 `state.json` 任务；另一来源不能受影响。
- 不再使用进程扫描作为降级数据源。未部署 hook 时，Codex 与 Claude Code 均显示“未安装”。
- Settings 按来源分别提供「卸载监控 / 重新安装监控」；卸载须确认，重新安装无需确认。
- 安装器卸载流程要提供默认不勾选的“移除 CC Traffic Light hooks”选项；静默卸载不移除。

## Current State Analysis

相关文件：

- `taskbar-widget/scripts/install-codex-hooks.ps1`：现有 Codex merge、备份和恢复脚本；工作树已开始加入 `-Uninstall`。
- `taskbar-widget/scripts/install-claude-hooks.ps1`：Claude 用户级安装脚本；需与 Codex 同等完成卸载测试。
- `taskbar-widget/src/settings_bridge.rs`：hook 状态判定与脚本进程启动；工作树已开始加入按来源卸载和状态清理。
- `taskbar-widget/src/agent_state.rs`：状态原子读写；工作树已开始加入 `clear_agent_tasks`。
- `taskbar-widget/src/detector.rs`：此前含进程 fallback；工作树已移除该路径，尚未完整回归。
- `crates/shared-core/src/tauri_ipc.rs`、`taskbar-widget/src/tauri_settings_ipc.rs`、`taskbar-settings-tauri/src-tauri/src/lib.rs`：需要完整暴露卸载命令。
- `taskbar-settings-tauri/src/pages/MonitoringPage.tsx`：已有安装入口；工作树已开始改为每来源操作行与确认弹窗。
- `installer.iss`：已在安装时部署两个来源 hook，但尚无卸载复选项或 `[UninstallRun]` 处理。

当前风险是部分实现尚未通过脚本 fixture、Rust workspace、前端与 release 构建验证；不得把现有工作树状态当作完成。

## Proposed Solution

将 hook 生命周期管理收敛为“脚本是唯一配置写入者，host 是唯一 UI 操作入口，状态文件由 host 原子清理”的模式。

1. 两个脚本都支持 `-Apply -Uninstall`：只按稳定管理标记移除条目，原子写回；配置不存在或没有管理条目时成功返回 no-op；JSON 损坏、只读或写入失败时不改文件并报告路径。
2. host 分别调用脚本。脚本成功后，调用 `agent_state::clear_agent_tasks(SourceId)`；清理只删除该来源的 task，再复用既有 summary 刷新和原子写入。
3. 重新安装在 host 内以“卸载成功（或 no-op）→ 安装”顺序执行；第二步失败时向 UI 返回明确的阶段化错误，不尝试还原用户文件。
4. detector 仅读取 state file；无来源状态时使用 Idle，不创建 `process` observation。hook 状态分类不再返回 `ProcessOnly`。
5. Settings 为 Codex 和 Claude 分别显示状态、卸载和重新安装按钮；卸载使用浏览器确认框；执行期间禁用四个来源操作；动作完成后重新取 snapshot 和 hook status。诊断区展示配置、备份与 hook EXE 路径，错误保留可复制文本。
6. 安装器在交互式卸载时给出未勾选复选项。选择后运行两份脚本的 `-Uninstall -Apply`；静默卸载不运行。若 Inno Setup 无法在卸载前安全承载复选项，则以明确确认对话为后备，并记录该产品差异。

## Alternatives Considered

### 恢复整个安装前备份

优点是简单；缺点是会覆盖用户安装后新增的设置和 hook。因违反“只删除本软件条目”，不采用。

### 在 Rust 中直接编辑 JSON

优点是少一个 PowerShell 子进程；缺点是要重写两种配置 schema 的 merge、识别、备份及原子写逻辑。现有脚本已经是配置写入边界，因此不采用。

### 保留 `claude.exe` / `codex.exe` 进程检测

优点是未安装时仍有弱信号；缺点是会把“进程存在”误表示为来源状态，且与已确认的 hook-only 产品语义冲突。不采用。

### 一个总的“卸载全部监控”按钮

优点是 UI 简短；缺点是一次修改两个独立用户配置，增加误操作范围。不采用。

## Implementation Plan

### Phase 1: 完成脚本级安全卸载

- Goal: 两个 PowerShell 脚本仅删除 CC Traffic Light 管理项并保留其他配置。
- Files: `install-codex-hooks.ps1`、`install-claude-hooks.ps1`。
- Tasks:
  - 固化 `-Uninstall` 参数、no-op 输出、稳定管理项匹配和原子写入。
  - 验证无配置、无管理项、混合其他 hook、重复管理项、损坏 JSON、只读文件和脚本路径缺失。
  - 保留现有备份文件，但卸载操作不使用它覆盖配置。
- Expected Result: `-Uninstall -Apply` 只删除自己的条目，并可由输出解释结果。

### Phase 2: Host、IPC 与状态清理

- Goal: UI 可按来源请求卸载或重新安装，状态清理具有来源隔离。
- Files: `agent_state.rs`、`settings_bridge.rs`、`tauri_ipc.rs`、`tauri_settings_ipc.rs`、Tauri backend。
- Tasks:
  - 补齐独立 install/uninstall DTO、pipe 分发和 Tauri command。
  - 为 `clear_agent_tasks` 写单元测试：清除 Claude 不删除 Codex，反之亦然，summary 正确重算。
  - 将“重新安装”实现为显式 remove-then-install 编排，并为每个失败阶段返回可定位错误。
  - 仅在脚本成功后清理状态；状态清理失败必须在 UI 显示，但不能谎称完整成功。
- Expected Result: 任何一来源操作不会影响另一来源配置或状态。

### Phase 3: Hook-only 检测语义

- Goal: 消除进程 fallback，使未安装来源统一显示未安装。
- Files: `detector.rs`、`settings_bridge.rs`、`MonitoringPage.tsx`、相关测试及文本。
- Tasks:
  - 删除 ToolHelp 进程扫描及 `DetectionMethod::Process` 的运行时产生路径。
  - 调整 hook status 分类和 UI 文案，不再将 `ProcessOnly` 显示为有效降级模式。
  - 更新/替换依赖 process fallback 的测试。
- Expected Result: 没有管理 hook 时，不论 CLI 进程是否存在，widget 来源为 Idle、设置页为未安装。

### Phase 4: Settings 交互和诊断

- Goal: 提供明确、安全、按来源操作的 Settings 界面。
- Files: `MonitoringPage.tsx`、`lib/tauri.ts`、必要的 CSS/类型文件。
- Tasks:
  - 按 Codex / Claude Code 分行显示状态、卸载和重新安装。
  - 卸载确认文案必须说明保留其他 hook/config 且清理此来源历史状态。
  - 统一忙碌状态、成功/失败消息和刷新后状态。
  - 加入只读诊断路径：配置、备份、hook EXE；支持复制错误和路径。
- Expected Result: 用户无需手工编辑 JSON 即可完成安全生命周期操作。

### Phase 5: 安装器卸载选项

- Goal: 交互式卸载可选清理 hook，默认不清理；静默卸载不清理。
- Files: `installer.iss`。
- Tasks:
  - 使用 Inno Setup uninstall UI 或等价安全确认，实现未勾选的选项。
  - 条件执行两份 `-Uninstall -Apply` 脚本，记录失败但不阻止主程序卸载。
  - 验证升级安装不会删除 hook；正常卸载默认保留；选中后仅移除管理项。
- Expected Result: 安装器行为与 Settings 行为一致且不意外篡改用户配置。

### Phase 6: 回归、release 与真实桌面验收

- Goal: 证明配置、IPC、状态和 UI 全链路有效。
- Files: 测试文件、`docs/checklist/`、`docs/reflections/` 或 `docs/handoff/`。
- Tasks:
  - 运行脚本 fixtures、Rust/前端测试和规定的独立 release 构建顺序。
  - 用已安装的 root `target\\release\\taskbar-widget.exe` 验证卸载、重新安装、Claude 与 Codex 的真实事件。
  - 记录安装器选项和失败路径证据。
- Expected Result: 每个操作都有命令、日志或 UI 证据，且不把用户豁免项标记为通过。

## Validation Strategy

- PowerShell fixture：为每种脚本构造带其他 hook 的临时 JSON；apply → uninstall → 断言其他字段/其他 hook 保留、管理条目移除；对 Claude 与 Codex 各运行一次。
- Rust tests：`cargo test -p taskbar-widget --offline` 覆盖 status 分类、来源状态清理与 detector 无进程兜底；`cargo test --workspace --offline` 作为回归。
- Frontend：`pnpm build`，并确认 Settings 可调用四个 IPC action。
- Build：先 `cargo build -p taskbar-settings-tauri --release --offline`，再 `cargo build -p taskbar-widget --release --offline`。
- Manual：在已安装 Settings 点击两个来源的卸载/重新安装；验证 JSON diff、`taskbar_widget_hook.exe list`、state.json、widget 与 UI 状态；一次 Claude 和一次 Codex 真实 hook 事件。
- Installer：新建安装器后验证默认卸载保留 hook、可选清理删除管理条目、静默卸载不清理。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| 用户配置损坏或只读 | 无法安全写入 | 脚本在解析/写入前失败，不执行部分变更；返回路径和错误。 |
| 管理项识别过宽 | 误删其他 hook | 只匹配稳定 status-message 前缀和预期 handler 形状；fixture 覆盖混合配置。 |
| state 清理失败 | hook 已移除但 UI 暂存旧状态 | 返回部分失败，下一次刷新保留诊断；不报告完整成功。 |
| Inno Setup 卸载 UI 时机受限 | 复选项不易在卸载前呈现 | 先制作最小原型并用 ISCC/交互式卸载验证；必要时采用明确确认对话。 |
| release 与当前源码不同 | UI/host IPC 不匹配 | 以单独 release 构建并确认 root target 路径后再进行桌面验证。 |

## Open Questions

无阻塞产品决策。实现阶段唯一待验证的是 Inno Setup 卸载 UI 能否在实际卸载前稳定承载默认未勾选的复选项；若不能，采用计划中定义的确认对话后备方案。

## Recommended Next Step

先完成并测试 Phase 1：对两份脚本各建立一个“保留用户 hook、删除管理 hook”的隔离 fixture。该步骤为 IPC、UI 和安装器行为提供可重复的安全基线。
