# 剩余发布就绪闭环 Checklist（2026-07-13）

## 目标与边界

目标：使 CC Traffic Light 达到可发布的可验证状态——清洁安装后 Codex 真实 hook 能驱动任务栏 widget；Claude 的支持等级由当前环境证据决定；任务栏在 Explorer 重启后能恢复；Settings、升级、卸载和失败路径均有证据与用户可理解的反馈。

范围：Explorer recovery、reduced motion、Settings 生命周期、Claude 支持定级、安装/升级/卸载、清洁环境、最终证据与发布 gate。

非目标：不重做已经通过的 Codex lifecycle/state/widget 状态验证；不覆盖工作树既有未提交改动；不将用户负责的 ISCC 或 pnpm 环境问题伪装为代码失败。

原始审计映射保留在 `docs/checklist/end-to-end-install-monitoring-audit.md`；本文件是后续执行的唯一**调度清单**。

## 闭环规格

### 工作流层

阶段依赖关系如下。每个阶段只在自己的 gate 有独立证据时进入下一阶段；下游失败可路由回其真正依赖的上游阶段，不能靠重复下游手工操作掩盖。

```text
P0 基线与证据
   ├─> P1 Explorer recovery ─┐
   ├─> P2 reduced motion ────┼─> P4 Settings/runtime 验收 ─> P6 最终桌面证据
   └─> P3 Claude 证据定级 ──┤                                  │
                              └─> P5 安装/升级/卸载/清洁环境 ──┴─> Release gate
```

### 单任务运行时闭环

每个 `RLS-*` 项按以下顺序推进，而非一次性连续修改：

1. **Inspect**：读取上次 evidence、当前 diff、相关源码和当前 checklist 状态。
2. **Hypothesize**：写出一个可证伪假设；不因旧结论直接编辑。
3. **Act**：做最小、可回滚的单一改动或受控手工操作。
4. **Observe**：保存原始命令输出、日志、截图/录屏、state JSON 或 diff 路径。
5. **Verify**：先跑独立 focused verifier，再跑该阶段 gate；不能由改动者的主观观察自证完成。
6. **Decide**：标记 complete、retry、replan、blocked 或 needs-decision，并更新本文件、原 audit 与 reflection。

### 状态与证据源

| 内容 | 唯一来源 / 位置 | 更新时机 |
|---|---|---|
| 调度状态、当前阻塞、下一步 | 本文件的「执行状态」和任务 checkbox | 每项 Act/Verify 后 |
| 原 AUD 的可追溯状态 | `docs/checklist/end-to-end-install-monitoring-audit.md` | RLS phase gate 通过或结论变化后 |
| 本轮背景与已知根因 | `docs/handoff/2026-07-13-1608.md` | 结论改变时 |
| 每项取舍与失败历史 | `docs/reflections/task-<RLS-ID>-<timestamp>.md` | 项目完成或明确 blocked 后 |
| 原始运行证据 | `%TEMP%\cc-traffic-light-*` 或本轮 evidence 目录；在下方登记绝对路径 | 每次 Observe 后 |
| 产品 runtime state | `%APPDATA%\CcTrafficLight\state.json` | 只读核对，不以人工摘要替代 |

### 执行状态

| 字段 | 当前值 |
|---|---|
| 当前阶段 | P1/P2/P3：源码与自动验证已完成，等待受控桌面验收 |
| 当前可执行最高优先级任务 | RLS-1-05：部署新 release host 后的 Explorer recovery 验收（需用户确认重启 Explorer） |
| 已知失败基线 | Explorer restart 后 widget 收到 `WM_NCDESTROY` 并消失，host 进程仍存活；证据：`%TEMP%\cc-traffic-light-explorer-recovery-1783929848740.log` |
| 已知实现缺口 | `appearance.reduced_motion` 无 UI 控件，且未进入 `widget_effects.rs` |
| 外部依赖 | 用户负责提供 pnpm build 证据与新 ISCC 安装器 |
| 当前正常运行态 | 已安装 release host 已由用户确认恢复可见；启动 PID 不作为长期证据 |
| 本轮假设与结果 | Explorer 的孤立 `WM_NCDESTROY` 未重建 HWND；现重建 HWND、tray 与 timers 并进入 attach retry。Tauri 初始 HWND 未可见被误判 stale；现按 PID 复用并 ShowWindow。 |
| 本轮证据 | `taskbar-widget/target/validate-tauri-settings-lifecycle/report.json`；`taskbar-widget/target/validate-tauri-settings-lifecycle/runtime.log`；release build 输出（2026-07-13 16:30） |
| 当前阻塞 | Explorer 重启、已安装 release 部署、桌面录屏、Claude 交互、ISCC 安装/清洁账户均需要用户或外部环境。 |

