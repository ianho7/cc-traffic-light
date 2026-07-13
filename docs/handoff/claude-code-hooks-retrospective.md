# Claude Code Command Hooks 对接复盘：一次"文档就在眼前却没认真读"的调试经历

## 1. 问题背景

CC Traffic Light 是一个 Windows 任务栏组件，通过 command hooks 接收 Codex 和 Claude Code 的活动状态，在任务栏上显示红绿灯指示。Codex 的 hooks 已成功对接：

```
Codex → hooks.json → taskbar_widget_hook.exe → state.json → 任务栏组件
```

现在需要为 **Claude Code** 做同样的对接。Claude Code 官方文档说它支持 command hooks，配置文件格式与 Codex 类似，放在 `.claude/settings.local.json`。

任务看起来很直接：把 Codex 那套配置改个名字和路径给 Claude Code 用就行了。

## 2. 为什么这个问题比看起来难

### 第一层陷阱：看起来很简单

Claude Code 官方文档提供了 hook 配置的完整格式说明，甚至给了 Windows 示例：

```json
{
  "type": "command",
  "command": "powershell.exe -Command \"...\""
}
```

而项目已经有一个 `.claude/settings.local.json`，里面已经配好了 6 个 hook 事件，command 指向 `taskbar_widget_hook.exe`——只是用了 `shell: "powershell"` 包装。

直觉反应是："格式不对，改改就好了。"

### 第二层陷阱：多个变量同时变化

每次修改都涉及路径格式（反斜杠 / 正斜杠 / MSYS2 / `${CLAUDE_PROJECT_DIR}`）和执行方式（shell form / exec form）两个维度。当测试结果始终是空状态时，很难判断到底是哪个变量出了问题。

### 第三层陷阱：静默失败

exec form 下 hooks **完全不执行**，且不报任何错误。没有日志、没有输出——这让排查无从下手。

## 3. 调查路径

### 3.1 初始状态：已有的 settings.local.json（行 1）

用户已经有一个 `.claude/settings.local.json`，配置了 6 个事件，全部使用 `shell: "powershell"` 包装：

```json
{
  "type": "command",
  "shell": "powershell",
  "command": "& 'D:\\project\\...\\taskbar_widget_hook.exe' claude PreToolUse"
}
```

测试结果：`tasks: {}`，没有任何写入。

### 3.2 弯路 1：认为反斜杠路径是唯一问题

用户第一次触发 hook 时，Claude Code 显示了错误信息：

```
/usr/bin/bash: line 1: D:projectcc-traffic-light...: command not found
```

关键证据：**反斜杠被 Git Bash 当转义符吃掉了**。而且最重要的是——**这个错误证明了 shell form 下的 hooks 确实能触发**，只是路径解析失败。

### 3.3 弯路 2：试了 5 种路径格式

在 shell form 下（不加 `args` 字段，command 字符串传给 Git Bash），逐一尝试：

| 尝试 | 路径格式 | 结果 |
|---|---|---|
| 去掉 `shell`，用反斜杠 | `D:\\project\\...` | Bash 报 `command not found` |
| 改为正斜杠 | `D:/project/...` | 无错误，无状态 |
| 改为 MSYS2 格式 | `/d/project/...` | 无错误，无状态 |
| 用 cmd.exe 包装 | `cmd.exe /c "D:\\project\\..."` | 无错误，无状态 |
| 用 ${CLAUDE_PROJECT_DIR} | `${CLAUDE_PROJECT_DIR}/...` | 无错误，无状态 |

所有尝试的结果完全一致：**state.json 为空**。

此时犯了第二个错误：**没有用一个最简单的写文件命令来验证 hook 是否真的触发了**。一直在看 state.json 是否被写入，但 state.json 为空可能是 hook 没触发、exe 没收到 stdin、exe 写入了但 session_id 为空等多种原因。

### 3.4 弯路 3：被文档的 exec form 推荐带偏

文档写道：

> Prefer exec form for any hook that references a path placeholder. Exec form spawns the process directly with no shell involved.

于是改成 exec form（加 `args` 字段）：

