# Codex hook `exited with code 1`：当前卡点与证据报告（2026-07-13）

## 结论先行

截至本报告，不能把当前 Codex 用户级 command hook 标记为“开箱即用”。

已证实 Codex 在至少一次真实会话中把 lifecycle payload 交给了 CC Traffic Light 的 hook，并写出了真实 `session_id` 的 `Stop -> done` state；但后续多个会话持续显示 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop` hook failed / `exited with code 1`，且未产生新的 state 或诊断日志。

**根因尚未被证明。** 当前最强的外部阻塞是 Codex CLI 自身的 Windows sandbox helper 启动失败：同一会话的普通只读工具也报 `codex-windows-sandbox-setup.exe` `program not found` 或 `Access denied`。这使得无法把 hook 的 code 1 与 sandbox 启动失败可靠分离。

本次应停止继续改 hook 实现或反复调整配置，先修复/重装 Codex CLI 的 Windows sandbox 运行环境，或获得 Codex 对 hook 子进程 stderr 与退出码的可观测性。

## 用户最初目标与当前判定

目标是“用户安装后无需额外手工配置，即可检测 Claude Code/Codex hook，并在任务栏 widget 上显示状态/闪烁”。

| 能力 | 当前证据 | 判定 |
|---|---|---|
| 用户级 `hooks.json` 可解析 | UTF-8 BOM 问题已修复 | 通过 |
| 用户级 release hook 路径存在 | `%LOCALAPPDATA%\Programs\CC Traffic Light\taskbar_widget_hook.exe` 存在，`--version` 成功 | 通过 |
| `/hooks` trust | 用户级 7 条 hook 的 `trusted_hash` 已写入 `%USERPROFILE%\.codex\config.toml` | 通过 |
| 真实 Codex payload/state 曾经到达 | 真实 `state.json` 中有 `codex_019f59b6-...`，`session_id_source=payload`，`hook_name=Stop`，`state=done` | 部分通过 |
| 真实 hook 无错误完成 | 最新会话仍显示 `hook exited with code 1` | **未通过** |
| 默认 state 路径稳定写入 | `%APPDATA%\CcTrafficLight\state.json` 未被本轮更新；固定 `%TEMP%` fixture 也未生成 state | **未通过** |
| Codex CLI 自身 shell 正常 | 同一会话报 sandbox helper `program not found`，偶发重试后成功 | **未通过，外部阻塞** |
| 任务栏 widget 闪烁验收 | 尚未在无错误的真实 Codex state 链路上完成 | 未通过 |

## 已做过的排查与结果

### 1. 配置解析与 executable 路径

- 早期 `hooks.json` 有 UTF-8 BOM，Codex 报 `expected value at line 1 column 1`；已修复安装脚本为 UTF-8 no-BOM，真实用户配置已去除 BOM。
- 早期用户配置指向不存在的旧路径 `%LOCALAPPDATA%\CcTrafficLight\bin\taskbar_widget_hook.exe`；已更新为正式安装路径 `%LOCALAPPDATA%\Programs\CC Traffic Light\taskbar_widget_hook.exe`。
- 当前用户级 7 条 command 均指向存在的正式 exe，release exe 的 `--version` 成功。

结论：**当前 code 1 不是 BOM 解析失败，也不是已知的旧 exe 路径不存在。**

### 2. Trust 与重复配置

- 用户级 hook 的 trust hash 已存在。
- 项目级 `D:\project\cc-traffic-light\.codex\hooks.json` 也包含 7 条 debug hook。根据 Codex 的合并规则，它们与用户级 hook 会同时触发。
- 用户已在 `/hooks` 禁用项目级 7 条；`%USERPROFILE%\.codex\config.toml` 中对应项均为 `enabled = false`。
- 禁用后，用户级 hook 仍报 code 1。

结论：**重复项目级 hook 是配置噪声，但不是当前用户级 code 1 的充分解释。**

### 3. hook CLI 的直接回放

对正式 release `taskbar_widget_hook.exe` 直接提供 Codex 格式 JSON：

- 可正常退出 0；
- 可写入隔离 `state.json`；
- 可提取 payload 内 `session_id`；
- release hook 的定向 Rust 测试通过（4/4）。

结论：**CLI 的参数解析、JSON 输入解析、状态模型和原子写入在非 Codex hook runner 环境中可工作。** 这不是对真实 Codex 子进程权限/环境的证明。

### 4. stdout 假设（已被证伪为充分原因）

旧实现对 Codex 成功路径输出 `{}`。考虑到 state 已写入而 UI 仍 code 1，曾假设 stdout pipe 在 `println!` 时失败。

已将成功路径改为无 stdout，重新构建并覆盖正式安装 exe；新 exe 的 SHA-256 与 release 构建一致，直接回放得到 `exit 0`、无 stdout/stderr、state 写入成功。

随后真实 Codex 会话仍报 code 1，且没有 state。因此：

> “`{}` stdout 是 code 1 的唯一根因”已经被否定。

无 stdout 仍然符合官方协议，应保留为正确的 hook 输出约束；但不能再把它当作当前 blocker 的解法。

### 5. opt-in 诊断日志

为获取被 Codex UI 隐藏的 stderr，hook CLI 加入了仅在 `TASKBAR_HOOK_DIAGNOSTIC_LOG` 存在时才写入的诊断日志。直接调用时能记录：

```text
... args=[codex Stop] state_home=[<default>] start
... args=[codex Stop] state_home=[<default>] success stdout_bytes=0
```

但在真实 Codex 会话中，已显式设置：

```powershell
$env:TASKBAR_HOOK_DIAGNOSTIC_LOG = "D:\project\cc-traffic-light\.cc-traffic-light-hook-diagnostic.log"
```

会话结束后该文件不存在，同时固定 `%TEMP%` state 文件也不存在。

这只能说明以下至少一个情况成立，**不能单独证明是哪一个**：

1. Codex 未将会话环境变量传给 hook 子进程；
2. hook 子进程未在应用入口前成功启动；
3. hook 子进程启动了，但 sandbox 阻止其写入该诊断路径；
4. Codex hook runner 在执行/收集阶段以 code 1 失败。

## `code 1` 可能意味着什么？是否逐项考虑过？

考虑过。下表区分“可能性”与“证据状态”，避免把 Codex UI 的泛化提示误读成 Rust 程序的具体错误。

| 可能性 | 证据状态 | 结论 |
|---|---|---|
| hook 配置无法解析 | BOM 曾导致该问题，但当前 `/hooks` 可加载且 trust 已记录 | 已排除为当前原因 |
| command 路径不存在/命令行引号错误 | 当前正式 exe 存在，直接运行与 `--version` 成功 | 已基本排除 |
| 未 Trust / 被禁用 | 用户级 hook 有 `trusted_hash`；项目级重复 hook 已 disabled | 已基本排除 |
| 项目级重复 hook 导致一个失败 | 禁用项目级 hook 后仍失败 | 已排除为充分原因 |
| stdin 不是 JSON 或 payload 解析失败 | 直接 JSON 回放通过；真实 payload 曾成功写 state；但最新会话 stderr 不可见 | 仍可能，未证明 |
| state 目录权限/沙箱写入限制 | 默认 APPDATA 与固定 TEMP fixture 都无新 state；历史 workspace state 曾成功 | 高可能，未证明具体拒绝原因 |
| stdout `{}` 导致失败 | 已改为无 stdout，真实会话仍失败 | 已排除为充分原因 |
| hook exe 根本没有启动 | 真实会话中无 opt-in 诊断文件，但该文件也可能被 sandbox 拦截 | 仍可能，未证明 |
| Codex hook runner / sandbox 自己失败 | 同会话工具明确报 `codex-windows-sandbox-setup.exe` `program not found`/`Access denied` | 高可能的外部 blocker |
| 旧 binary 仍被调用 | 已比较正式安装 exe 与当前 release 构建的 SHA-256，一致 | 已排除 |

## 官方文档是否已被参考？

已参考官方 [Codex Hooks 文档](https://developers.openai.com/codex/hooks)。本轮采用的、并已验证与本机配置一致的规则包括：

- Codex 会从 `~/.codex/hooks.json`、`~/.codex/config.toml`、项目 `.codex/hooks.json` 和项目 `.codex/config.toml` 发现 hook；多个来源的匹配 hook **都会运行**，不会互相覆盖。这个规则解释了用户级 release hook 与项目级 debug hook 曾经同时触发。 
- 非 managed command hook 必须经过 review/trust；trust 与当前 hook 定义 hash 绑定。 
- 每个 command hook 从 stdin 接收一个 JSON 对象，其中含 `session_id`、`cwd`、`hook_event_name` 等字段。 
- `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop` 都是正式 lifecycle event。 
- **退出码 0 且没有 stdout 被视为成功。** 因此将成功 hook 改为无 stdout 的方向符合官方协议。

官方文档没有提供以下内容：

1. 将 UI 中笼统的 `hook exited with code 1` 映射为具体的 Rust/Windows 错误；
2. 如何取得被 Codex hook runner 隐藏的 stderr；
3. command hook 是否继承所有 PowerShell 环境变量；
4. command hook 写入 `%APPDATA%`、`%TEMP%` 时与 Windows sandbox 的 ACL/隔离边界；
5. `codex-windows-sandbox-setup.exe` `program not found` 的修复步骤。

因此，官方文档足以确认配置、trust、stdin/stdout 协议与多来源合并规则；**不足以单独诊断这台机器上的 Windows sandbox/helper 故障。**

## 当前真正的 blocker

当前 blocker 不是“还没找到正确的 hook JSON 语法”，而是：

```text
Codex CLI 0.144.1 的 Windows sandbox helper 在同一会话中无法稳定启动：
codex-windows-sandbox-setup.exe -> program not found / Access denied