每次启动新一轮时，在此表新增或更新：`当前任务`、`假设`、`尝试次数`、`最后 verifier`、`证据路径`、`下一个决定`。不要靠对话上下文保存这些内容。

### 下一步选择规则

1. 只从依赖已满足的未完成项中选任务；优先选能解除最多下游 gate 的最低成本 verifier。
2. 若已有失败证据，先运行回归基线，再修改实现；禁止凭记忆修改。
3. 同一 verifier 失败时，仅重试**一次**以排除瞬时环境；第二次同类失败必须更新假设或切换策略。
4. 需要用户安装器、pnpm 输出、Claude 交互或产品选择时，立即转为 `blocked`/`needs-decision`，继续执行不依赖它的分支。
5. 下游手工验收失败时，回到最靠近失败的实现 phase；不得只靠重启应用把失败标为通过。

### Verifier 阶梯

每项按成本从低到高验证；后一级不能替代前一级。

1. focused source/test/fixture 或纯只读诊断；
2. 包级 Rust test/check 或前端 type/build；
3. 单独 release package build（settings 先、host 后）；
4. 已安装 release runtime 的日志/state 检查；
5. 独立人工 Windows 桌面观察、截图/录屏；
6. 清洁用户环境安装/升级/卸载。

### 失败语义与重试预算

| 类别 | 判定 | 允许动作 | 上限 / 退出 |
|---|---|---|---|
| transient | 一次性进程、Explorer、文件锁或网络波动，且同一输入曾通过 | 原样重试一次并保留两次输出 | 第二次失败转 unknown，不继续重复 |
| strategy | 相同实现/路径连续两次无法改变 verifier | 写新假设，缩小最小复现，换设计或工具 | 不允许第三次同策略 |
| environment | ISCC/pnpm/Claude/Git Bash 缺失、权限或外部配置不在本轮控制内 | 保存版本/错误/最小复现，转 blocked | 等用户或环境变化，不循环 |
| policy/risk | 需重启 Explorer、改用户 hooks、卸载、删除证据或影响用户桌面 | 先说明影响并等待确认 | 无确认不执行 |
| unknown | 无法归类或证据相互矛盾 | 只做一次只读最小诊断，更新 handoff | 仍不明则 needs-decision |

### 退出条件

- **complete**：Release gate 的所有 required verifier 和原始证据完整，或每个例外均被明确接受并更新 UI/文档。
- **blocked**：同一外部环境/权限/用户输入阻塞连续两轮，且没有独立分支可推进。
- **needs-decision**：reduced-motion 视觉语义、Claude 支持承诺、Settings 常驻策略等产品选择不唯一。
- **risk exit**：下一步会清除用户配置、卸载产品或中断桌面，但尚未获确认。
- **pause/handoff**：每个暂停点更新「执行状态」、原始 evidence 路径和 reflection，保证不需要重放聊天记录。

## P0：基线、权限与证据 Bundle

**进入条件：** 无。  
**Actor：** 只读检查、创建新的 evidence 目录。  
**Verifier：** 路径存在且可以由下一位执行者重放。  
**阶段 gate：** 当前 worktree、版本、安装路径、已有失败基线均有记录。

- [x] RLS-PRE-01 读取 `docs/handoff/2026-07-13-1608.md`、原 audit checklist 与 `git status`；记录既有 dirty files，确认后续不覆盖它们。
- [x] RLS-PRE-02 记录已安装 host/settings/hook 的绝对路径、文件哈希、当前 config/state 路径和 host PID。（AUD-PRE-04）
- [x] RLS-PRE-03 记录 Codex、Claude Code、Windows、PowerShell、Rust、Node/pnpm 版本；ISCC 未发现、Git Bash 非 Git for Windows，标为环境待回填。（AUD-PRE-04）
- [x] RLS-PRE-04 建立 evidence 索引：生命周期 report/runtime log、历史 Explorer log、当前 config/state 与构建输出均已登记于本文件的执行状态。（AUD-DOC-06）

## P1：Explorer Recovery（最高优先级实现闭环）

**进入条件：** P0 完成；用户确认允许短暂重启 Explorer。  
**成功行为：** Explorer 重启后不杀/重启 host，widget 自动重新出现；host 只有一个 tray icon 与一个可见 widget。  
**独立 verifier：** runtime log 必须包含 destroy 后新的 attach/recovery；人工观察和进程/PID 仅作为补充。  
**失败路由：** 若没有重建路径，回到 RLS-1-02；若重建后多 tray icon/多个窗口，回到 RLS-1-03；若需要新的产品策略，转 needs-decision。

