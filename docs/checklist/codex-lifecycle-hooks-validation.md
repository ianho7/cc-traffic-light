# Codex Lifecycle Hooks 验证 Checklist

日期：2026-07-01

## 目标

验证本机 Codex 是否支持并加载正式 lifecycle hooks，确认 command hook stdin 是否能提供多任务状态所需的 identity。

本 checklist 修正一个关键边界：`notify` 实测为空 stdin，只证明 `notify` 不适合做主状态路径；不证明 Codex lifecycle hooks 拿不到状态。

## 官方依据

OpenAI Codex Hooks 文档说明：

- hooks 默认启用，可通过 `[features].hooks = false` 关闭。
- Codex 会从 `hooks.json` 或 `config.toml` inline `[hooks]` 加载 lifecycle hooks。
- 常用位置包括 `~/.codex/hooks.json`、`~/.codex/config.toml`、项目 `.codex/hooks.json`、项目 `.codex/config.toml`。
- 当前真正运行的是 `type = "command"` handler。
- command hook 会从 stdin 接收 JSON。
- Windows 可使用 `commandWindows`；TOML 中可用 `command_windows` 或 `commandWindows`。
- 非 managed command hook 需要通过 `/hooks` review/trust 后才会运行。

Source: https://developers.openai.com/codex/hooks

## 验证范围

需要确认的字段：

- `session_id`：主任务 identity。
- `hook_event_name`：状态映射依据。
- `cwd`：展示分组和排障辅助。
- `model`：诊断辅助。
- `turn_id`：turn scope 辅助 identity，不能替代 session 主键。

不做的事：

- 不保存完整 prompt、代码、命令参数或完整路径。
- 不把 `notify` 重新提升为主状态来源。
- 不自动修改用户级 Codex 配置。
- 不引入 daemon、HTTP server、IPC 或进程检测。

## 建议验证步骤

1. 确认 Codex hooks 没被禁用：

```toml
[features]
hooks = true
```

如果没有显式配置，按官方文档默认启用处理。

2. 使用最小 dump hook，先只记录 shape。

推荐先用用户级或项目级 `hooks.json`，不要和 inline `[hooks]` 同层混用，避免启动警告。

3. Windows 下 command 使用 `commandWindows` 或 TOML `command_windows`。

4. 启动 Codex 后用 `/hooks` 检查 hook source、review 状态和 trust 状态。

5. 触发最小事件集：

- `SessionStart`
- `UserPromptSubmit`
- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `SubagentStop`
- `Stop`

6. 检查 dump 输出中是否有：

- `session_id`
- `hook_event_name`
- `cwd`
- `model`
- turn scope 事件中的 `turn_id`

7. 如果字段存在，再把 command 指向：

```text
D:\project\cc-traffic-light\taskbar-widget\target\debug\taskbar_widget_hook.exe codex <HookName>
```

8. 再用 `taskbar_widget_hook.exe list` 检查 `codex_<session_id>` 是否按事件更新。

## 当前仓库已准备的测试文件

2026-07-01 已新增项目级测试配置，不修改用户级 `C:\Users\admin\.codex\config.toml`：

- `.codex/hooks.json`
- `taskbar-widget/scripts/codex-lifecycle-hook-dump.ps1`

`.codex/hooks.json` 配置了这些事件的 shape-only dump：

- `SessionStart`
- `UserPromptSubmit`
- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `SubagentStop`
- `Stop`

dump 输出默认写入：

```text
%TEMP%\cc-traffic-light-codex-lifecycle-hooks
```

本地人工 JSON 自测已通过：

- stdin 可解析为 JSON。
- `session_id`、`turn_id`、`hook_event_name`、`cwd`、`model` candidate paths 可识别。
- 字段值只保存为 `<redacted>`。
- 未保存完整 prompt、代码、命令参数或完整路径。

## 真实 Codex Lifecycle Hooks 观察记录

2026-07-01 用户提供了真实 Codex lifecycle hooks dump。结论：验证通过。

观察到的事件：

