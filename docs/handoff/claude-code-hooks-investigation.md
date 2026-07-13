# Claude Code Command Hook 对接问题分析

## 1. 项目背景（Project Context）

**CC Traffic Light** 是一个 Windows 任务栏小组件，用于实时监控 Codex 和 Claude Code 的活动状态。它在任务栏上显示红绿灯指示：Working（工作中）、NeedsAttention（需要输入）、Completed（完成）等。

**技术栈：**
- Rust Win32 原生窗口（taskbar-widget）
- Tauri + React 设置界面（taskbar-settings-tauri）
- 共享业务层（crates/shared-core）

**监控架构：**
```
Codex / Claude Code
    │ (hook event: JSON on stdin)
    ▼
taskbar_widget_hook.exe
    │ (写入 state.json)
    ▼
%APPDATA%\CcTrafficLight\state.json
    │ (每秒轮询)
    ▼
taskbar-widget.exe → 红绿灯显示
```

**Hook 的执行方式：**
- Claude Code 在特定生命周期事件（`PreToolUse`, `PostToolUse`, `Stop` 等）触发时，向 command hook 的 stdin 写入 JSON payload
- hook 可执行文件从 stdin 读取 JSON，提取 `session_id`、`hook_event_name` 等字段，写入 `state.json`
- 如果 stdin 为空或无法解析，hook 不会写入有效任务

**当前开发目标：**
- 让 Claude Code 通过 command hooks 将活动状态写入 `state.json`
- 已确认：`taskbar_widget_hook.exe` 本身工作正常（通过 cmd.exe 管道传入 JSON 测试通过）
- 已确认：Codex 的 command hooks 在这个开发机上可正常使用

---

## 2. 当前问题描述（Problem Statement）

**Expected:**

```
在 Claude Code 中触发 tool call 后：
1. taskbar_widget_hook.exe 通过 stdin 收到 JSON payload
2. state.json 中出现 claude_<session_id> 条目
3. taskbar-widget.exe 显示 Claude Code 的活动状态（Working / NeedsAttention 等）
```

**Actual:**

```
无论改成什么 hook 配置，state.json 中的 tasks 始终为空：
"tasks": {}
```

**问题首次发现时间：** 2026-07-08 首次尝试用 `.claude/settings.local.json` 配置 command hooks。

**复现步骤：**
1. 确保 `.claude/settings.local.json` 中有 command hooks 配置
2. 启动 Claude Code（`claude`）
3. 在 Claude Code 中运行 `/hooks`，确认 hooks 已加载
4. 发送任意 prompt（如 `test`）
5. 在 PowerShell 中运行 `& "D:\project\cc-traffic-light\target\debug\taskbar_widget_hook.exe" list`
6. 观察 `tasks: {}` —— 始终为空

---

## 3. 环境信息（Environment）

| 项目 | 值 |
|---|---|
| 操作系统 | Windows (确认版本 UNKNOWN) |
| 终端 | PowerShell (在 Windows Terminal 中) |
| Claude Code 版本 | 从 `/hooks` 输出确认支持 command hooks，具体版本号 UNKNOWN |
| Claude Code 安装方式 | UNKNOWN（可能是 npm global install） |
| Git | 已安装（Git Bash 存在，路径可访问） |
| taskbar_widget_hook.exe | 可用且已验证 |
| Rust 工具链 | cargo 1.93.0, rustc 1.93.0 |

**已知的 Claude Code 钩子系统行为（来自官方文档）：**

- Shell form（无 `args` 字段）：command 字符串传给 `sh -c`（macOS/Linux）或 **Git Bash on Windows**，或 PowerShell（当 Git Bash 未安装时）
- Exec form（有 `args` 字段）：command 解析为 PATH 上的可执行文件，直接 spawn，**没有 shell 参与**
- Windows 上 exec form 要求 command 是真实的 `.exe` 文件
- 推荐使用 `${CLAUDE_PROJECT_DIR}` 路径占位符引用项目目录下的脚本

---

