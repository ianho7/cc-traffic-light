# 安装后首次运行体验：完整监控链路实施计划

> 状态说明（2026-07-10）：本文保留为历史实施计划。当前 installer.iss 已包含三个 exe，scripts/pack-all.ps1 已显式构建并验收 hook CLI；Claude Code command hooks 在当前 Windows 环境仍不能作为稳定生产能力，当前结论以 docs/plan/end-to-end-install-monitoring-audit.md 和最新 retrospective 为准。

## Objective

使用户安装 CC Traffic Light 后，首次打开就能**高置信度地**（不只是进程存在检测）监控 Codex 和 Claude Code 的活动状态，覆盖 Working / NeedsAttention / Completed 等完整红绿灯语义。

### 解决的问题

- 历史版本的安装器只装了两个 exe，曾缺少 hook 写入二进制；当前 installer.iss 已包含 taskbar_widget_hook.exe
- 项目级 `.codex/hooks.json` 仍硬编码开发路径，release 安装后无效
- 全局 hook 安装脚本已实现但未接入安装器，且默认二进制路径不一致
- Claude Code 的 hook payload 形状从未被验证过，属于未知可行性
- 首次运行完全没有信任引导

### 预期结果

安装后，开机自启状态下：
- Codex 会话自动写入 `%APPDATA%\CcTrafficLight\state.json`
- Widget 每 1 秒轮询状态文件，显示 Working / NeedsAttention / Completed
- Claude Code 至少达到进程存在检测级别（若 hooks 验证通过，则达到同等高置信度）
- 用户只需在 Codex/Claude Code 中运行一次 `/hooks` trust

### 范围内

- 安装器打包 `taskbar_widget_hook.exe`
- 安装器在安装后自动执行全局 hooks 部署
- 首次运行的信任引导提示
- Claude Code 真实 payload 采样验证

### 范围外

- 纯 Web/云端 Codex 会话（没有本地 hooks 支持）
- 企业级 managed hooks 绕过 trust
- 多显示器 taskbar 兼容性
- runtime hardening（P4 计划的内容）

---

## Background and Context

### 现有架构

```
Codex / Claude Code
    │ (hook event + stdin JSON)
    ▼
taskbar_widget_hook.exe
    │ (apply_hook_event → atomic write)
    ▼
%APPDATA%\CcTrafficLight\state.json
    │ (WM_TIMER 每 1 秒 poll)
    ▼
taskbar-widget.exe → detector::build_status_snapshot → InvalidateRect → redraw
```

### 已有资产（已验证通过）

| 资产 | 状态 |
|---|---|
| `taskbar_widget_hook.rs` 二进制入口 | ✅ 完整，支持 `codex|claude <HookName>` / `sample` / `list` / `set` / `clear` |
| `agent_state.rs` 状态写入 | ✅ 多任务 `codex_<session_id>`，mutex 保护，automatic summary，过期清理 |
| `detector.rs` 双路径检测 | ✅ hook state 优先，进程 fallback 兜底 |
| `hook_rules.rs` 状态映射 | ✅ 7 个 Codex 事件 → Idle/Working/Waiting/Done/Error |
| `.codex/hooks.json` 项目级配置 | ✅ 已验证通过真实 Codex 会话能写入 `codex_<session_id>` |
| `ui_state.rs` 红绿灯映射 | ✅ AgentState → SourceVisualState（Working → NeedsAttention → Completed） |
| widget 重绘链路 | ✅ 1 秒轮询 + 80ms 动画，已通过 debug CLI 和真实 Codex 事件验证 |
| `install-codex-hooks.ps1` | ✅ 已实现 dry-run/apply/restore、backup、merge，但默认路径与安装器不一致 |
| `examples.claude-hooks.json` | ⚠️ 示例配置已写，但从未被真实 Claude Code 验证过 |

### 已知限制（当前已确认）

- 没有 `event_order` 字段在 Codex payload 中，使用 `received_at` 兜底排序
- `PostToolUse` 失败场景保守映射为 `Working`（非 `Error`），等真实 payload 证据再收紧
- widget 在长时间无状态变化后可能被 Windows 回收或重绘被覆盖（P4 问题）

---

## Current State Analysis

### 关键文件

