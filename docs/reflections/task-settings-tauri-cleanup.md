# Reflection: taskbar-settings-tauri 代码清理

## Task

执行 `docs/checklist/settings-tauri-code-cleanup.md` 中的 Phase 1-3。

## 问题

### 1. dead branch + 重复的 command 骨架

5 个 Tauri command 各自写了 ~15 行的 `call_pipe → match 5 arm → fake fallback` 骨架，
其中 `pipe_or_fake_error` 函数包含了从未执行的 dead branch。

### 2. TauriCommandError 多余包装

`TauriCommandError` 只包裹一个 `String`，通过 `error()` 辅助函数创建，增加了一层不必要的间接。

### 3. 前端状态别名与实际后端脱节

`App.tsx` 的 `stateLabels`/`stateHints` 中有 `attention`/`blocking`/`undiscovered`/`untrusted` 等键，
Rust 后端从不产生这些值。

## 分析过程

1. 对比 5 个 command 的 match 逻辑，发现差异点仅在：
   - `call_pipe` 传入的 command variant
   - match 中提取的 response 字段
   - fake fallback 的返回值

2. `bootstrap_window` 使用两个 pipe 调用且有独立的复合逻辑，不适合参与抽取

3. 前端 `stateLabel()` 和 `stateHint()` 有 `?? value` 和 `?? stateHints.idle` 回退保护，
   删除冗余 key 不会导致崩溃

## 考虑的方案

| 方案 | 选择 |
|------|------|
| 抽取 `call_or_fake` 泛型辅助函数 | ✅ 选用 |
| 只修 dead branch 不改结构 | ❌ 不消除重复 |
| 保留 TauriCommandError | ❌ 用户确认删除 |
| 保留所有状态别名 | ❌ 用户确认清理 |

## 最终方案

1. **`call_or_fake<T>`** — 接收 command + extract 闭包 + fake_fallback 闭包，
   统一处理 pipe/fake 切换和错误处理。5 个 command 从 ~75 行 match 骨架缩减到 ~40 行闭包调用

2. **`call_pipe` 返回 `Result<..., String>`** — 删除了 `TauriCommandError` 和 `error()` 辅助函数，
   错误消息直接用字符串字面量或 `to_string()`

3. **`std::mem::replace` 消除 fake fallback 中的 clone** — `save_settings` 的 fake fallback 从
   `guard.settings.clone()` + `settings.clone()` 改为 `std::mem::replace(&mut guard.settings, settings)`

4. **前端** — 添加 ErrorBoundary，轮询从 1s 改为 5s，删除 4 个冗余状态别名