- [x] RLS-1-01 已用带 `TASKBAR_MVP_RUNTIME_LOG_FILE` 的已安装 release host 重放 Explorer restart 基线；历史日志 `%TEMP%\\cc-traffic-light-explorer-recovery-1783929848740.log` 保存了 `WM_NCDESTROY`、widget 消失、host PID 存活。（AUD-5-09）
- [x] RLS-1-02 已基于该最小复现实现 window 重建/重新 attach：异常 `WM_NCDESTROY` 重建 widget HWND、重新绑定 settings bridge、恢复计时器/attach retry，并以 runtime state 防止重复 tray 注册。（AUD-5-09）
- [x] RLS-1-03 recovery 的可重复诊断已证明 `WM_NCDESTROY → new HWND → tray pending → retry registered → attach`；同一 PID 存活且 host_count=1，未重复注册 tray/pipe/定时器。证据：`%TEMP%\\cc-traffic-light-explorer-recovery-20260713-163739.log`。（AUD-5-08）
- [x] RLS-1-04 已复跑 `cargo test -p taskbar-widget --offline`（35 tests）和 `cargo build -p taskbar-widget --release --offline`，均通过；独立 release host 为最终验收构建。（AUD-VAL-01/04）
- [x] RLS-1-05 已部署 root `target\\release\\taskbar-widget.exe`；Explorer restart 后同一 PID 存活且仅一个 host。日志记录 `WM_NCDESTROY`、新 HWND、首次 tray pending、下一 retry tray registered 与重新 attach：`%TEMP%\\cc-traffic-light-explorer-recovery-20260713-163739.log`。（AUD-5-08/09、AUD-VAL-14）
- [x] RLS-1-06 已更新本 checklist、handoff/reflection；recovery runtime gate 通过，P5 为用户接受例外。（AUD-5-08/09）

## P2：Reduced Motion（独立可观察的产品闭环）

**进入条件：** P0 完成；视觉语义已冻结：reduced motion 下灯可常亮/常暗，但 Working、Waiting、Error 必须仍可区分。  
**成功行为：** Appearance 页面可修改并持久化设置；运行 host 读取设置后，动态状态不闪烁。  
**独立 verifier：** config JSON、widget-effects unit test、已安装桌面录屏三者一致。  
**开发者反馈点：** 若“无动画时三种状态如何区分”没有产品决定，不写实现，转 needs-decision。

已冻结规格：reduced motion 下 Working=绿灯常亮、Waiting=黄灯常亮、Error=红灯常亮；颜色和位置保持不变，所有 blink alpha 固定为常亮。

**用户接受的例外（2026-07-13）：** 不要求 reduced-motion 下状态注入、桌面录屏或截图；本 phase 以 UI 持久化、配置读取和 focused renderer test 作为完成依据，不得声称存在最终桌面视觉录屏。

- [x] RLS-2-01 确认 reduced-motion 的非动画视觉规格（每个状态的颜色/亮度/常亮规则），写入本文件或专门规格后再编码。（AUD-5-07）
- [x] RLS-2-02 在 `AppearancePage.tsx` 添加可访问、可持久化的 `appearance.reduced_motion` 控件；构建通过，已安装 Settings IPC/桌面保存仍待验。（AUD-5-07）
- [x] RLS-2-03 将配置传入 `WidgetEffectsState`/renderer，使动画 timer 和 blink alpha 遵守 reduced motion；新增 focused unit tests。（AUD-5-07）
- [x] RLS-2-04 相关 tests、独立 release settings/host build 均通过，且已部署至当前安装目录。（AUD-VAL-13）
- [x] RLS-2-05 用户接受的例外：已安装 Settings UI 切换确实持久化 `config.json`，随后恢复原值；不要求状态注入与桌面录屏。（AUD-5-07/5-10）
- [x] RLS-2-06 已更新 audit/reflection；UI persistence、host config reload 与 focused renderer test 一致。（AUD-5-07）

## P3：Settings 生命周期与可诊断性

**进入条件：** P0 完成。  
**成功行为：** Settings spawn/reuse/close 有明确且已验证的生命周期；所有已知失败状态有用户文案或可定位日志。  
**独立 verifier：** lifecycle report 的 PID/退出状态 + 已安装 Settings 真实 IPC 操作。  
**决策点：** “关闭即退出”与“隐藏/常驻”均可行，但必须由产品选择并与 verifier/UX 一致。