| 文件 | 角色 | 状态 |
|---|---|---|
| `installer.iss` | 安装器定义 | ✅ 已包含三个 exe；仍需完成最终清洁安装、升级和卸载验收 |
| `.codex/hooks.json` | 项目级 Codex hooks | ⚠️ 仅作为开发 fixture；正式路径由全局安装脚本管理 |
| `taskbar-widget/scripts/install-codex-hooks.ps1` | 全局 Codex hooks 安装器 | ✅ 已有 merge/backup/restore；需继续完成清洁安装和真实 Codex 事件验收 |
| `taskbar-widget/examples.claude-hooks.json` | Claude Code hooks 示例 | ❌ 从未经过真实验证 |
| `taskbar-widget/src/bin/taskbar_widget_hook.rs` | hook 写入二进制 | ✅ 完整 |
| `taskbar-widget/src/agent_state.rs` | 状态文件读写 | ✅ 完整 |
| `taskbar-widget/src/hook_rules.rs` | 状态映射规则 | ✅ 完整 |

### 路径冲突

当前 `install-codex-hooks.ps1` 的默认二进制路径：
```
$env:LOCALAPPDATA\Programs\CC Traffic Light\taskbar_widget_hook.exe
```

安装器实际路径：
```
{localappdata}\Programs\CC Traffic Light\taskbar_widget_hook.exe
```

与 installer.iss 的 `{app}` 目录契约一致；仍需在真实安装后的路径上验收。

### Claude Code 不确定项

| 不确定项 | 当前假设 | 需要验证 |
|---|---|---|
| `type = "command"` hooks 是否被支持 | 推测支持，与 Codex 同理 | 真实部署 + 触发 |
| payload 中 `session_id` 字段是否存在和字段名 | 猜测 `session_id` | 真实 dump |
| event name 列表 | 猜测 `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Notification`, `PostToolUseFailure` | 真实 dump |
| stdin 编码 | 假设与 Codex 一致（JSON, 多种编码） | 真实 dump |
| 是否需要 `/hooks` trust | 推测需要 | 真实环境验证 |

---

## Proposed Solution

### 整体策略

分四个阶段推进，每阶段交付一个可独立验证的结果：

1. **修复部署断裂**：安装器加 hook 二进制 + 修复路径一致性
2. **接入全局 hooks 安装**：安装器运行 `install-codex-hooks.ps1`，首次启动时或 settings UI 提供手动按钮
3. **Claude Code 验证**：先 dump 采样，再决定是否走通高置信度监控
4. **信任体验闭环**：通知提示 + settings UI 引导

### 设计决策

1. **统一 hook 二进制路径**：`{app}\taskbar_widget_hook.exe`（跟主 exe 同目录），`install-codex-hooks.ps1` 默认路径同步更新
2. **安装时部署 vs 运行时部署**：安装器在 `[Run]` 段执行 `install-codex-hooks.ps1 -Apply`；settings UI 提供"重新部署 hooks"按钮用于升级/修复场景
3. **Claude Code 先采样再承诺**：不做 Claude Code hooks 自动部署，直到真实 payload 证据确认可行

---

## Alternatives Considered

### 替代方案 1：运行时由 Rust 代码直接写入 hooks.json

- 优势：不需要 PowerShell 脚本依赖
- 劣势：Windows JSON 文件操作在 Rust 中比 PowerShell 繁琐；备份/merge/恢复逻辑需要重写；安装器后的首次部署时机更难确定
- **未选择原因**：已有完整的 PowerShell 脚本，复用比重写更安全

### 替代方案 2：只做项目级 hooks，不做全局

- 优势：最简单，trust 只影响当前项目
- 劣势：用户必须在每个项目目录下放 `.codex/hooks.json`，体验极差
- **未选择原因**：P1 计划已经否决了这个方向

### 替代方案 3：Claude Code hooks 不做验证直接上

- 优势：快
- 劣势：如果 Claude Code 不兼容，用户看到的要么是静默失败，要么是垃圾状态
- **未选择原因**：风险过高，不值得冒险

---

## Implementation Plan

### Phase 1: 修复部署断裂（半天）

**Goal:** 安装器包含 `taskbar_widget_hook.exe`，路径与 install 脚本一致