## 4. 相关代码路径（Relevant Code）

### 4.1 配置文件

```
.claude/settings.local.json
```

作用：Claude Code 项目级本地 hook 配置（gitignored）
关键位置：hook 事件和 command 定义

```
.codex/hooks.json
```

作用：Codex 项目级 hook 配置（已验证能工作）
关键位置：作为对比参考

### 4.2 hook 二进制

```
taskbar-widget/src/bin/taskbar_widget_hook.rs
```

作用：接收 hook 事件的主入口。从 stdin 读取 JSON payload，解析 session_id、hook_event_name 等字段，写入 state.json
关键位置：
- `read_stdin()` (行 136-139)：从 stdin 读取字节
- `decode_stdin_bytes()` (行 142-155)：处理 UTF-8 BOM / UTF-16 LE 编码
- `handle_hook()` (行 92-133)：解析 JSON payload，提取 session_id，写入状态
- `usage()` (行 179-182)：命令行接口定义

相关问题：如果 stdin 为空，`parse_json_input(input, true)` 返回空对象，`extract_session_id()` 返回 None，任务以 `claude_unknown` 写入但 `summary_eligible = false`（不参与显示摘要）

### 4.3 状态管理

```
taskbar-widget/src/agent_state.rs
```

作用：读写 `%APPDATA%\CcTrafficLight\state.json`
关键位置：
- `state_file_path()` (行 188-195)：状态文件路径
- `apply_hook_event()` (行 253-290)：写入 hook 事件
- `task_key()` (行 322-327)：生成任务键（`<agent>_<session_id>` 或 `<agent>_unknown`）

### 4.4 文档

```
docs/plan/post-install-monitoring-readiness.md
docs/checklist/post-install-monitoring-readiness.md
```

作用：安装后监控链路的实施计划和 checklist

---

## 5. 当前执行流程（Execution Flow）

### 期望流程

```
用户在 Claude Code 输入 prompt
 ↓
Claude Code 触发 PreToolUse 事件
 ↓
Claude Code 将 JSON payload 写入 stdin pipe
 ↓
Claude Code 执行 command hook（spawn 进程）
 ↓
taskbar_widget_hook.exe 从 stdin 读取 JSON
 ↓
提取 session_id → 写入 state.json
 ↓
taskbar-widget.exe 轮询并显示状态
```

### 实际流程

```
用户在 Claude Code 输入 prompt
 ↓
Claude Code 触发 PreToolUse 事件（/hooks 菜单确认 hooks 已加载）
 ↓
Claude Code 尝试执行 command hook
 ↓
???         ← 异常点：没有任何反馈（无错误、无输出、无状态写入）
 ↓
state.json 始终为 {}
```

### 异常点分析

- Codex 的 hook 配置使用 `commandWindows` 字段，Claude Code 使用不同的 schema
- Claude Code 文档明确说 shell form 走 Git Bash，exec form 直接 spawn
- 两种 form 都试过了，均未产生状态写入
- 唯一一次看到 hook 有输出，是 shell form + backslash 路径时 Git Bash 报 `command not found`

---

## 6. 已尝试方案（Attempted Solutions）

| # | 方案 | 配置格式 | 结果 | 失败原因 |
|---|---|---|---|---|
| 1 | `shell: "powershell"` + `& 'D:\...exe' claude <Event>` | Shell form | Git Bash 报 `command not found`，反斜杠被 bash 吃掉 | 原配置就存在且不工作 |
| 2 | 去掉 `shell`，直接写 `D:\...exe claude <Event>` | Shell form | Git Bash 报同样的错 | 反斜杠被 bash 解释为转义符 |
| 3 | 改为正斜杠 `D:/...exe claude <Event>` | Shell form | 无错误，也无状态写入 | UNKNOWN — 可能路径未解析或 stdin 未传递 |
| 4 | 改为 MSYS2 格式 `/d/project/...exe claude <Event>` | Shell form | 无错误，无状态写入 | UNKNOWN — 同上 |
| 5 | 去掉 `shell`，改用 `cmd.exe /c "D:\...exe" claude <Event>` | Shell form | 无错误，无状态写入 | UNKNOWN — cmd.exe 可能不转发 stdin |
| 6 | 加 `"args": ["claude", "PreToolUse"]`，`command` 用 `${CLAUDE_PROJECT_DIR}/...` | Exec form (推荐方式) | hooks 静默不触发（无任何输出、无错误、marker 文件未生成） | UNKNOWN — exec form 可能在此环境不支持或 spawn 失败 |
| 7 | 加 `"args": []`，`command` 用 `cmd.exe` 写 marker 文件 | Exec form | 无输出，marker 文件未生成 | UNKNOWN — exec form 在此环境下可能完全不执行 |

