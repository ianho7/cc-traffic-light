# Codex Notify 探针方案

日期：2026-07-01

## 目标

确认当前 Codex `notify` 入口实际传给命令的输入形态，判断它是否能作为 hook 状态来源。

本探针只回答这些问题：

- Codex 是否给 notify wrapper 传递 argv。
- Codex 是否给 notify wrapper 传递 stdin。
- stdin 是否为 JSON。
- 输入 shape 中是否存在 session、thread、turn 或 event order 相关字段。
- wrapper 是否能在记录脱敏 shape 后继续转发原始 notify 命令。

## 非目标

- 不自动修改 `C:\Users\admin\.codex\config.toml`。
- 不替换或吞掉原 notify。
- 不保存完整 payload、prompt、代码、命令参数或完整路径。
- 不把 notify 直接作为 task 状态主来源，除非它携带足够 task 上下文。

## 当前前提

当前可见 Codex 配置形态是 `notify = [ ... , "turn-ended" ]`，不是已确认的多事件 hooks TOML 表。

该文档只评估 `notify` 入口，不评估 Codex lifecycle hooks。`notify` 的低保真实测结果不能上升为“Codex hooks 无法获取状态”。

官方 Codex lifecycle hooks 是独立机制，支持 `hooks.json` 和 inline `[hooks]`，command hook 从 stdin 接收 JSON。Codex 多任务状态主路径应继续验证 lifecycle hooks，而不是继续依赖 `notify`。

## Wrapper 记录格式

wrapper 只能记录 shape，不记录原值：

```json
{
  "captured_at": "<timestamp>",
  "argv": {
    "count": 2,
    "items": [
      { "index": 0, "kind": "string", "value": "<redacted>" },
      { "index": 1, "kind": "string", "value": "<redacted>" }
    ]
  },
  "stdin": {
    "present": true,
    "encoding": "utf8_or_utf16",
    "is_json": true,
    "shape": {
      "session_id": { "type": "string", "value": "<redacted>" }
    },
    "candidate_paths": {
      "session": ["$.session_id"],
      "thread": [],
      "turn": [],
      "event_order": []
    }
  },
  "forward": {
    "attempted": true,
    "exit_code": 0
  }
}
```

## 转发规则

- wrapper 必须逐字保留原 notify 命令路径和参数。
- wrapper 记录失败时，仍应尝试执行原 notify。
- 原 notify 的 stdout/stderr/exit code 不应被静默吞掉。
- wrapper 的探针输出应写到用户明确指定的临时目录。
- 如果原 notify 失败，记录 `forward.exit_code`，但不要把完整参数写入日志。

## 人工步骤

1. 备份当前 Codex 配置。
2. 复制当前 `notify` 的原命令路径和参数。
3. 构造一个临时 wrapper，让 wrapper 先记录 shape，再调用原命令。
4. 临时把 Codex `notify` 指到 wrapper。
5. 触发一次 Codex turn-ended。
6. 检查 wrapper 输出是否包含 argv/stdin/JSON shape。
7. 恢复原 Codex 配置。

本步骤需要用户确认后再执行；当前仓库不会自动修改外部配置。

仓库内已提供可审阅 wrapper：

```text
D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-wrapper.ps1
```

仓库内也提供配置辅助脚本：

```text
D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-config.ps1
```

该脚本默认 dry-run，只读取当前 Codex config，输出脱敏摘要，不写入文件：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-config.ps1
```

如需实际安装探针，必须用户确认后显式加 `-Apply`。恢复也必须显式加 `-Restore -Apply`：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-config.ps1 -Apply
powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-config.ps1 -Restore -Apply
```

调用形式：

```powershell
$forwardJson = '["<original-notify-command>","<original-notify-arg-1>"]'
$forwardJsonBase64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($forwardJson))
powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:\project\cc-traffic-light\taskbar-widget\scripts\codex-notify-probe-wrapper.ps1 -OutDir <probe-output-dir> -ForwardJsonBase64 $forwardJsonBase64
```