**Files:**
- `installer.iss`
- `taskbar-widget/scripts/install-codex-hooks.ps1`（默认路径）
- 所有开发文档中的硬编码路径样例

**Tasks:**

1. 更新 `installer.iss` `[Files]` 段，加上 `taskbar_widget_hook.exe`
2. 同步 `install-codex-hooks.ps1` 的 `$HookExecutablePath` 默认值从 `$env:LOCALAPPDATA\CcTrafficLight\bin\` 改为 `$env:LOCALAPPDATA\Programs\CC Traffic Light\`
3. 审核所有示例配置中的路径引用，更新为 `{app}\taskbar_widget_hook.exe` 模板路径
4. 在安装器 `[Run]` 段中加入执行 install-codex-hooks.ps1 的步骤（-Apply 模式），安装后立即部署全局 hooks

**Expected Result:** 安装后 `%LOCALAPPDATA%\Programs\CC Traffic Light\` 下出现 `taskbar_widget_hook.exe`，且 `%USERPROFILE%\.codex\hooks.json` 被自动写入指向该路径的 hooks

---

### Phase 2: 接入全局 hooks 安装 + 安装器联动（1 天）

**Goal:** 安装完成后，全局 Codex hooks 自动就位，无需用户手动操作

**Files:**
- `installer.iss`
- `taskbar-widget/scripts/install-codex-hooks.ps1`
- `taskbar-widget/src/bin/taskbar_widget_hook.rs`（可能需要加 `--validate-path` 或版本号自检）

**Tasks:**

1. **验证 `install-codex-hooks.ps1` 在 release 路径下的行为**
   - 用 release 构建的二进制路径测试 dry-run 模式
   - 确认 `Assert-StableHookExecutablePath` 不会拒绝 `Programs\CC Traffic Light\` 路径
   - 验证 backup/restore 在无既有配置和有其配置两种情况下的表现

2. **在安装器 `[Run]` 段添加 hooks 部署**
   ```
   Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Apply"; Flags: runhidden
   ```
   或者将 install 脚本逻辑内联到安装器（Inno Setup Pascal 脚本），减少对 PowerShell 执行策略的依赖

3. **可选：在 taskbar_widget_hook 中加 `--version` 自检**
   ```
   taskbar_widget_hook.exe --version  # 输出 0.1.0
   ```
   辅助安装脚本验证二进制完整性

4. **在 taskbar-widget settings UI（Tauri）中添加"重新部署 hooks"按钮**
   - 调用 `install-codex-hooks.ps1 -Apply`，覆盖升级场景
   - 显示当前 hooks 状态（已安装/未安装/版本过期）

**Expected Result:** 安装后首次启动，widget 即可通过状态文件读取 Codex 的活动状态（前提是 Codex 已信任 hooks）

---

### Phase 3: Claude Code payload 采样验证（几天到一周）

**Goal:** 确认真实 Claude Code hook payload 的形状，决定是否能走通高置信度监控

**Files:**
- `.claude/hooks.json`（新建项目级测试配置）
- `taskbar-widget/scripts/`（新增 claude 助手脚本）
- `taskbar-widget/src/hook_rules.rs`（可能根据采样结果更新状态映射）

**Tasks:**

1. **准备 Claude Code 项目级 dump hook 配置**
   - 在项目根新建 `.claude/hooks.json`
   - 参考 `examples.claude-hooks.json`，但 command 指向 dump-only wrapper 脚本
   - 5 个事件: `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Notification`, `PostToolUseFailure`

2. **信任 hooks 并触发真实事件**
   - 在 Claude Code 中运行 `/hooks`，review/trust
   - 触发一个简单只读任务
   - 收集 dump 输出

3. **分析 payload 证据**
   - 确认 stdin 是否为 JSON
   - 确认 `session_id` 是否存在及字段名
   - 确认 event name 列表是否与预期一致
   - 确认 `event_order` 或 `timestamp` 是否存在
   - 记录所有候选字段路径

4. **根据证据决策**
   - 如果 payload 有 `session_id`：Claude Code 可跟 Codex 一样走通高置信度链路，更新 `hook_rules.rs` 的映射表
   - 如果 payload 没有 `session_id`：退化为单 session 模型（`claude_unknown`），降低预期
   - 如果 command hooks 完全不被支持：Claude Code 只保留进程检测（Degraded 置信度）
   - 更新 `hook_rules.rs` 中的 Claude Code 事件映射

5. **部署 Claude Code 全局 hooks**
   - 参考 `install-codex-hooks.ps1` 实现 `install-claude-hooks.ps1`
   - 或在原脚本中增加 `-Agent claude` 参数复用同一逻辑
   - 更新安装器
   - 更新 settings UI 中的状态展示

**Expected Result:** 确定 Claude Code 能否达到高置信度监控，并有对应的 hooks 部署脚本

---

### Phase 4: 信任引导体验闭环（半天）

**Goal:** 用户首次启动或检测到 hooks 未信任时，收到清晰的操作指引

**Files:**
- `taskbar-widget/src/main.rs`（首次运行检测 + 通知）
- `taskbar-settings-tauri/src/`（UI 信任引导面板）
- `taskbar-widget/src/settings_bridge.rs`（暴露信任状态给 UI）

**Tasks:**

1. **检测 hooks 状态**
   - 检查 `%APPDATA%\CcTrafficLight\state.json` 是否存在
   - 检查 `%USERPROFILE%\.codex\hooks.json` 是否包含本软件条目
   - 状态：`not_installed` / `installed_but_untrusted` / `active_and_working`

2. **首次运行通知**
   - 如果检测到 `not_installed` 或 `installed_but_untrusted`：
     - tray 区域弹出 balloon notification
     - 内容："CC Traffic Light 已安装 Codex 监控 hooks。请在你的 Codex 终端中运行 /hooks 并 trust 相关命令。"
   - 只在首次检测到未信任状态时弹出一次，避免反复打扰

3. **Settings UI 信任面板**
   - 在 Tauri settings 页面新增"监控配置"区域
   - 显示每个 agent（Codex / Claude Code）的 hook 状态
   - "❌ 未信任 - 请在你的终端运行 /hooks" / "✅ 已就绪"
   - "重新部署 hooks" 按钮

4. **安装器最后一步提示**
   - 如果在 `[Run]` 后仍有交互界面，显示信任引导文字
   - 或者在安装完成后自动打开一个说明页面

**Expected Result:** 用户无论从哪个入口（安装器/tray/settings UI）都能知道下一步要做什么

---

## Validation Strategy

### Phase 1 验证

```powershell
# 确认二进制已打包
$installer = "dist\installer\CC-Traffic-Light-Setup-0.1.0.exe"
# 手动安装后确认
Get-ChildItem "$env:LOCALAPPDATA\Programs\CC Traffic Light\"
# 预期出现：taskbar-widget.exe, taskbar-settings-tauri.exe, taskbar_widget_hook.exe