**每一步都在重启 Claude Code 后测试。**

---

## 7. 更新后的诊断（2026-07-09 追加，two rounds of testing）

### 7.1 修正：settings.json hooks **在启动时正常加载**

通过最终干净测试（全新目录 C:\hook-test2，只用 hook binary 不依赖 cmd.exe）证实：

**`.claude/settings.json` 中的 hooks 在启动时就会被加载和执行。** 之前的"启动时不加载"结论是错误的——错误的 root cause 是测试方法问题：

1. 之前的 marker 文件测试使用了 `cmd.exe /c "echo ... > \"%TEMP%\\...\""`，但 `%TEMP%` 环境变量在 exec form 的 spawn 上下文中不可用 → marker 文件未生成 → 误判为 hooks 没触发
2. `"Hooks: Found 0 total hooks in registry"` debug log 信息可能只统计 plugin hooks，不包含 settings hooks
3. 反编译二进制得出的调用图结论应整体撤回——minified Bun bundle 的函数名映射不可靠

**干净测试结果：**
```
state.json 中成功写入:
  claude_421e3232: hook=Stop, state=done, session_source=payload  ← 第1次运行
  claude_97ffd79d: hook=Stop, state=done, session_source=payload  ← 第2次运行（独立session）
```

两个独立 session 都产生了有效的 state.json 条目，说明完整 hook pipeline 正常工作。

### 7.2 当前已验证的结论

| 假设 | 结论 | 证据 |
|---|---|---|
| settings.json hooks 启动时不加载 | **已证伪** | 干净测试中 hooks 正常触发并写入 state.json |
| `%TEMP%` 在 exec form 中不可用 | **确认** | `cmd.exe /c echo > %TEMP%\file` 失败，字面路径成功 |
| Plugin hooks 机制 | **确认可用** | 修改 UA plugin hooks.json 后 hooks 注册成功 |
| 本地 Plugin marketplace | **确认受阻** | `marketplace-not-found` 错误 |
| JSON/BOM/Workspace trust | **已排除** | 所有检查通过 |

### 7.3 当前推荐工作路径

**直接使用 `.claude/settings.json`（项目级共享配置）即可。** hooks 配置如下（exec form + 绝对路径）：

```json
{
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "UserPromptSubmit"]
      }]
    }],
    "PreToolUse": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "PreToolUse"]
      }]
    }],
    "PostToolUse": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "PostToolUse"]
      }]
    }],
    "Notification": [{
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "Notification"]
      }]
    }],
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "Stop"]
      }]
    }]
  }
}
```

---

## 8. 已排除的问题（Ruled Out）

| 假设 | 排除理由 |
|---|---|
| `.claude/settings.local.json` 语法错误 | `ConvertFrom-Json` 解析成功，`/hooks` 菜单也能正确显示 |
| hook 事件不匹配 | `PreToolUse` 是 tool call 之前触发，发 prompt + tool call 应该触发 |
| `taskbar_widget_hook.exe` 本身有问题 | 直接 `cmd /c 'type payload \| exe'` 测试能写状态 |
| 路径不存在 | `Test-Path` 确认 exe 存在 |
| 需要特殊转义 | 试了反斜杠、正斜杠、双引号、MSYS2 格式、${CLAUDE_PROJECT_DIR} 占位符——均无效 |
| hook 数量太多导致问题 | 精简到 1 个事件（PreToolUse）+ 1 个 command——仍无效 |

