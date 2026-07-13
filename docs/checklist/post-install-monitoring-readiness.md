# Post-Install Monitoring Readiness Checklist

基于 `docs/plan/post-install-monitoring-readiness.md` 的逐阶段可执行任务清单。

## Checklist Objective

使安装 CC Traffic Light 后首次打开，就能高置信度监控 Codex 和 Claude Code 的活动状态。

> 当前状态（2026-07-10）：本 checklist 的部分早期条目描述的是历史缺口。installer.iss 当前已包含三个 exe，hook CLI 已支持 --version，Codex merge/restore fixture 已通过。持续执行请以 end-to-end-install-monitoring-audit.md 为主。

**Target outcome:**
- 安装器包含 `taskbar_widget_hook.exe`，安装后自动部署全局 Codex hooks
- Claude Code payload 已验证或明确记录其限制
- 用户通过 tray/settings UI 接收信任引导

**Scope:** 安装器 + hooks 部署 + Claude 采样 + 信任引导，不包含 P4 runtime hardening。

**Non-goals:** 多显示器、云端 Codex 会话、managed hooks 绕过 trust。

---

## Pre-Implementation Checks

- [ ] CSW-PRE-01 确认计划文档已写入 `docs/plan/post-install-monitoring-readiness.md`
- [x] CSW-PRE-02 阅读 `installer.iss`；历史上只包含两个 exe，当前已包含 taskbar-widget、taskbar-settings-tauri 和 taskbar_widget_hook 三个 exe
- [ ] CSW-PRE-03 阅读 `taskbar-widget/scripts/install-codex-hooks.ps1`，确认第 3 行 `$HookExecutablePath` 的默认值
- [ ] CSW-PRE-04 阅读 `.codex/hooks.json`，确认当前所有 command 路径指向 dev debug 目录
- [ ] CSW-PRE-05 阅读 `taskbar-widget/examples.claude-hooks.json`，确认当前示例配置的结构
- [x] CSW-PRE-06 阅读 `taskbar-widget/src/bin/taskbar_widget_hook.rs`，确认当前已支持 `--version`
- [ ] CSW-PRE-07 确认 `cargo build -p taskbar-widget --release --offline` 能否正常生成 `taskbar_widget_hook.exe`

---

## Implementation Checklist

### Phase 1: 修复部署断裂

目标：安装器包含 `taskbar_widget_hook.exe`，脚本路径与安装器一致。

