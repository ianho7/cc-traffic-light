# Claude Code Hooks 接入精简版

> 来源：Claude Code 官方 Hooks 文档：<https://code.claude.com/docs/en/hooks>  
> 目标：只保留“接入 hook”真正需要知道的配置、事件、输入输出、调试与落地模板。

---

## 1. Hooks 是什么

Claude Code Hooks 是在 Claude Code 生命周期中的固定节点自动执行的处理器。

它可以用来：

- Claude 准备调用工具前拦截命令
- Claude 修改文件后自动格式化、测试、记录日志
- Claude 等待用户输入时发通知
- Claude 完成一轮回复时做收尾处理
- Claude 会话开始/结束时注入上下文或清理状态

一句话：

> Hooks 是 Claude Code 的“确定性自动化入口”，不是靠模型自觉执行，而是在指定事件触发时必定执行。

---

## 2. 配置文件位置

Hooks 写在 Claude Code 的 JSON settings 文件里。

| 位置 | 作用范围 | 是否适合提交到仓库 |
|---|---|---|
| `~/.claude/settings.json` | 当前用户的所有项目 | 不建议 |
| `.claude/settings.json` | 当前项目 | 可以提交 |
| `.claude/settings.local.json` | 当前项目本地配置 | 不提交，适合个人调试 |
| Managed policy settings | 组织级 | 管理员控制 |
| Plugin `hooks/hooks.json` | 插件启用时 | 插件分发 |
| Skill / Agent frontmatter | 组件激活时 | 随组件分发 |

推荐项目接入时使用：

```text
.claude/settings.json
```

个人本地实验时使用：

```text
.claude/settings.local.json
```

---

## 3. 最小配置结构

Hooks 配置是三层结构：

```json
{
  "hooks": {
    "事件名": [
      {
        "matcher": "匹配条件",
        "hooks": [
          {
            "type": "command",
            "command": "要执行的命令"
          }
        ]
      }
    ]
  }
}
```

对应关系：

```text
hooks
└── Event，例如 PreToolUse / PostToolUse / Notification / Stop
    └── matcher group，例如 Bash / Edit|Write / 空字符串
        └── hook handler，例如 command / http / prompt / agent / mcp_tool
```

---

## 4. 最常用事件

完整事件很多，实际接入一般先关注这些。

| 事件 | 触发时机 | 常见用途 |
|---|---|---|
| `SessionStart` | Claude Code 会话开始或恢复 | 注入环境信息、读取项目状态 |
| `UserPromptSubmit` | 用户提交 prompt 后，Claude 处理前 | 校验 prompt、记录输入、阻止某些请求 |
| `PreToolUse` | 工具调用前 | 拦截危险命令、权限控制、审计 |
| `PostToolUse` | 工具调用成功后 | 格式化、测试、记录文件变更 |
| `PostToolUseFailure` | 工具调用失败后 | 捕获失败、错误分析、日志记录 |
| `Notification` | Claude Code 发送通知时 | 桌面提醒、任务状态提示 |
| `Stop` | Claude 完成一轮响应时 | 回合结束通知、检查任务是否完成 |
| `StopFailure` | 回合因 API 错误结束 | 错误记录；输出和退出码会被忽略 |
| `SessionEnd` | 会话结束 | 清理资源、保存状态 |
| `FileChanged` | 被监听文件变化 | reload env、更新配置 |
| `CwdChanged` | 工作目录变化 | 切换环境、动态 watch 文件 |

---

## 5. matcher 怎么写

`matcher` 用来过滤哪些情况触发 hook。

### 5.1 匹配全部

```json
"matcher": ""
```

或：

```json
"matcher": "*"
```

或直接省略。

### 5.2 精确匹配工具

```json
"matcher": "Bash"
```

只匹配 Bash 工具。

### 5.3 匹配多个工具

```json
"matcher": "Edit|Write"
```

匹配 `Edit` 或 `Write`。

较新版本也可写成：

```json
"matcher": "Edit,Write"
```

