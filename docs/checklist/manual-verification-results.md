# CC Traffic Light 人工验收回填表

## 使用说明

1. 按“操作步骤”逐行执行。
2. 只填写最后一列“回填结果”，前面的验收定义不要改动。
3. 回填结果建议使用以下格式：

   ```text
   PASS
   证据：<截图、日志、state.json 或命令输出路径>
   备注：<必要的补充说明>
   ```

   失败或无法执行时使用 `FAIL` 或 `BLOCKED`，并写明实际错误和证据路径。
4. 不要回填完整 payload、token、个人隐私或其他 secrets；只保留必要字段和脱敏后的路径。
5. ISCC 编译和 Volta/pnpm 环境由用户单独处理；安装器和 frontend 的后续运行结果仍可填在对应行。

## 统一准备

以下路径约定必须保持不变。表格中的 `state.json` 默认都指 `$evidence\state\state.json`，不是用户真实目录中的旧文件。

| 名称 | 绝对路径 / 含义 |
|---|---|
| 审计证据根目录 | `$evidence`，例如 `C:\Users\admin\AppData\Local\Temp\cc-traffic-light-audit-20260711-1030` |
| 本轮 state 目录 | `$evidence\state` |
| 本轮 state 文件 | `$evidence\state\state.json` |
| 用户真实 Codex 配置 | `$env:USERPROFILE\.codex\hooks.json` |
| 用户真实 Claude 配置 | `$env:USERPROFILE\.claude\settings.json` |
| 本轮 Codex 工作目录 | `$evidence\codex-project` |
| 本轮 Claude dump 目录 | `$env:TEMP\cc-traffic-light-claude-hooks` |
| 本轮 host runtime log | `$evidence\runtime.log` |
| release hook CLI | `D:\project\cc-traffic-light\target\release\taskbar_widget_hook.exe` |
| release widget host | `D:\project\cc-traffic-light\target\release\taskbar-widget.exe` |
| release Tauri settings | `D:\project\cc-traffic-light\target\release\taskbar-settings-tauri.exe` |
| lifecycle report | `D:\project\cc-traffic-light\taskbar-widget\target\validate-tauri-settings-lifecycle\report.json` |

在开始前执行：

```powershell
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$evidence = Join-Path $env:TEMP "cc-traffic-light-audit-$stamp"
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
New-Item -ItemType Directory -Path "$evidence\state" -Force | Out-Null
New-Item -ItemType Directory -Path "$evidence\codex-project" -Force | Out-Null

$env:TASKBAR_WIDGET_STATE_HOME = "$evidence\state"
$env:TASKBAR_MVP_RUNTIME_LOG_FILE = "$evidence\runtime.log"

$hook = "D:\project\cc-traffic-light\target\release\taskbar_widget_hook.exe"
$widget = "D:\project\cc-traffic-light\target\release\taskbar-widget.exe"
```

确认当前 state 和配置路径：

```powershell
"state_dir=$env:TASKBAR_WIDGET_STATE_HOME"
"state_file=$(Join-Path $env:TASKBAR_WIDGET_STATE_HOME 'state.json')"
"codex_config=$(Join-Path $env:USERPROFILE '.codex\hooks.json')"
"claude_config=$(Join-Path $env:USERPROFILE '.claude\settings.json')"
```

路径区别：

- `$evidence\state\state.json` 是本轮隔离验证使用的 state。
- `$env:APPDATA\CcTrafficLight\state.json` 是未设置 `TASKBAR_WIDGET_STATE_HOME` 时的真实默认 state，本表不把它当作隔离证据。
- `$env:USERPROFILE\.codex\hooks.json` 和 `$env:USERPROFILE\.claude\settings.json` 是真实用户配置，修改前必须备份。
- Codex 必须在 `$evidence\codex-project` 中启动，但仍读取用户级 `$env:USERPROFILE\.codex\hooks.json`。**2026-07-13 修正：** 空临时目录中出现的 Windows sandbox helper 查找失败在执行 `git init` 后仍复现，不能将 Git 初始化作为修复；真实 lifecycle 验收应优先在实际 Git 仓库中进行，并将临时 state 放入该仓库可写目录。
- PowerShell 的 `$state`、`$hook` 和 `$env:TASKBAR_WIDGET_STATE_HOME` 只对当前 PowerShell 及其子进程有效；**新开窗口必须重新设置**。跨窗口检查请使用已打印出的绝对路径，或使用固定路径（例如 `$env:TEMP\cc-traffic-light-codex-verify\state.json`），不能继续引用未定义的 `$state`。