---

## 9. 最小复现路径（Minimal Reproduction）

### 前提：已安装 Claude Code，已 clone 项目

```
# 1. 确保配置文件存在
# 内容见 .claude/settings.local.json

# 2. 启动 Claude Code
cd D:\project\cc-traffic-light
claude

# 3. 在 Claude Code 中确认 hooks 加载
/hooks
# → PreToolUse / PostToolUse / Stop 应出现

# 4. 触发 hook 事件
列出当前目录文件

# 5. 检查状态
& "D:\project\cc-traffic-light\target\debug\taskbar_widget_hook.exe" list
# → tasks: {}  （应为 claude_<session_id>）
```

### 最简单的验证——不依赖项目 exe：

在 `.claude/settings.local.json` 中使用：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "cmd.exe /c echo fired > \"%TEMP%\\cc-hook-test.txt\""
          }
        ]
      }
    ]
  }
}
```

然后重启 Claude Code，发 `test`，检查 `Test-Path "$env:TEMP\cc-hook-test.txt"`。

**注意：此测试结果的最后状态是 False（hook 未执行）。**

---

## 10. 已验证的解决方案（Proven Solution）

### 推荐方案：使用 `.claude/settings.json`（已验证可行）

在项目根目录创建 `.claude/settings.json`，使用 exec form + 绝对路径：

```
D:\project\cc-traffic-light\.claude\settings.json
```

```json
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "PreToolUse"]
      }]
    }],
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "D:\\project\\cc-traffic-light\\target\\debug\\taskbar_widget_hook.exe",
        "args": ["claude", "Stop"]
      }]
    }]
  }
}
```

**关键注意事项：**
- 必须使用 exec form（即包含 `"args": [...]` 字段）
- 命令路径使用 Windows 绝对路径，反斜杠需要转义
- 不要使用 `cmd.exe + %TEMP%` 做测试——环境变量在 exec form 中不可用

### 备选方案：修改 understand-anything plugin 的 hooks.json

```
%USERPROFILE%\.claude\plugins\cache\understand-anything\understand-anything\2.7.5\hooks\hooks.json
```

**缺点：** 会被 plugin 更新覆盖。

---

## 11. 诊断过程回顾

### 第一轮错误结论（已撤回）

最初基于 debug log `"Hooks: Found 0 total hooks in registry"` 和 `cmd.exe + %TEMP%` marker 测试失败，错误地认为"settings.json hooks 启动时不加载"。这个结论被以下干净测试证伪：

1. 使用 hook binary 直接写入 state.json（绕过了 `cmd.exe + %TEMP%` 的问题）
2. 不使用任何特殊标志（`--debug-file`、`--dangerously-skip-permissions`）
3. 两次独立运行都产生正确 state.json 条目

**错误 root cause：** `cmd.exe /c "echo > \"%TEMP%\\file\""` 中的 `%TEMP%` 环境变量在 exec form 的 spawn 上下文中不可用。hook 二进制可以正常工作（不依赖 `%TEMP%`），但 marker 文件测试不会产生预期结果。

---

## 12. AI Agent Continuation Prompt

```text
当前状态（2026-07-09 最终修正）：
- Claude Code v2.1.204 在 Windows 上：settings.json hooks **在启动时正常加载并执行**
- exec form（args: []）+ 绝对路径可正常工作，直接写入 state.json
- 之前"启动时不加载"的结论已撤回——错误由 cmd.exe + %TEMP% 测试方法导致

已验证：
- 直接使用 .claude/settings.json 即可，plugin 修改不是必须的
- 反编译二进制得出的函数调用图结论不可靠，已撤回
- JSON/BOM/Workspace trust/项目路径——均排除

待完成：
- 将当前 project 的 .claude/settings.json 配置为完整的 hooks set（所有事件映射）
- 已恢复 API key
```