`-ForwardJsonBase64` 解码后是原 notify 命令数组，必须按当前配置逐字填写。wrapper 只在输出中记录参数 shape，不记录原值。

## 判定规则

- 如果 notify 输入含稳定 `session_id`、thread id 或 turn id，可继续评估是否作为 Codex 低保真状态来源。
- 如果 notify 只表达 `turn-ended`，且没有 task/session 上下文，只能记录为低保真来源，不进入 task 状态主路径。
- 如果发现正式多事件 hooks 配置格式，再单独更新 Codex hook 接入计划。

## Wrapper 自测记录

2026-07-01 已完成仓库内 wrapper 自测，未修改外部 Codex 配置。

自测方式：

- 使用临时 `%TEMP%` 输出目录。
- stdin 传入人工 JSON：包含 `session_id`、`thread_id`、`turn_id`、`timestamp`。
- fake 原 notify 使用 `cmd.exe /C echo forwarded`。

自测结果：

- wrapper 生成 `codex-notify-probe-<timestamp>-<pid>.json`。
- `stdin.is_json = true`。
- `candidate_paths.session` 包含 `$.session_id`。
- payload shape 中字段值为 `<redacted>`。
- 原 notify 参数只记录 count 和 `<redacted>` item，不记录原值。
- forward 被执行，stdout 转发为 `forwarded`，exit code 为 `0`。

真实 Codex `notify` 探针已执行，结果见下方“真实 Codex notify 观察记录”。

## Config 辅助脚本自测记录

2026-07-01 已完成 `codex-notify-probe-config.ps1` 自测：

- 临时 config dry-run 通过，未泄露原 notify 值。
- 临时 config `-Apply` 通过，生成备份并写入 wrapper notify。
- 临时 config `-Restore -Apply` 通过，恢复原 notify。
- 真实 `C:\Users\admin\.codex\config.toml` dry-run 通过，检测到当前 `notify` 有 2 项，未写入真实配置。

真实配置后续已由用户执行 `-Apply` 并触发一次 Codex turn-ended，结果见下方观察记录。测试完成后应执行 `-Restore -Apply` 恢复原配置。

## 真实 Codex notify 观察记录

2026-07-01 用户提供了一次真实 Codex turn-ended 探针输出，记录仅保留脱敏 shape 和判定结论。

实际观察：

- `argv.forward_json_base64_present = true`。
- `stdin.present = false`。
- `stdin.encoding = empty`。
- `stdin.byte_length = 0`。
- `stdin.is_json = false`。
- `stdin.candidate_paths.session = []`。
- `stdin.candidate_paths.thread = []`。
- `stdin.candidate_paths.turn = []`。
- `stdin.candidate_paths.event_order = []`。
- `original_notify.args.count = 1`，参数值已脱敏。
- `original_notify.forward_token_count = 2`。
- `forward.attempted = true`。
- `forward.exit_code = 1`。

判定：

- 当前可见 Codex `notify` 没有向 wrapper 提供 stdin。
- 当前可见 Codex `notify` 没有提供可用于 task identity 的 `session_id`、thread id、turn id 或 event order。
- 该来源只能作为低保真 turn-ended 通知，不进入 task 状态主路径。
- 该结论只适用于 `notify`，不适用于正式 Codex lifecycle hooks。
- 原 notify 已被 wrapper 尝试转发，但原 notify 返回 `exit_code = 1`，需要在真实接入前单独排查或避免依赖该路径的成功退出码。
- 测试结束后必须恢复 Codex 原始 `notify` 配置，避免 wrapper 长期驻留在用户配置中。

## 停机条件

- 用户未确认临时修改外部 Codex 配置。
- 无法保证原 notify 被转发。
- wrapper 输出可能包含完整 prompt、代码、命令参数或完整路径。
- notify 不提供 session/thread/turn 信息，且继续推断会污染 task 状态。
