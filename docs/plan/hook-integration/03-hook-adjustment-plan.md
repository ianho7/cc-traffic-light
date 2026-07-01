# Hook 接入调整计划

日期：2026-07-01

## 背景

参考 Electron 项目的实现链路后，需要修正当前 Rust MVP 的一个关键假设：Codex 和 Claude Code 不一定提供完全相同的 hook 配置形态。

Electron 项目的完整链路是：

```text
对话生命周期事件
-> hook 进程
-> hookRules 状态推断
-> 写 state.json
-> 主进程轮询
-> IPC 推送
-> 前端显示
```

Rust 任务栏版本不需要 Electron 的 IPC 和 renderer，但前半段仍然适用：

```text
对话生命周期事件
-> Rust hook CLI
-> hook_rules 状态推断
-> 写 state.json
-> Win32 timer 轮询
-> GDI 重绘
```

当前 Rust 实现已经完成这条主链路的人工验证，但真实 Codex 接入方式仍需调整。

## 需要调整的结论

### 1. Codex 接入应区分 notify 和 lifecycle hooks

当前本机 `C:\Users\admin\.codex\config.toml` 没有明确的 `hooks` 配置块，只存在：

```toml
notify = [ "...codex-computer-use.exe", "turn-ended" ]
```

同时 `codex features list` 显示：

```text
hooks stable true
```

这说明 Codex 具备 hooks 能力，但当前可见配置只证明本机已有 `notify` 入口，不能把 `notify` 的低保真实测结果等同于 Codex lifecycle hooks 的能力。

官方 Codex hooks 文档说明：lifecycle hooks 默认启用，可通过 `hooks.json` 或 `config.toml` 的 inline `[hooks]` 配置；当前真正运行的是 `type = "command"` handler；command hook 从 stdin 接收 JSON，包含 `session_id`、`cwd`、`hook_event_name`、`model`，turn scope 事件还可包含 `turn_id`。

调整方向：

- 不直接覆盖用户现有 `notify`。
- `notify` 只作为兼容/兜底通知入口，不作为 Codex 多任务状态主路径。
- Codex 主状态来源应改为正式 lifecycle hooks，前提是本机 Codex 版本支持、hooks 已启用、配置已加载且 hook 已 trust。
- 新增 Codex lifecycle hooks 最小验证计划。
- wrapper 先记录脱敏 argv/stdin shape，再转发到原 notify 命令。
- 只有确认正式 hooks 的真实输入 shape 后，才进入 Codex 主状态映射。

MVP 影响：

- Codex notify 已实测为低保真，不应继续作为主路径。
- Codex lifecycle hooks 是下一步主验证路径。
- Codex 可先通过 debug CLI 或 lifecycle hooks dump 验证真实 payload。
- Claude Code hook 可继续按明确 hook 事件路线推进。

### 2. 把 hook rules 从 CLI 中拆出

Electron 项目将状态推断集中在 `hookRules.ts`。Rust 当前把以下逻辑放在 `taskbar_widget_hook.rs`：

- hook name 到 `AgentState` 的映射。
- `Stop` 的 waiting 判断。
- payload 文本提取。
- payload 字段提取。

调整方向：

- 新增 `taskbar-widget/src/hook_rules.rs`。
- CLI 只负责 argv/stdin、错误输出和调用状态写入。
- `hook_rules.rs` 负责纯规则函数，便于单元测试和对齐 Electron 行为。

MVP 影响：

- 不改变状态 schema。
- 不改变 state.json 写入路径。
- 提升后续真实 payload 调整的局部性。

### 3. 补齐 `SubagentStop -> done`

Electron 参考映射包含：

```text
SubagentStop -> done
```

Rust 当前映射缺少 `SubagentStop`。

调整方向：

- 在规则层补 `SubagentStop -> done`。
- 验证不会影响普通 `Stop` 的 waiting heuristic。

MVP 影响：

- 对 Claude Code 子任务结束更准确。
- 不扩大 UI 范围。

### 4. 明确在线检测不是当前 MVP 的状态来源

Electron 项目用 detector 决定 agent 列是否显示：

```text
hook state 决定颜色
process detector 决定是否显示
```

Rust MVP 当前没有进程检测，任务栏只显示 `global_summary`。

调整方向：

- 文档明确当前显示的是“最近 hook 状态聚合”，不是“当前 agent 在线状态”。
- 不在本轮加入进程检测。
- 后续如果要按-agent显示或隐藏 agent，再新增 detector 阶段。

MVP 影响：

- 避免用户误解红绿灯代表进程在线。
- 保持当前实现窄范围。

## 调整后的阶段计划

### Phase A: Codex notify 探针

Goal: 在不破坏现有 Codex notify 的前提下，确认 Codex notify 实际传入方式。

Tasks:

- 新增一个 wrapper 方案文档，说明如何包装当前 `notify`。
- wrapper 输入只记录脱敏 argv shape 和 stdin shape。
- wrapper 调用原始 `codex-computer-use.exe turn-ended`，避免破坏 Codex Desktop 行为。
- 验证 Codex notify 是否提供 JSON stdin。
- 验证 notify 是否携带 session/thread/turn 信息。