### 5.4 正则匹配

只要 matcher 里包含普通字母、数字、`_`、`|` 之外的字符，就会按 JavaScript 正则处理。

例如：

```json
"matcher": "mcp__.*"
```

匹配所有 MCP 工具。

---

## 6. command hook

最常用的是 `type: "command"`。

```json
{
  "type": "command",
  "command": "node .claude/hooks/on-stop.js"
}
```

Claude Code 会：

1. 在事件触发时执行这个命令
2. 把事件上下文 JSON 通过 `stdin` 传给命令
3. 根据命令的 `stdout`、`stderr`、退出码判断结果

---

## 7. Hook 输入：stdin JSON

command hook 会从 `stdin` 收到 JSON。

常见公共字段：

| 字段 | 说明 |
|---|---|
| `session_id` | 当前 Claude Code 会话 ID |
| `transcript_path` | 当前会话 transcript JSONL 路径 |
| `cwd` | hook 执行时的工作目录 |
| `permission_mode` | 当前权限模式，部分事件才有 |
| `hook_event_name` | 当前触发的事件名 |
| `agent_id` | 子 agent 场景下的 agent ID，部分场景才有 |
| `agent_type` | agent 类型，部分场景才有 |

`PreToolUse` 示例输入：

```json
{
  "session_id": "abc123",
  "transcript_path": "/home/user/.claude/projects/.../transcript.jsonl",
  "cwd": "/home/user/my-project",
  "permission_mode": "default",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash",
  "tool_input": {
    "command": "npm test"
  }
}
```

不同事件会有额外字段，例如：

- `PreToolUse` / `PostToolUse`：`tool_name`、`tool_input`
- `Notification`：通知类型、消息内容
- `Stop`：最后一次 assistant message、后台任务信息等
- `SessionEnd`：`reason`
- `FileChanged`：`file_path`、`event`

> 注意：不是每个事件都会提供同样的字段，接入时必须做字段存在性判断。

---

## 8. Hook 输出：退出码与 stdout/stderr

### 8.1 退出码规则

| 退出码 | 含义 |
|---|---|
| `0` | 成功。Claude Code 会尝试解析 stdout 中的 JSON 输出 |
| `2` | 阻止/拦截，具体效果取决于事件 |
| 其他非 0 | 非阻塞错误，大多数情况下继续执行 |

### 8.2 stderr 的作用

如果 hook `exit 2`，Claude Code 会忽略 stdout，把 stderr 作为错误原因反馈给 Claude 或用户。

示例：拦截危险 Bash 命令。

```bash
#!/bin/bash
command=$(jq -r '.tool_input.command // empty' < /dev/stdin)

if [[ "$command" == rm* ]]; then
  echo "Blocked: rm commands are not allowed" >&2
  exit 2
fi

exit 0
```

---

## 9. JSON 输出结构

如果需要更细控制，可以 `exit 0`，并在 stdout 输出 JSON。

注意：

- stdout 必须只包含 JSON
- shell 启动时不要打印额外文本
- JSON 只会在 `exit 0` 时被解析
- 不要同时依赖 `exit 2` 和 JSON 决策

通用字段：

```json
{
  "continue": true,
  "stopReason": "message shown to user",
  "suppressOutput": false,
  "systemMessage": "warning shown to user"
}
```