- [x] RLS-3-01 复现 lifecycle failure，保存 report、host/settings PID 时间线与配置；根因是初始 Tauri HWND 尚未可见即被判 stale。（AUD-6-05）
- [x] RLS-3-02 决定 Settings 关闭即退出；这是当前 verifier 和既有 UX 的一致语义。（AUD-6-05）
- [x] RLS-3-03 复用同 PID 的初始 HWND 并 ShowWindow；focused test、debug 与 root `target\\release` lifecycle verifier、workspace test 和单独 release build 均通过。（AUD-6-05）
- [x] RLS-3-04 已在已安装 Settings 的「03 数据源」真实执行「重新部署 Codex hooks」和「立即刷新检测」：分别显示 `hooks deployed successfully`、`检测刷新已请求`，刷新后显示 `Codex：✅ 已就绪`、`state_file / confirmed / 最近事件`，诊断为 `attached`、最近错误「无」。（AUD-6-GATE）
- [x] RLS-3-05 失败/降级路径已具备可定位 UX 或日志：已安装页面明确说明 Claude Code 仅进程检测；hook 状态单测覆盖 `process_only`、配置未验证和损坏 `error`；attach 失败保留 tray-only 运行日志基线；state 不可写的正式 hook 证据为 `os error 183`，位于 `%TEMP%\\cc-traffic-light-audit-2-12-20260713-142630`；未 trust 使用「已配置但尚未验证触发（请在终端运行 /hooks trust）」文案。（AUD-VAL-15）
- [x] RLS-3-06 已更新本 checklist、handoff 与 reflection；debug/release lifecycle verifier 已通过，AUD-6-GATE 的 Settings 真实操作证据已归档。

## P4：Claude 支持等级（证据优先，不预设结论）

**进入条件：** P0 完成，且 Claude Code、终端和配置层级可确定。  
**成功行为：** 得到 Active、实验性或 DegradedProcessOnly 的可复现实证；UI、示例和文档完全匹配该等级。  
**独立 verifier：** 最小 marker、受控 payload dump、真实事件矩阵和 state/UI 观察分开保存。  
**失败路由：** 同一失败两次后停止路径试错；如 command hook 不稳定，选择 ProcessOnly 或请求是否值得建设 HTTP prototype。

**产品决定（2026-07-13）：** PowerShell shell form 是本轮唯一需要支持的 Claude command-hook 形式。该形式在 Claude Code 2.1.204 / Windows 11 下已验证 Active；Git Bash、exec form、Notification 与失败事件不属于本轮承诺，作为 accepted limitation。

- [x] RLS-4-01 固定 Claude Code 2.1.204 / Windows 11 / PowerShell 5.1 / 项目级配置层级。（AUD-3-01）
- [x] RLS-4-02 最小 marker 以 PowerShell shell form 成功触发；首次 wrapper stdin bug 修复后仅重试一次。（AUD-3-02/09）
- [x] RLS-4-03 dump-only wrapper 采集到 JSON shape、`session_id` 与 `hook_event_name`，不保存 raw payload。（AUD-3-03/08）
- [x] RLS-4-04 用户接受限制：只验证 `shell=powershell`；Git Bash 与 exec form 不承诺。（AUD-3-04/05/06）
- [x] RLS-4-05 验证 SessionStart、UserPromptSubmit、PreToolUse、PostToolUse、Stop；Notification/失败事件为 accepted limitation。（AUD-3-07）
- [x] RLS-4-06 交互式 Claude session 已完成 hook trust/review；用户级部署不属于本轮承诺。（AUD-3-12）
- [x] RLS-4-07 支持等级为 Active（PowerShell shell form）；HTTP prototype 不需要。（AUD-3-10/11/13/GATE）
- [x] RLS-4-08 README/UI 已保留比当前环境更保守的默认 ProcessOnly 表述；本 checklist 记录 opt-in PowerShell Active matrix。（AUD-3-13）

## P5：安装、升级、卸载与清洁环境

**进入条件：** P1/P2/P3 的 required 代码行为通过；用户提供新的 ISCC 安装器和 pnpm build 证据。  
**成功行为：** 新用户从安装到 Codex real event/widget，升级和卸载都不静默破坏配置或 hooks。  
**独立 verifier：** 临时安装目录/测试账户中的文件、注册表、自启动、hooks、state、UI 与卸载后状态。  
**外部阻塞：** 缺 ISCC 或 pnpm 输出时标 environment blocked，继续 P4/P6 可执行分支。