```json
{
  "type": "command",
  "command": "${CLAUDE_PROJECT_DIR}/taskbar-widget/target/debug/taskbar_widget_hook.exe",
  "args": ["claude", "PreToolUse"]
}
```

结果：hooks 在 `/hooks` 菜单中正常显示，但 **完全不执行**，静默失败，没有任何错误。

这是最大的弯路——花了大量时间调试 exec form 的路径格式，但根本问题是 exec form 在这个环境里就不工作。

### 3.5 转折点：用写文件验证 hook 是否真的触发

最终用了一条最简单的命令：

```json
{
  "type": "command",
  "command": "cmd.exe /c echo fired > \"%TEMP%\\cc-hook-test.txt\""
}
```

结果：**文件没生成**。

这个测试排除了所有与 exe、路径、stdin 格式相关的变量。问题回到了原点：**Shell form 下 hooks 能触发（早期报过 bash 错误），但 exec form 下 hooks 根本不执行**。

### 3.6 最终确认：shell form 可触发但不工作

回到 shell form + cmd.exe 包装：

```json
{
  "type": "command",
  "command": "cmd.exe /c \"D:\\project\\...\\taskbar_widget_hook.exe\" claude PreToolUse"
}
```

hooks 在 `/hooks` 菜单显示正确，但：

- 测试文件未生成 → cmd.exe 可能没执行或 stdin 没传递
- state.json 为空 → exe 没收到有效 stdin

### 3.7 最终结论

在这个环境下，Claude Code 的 command hooks 机制无法与 `taskbar_widget_hook.exe` 可靠对接：

- **Shell form**（走 Git Bash）：hooks 能触发，但通过多重 shell 嵌套后 stdin 丢失或路径转换失败
- **Exec form**（直接 spawn）：hooks 完全不执行，静默失败

**可行的方案**：

1. **Claude Code 保持进程检测**（Degraded 置信度：只能显示"进程存在"，不能显示 Working / NeedsAttention）
2. **未来可探索 HTTP hook**（`type: "http"`——在本地启动 HTTP endpoint 接收 hook 事件，绕过 shell 路径问题）

## 4. 关键证据

| 证据 | 内容 | 强度 |
|---|---|---|
| 首次 shell form 报错 | `/usr/bin/bash: line 1: D:projectcc-traffic-light...: command not found` | 强 — 直接错误输出 |
| exec form 写文件验证 | `cmd.exe /c echo fired > %TEMP%\cc-hook-test.txt` 未生成文件 | 强 — 可复现 |
| shell form cmd.exe 测试 | 同一条命令走 shell form 也未生成文件 | 强 — 可复现 |
| `/hooks` 菜单显示 | 所有配置的 hooks 加载正常 | 强 — 多次确认 |
| 直接 cmd.exe 管道测试 | `cmd /c 'type payload \| hook.exe'` 能正常写状态 | 强 — 证明 exe 本身没问题 |
| Codex hooks 正常工作 | 同一台机器、同一条 exe 路径、Codex 的 `commandWindows` 字段能工作 | 强 — 对比证据 |

## 5. 走了哪些弯路，如何纠正的

### 弯路 1：在 shell form 下反复改路径格式

**为什么会走：** 看到 bash 报 `command not found`，直觉是路径不对。改了 5 种格式，每种都重启 Claude Code 测试。

**为什么是弯路：** 路径格式只是表象——shell form 走 Git Bash 本身就有 stdin 传递问题，改路径根本解决不了。

**如何纠正的：** 当所有路径格式都失败后，意识到问题不在路径，在 hook 执行机制本身。

### 弯路 2：没有及时用写文件来验证 hook 是否触发

**为什么会走：** 依赖 state.json 是否被写入来判断 hook 是否工作。但 state.json 为空有多种原因。



### 弯路 3：被文档的 exec form 推荐带偏

**为什么会走：** 文档说"推荐 exec form，没有 shell 参与"，看起来正是需要的方案。

**为什么是弯路：** 花了大量时间调试 exec form 的各种路径变体，但 exec form 在这个环境下**根本不执行**——先确认它能工作再调试会更快。