向 Claude 注入上下文：

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "This file is generated. Edit src/schema.ts and run `bun generate` instead."
  }
}
```

---

## 10. 哪些事件可以阻止执行

| 事件 | 是否可阻止 | 阻止效果 |
|---|---:|---|
| `PreToolUse` | 是 | 阻止工具调用 |
| `PermissionRequest` | 是 | 拒绝权限请求 |
| `UserPromptSubmit` | 是 | 阻止 prompt 处理，并清空 prompt |
| `UserPromptExpansion` | 是 | 阻止命令展开 |
| `Stop` | 是 | 阻止 Claude 停止，继续对话 |
| `SubagentStop` | 是 | 阻止 subagent 停止 |
| `PostToolBatch` | 是 | 下一次模型调用前停止 agent loop |
| `TaskCreated` | 是 | 回滚任务创建 |
| `TaskCompleted` | 是 | 阻止任务标记完成 |
| `ConfigChange` | 是 | 阻止配置变更生效 |
| `PreCompact` | 是 | 阻止 compact |
| `Elicitation` | 是 | 拒绝 elicitation |
| `ElicitationResult` | 是 | 阻止响应 |
| `WorktreeCreate` | 是 | 非 0 退出会导致 worktree 创建失败 |

不可阻止但可观察/记录的常见事件：

- `PostToolUse`
- `PostToolUseFailure`
- `Notification`
- `SessionStart`
- `SessionEnd`
- `CwdChanged`
- `FileChanged`
- `PostCompact`
- `StopFailure`

---

## 11. 常用接入模板

### 11.1 Claude 等待输入时通知

适合：让外部系统知道 Claude Code 已经空闲或需要用户处理。

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "node .claude/hooks/notify.js"
          }
        ]
      }
    ]
  }
}
```

`.claude/hooks/notify.js`：

```js
#!/usr/bin/env node

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  const event = input ? JSON.parse(input) : {};

  console.log(JSON.stringify({
    systemMessage: `Claude notification: ${event.message || event.hook_event_name || 'unknown'}`
  }));
});
```

---

### 11.2 Claude 完成一轮回复时记录状态

适合：捕获 turn 结束，写入本地日志、通知你的 Rust/Node 服务。

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "node .claude/hooks/on-stop.js"
          }
        ]
      }
    ]
  }
}
```

`.claude/hooks/on-stop.js`：

```js
#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  const event = input ? JSON.parse(input) : {};

  const logDir = path.join(process.cwd(), '.claude', 'hook-logs');
  fs.mkdirSync(logDir, { recursive: true });

  fs.appendFileSync(
    path.join(logDir, 'stop.jsonl'),
    JSON.stringify({
      at: new Date().toISOString(),
      session_id: event.session_id,
      hook_event_name: event.hook_event_name,
      cwd: event.cwd,
      transcript_path: event.transcript_path,
      last_assistant_message: event.last_assistant_message
    }) + '\n'
  );

  process.exit(0);
});
```

---

### 11.3 工具调用前拦截危险命令

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/block-dangerous-bash.sh"
          }
        ]
      }
    ]
  }
}
```

`.claude/hooks/block-dangerous-bash.sh`：

```bash
#!/bin/bash
command=$(jq -r '.tool_input.command // empty' < /dev/stdin)

if [[ "$command" =~ rm[[:space:]]+-rf[[:space:]]+/ ]]; then
  echo "Blocked dangerous command: $command" >&2
  exit 2
fi

exit 0
```

---

### 11.4 文件修改后自动格式化

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "jq -r '.tool_input.file_path' | xargs npx prettier --write"
          }
        ]
      }
    ]
  }
}
```

---

### 11.5 Windows PowerShell hook

Windows 下可以显式指定 PowerShell：

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "shell": "powershell",
            "command": "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); [System.Windows.Forms.MessageBox]::Show('Claude Code needs your attention', 'Claude Code')"
          }
        ]
      }
    ]
  }
}
```

---

## 12. HTTP hook

如果你要把事件发给本地服务或远程服务，可以使用 `type: "http"`。

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "http",
            "url": "http://localhost:8787/claude/hooks/stop",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

HTTP hook 行为：

| HTTP 结果 | Claude Code 行为 |
|---|---|
| `2xx` 空 body | 成功，无额外输出 |
| `2xx` 文本 body | 文本作为上下文 |
| `2xx` JSON body | 按 command hook JSON 输出解析 |
| 非 `2xx` | 非阻塞错误，继续执行 |
| 连接失败/超时 | 非阻塞错误，继续执行 |

注意：