- `PreToolUse`
- `PostToolUse`
- `Stop`

共同事实：

- `stdin.present = true`
- `stdin.is_json = true`
- `stdin.parse_error = null`
- `stdin.candidate_paths.session = ["$.session_id"]`
- `stdin.candidate_paths.turn = ["$.turn_id"]`
- `stdin.candidate_paths.hook_event = ["$.hook_event_name"]`
- `stdin.candidate_paths.cwd = ["$.cwd"]`
- `stdin.candidate_paths.model = ["$.model"]`
- `stdin.candidate_paths.event_order = []`

字段 shape：

- `session_id`: string
- `turn_id`: string
- `transcript_path`: string
- `cwd`: string
- `hook_event_name`: string
- `model`: string
- `permission_mode`: string
- `tool_name`: string, observed in tool events
- `tool_input.command`: string, observed in tool events
- `tool_response`: string, observed in `PostToolUse`
- `tool_use_id`: string, observed in tool events
- `stop_hook_active`: boolean, observed in `Stop`
- `last_assistant_message`: string, observed in `Stop`

判定：

- Codex lifecycle hooks 可作为 Codex 主状态来源。
- `session_id` 可作为多任务主键，形成 `codex_<session_id>`。
- `turn_id` 可作为 turn 级辅助 identity，当前状态 schema 可先不保存，后续需要更细粒度排障时再扩展。
- 当前 payload 未观察到独立 event order 字段；MVP 可继续使用 hook CLI 的 `received_at` 作为排序兜底。
- `notify` 继续保持低保真兼容入口结论，不进入主状态路径。

## 本机 Codex 实测步骤

1. 重启当前项目的 Codex 会话，或新开一个指向 `D:\project\cc-traffic-light` 的 Codex 会话，让 Codex 重新加载项目 `.codex/hooks.json`。

2. 在 Codex 输入：

```text
/hooks
```

检查 `.codex/hooks.json` 是否出现，并按 Codex 提示 review/trust 这些 command hooks。

3. 触发一个简单任务，例如：

```text
请回复 ok，并运行一个最小只读命令检查当前目录。
```

4. 在 PowerShell 查看是否有真实 hook dump：

```powershell
Get-ChildItem $env:TEMP\cc-traffic-light-codex-lifecycle-hooks -Filter "codex-lifecycle-hook-*.json" |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 5 |
  Get-Content
```

5. 重点检查：

- `stdin.present`
- `stdin.is_json`
- `stdin.candidate_paths.session`
- `stdin.candidate_paths.turn`
- `stdin.candidate_paths.hook_event`
- `stdin.candidate_paths.cwd`
- `stdin.candidate_paths.model`

6. 如果真实 dump 含稳定 `session_id`，再把项目 hook command 从 dump 脚本替换为 `taskbar_widget_hook.exe codex <HookName>`，进入主状态写入验证。

## 判定

- Pass: lifecycle hook 触发，stdin 是 JSON，且包含稳定 `session_id`。（2026-07-01 已通过）
- Partial: lifecycle hook 触发，但缺少 `turn_id`；可以先按 session 维度接入，turn 只做缺省。
- Fail: hook 未触发；优先排查版本、配置位置、`[features].hooks`、项目 trust 和 `/hooks` trust。
- Reject as main path: 只有 `notify` 触发，且没有 session identity。

## 状态映射建议

- `SessionStart` -> `idle` 或诊断事件；MVP 可暂不更新主状态。
- `UserPromptSubmit` -> `working`
- `PreToolUse` -> `working`
- `PermissionRequest` -> `waiting`
- `PostToolUse` 成功 -> `working`
- `PostToolUse` 失败 -> `error` 或诊断 warning，等待真实 payload shape 后决定。
- `SubagentStop` -> `done`
- `Stop` -> `done`；如果 previous state 是 `waiting`，保守保持 `waiting`。

## 后续实现门槛

只有 lifecycle hooks 验证通过后，才把 Codex 正式接入主状态路径。否则 Codex 只能保留 debug CLI 和 notify 低保真记录。