**如何纠正的：** 写文件验证暴露了 exec form 完全静默失败的事实。

### 弯路 4：没有仔细阅读文档的 Windows 章节

**为什么会走：** 看了文档的 hook 示例和 exec form 说明，但没仔细看这段：

> Shell form runs when `args` is absent. The command string is passed to a shell: **Git Bash on Windows**...

用户给了完整文档，但我没逐字读完，而是选择性阅读了自己想要的方案。

**如何纠正的：** 用户反复提醒"你确定你的代码都遵守了我给你的2个文档？"，最后才去逐行重读。

## 6. 可复用的经验

### 调试原则

1. **先证 hook 触发，再调 payload 处理。** 用 `echo > file` 这种零依赖的命令验证 hooks 是否真的执行，比看复杂 exe 的输出要快得多。

2. **一次只变一个变量。** 路径格式和执行方式是正交维度，不应该同时改。先固定 shell form 调通路经，再试 exec form。

3. **区分"不触发"和"处理错"。** state.json 为空可能是 hook 没触发、exe 没收到 stdin、exe 解析失败等多种原因。一个简单的写文件命令可以直接回答"触发了没有"。

### 与 AI Agent 协作

1. **AI Agent 容易"选择性阅读"文档。** 它会倾向于寻找支持当前假设的章节，而不是通读全文。作为人类审稿者，直接定位到相关段落提问最有效。

2. **当 Agent 反复在一个方向上碰壁时，它可能需要外部的"推一把"。** 这次的转折点是两个触发：
   - 用户问"你确定你的代码遵守了文档？"
   - 用户提供了完整的官方文档链接

3. **好的 Agent prompt 应该明确禁止已试过的方向。** 最终的 handoff 文档中的 "禁止重复尝试的方向" 列表就是为此设计的。

### Windows 平台上 Claude Code hooks 的特殊性

1. Claude Code 在 Windows 上默认通过 Git Bash 运行 shell form hooks——不是 cmd.exe，不是 PowerShell
2. Git Bash 的反斜杠转义会导致 Windows 绝对路径损坏
3. Exec form（直接 spawn）在当前版本上可能不可靠，需要在具体环境中验证
4. Codex 有 `commandWindows` 字段专门处理 Windows 路径，Claude Code 没有等价机制

## 7. 最终结果

| 项目 | 状态 |
|---|---|
| **Codex hooks** | ✅ 可正常工作（通过 `commandWindows` + 反斜杠路径） |
| **Claude Code command hooks** | ❌ 在当前 Windows + Git Bash 环境下不可用 |
| **Claude Code 进程检测** | ✅ 已实现（Degraded 置信度） |
| **Handoff 文档** | ✅ `docs/handoff/claude-code-hooks-investigation.md` |
| **配置清理** | ✅ `.claude/settings.local.json` 已删除 |

### 未来可探索

- Claude Code 的 `type: "http"` hook 可能绕过 shell 路径问题——在本地启动 HTTP 端点接收 hook 事件
- 如果 Claude Code 后续版本更新了 Windows 上的 hook 执行机制，可以重新验证
- 如果用户切换到不使用 Git Bash 的终端环境（如 Windows Terminal + PowerShell only），shell form 可能走 PowerShell 路径，行为可能不同

## 8. 耗时统计

| 阶段 | 耗时 | 说明 |
|---|---|---|
| 尝试各种路径格式 | 长 | 5 种路径格式 × 多次重启 Claude Code |
| 尝试 exec form | 中 | 验证了 ${CLAUDE_PROJECT_DIR} + args |
| 写文件验证 | 短 | 一次测试就暴露了根本问题 |
| 阅读文档 | 短 | 但发生在过程的最后 |

**如果不是走了弯路，整个过程可以在 3-4 次测试内完成：**

1. 先跑写文件验证 → 发现 shell form 下 hooks 不产生副作用
2. 再跑 exec form 写文件验证 → 发现 exec form 完全不执行
3. 三次测试锁定结论：command hooks 在当前环境不可靠
4. 记录到文档，切换为进程检测

实际上花了远多于这个时间的根源：**没有优先用最简单的验证手段。**