> HTTP hook 不能只靠非 2xx 状态码阻止工具调用。要阻止，必须返回 `2xx`，body 里放 JSON 决策字段。

---

## 13. async hook

长任务可以设置后台执行：

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/run-tests.sh",
            "async": true,
            "timeout": 120
          }
        ]
      }
    ]
  }
}
```

注意：

- 只有 `command` hook 支持 `async`
- async hook 不会阻塞 Claude
- async hook 的 `decision`、`continue` 等控制字段无效
- 适合跑测试、发送通知、写日志、调外部服务

---

## 14. 调试方法

### 14.1 查看已注册 hooks

在 Claude Code 中输入：

```text
/hooks
```

它可以查看：

- 哪些事件配置了 hook
- matcher 是什么
- hook 来源文件
- command / prompt / URL 等细节

注意：`/hooks` 是只读的，修改仍然需要编辑 settings JSON。

### 14.2 开启 debug

启动 Claude Code 时加：

```bash
claude --debug
```

用于查看 hook 执行错误、stderr、JSON 解析失败等。

### 14.3 给 hook 写本地日志

建议每个接入脚本都先写原始输入：

```js
const fs = require('fs');
const input = fs.readFileSync(0, 'utf8');
fs.appendFileSync('.claude/hook-logs/raw.jsonl', input + '\n');
```

这比盲猜字段可靠很多。

---

## 15. 安全注意事项

Hooks 会自动执行本地命令，所以需要谨慎：

- 不要在 hook 中直接执行来自 `stdin` 的任意字符串
- 对 `tool_input.command` 做严格校验
- 不要把 secrets 打印到 stdout/stderr
- 项目共享的 `.claude/settings.json` 应该只放团队都能接受的 hook
- 个人实验放 `.claude/settings.local.json`
- 对外 HTTP hook 建议配置允许访问的 URL
- Windows 下注意 Git Bash / PowerShell 差异

---

## 16. 推荐项目结构

```text
your-project/
├── .claude/
│   ├── settings.json
│   ├── settings.local.json
│   ├── hooks/
│   │   ├── notify.js
│   │   ├── on-stop.js
│   │   ├── block-dangerous-bash.sh
│   │   └── run-tests.sh
│   └── hook-logs/
│       ├── raw.jsonl
│       └── stop.jsonl
└── package.json
```

建议把 `.claude/hook-logs/` 加入 `.gitignore`。

```gitignore
.claude/hook-logs/
```

---

## 17. 最小可用方案：只接入任务状态通知

如果你的目标只是知道 Claude Code 什么时候需要用户输入、什么时候一轮结束，可以先接两个事件：

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "node .claude/hooks/notify.js"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "node .claude/hooks/on-stop.js"
          }
        ]
      }
    ]
  }
}
```

这两个事件的定位：

| 事件 | 你能知道什么 |
|---|---|
| `Notification` | Claude 需要输入、需要权限、认证完成、MCP elicitation 等 |
| `Stop` | Claude 当前 turn 已结束 |

如果要做多任务状态管理，建议 hook 脚本至少记录：

- `session_id`
- `hook_event_name`
- `cwd`
- `transcript_path`
- 当前时间戳
- `last_assistant_message`，如果事件提供
- Notification 类型与 message，如果事件提供

---

## 18. 接入 checklist

- [ ] 创建 `.claude/settings.json` 或 `.claude/settings.local.json`
- [ ] 添加 `hooks` 配置块
- [ ] 先接 `Notification` 和 `Stop`
- [ ] hook 脚本读取 stdin
- [ ] hook 脚本把原始 JSON 写到本地日志
- [ ] 使用 `/hooks` 确认配置被 Claude Code 识别
- [ ] 运行一次 Claude Code，确认 hook 触发
- [ ] 再按需求增加 `PreToolUse` / `PostToolUse`
- [ ] 对阻断类 hook 使用 `exit 2` 或结构化 JSON，二选一
- [ ] 把 `.claude/hook-logs/` 加入 `.gitignore`