同时，Codex hook runner 只显示 code 1，未暴露子进程 stderr；
因此无法确认 hook 是未启动、未继承环境、被 sandbox 拒绝写 state，还是由 runner 收集阶段失败。
```

这意味着目前没有足够证据承诺“普通用户安装后、无需处理 Codex sandbox 环境，就能稳定检测 Codex”。这一项必须保持未通过。

## 建议的停点与恢复条件

本报告后不应继续改动 hook 代码、hooks.json 或 installer，直到外部条件满足其一：

1. Codex CLI/桌面应用更新或重装后，`codex-windows-sandbox-setup.exe` 不再在普通 `Get-ChildItem` 上报 `program not found` 或 `Access denied`；或
2. 可以从 Codex 获取 command hook 子进程的原始 stderr/实际启动错误；或
3. 用户明确授权使用仅用于诊断的 Codex sandbox bypass 对照测试（该测试不能作为“开箱即用”验收证据）。

恢复后，最小验收应只做一次：在真实仓库启动新 Codex 会话，保留用户级 release hook、禁用项目级 debug hooks，运行一个只读任务，并捕获 state、hook stderr 和 sandbox log。

## 当前工作区/安装状态备注

- 本轮已经对 `taskbar-widget/src/bin/taskbar_widget_hook.rs` 做过两项诊断性改动：成功 hook 无 stdout；可选 `TASKBAR_HOOK_DIAGNOSTIC_LOG` 日志。
- 已构建 release hook 并复制到用户的正式安装路径；旧 exe 留有时间戳 backup。
- 本报告生成后不再进行代码、用户配置或安装目录修改。
- 这些诊断性改动尚未通过真实 Codex CLI 的无错误验收，不能宣称为最终产品修复。

## 后续诊断更新（2026-07-13）

`code 1` 的上游阻塞现已能精确定位到 Codex standalone launcher 的 sandbox helper 资源解析，而不是 CC Traffic Light hook 的 Rust 实现：

- `C:\Users\admin\AppData\Local\Programs\OpenAI\Codex\bin` 是指向 `~\.codex\packages\standalone\current\bin` 的 junction；其中 `codex.exe` 与 `0.144.2` release binary 哈希相同，但该 launcher 路径的父目录没有 `codex-resources`。
- `0.144.2` release 的正确资源位于 `~\.codex\packages\standalone\releases\0.144.2-x86_64-pc-windows-msvc\codex-resources\`，含 `codex-windows-sandbox-setup.exe` 和 `codex-command-runner.exe`。
- 从 launcher 路径运行的真实只读 shell probe 在 sandbox log 中尝试启动裸名称 `codex-windows-sandbox-setup.exe`，并复现 `program not found` 或 `拒绝访问 (os error 5)`。
- 从 release 的真实 `bin\codex.exe` 路径运行完全相同的 probe，则解析并启动绝对路径的 `codex-resources\codex-windows-sandbox-setup.exe`，`Get-Date` 成功。
- Microsoft Store 包内的同名 helper 对普通 PowerShell 进程也直接返回 `Access is denied`，因此它不能作为 standalone launcher 丢失资源定位时的可靠回退目标。

因此当前应修复/重装 Codex 的 standalone launcher 布局（使 launcher 与 `codex-resources` 同级可见，或让 launcher 在 junction/hardlink 下解析到 canonical release root），然后再做一次真实 hook 验收。不要继续修改 hook、hooks.json 或 state 写入逻辑。

## Sandbox 复验更新（2026-07-13）

用户将 `%USERPROFILE%\.codex\config.toml` 的 `[windows] sandbox` 调整为 `"unelevated"` 后，默认 launcher 路径下四次无写入 `Get-Date` sandbox probe 均成功，且没有新的 helper 启动失败记录。

为区分“根因已修复”和“配置绕过”，对单次 probe 仅以命令行覆盖 `-c 'windows.sandbox="elevated"'`（未写入 config）进行复验；该 probe 立即复现：

```text
windows sandbox: orchestrator_helper_launch_failed
setup refresh failed to launch helper: helper=codex-windows-sandbox-setup.exe
error=拒绝访问。 (os error 5)
```

结论：`unelevated` 是当前有效的运行时绕过，允许 sandboxed shell 正常执行；elevated sandbox helper 的启动问题仍然存在，尚未被修复。后续真实 Codex hook 验收应在保持 `unelevated` 配置的前提下进行，并将其作为部署前提而非“helper 已恢复”的证据。

## 真实 hook 复现更新（2026-07-13 14:31）

在保持 `unelevated`、用户级 7 条 hook 已 trust、项目级 7 条 debug hook 均 disabled 的条件下，交互式 Codex 会话完成一次只读目录列表。普通工具调用成功，但 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop` 均显示 `hook exited with code 1`；隔离 state 目录和默认 `%APPDATA%\CcTrafficLight\state.json` 都没有新写入。