**用户接受的例外（2026-07-13）：** 用户明确指示 ISCC 新安装器、隔离账户以及安装/升级/卸载验收无需处理，视为本轮已接受的 release limitation；不得将其表述为已获得独立安装器证据。

- [x] RLS-5-01 用户接受的例外：不构建/验收新 ISCC 安装器；本轮 pnpm build 已通过。（AUD-1-06、AUD-VAL-04）
- [x] RLS-5-02 用户接受的例外：不执行临时安装和注册表验收。（AUD-1-07、AUD-7-01/02）
- [x] RLS-5-03 用户接受的例外：不执行清洁用户 Codex 安装验收。（AUD-7-03、AUD-2-GATE）
- [x] RLS-5-04 用户接受的例外：不执行清洁用户 Claude 验收。（AUD-7-04）
- [x] RLS-5-05 用户接受的例外：不执行清洁环境异常矩阵。（AUD-7-05/06）
- [x] RLS-5-06 用户接受的例外：不执行测试账户升级验收。（AUD-1-09、AUD-7-07）
- [x] RLS-5-07 用户接受的例外：不执行卸载及残留验收。（AUD-1-10、AUD-7-08）
- [x] RLS-5-08 已在本文件记录接受例外；不进行破坏性安装器操作。

## P6：最终证据、质量与 Release Gate

**进入条件：** P1–P5 的 required tasks 通过，或每个未通过项有明确 accepted limitation。  
**成功行为：** 任何“通过”都有可复查原始证据；任何“未通过”不会被 UI/文档掩盖。  
**独立 verifier：** 另一执行者可仅依据 evidence manifest、命令、截图和状态文件复核结论。  
**退出：** 仅在 release report 通过或以明确 Blocked/Accepted limitation 交接时退出。

- [x] RLS-6-01 用户接受的例外：不要求正常动效的最终截图或录屏。已保留 2026-07-13 人工状态确认、state/runtime 诊断与 renderer tests；不得将其表述为已保存影像证据。（AUD-5-10/5-GATE、AUD-VAL-13）
- [x] RLS-6-02 用户接受的例外：不要求 reduced-motion 桌面状态矩阵；保留 UI persistence、config reload 和 unit evidence。（AUD-5-07）
- [x] RLS-6-03 已在 Explorer restart 后不重启 host 验证自动恢复：同 PID、host_count=1，日志有 tray pending→retry registered→attach；tray-only 基线与 recovery 均已归档。（AUD-5-08/09、AUD-VAL-14）
- [x] RLS-6-04 `cargo test --workspace --offline`、`pnpm build`、单独 release settings/host build 均通过；`cargo fmt --all -- --check` 已运行但报告既有 dirty 文件格式差异，按约束未静默格式化，作为最终 gate limitation 记录。（AUD-VAL-01/04）
- [x] RLS-6-05 evidence manifest 已汇总于 `docs/handoff/release-evidence-manifest-2026-07-13.md`；明确记录构建、生命周期、Settings IPC、Explorer recovery、状态观察和缺失影像证据。（AUD-DOC-06）
- [x] RLS-6-06 README 的 Claude 降级声明、支持矩阵与原 audit 已更新为当前证据：默认 ProcessOnly，PowerShell shell-form 仅为 opt-in 限定 Active，不承诺其他 form；Settings gate 已更新。（AUD-3-13、AUD-6-GATE）
- [x] RLS-6-07 已移除 `%TEMP%\\cc-traffic-light-claude-hooks` 临时 dump；保留 Explorer/state-write 原始证据，执行 diff/status 与 API-key-shaped value 审查，未发现明显 secret，未格式化或暂存既有 dirty files。（AUD-CLN-01/02/04）
- [x] RLS-6-08 已生成最终 gate 报告与 reflections；报告状态为「影像证据待补，尚不批准 release」，列出已通过、accepted limitation、格式基线和后续 owner。（AUD-0/1/2/3/5/6/7-GATE、AUD-7-09）

## Completion Criteria

- Explorer restart 自动恢复，无需杀/重启 host；reduced motion 可配置、持久化并有无闪烁桌面证据。
- Settings 生命周期策略已选定并通过对应 verifier；所有失败状态有可理解 UI 或可定位日志。
- Claude 支持等级由当前环境 evidence 决定，UI/文档不夸大；Codex 清洁安装到真实事件和 widget 闭环通过。
- 安装、升级、卸载、异常路径、构建和最终视觉证据完整；所有例外均被明确标记为 blocked 或 accepted limitation。
- `RLS-6-08` 的 release report 与 reflections 完整，下一位执行者无需重放聊天记录即可继续或复核。