Exit:

- 如果 notify 没有足够上下文，只将其作为低保真 `turn-ended` 事件，不作为 task 状态主来源。
- notify 实测完成后，转入 Codex lifecycle hooks 验证，不继续从 notify 推断多任务状态。

### Phase A2: Codex lifecycle hooks 最小验证

Goal: 验证本机 Codex 是否加载正式 lifecycle hooks，并确认 command hook stdin 是否提供 `session_id`、`hook_event_name`、`cwd`、`model` 和 turn 级 `turn_id`。

Tasks:

- 新增或更新 `docs/checklist/codex-lifecycle-hooks-validation.md`。
- 使用 `hooks.json` 或 inline TOML 配置一个 shape-only dump hook。
- Windows 下使用 `commandWindows` / `command_windows` 指向仓库内或用户级 dump 脚本。
- 通过 `/hooks` 或启动提示完成 hook review/trust。
- 触发 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`PostToolUse`、`SubagentStop`、`Stop` 中的最小事件集。
- 只记录字段 shape 和 candidate paths，不保存 prompt、代码、命令参数或完整路径。

Exit:

- 如果 payload 提供稳定 `session_id`，Codex lifecycle hooks 可作为主状态来源。
- 如果 turn 级事件提供 `turn_id`，把它记录为辅助 turn identity，不替代 session 主键。
- 如果 hooks 没触发，先排查版本、`[features].hooks`、配置位置、project trust 和 hook trust。

### Phase B: 规则层拆分

Goal: 将状态推断从 CLI 剥离为纯规则模块。

Tasks:

- 新增 `taskbar-widget/src/hook_rules.rs`。
- 移动 hook name 映射、waiting heuristic、payload 字段提取。
- CLI 保留 stdin decode、argv parse、错误输出、成功输出兼容。
- 为规则层添加 focused 验证命令或人工断言。

Exit:

- `cargo check` 通过。
- 人工 hook 验证结果与拆分前一致。

### Phase C: 补齐 Electron 对齐项

Goal: 对齐参考实现中已经明确且低风险的事件语义。

Tasks:

- 增加 `SubagentStop -> done`。
- 明确未知 hook 的处理策略：保守设为 `working`，同时记录 hook name。
- 扩充 waiting pattern 只基于真实 payload 证据，不提前无限添加。

Exit:

- `SubagentStop` 人工验证进入 `done`。
- `Stop` waiting 逻辑仍保持 previous waiting 优先。

### Phase D: 文档和 checklist 同步

Goal: 避免旧文档继续暗示 Codex TOML 多事件 hooks 已确认。

Tasks:

- 更新 `docs/checklist/hook-payload-sampling.md`，标记 Codex notify 已实测为低保真，正式 hooks 是主路径。
- 更新 `taskbar-widget/examples.codex-hooks.toml`，使用官方 lifecycle hooks 的 inline TOML 结构。
- 新增 lifecycle hooks 最小验证 checklist。
- 更新 checklist，增加 Codex notify 探针和 `hook_rules.rs` 拆分任务。
- 保留“不自动修改用户外部配置”的 policy。

Exit:

- 文档不再误导用户直接套用 Codex hooks TOML。
- 下一轮实现有明确任务边界。

## 不调整的内容

- 不引入 Electron、Node、daemon、HTTP server 或 IPC。
- 不改 taskbar probing、SetParent、positioning。
- 不保存完整 payload。
- 不自动修改 `C:\Users\admin\.codex\config.toml`。
- 不把进程检测加入当前 MVP。

## 风险

Risk: Codex notify 不提供 JSON stdin。  
Impact: 无法通过 notify 获取 session/task 粒度状态。  
Mitigation: notify 仅作为兼容/低保真 turn-ended；正式状态改走 lifecycle hooks。

Risk: lifecycle hooks 配置存在但未被 trust。  
Impact: hook 不运行，误判为 Codex 无 hook 能力。  
Mitigation: 用 `/hooks` 检查 hook source、review 状态和 trust 状态，必要时单独记录阻塞原因。

Risk: wrapper 转发原 notify 行为不完整。  
Impact: 可能影响 Codex Desktop 通知或 computer-use 集成。  
Mitigation: wrapper 先做文档和手工审阅，不自动安装；转发参数必须逐字保留。

Risk: 规则层拆分引入行为回归。  
Impact: 人工 hook 验证结果变化。  
Mitigation: 拆分前后跑同一组人工验证：working、waiting、error、Stop waiting、缺失 session、乱序。

## 推荐下一步

Phase B 和 Phase C 已完成。下一步应执行 Phase A2：Codex lifecycle hooks 最小验证。

Codex notify wrapper 属于外部配置探针，已验证为低保真兼容入口。后续不应继续扩大 notify 推断，除非只是排查现有通知兼容性。