这证明 `unelevated` 修复的是普通 sandboxed tool 命令，不足以证明 command hook runner 本身可用。正式安装 hook 与当前 release hook 的 SHA-256 一致。使用与用户 `hooks.json` 相同的、含空格绝对路径的 Windows command line，经 `cmd.exe` 直接运行正式 hook、输入真实形状的 `UserPromptSubmit` JSON 后得到 exit 0、state 写入成功，且诊断确认 stdout 为 0 字节。故已排除 release EXE、基本 command quoting、payload JSON 和成功 stdout 作为充分根因。

当前仍待区分的根因仅限于 Codex Windows hook runner：它可能在启动 hook 前失败，使用了不同的受限令牌/文件 ACL，或清理了传入环境。下一次真实交互复现必须在启动 Codex 前设置 `TASKBAR_HOOK_DIAGNOSTIC_LOG`；若日志存在，可取得 hook 内部失败信息；若日志缺失，则可将问题收敛到 runner 启动/环境层。官方 hooks 文档规定 command hook 的 stdin JSON、`commandWindows` 和 exit 0/no-output 成功，但未定义 Windows hook 子进程的 sandbox、环境继承或 code 1 的细分原因。

## 修复与最终真实验收（2026-07-13）

marker 诊断证明 runner 未进入 hook EXE；进一步对照表明用户级 `cmd.exe /d /s /c exit 0` hook 可以完成、直接 hook EXE 的 `--version` 仍失败、无空格路径的 `.cmd` wrapper 则可启动同一 EXE 并完成 hook。故根因是当前 Codex Windows hook runner 对直接本地 EXE 启动的限制/故障，而非 hook 逻辑、payload、stdout 或 state 权限。

正式修复写入 `install-codex-hooks.ps1`：部署 `commandWindows = cmd.exe /d /s /c call "<LOCALAPPDATA>\\CcTrafficLight\\codex-taskbar-widget-hook.cmd" codex <Event>`，wrapper 将 `%*` 转发给安装目录中的 `taskbar_widget_hook.exe`。安装脚本 clean-config fixture、wrapper stdin 转发和 release hook 定向测试均通过。临时 marker、诊断日志和 runner-control wrapper 已删除。

真实最终验收在用户 trust 全部 7 条正式用户级 hooks 后通过：`SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop` 均进入 hook 并成功退出；state 记录真实 session `019f5a5a-cee5-7462-ba3a-78ce6171141f`，最终为 `Stop -> done`。AUD-2-08 可标记完成；Settings 立即刷新/Active 与 widget 桌面闪烁仍需后续验收。