## 验证回填表

## Codex Hook Trust 的具体操作

本项目当前安装脚本写入的是用户级配置：

```text
%USERPROFILE%\.codex\hooks.json
```

Codex 官方支持的 hook 层级还包括：

```text
%USERPROFILE%\.codex\hooks.json
%USERPROFILE%\.codex\config.toml
<当前项目>\.codex\hooks.json
<当前项目>\.codex\config.toml
```

实际操作：

1. 确认 `hooks.json` 中的 command 指向本次要验收的 release executable，例如：

   ```text
   C:\Users\admin\AppData\Local\Programs\CC Traffic Light\taskbar_widget_hook.exe codex PreToolUse
   ```

2. 在**同一个已经设置好 `TASKBAR_WIDGET_STATE_HOME` 的 PowerShell 窗口**进入测试项目目录：

   ```powershell
   Set-Location $evidence\codex-project
   codex
   ```

3. 在 Codex CLI 输入：

   ```text
   /hooks
   ```

4. 在 hooks 浏览界面中按 event 展开配置，找到来源为用户级 `.codex/hooks.json` 或当前项目 `.codex/hooks.json` 的 CC Traffic Light command。
5. 逐个打开新 hook 或 hash 已变化的 command，先检查：

   - executable 是本次验收的 `taskbar_widget_hook.exe`；
   - 参数是 `codex SessionStart`、`codex PreToolUse` 等预期 event；
   - 没有指向 `target\debug`、陌生目录或未知脚本；
   - command 的 stdout/stderr 行为符合预期。

6. 在该 hook 的审核界面选择 **Review** 后查看定义，再选择 **Trust**（不同 Codex 版本的按钮文字可能略有差异，但语义是 review/trust）。如果界面提供按组信任，只有在确认组内每个 command 都属于本项目后才选择全部信任。
7. 关闭 `/hooks` 菜单，执行一个只读任务；然后检查：

   ```powershell
   Get-Content "$env:TASKBAR_WIDGET_STATE_HOME\state.json"
   & $hook list
   ```

8. 如果 command 被修改过，Codex 会按新的 hook hash 再次要求 review/trust；不能把上一次 trust 当作新定义的 trust。

9. 验收时不要使用 `--dangerously-bypass-hook-trust`，因为它绕过的是本次要验证的 trust 闭环。

如果 `/hooks` 报 `failed to parse hooks config ... expected value at line 1 column 1`，先检查文件前三个字节是否为 `239 187 191`。这是 UTF-8 BOM；当前安装脚本已改为 UTF-8 no-BOM 写入。修复前先备份 `hooks.json`，修复后重新启动 Codex，再执行 `/hooks`。