# 确认 install-codex-hooks.ps1 默认路径已更新
Select-String -Path "taskbar-widget\scripts\install-codex-hooks.ps1" -Pattern "Programs"
# 预期匹配到新路径
```

### Phase 2 验证

```powershell
# 安装后检查全局 hooks 是否已部署
Get-Content "$env:USERPROFILE\.codex\hooks.json"
# 预期包含指向 release 路径的 command

# 验证命令路径存在
& "$env:LOCALAPPDATA\Programs\CC Traffic Light\taskbar_widget_hook.exe" list
# 预期成功执行

# 验证 dry-run 模式
& ".\taskbar-widget\scripts\install-codex-hooks.ps1" -ShowPaths
# 预期输出当前 hooks 分析摘要

# 验证 restore
& ".\taskbar-widget\scripts\install-codex-hooks.ps1" -Restore -ShowPaths
# 预期恢复到安装前状态
```

### Phase 3 验证

```powershell
# 项目级 dump hook 配置后，在 Claude Code 中触发事件
# 检查 dump 输出
Get-ChildItem "$env:TEMP\cc-traffic-light-claude-hooks" -Filter "claude-hook-*.json"

# 如果走通：用真实写入验证
& "$env:LOCALAPPDATA\Programs\CC Traffic Light\taskbar_widget_hook.exe" list
# 预期出现 claude_<session_id> 条目
```

### Phase 4 验证

```powershell
# 首次运行后检查 tray 通知是否出现
# 打开 settings UI 查看信任面板
# 预期看到正确的 hook 状态展示
```

### 集成验证（最终验收）

```powershell
# 全新安装全程验证
# 1. 运行安装器
# 2. 确认所有文件到位
# 3. 确认 %USERPROFILE%\.codex\hooks.json 已写入
# 4. 启动 widget
# 5. 在 Codex 中运行 /hooks → trust
# 6. 触发一个 prompt
# 7. 观察 widget 显示 "Working"
# 8. 等待完成
# 9. 观察 widget 显示 "Completed" 或 "Idle"
```

---

## Risks and Mitigations

| # | Risk | Impact | Likelihood | Mitigation | Fallback |
|---|---|---|---|---|---|
| 1 | 安装器 PowerShell 执行策略阻止脚本运行 | hooks 部署静默失败 | 中 | [Run] 加 `-ExecutionPolicy Bypass` | 在 settings UI 提供手动部署按钮；widget 首次运行时检测并提示 |
| 2 | 用户已有自定义 hooks，merge 后行为异常 | 用户原有 hook 被影响 | 低 | install 脚本只添加不删除，且会 backup + merge | restore 模式一键回滚 |
| 3 | Claude Code command hooks 不支持 | Claude 监控退化为纯进程检测 | 中 | Phase 3 先采样再决定 | 保持进程检测路径（已实现），Degraded 置信度 |
| 4 | hook 路径在版本升级后变化，trust hash 失效 | 用户被反复要求 trust | 低 | 安装器确保路径稳定在 `{app}\` 下 | 升级时跑一次 `install-codex-hooks.ps1 -Apply` 更新路径 |
| 5 | 用户从不运行 `/hooks` trust | hooks 永远不触发，状态文件为空 | 高（常见用户行为） | Phase 4 的多入口引导 | 进程检测作为兜底，至少能感知"存在" |

---

## Open Questions

> Historical status update (2026-07-10): the old path-conflict question below is resolved. The current default is the stable Programs\CC Traffic Light installation path, matching installer.iss. The remaining work is real installation, upgrade and uninstall verification.

1. **`install-codex-hooks.ps1` 的路径统一问题**：当前默认路径是 `$env:LOCALAPPDATA\CcTrafficLight\bin\`，安装器路径是 `{localappdata}\Programs\CC Traffic Light\`。是改脚本默认值，还是改安装器路径？—— 推荐改脚本默认值，因为安装器路径已发布过，更稳定。

2. **安装器是否该内联 PowerShell 脚本逻辑？** 用 Inno Setup Pascal 脚本直接操作 JSON 可以避免 PowerShell 执行策略问题，但代码量大得多。当前推荐先走 `[Run]` + PowerShell，P4 阶段再优化。

3. **Claude Code hooks 部署是否复用同一个 PowerShell 脚本？** 推荐扩展 `install-codex-hooks.ps1` 为 `install-hooks.ps1`，增加 `-Agent claude` 参数，避免维护两个几乎相同的脚本。

4. **trust 状态能否编程检测？** Codex/Claude Code 的 trust 状态是否存在文件或 API 可以读取？目前不确定。如果不能，只能通过"状态文件是否被写入"间接推断。

---

## Recommended Next Step

> The historical Phase 1 next step has been completed. Continue with docs/checklist/end-to-end-install-monitoring-audit.md, starting at AUD-1-04 and the remaining installation gates.

**历史 next step 已完成。当前 next step 请执行 docs/checklist/end-to-end-install-monitoring-audit.md 的 AUD-1-04 及之后任务。**

这是最简单的步骤，而且是后续所有步骤的依赖——没有二进制、路径不统一，后面什么都做不了。具体来说：

1. `installer.iss` 的 `[Files]` 增加 `taskbar_widget_hook.exe`
2. `install-codex-hooks.ps1` 第 3 行的 `$HookExecutablePath` 默认值从 `$env:LOCALAPPDATA\CcTrafficLight\bin\` 改为 `$env:LOCALAPPDATA\Programs\CC Traffic Light\`
3. 运行一次 release 构建确认 `taskbar_widget_hook.exe` 正常生成