- [ ] CSW-1-01 在 `installer.iss` 的 `[Files]` 段加上 `taskbar_widget_hook.exe`，与主 exe 同目录
- [ ] CSW-1-02 同步 `install-codex-hooks.ps1` 的 `$HookExecutablePath` 默认值从 `$env:LOCALAPPDATA\CcTrafficLight\bin\` 改为 `$env:LOCALAPPDATA\Programs\CC Traffic Light\`
- [ ] CSW-1-03 审核并更新所有示例配置中的硬编码路径引用：
      - `.codex/hooks.json`
      - `examples.claude-hooks.json`
      - `examples.codex-hooks.toml`
      - `README.md` 中写死的路径示例（改为 `{app}\taskbar_widget_hook.exe` 模板表述或相对路径）
- [ ] CSW-1-04 运行 `cargo build -p taskbar-widget --release --offline`，确认 `target/release/taskbar_widget_hook.exe` 成功生成
- [ ] CSW-1-05 运行 `cargo check -p taskbar-widget --offline`，确认无编译警告

### Phase 2: 接入全局 hooks + 安装器联动

目标：安装后全局 Codex hooks 自动就位，settings UI 提供修复/重部署能力。

- [ ] CSW-2-01 用 release 路径测试 `install-codex-hooks.ps1` 的 dry-run 模式：
      ```powershell
      .\taskbar-widget\scripts\install-codex-hooks.ps1 -HookExecutablePath "$env:LOCALAPPDATA\Programs\CC Traffic Light\taskbar_widget_hook.exe" -ShowPaths
      ```
      确认 `Assert-StableHookExecutablePath` 不拒绝该路径，且输出摘要正确
- [ ] CSW-2-02 在既有 hooks.json 的场景下测试 apply/restore 往返：
      ```powershell
      # 备份当前 hooks
      copy "$env:USERPROFILE\.codex\hooks.json" "$env:USERPROFILE\.codex\hooks.json.bak"
      # apply
      .\taskbar-widget\scripts\install-codex-hooks.ps1 -HookExecutablePath "..." -Apply -ShowPaths
      # 检查结果
      Get-Content "$env:USERPROFILE\.codex\hooks.json"
      # restore
      .\taskbar-widget\scripts\install-codex-hooks.ps1 -Restore -ShowPaths
      # 确认恢复
      ```
- [ ] CSW-2-03 在 `installer.iss` 的 `[Run]` 段添加 hooks 部署命令：
      ```
      Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Apply"; Flags: runhidden
      ```
      或将 install 脚本逻辑内联为 Inno Setup Pascal 脚本（备选）
- [ ] CSW-2-04 在 `taskbar_widget_hook.rs` 中添加 `--version` 自检支持（输出 `env!("CARGO_PKG_VERSION")`）
- [ ] CSW-2-05 在 taskbar-settings-tauri UI 中添加"监控配置"区域：
      - 显示 Codex hook 状态（已安装 / 未安装 / 版本过期）
      - "重新部署 hooks"按钮
      - 调用 `install-codex-hooks.ps1 -Apply`（通过 Tauri 后端 shell command 或 named pipe 触发）
- [ ] CSW-2-06 在 `settings_bridge.rs` 中添加 hook 状态检测接口，暴露给 settings UI

### Phase 3: Claude Code payload 采样验证

目标：拿到真实 Claude Code hook payload，决定高置信度监控是否可行。

注意：本阶段大多数任务需要真实的 Claude Code 环境和人在回路中 trust hooks，纯 coding agent 无法独立完成。标注 `[manual]` 的任务需要人工执行。

- [ ] CSW-3-01 在项目根新建 `.claude/hooks.json`，参考 `examples.claude-hooks.json` 但 command 指向 dump-only wrapper 脚本（避免误写入状态文件）
- [ ] CSW-3-02 编写 dump-only wrapper 脚本 `taskbar-widget/scripts/claude-lifecycle-hook-dump.ps1`，把 stdin JSON shape 保存到 `%TEMP%\cc-traffic-light-claude-hooks\`
- [ ] CSW-3-03 [manual] 在当前项目目录启动 Claude Code
- [ ] CSW-3-04 [manual] 在 Claude Code 中运行 `/hooks`，检查 `.claude/hooks.json` 是否加载，完成 review/trust
- [ ] CSW-3-05 [manual] 触发一个简单只读任务（例如"列出当前目录文件"），确保触发 `PreToolUse` 和 `PostToolUse`
- [ ] CSW-3-06 [manual] 收集 dump 输出：
      ```powershell
      Get-ChildItem "$env:TEMP\cc-traffic-light-claude-hooks" -Filter "claude-hook-*.json" | Sort-Object LastWriteTime -Descending | Select-Object -First 5 | Get-Content
      ```
- [ ] CSW-3-07 分析采样证据，记录以下结论到 `docs/checklist/` 或 `docs/reflections/`：
      - stdin 是否为 JSON: 是 / 否
      - `session_id` 字段名及存在性
      - event name 列表是否匹配 `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Notification`, `PostToolUseFailure`
      - `event_order` 或 `timestamp` 是否存在
      - 是否需要 `/hooks` trust
- [ ] CSW-3-08 基于采样证据更新决策记录：
      - 情况 A：payload 有 `session_id` → 走通高置信度链路，更新 `hook_rules.rs`
      - 情况 B：payload 无 `session_id` → 接受单 session 模型（`claude_unknown`）
      - 情况 C：command hooks 不支持 → Claude 只保留进程检测
- [ ] CSW-3-09 如果走通（情况 A）：更新 `hook_rules.rs` 中的 Claude Code 事件映射表，添加 `Notification` → `Waiting`、`PostToolUseFailure` → `Error` 等
- [ ] CSW-3-10 如果走通（情况 A）：扩展 `install-codex-hooks.ps1` 为 `install-hooks.ps1`，增加 `-Agent claude` 参数，支持同时部署 Claude Code hooks
- [ ] CSW-3-11 如果走通（情况 A）：在安装器中加入 Claude Code hooks 部署步骤
- [ ] CSW-3-12 更新 settings UI 展示 Claude Code hook 状态

### Phase 4: 信任引导体验闭环

目标：用户首次启动时收到清晰的信任引导，不依赖外部文档。

- [ ] CSW-4-01 在 `main.rs` 的 `poll_hook_state()` 或首次运行检测路径中增加 hook 状态检测：
      - 检查 `%APPDATA%\CcTrafficLight\state.json` 是否存在且非空
      - 检查 `%USERPROFILE%\.codex\hooks.json` 是否包含 `CcTrafficLight` 命名的条目
      - 定义三态枚举：`NotInstalled` / `InstalledButUntrusted` / `ActiveAndWorking`
- [ ] CSW-4-02 在 `tray_icon.rs` 或 `main.rs` 的初始化流程中，如果检测到 `NotInstalled` 或 `InstalledButUntrusted` 状态，弹出 tray balloon notification：
      - 只在首次检测到时弹出一次（持久化标记到 reg 或状态文件）
      - 内容："CC Traffic Light 已安装监控 hooks，请在 Codex 终端中运行 /hooks 并 trust 相关命令"
- [ ] CSW-4-03 在 Tauri settings UI 的"监控配置"区域增加信任状态指示：
      - Codex: "❌ 未信任 - 请在 Codex 中运行 /hooks" / "✅ 已就绪"
      - Claude Code: "❌ 未信任" / "✅ 已就绪" / "不适用（仅进程检测）"
- [ ] CSW-4-04 更新 `settings_bridge.rs` 暴露信任状态枚举给 settings UI（可通过 IPC 或共享状态）

---

## Validation Checklist

### Phase 1 验证

- [ ] CSW-VAL-01 运行 `cargo build -p taskbar-widget --release --offline`，确认成功，无错误
- [ ] CSW-VAL-02 运行 `cargo check -p taskbar-widget --offline`，确认无警告
- [ ] CSW-VAL-03 确认 `target/release/taskbar_widget_hook.exe` 文件存在
- [ ] CSW-VAL-04 运行 `target/release/taskbar_widget_hook.exe list`，预期输出当前 state.json 内容或空状态
- [ ] CSW-VAL-05 运行 `target/release/taskbar_widget_hook.exe --version`，预期输出版本号
- [ ] CSW-VAL-06 检查 `installer.iss` 中新加的 `[Files]` 行与现有格式一致
- [ ] CSW-VAL-07 检查 `install-codex-hooks.ps1` 中路径默认值已更新

### Phase 2 验证

- [ ] CSW-VAL-08 运行 `install-codex-hooks.ps1 -ShowPaths` dry-run，预期输出无错误
- [ ] CSW-VAL-09 用临时目录测试 `install-codex-hooks.ps1 -Apply`，确认 backup + 写入 + restore 往返完整
- [ ] CSW-VAL-10 检查 `installer.iss` `[Run]` 段的命令格式是否正确
- [ ] CSW-VAL-11 运行 Tauri settings 前端，确认"重新部署 hooks"按钮存在且能触发回调

### Phase 3 验证

- [ ] CSW-VAL-12 [manual] Claude Code 中 `/hooks` 能看到 `.claude/hooks.json` 的配置
- [ ] CSW-VAL-13 [manual] 触发事件后 `%TEMP%\cc-traffic-light-claude-hooks\` 下有 dump 文件生成
- [ ] CSW-VAL-14 采样证据已记录到 `docs/reflections/`
- [ ] CSW-VAL-15 如果走通：`hook_rules.rs` 中有 Claude Code 事件映射
- [ ] CSW-VAL-16 如果走通：`install-hooks.ps1`（或扩展后的脚本）支持 `-Agent claude`

### Phase 4 验证

- [ ] CSW-VAL-17 在没有 state.json 的干净环境下启动 widget，tray 弹出信任引导通知
- [ ] CSW-VAL-18 打开 settings UI，看到每个 agent 的信任状态指示
- [ ] CSW-VAL-19 信任引导通知不会在每次启动时重复弹出

### 集成验证（最终验收）

- [ ] CSW-VAL-20 [manual] 完整流程验证：
      1. 运行安装器 → 检查 `%LOCALAPPDATA%\Programs\CC Traffic Light\` 含三个 exe
      2. 检查 `%USERPROFILE%\.codex\hooks.json` 已被写入
      3. 启动 widget → 看到 tray 图标
      4. 打开 Codex → 运行 `/hooks` → trust
      5. 触发一个 prompt
      6. 观察 widget 从 Idle → Working → Completed/Idle 变化

---

## Documentation Checklist

- [ ] CSW-DOC-01 更新 `docs/plan/post-install-monitoring-readiness.md` 在执行过程中如有偏差
- [ ] CSW-DOC-02 更新 `installer.iss` 的注释说明
- [ ] CSW-DOC-03 更新 `taskbar-widget/README.md` 中的 hook 路径描述
- [ ] CSW-DOC-04 记录 Claude Code payload 采样结论到 `docs/reflections/`
- [ ] CSW-DOC-05 更新 `docs/checklist/` 索引或 README 指向新 checklist

---

## Cleanup Checklist

- [ ] CSW-CLN-01 确认没有在 repo 中提交 `%APPDATA%` 或 `%TEMP%` 的临时文件
- [ ] CSW-CLN-02 确认 `.codex/hooks.json` 的 dev 调试路径已被模板路径替代
- [ ] CSW-CLN-03 确认 `install-codex-hooks.ps1` 中的 ShowPaths 模式在生产路径下不会泄露用户路径
- [ ] CSW-CLN-04 确认 `taskbar_widget_hook.exe --version` 使用 `CARGO_PKG_VERSION` 而非硬编码
- [ ] CSW-CLN-05 确认没有在 Rust 或 Tauri 代码中嵌入可执行文件路径或用户级绝对路径

---

## Completion Criteria

以下条件全部满足时，本轮可判定完成：

- `installer.iss` 包含 `taskbar_widget_hook.exe`，且路径与 `install-codex-hooks.ps1` 的默认值一致
- 安装后 `%USERPROFILE%\.codex\hooks.json` 能被安装器自动写入（[Run] 段或首次运行逻辑）
- `taskbar_widget_hook.exe` 支持 `--version` 自检
- Tauri settings UI 能显示 hook 状态并提供"重新部署"能力
- Claude Code payload 采样已执行，结论已记录，`hook_rules.rs` 已相应更新（或已明确记录限制）
- 首次运行时 tray 通知能引导用户执行 `/hooks` trust
- `cargo build -p taskbar-widget --release --offline` 通过

**可接受的已知限制：**
- PowerShell 执行策略可能阻止安装器 [Run] 段脚本——由 settings UI 的"重新部署"按钮作为兜底
- Claude Code 如果 command hooks 不支持，只保留进程检测（Degraded 置信度）
- trust 状态目前只能通过"状态文件是否被写入"间接推断，无法直接查询 Codex/Claude Code 内部状态

---

## Reflection / Task Summary Generation

每完成一个 checklist item，自动生成：

```
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

- task id 必须对应本 checklist 中的条目，例如 `CSW-1-02`
- 完成、阻塞、跳过都要生成 reflection，并写明原因
- Phase 3 涉及 Claude Code 人工采样的任务（标注 `[manual]`），如果当前回合无法执行，记录为 blocked 并继续后续任务
- 涉及 `installer.iss` 的修改，需同时提供 `cargo build` 验证结果