官方依据：[Codex Hooks — Review and trust hooks](https://developers.openai.com/codex/hooks)。官方说明：非 managed command hook 在执行前需要 review/trust；`/hooks` 用于检查来源、审核新/变更 hook、trust 或禁用 hook；项目级 hooks 只有在项目层被信任时才会加载。

| ID | 验证项目 | 操作步骤 | 预期结果 / 证据 | 停止条件 | 回填结果 |
|---|---|---|---|---|---|
| AUD-2-08 | Codex 真实 lifecycle | **前提：** `%USERPROFILE%\.codex\config.toml` 保持 `[windows] sandbox = "unelevated"`；项目级 `D:\project\cc-traffic-light\.codex\hooks.json` 的 7 条 hook 保持 Disabled。**工作目录：** 实际 Git 仓库 `D:\project\cc-traffic-light`。**配置文件：** `$env:USERPROFILE\.codex\hooks.json`。**本轮 state 文件：** `D:\project\cc-traffic-light\.cc-traffic-light-hook-evidence-<stamp>\state.json`。<br>1. 在实际仓库下创建 state 目录并设置 `TASKBAR_WIDGET_STATE_HOME`。<br>2. 执行 `codex`。<br>3. 输入 `/hooks`，确认用户级 CC Traffic Light command 已 Trust 且 Active，项目级开发 hook 保持 Disabled。<br>4. 执行只读任务：`请只读取当前目录文件列表，不要修改文件`。<br>5. 在同一 PowerShell 执行 `& $hook list`。 | state 文件出现 `codex_<session_id>`；运行时为 `working`，完成后为 `done`；且 Codex UI 不再报告 CC Traffic Light hook code 1。2026-07-13：默认 launcher 的 `codex exec` 只读 sandbox probe 已通过，但该模式不触发生命周期 hooks，不能代替本项。release hook 成功时无 stdout，定向测试 4/4、直接 payload 回放 exit 0 且无 stdout/stderr。 | 若新会话仍报告 code 1，回填对应 event、state 文件和 `$env:USERPROFILE\.codex\.sandbox\sandbox.<date>.log` 末尾内容；不要重新启用项目级开发 hooks。 | PASS（2026-07-13）：真实会话中 SessionStart、UserPromptSubmit、PreToolUse、PostToolUse、Stop 全部成功；真实 session `019f5a5a-cee5-7462-ba3a-78ce6171141f` 最终 `Stop → done`。修复为 Windows `cmd.exe` + `.cmd` wrapper 转发安装 hook EXE。 |
| AUD-2-11 | Codex trust 后状态分类 | **仍使用：** `$evidence\codex-project`、`$env:USERPROFILE\.codex\hooks.json`、`$evidence\state\state.json`。在 `/hooks` 中确认每个新/变更 command 已 Trust 后，点击设置页“立即刷新检测”，并执行 `& $hook list`。 | 设置页 Codex 状态从 `ConfiguredUnverified` 变为 `Active`；若未变更，记录 `/hooks` 显示的 source/review 状态和该 state 文件内容。 | 不要把 hooks.json 存在单独当作 Active；不要用 bypass trust 代替 Trust。 | |
| AUD-2-12 | state 写入失败诊断 | **正常 state：** `$evidence\state\state.json`。**失败 fixture：** `$evidence\state-root-file`，它是普通文件而不是目录。<br>1. 执行 `$badStateRoot = Join-Path $evidence 'state-root-file'; Set-Content $badStateRoot 'not-a-directory'`。<br>2. 临时设置 `$oldStateHome=$env:TASKBAR_WIDGET_STATE_HOME; $env:TASKBAR_WIDGET_STATE_HOME=$badStateRoot`。<br>3. 执行 `'{"session_id":"write-failure"}' | & $hook codex PreToolUse 2>&1`。<br>4. 恢复 `$env:TASKBAR_WIDGET_STATE_HOME=$oldStateHome`。 | 退出码非 0；stderr 明确说明无法创建/写入 `$badStateRoot\state.json`；正常 state 和真实 `%APPDATA%\CcTrafficLight\state.json` 均未被修改。 | 不要修改真实 `%APPDATA%\CcTrafficLight` 的 ACL。 | PASS（2026-07-13）：正式安装 hook exit code 1；stderr：`当文件已存在时，无法创建该文件。 (os error 183)`；fixture 未变。证据：`C:\Users\admin\AppData\Local\Temp\cc-traffic-light-audit-2-12-20260713-142630`。 |
| AUD-2-GATE | Codex 全链路 | 汇总安装后的 hook 配置、trust、真实事件、state、设置页状态和 widget 截图。 | 安装 → trust → 事件 → state → detector → UI 全部有证据。 | 任一环节只有间接单测而没有真实证据时保持未通过。 | |
| AUD-3-01 | Claude 测试环境固定 | **工作目录：** `$evidence\claude-project`。先执行 `New-Item -ItemType Directory "$evidence\claude-project" -Force` 和 `Set-Location "$evidence\claude-project"`。**配置文件：** `$env:USERPROFILE\.claude\settings.json`。执行 `claude --version`、`$PSVersionTable`、`git --version`、`Get-Command bash`。 | 记录版本、启动终端、当前工作目录、Git Bash/PowerShell、配置层级。 | 版本或配置层级不固定时不比较不同 form 的结果。 | |
| AUD-3-02 / 03 | Claude marker 和 dump-only | **wrapper：** `D:\project\cc-traffic-light\taskbar-widget\scripts\claude-lifecycle-hook-dump.ps1`。**配置文件：** `$env:USERPROFILE\.claude\settings.json`。将单个 event 配置为该 wrapper，先只测 `UserPromptSubmit`，再测 `PreToolUse`。查看 `$env:TEMP\cc-traffic-light-claude-hooks\claude-hook-*.json`。 | dump 文件生成；记录 dump 的绝对路径、`stdin_present`、`stdin_is_json`、top-level keys、session/event candidate paths。 | 同一配置连续两次无 dump 后停止，不继续盲改路径。 | |
| AUD-3-04 | Claude shell form | 按官方 shell-form 规则配置一个 event，使用绝对路径 wrapper；启动 Claude 并触发该 event。 | 记录是否执行、stdin 是否完整、退出码和 Claude UI 错误。 | 同一 shell form 连续失败两次停止。 | |
| AUD-3-05 | Claude exec form | 使用 exec form：command 只指向真实 executable，参数放 `args`；只测试一个 event。 | 记录是否生成 dump；确认没有把参数拼进 executable path。 | 连续两次静默不执行就停止。 | |
| AUD-3-06 | Claude PowerShell shell | 设置 `shell = "powershell"`，重复同一个最小 event。 | 与 Git Bash shell form 对比 stdin、路径和退出码。 | 不再同时修改 shell、路径和 payload。 | |
| AUD-3-07 / 08 | Claude 事件与 payload | 在已证明可触发的 form 下，逐个测试 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Notification`、`Stop`；保存 shape-only 结果。 | 记录 event name、session id、event order、编码和缺失字段。 | 不要求在失败 form 上继续扩展全部 event。 | |
| AUD-3-09 / 10 | Claude 支持等级 | 根据前面结果判断 command hook 是否稳定；同一原因失败两次后停止。 | 若仍不稳定，明确回填 `DegradedProcessOnly`，并保留失败证据。 | 不得用历史 investigation 的一次成功替代当前证据。 | |
| AUD-3-12 / 13 / GATE | Claude 产品决策 | 确认是否需要 trust/review、是否允许用户级 settings.json 部署；更新 Claude 示例和支持矩阵。 | 最终支持等级为 Active、实验性或 DegradedProcessOnly，且 UI/README 一致。 | 若需要 HTTP 高置信度接入，另行立项，不在本表继续扩展。 | |
| AUD-5-01 | Widget Idle | **工作目录：** `$evidence`。**state 文件：** `$evidence\state\state.json`。执行 `Remove-Item "$evidence\state\state.json" -Force -ErrorAction SilentlyContinue`，然后在同一 PowerShell 执行 `& $widget`。 | widget 可见；整体 Idle；runtime log 为 `$evidence\runtime.log`，且来自当前 release host。 | 不要用旧截图或 `$env:APPDATA\CcTrafficLight\state.json` 证明本轮结果。 | |
| AUD-5-02 | Working 绿灯 | `& $hook set codex_visual working`；等待至少两个轮询周期。 | 左侧绿灯慢闪；保存 `working.png`、state 和 runtime log。 | 若 widget 不可见，转到 AUD-5-08/09，不连续重启。 | |
| AUD-5-03 | Waiting 黄灯 | `& $hook set codex_visual waiting`。 | 中间黄灯快闪；保存截图和 state。 | 同上。 | |
| AUD-5-04 | Error 红灯 | `& $hook set codex_visual error`。 | 右侧红灯慢闪；保存截图和 state。 | 同上。 | |
| AUD-5-05 | Done 完成态 | `& $hook set codex_visual done`，等待 Done retention TTL 后再次观察。 | 完成态常亮，随后回落 Idle；记录时间。 | 不要把单次截图当作 TTL 证据。 | |
| AUD-5-06 | Codex + Claude 同时运行 | `& $hook set codex_dual working`；`& $hook set claude_dual waiting`；执行 `& $hook list`。 | 两个 source 不互相覆盖；overall priority 正确；设置页显示 canonical key。 | 发现 source key 混乱时停止并保留 state。 | |
| AUD-5-07 | Reduced motion | 在设置页开启 Reduced Motion；重复 Working/Waiting/Error；再关闭设置。 | 状态颜色正确，动画按配置减少或关闭。 | 若设置页无法启动，记录 Tauri/fallback 证据。 | |
| AUD-5-08 | Attach failure / tray-only | 执行 `diagnose-taskbar-loop.ps1 -SkipBuild -Parents shell -Anchors tray_notify -CoordModes rect_delta`；保存诊断目录。 | attach 失败有 retry、tray-only 或明确 runtime log。 | 到达 retry window 后停止，不连续重启 Explorer。 | |
| AUD-5-09 / 14 | Explorer/taskbar recovery | 保存当前截图；在任务管理器中 Restart Windows Explorer；等待任务栏恢复。 | widget 自动恢复，或明确进入 retrying/tray-only；保存前后截图和 log。 | 若任务栏无法恢复，停止并保留系统状态。 | |
| AUD-5-10 / GATE / VAL-13 | 最终桌面证据 | 汇总 Idle、Working、Waiting、Error、Done、双 Agent、reduced motion 的截图/录屏、state 和 log。 | 每个用户可见状态都有同一版本的桌面证据。 | 不用单元测试替代桌面截图。 | |
| AUD-6-05 | Tauri settings 生命周期 | **host：** `D:\project\cc-traffic-light\target\release\taskbar-widget.exe`。**settings：** `D:\project\cc-traffic-light\target\release\taskbar-settings-tauri.exe`。**report：** `D:\project\cc-traffic-light\taskbar-widget\target\validate-tauri-settings-lifecycle\report.json`。<br>1. 从该 release host 托盘打开 settings。<br>2. 重复打开确认复用同一 settings PID。<br>3. 点击 settings 右上角关闭。<br>4. 检查该绝对路径的 settings 进程是否退出。<br>5. 再次打开。<br>6. 强制结束该 PID 后再次打开。 | spawn、reuse、close、reopen、kill-recover 全部通过；记录 PID、截图、report 和 runtime log。 | 当前 verifier 已在 close 阶段超时；若手工仍不退出，不强杀后伪造 PASS。 | |
| AUD-6-GATE | Settings / fallback UX | 分别验证 Tauri 正常、Tauri 不可用、Win32 fallback、hook 部署失败、立即刷新失败。 | 用户能看到状态、下一步和失败原因；日志可定位。 | 任一失败只写 log 而无用户提示时保持未通过。 | |
| AUD-1-06 / 07 | 安装后文件与自启动 | 在安装器生成后安装；检查安装目录、三个 exe、scripts、快捷方式和 Run 注册表。 | 文件完整；`HKCU\Software\Microsoft\Windows\CurrentVersion\Run\CcTrafficLight` 指向当前 exe。 | 不使用旧 `dist\installer` 作为证据。 | |
| AUD-1-09 / 10 | 升级与卸载 | 先备份用户 hooks；安装旧版/当前版升级；再卸载；检查 hooks、backup、restore、Run 项和残留文件。 | 用户 hooks 保留；升级路径不变；卸载清理符合预期。 | 不在真实用户配置上做不可逆删除；优先使用临时 Windows 用户。 | |
| AUD-7-01 / 02 | 清洁安装记录 | 使用无既有 Agent 配置的临时 Windows 用户安装；记录安装目录、文件、注册表、配置和 state 路径。 | 形成完整 `clean-install` 证据目录。 | 缺少干净用户环境时标记 BLOCKED，不用当前用户环境冒充。 | |
| AUD-7-03 / 04 / 05 | 单 Agent 与双 Agent | 分别只启用 Codex、只启用 Claude、同时启用两者；重复真实事件和 widget 状态验证。 | Codex 闭环成立；Claude 文案符合实际支持等级；两者状态不覆盖。 | 不把 Claude 进程出现当作 hook Active。 | |
| AUD-7-06 / 07 / 08 | 异常、升级、卸载 | 在临时用户中验证已有 hooks、非法 JSON、PowerShell 拒绝、未 trust、state 不可写、升级和卸载 restore。 | 每条异常都有用户文案或定位日志；用户配置不静默丢失。 | 失败后停止，不继续修改临时配置以“修到通过”。 | |
| AUD-7-09 / GATE | 最终 release gate | 汇总安装器、Codex、Claude、Widget、Settings、异常路径和 cleanup 结果。 | 所有 gate 有 PASS 或明确 BLOCKED/Accepted limitation；生成最终报告。 | 任何关键项缺少原始证据时不宣布完成。 | |
| AUD-PRE-04 / 0-GATE | 环境与基线回填 | 回填 Codex CLI 可用性、Claude 版本、终端、Git Bash、PowerShell、Rust，以及 Phase 0 evidence matrix。 | 版本和证据链完整，不再混用历史文档结论。 | 工具不可用时记录原因，不伪造版本。 | |
| AUD-VAL-01 | Rust 格式检查 | 在确认工作区变更边界后执行 `cargo fmt --all -- --check`；若有用户未完成漂移，不覆盖文件。 | 通过，或记录明确的既有漂移文件列表。 | 不执行 `cargo fmt` 覆盖用户未完成改动。 | |
| AUD-VAL-15 | 失败路径审计 | 逐项检查 hook CLI、state、安装脚本、Tauri pipe、Tauri spawn、fallback、taskbar attach 的用户文案或 runtime log。 | 每条失败路径至少有一个可定位证据。 | 发现静默失败时回填 FAIL，并附日志。 | |
| AUD-DOC-06 | 原始证据归档 | 将命令输出、截图、runtime log、state JSON、安装/卸载记录集中到一个证据目录；脱敏后记录路径。 | 证据目录可复查，路径写入本表和最终报告。 | 不上传 secrets 或完整敏感 payload。 | |
| AUD-CLN-01 / 02 | 清理验证 | 验收结束后删除临时 state、marker、dump、实验 endpoint；检查 git/status 和证据目录是否误入源码。 | 无临时 secrets、debug artifact、真实用户路径或无意配置。 | 不删除用户原有配置；先确认备份。 | |
| AUD-CLN-04 | 最终诊断代码/链接清理 | 搜索未使用诊断代码、重复配置、失效链接；运行 tests/type-check。 | 无已知未使用项和失效链接；回归仍通过。 | 不删除仍被 checklist 或 verifier 使用的脚本。 | |

## 回填完成后的处理

填写完成后，将本文件保存并告知 Codex。下一轮将：

1. 逐行解析“回填结果”；
2. 检查证据路径是否存在、是否与项目/版本匹配；
3. 将 PASS 项同步到 `docs/checklist/end-to-end-install-monitoring-audit.md`；
4. 对 FAIL/BLOCKED 项补充原因和下一步；
5. 重新判断各 Phase gate 和最终 completion criteria。
