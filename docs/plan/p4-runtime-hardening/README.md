# P4: 运行时硬化与性能

## 目标

对 hook 监控链路做长期运行验证和硬化：覆盖 Codex/Claude 并发会话、高频事件、stale 清理、状态文件损坏恢复、hook 失败和 widget 轮询成本。预期结果是：可以对任务栏红绿灯长期运行有足够信心，不会污染状态，也不会带来明显开销。

范围内：性能预算、压力测试、恢复行为、日志边界和安装回滚验证。范围外：新增 UI 功能、云端同步，以及把文件桥接替换成 IPC。

## 背景与上下文

当前设计使用短生命周期 hook 进程写共享 JSON 状态文件，并通过 Win32 named mutex 做保护。任务栏 widget 每 1000 ms 轮询一次状态文件。这条路径有意保持简单，适合 MVP，但在进入产品化前必须做压力验证。

## 当前状态分析

相关文件：

- `taskbar-widget/src/agent_state.rs`：mutex、原子写入、TTL/stale、损坏恢复
- `taskbar-widget/src/main.rs`：1000 ms 轮询和 summary 变化时重绘
- `taskbar-widget/src/bin/taskbar_widget_hook.rs`：hook 进程入口
- 未来来自 P1 的安装脚本

当前已知限制：

- Codex 没有观测到独立 `event_order` 字段，因此当前使用 `received_at` 排序兜底。
- 状态文件使用 pretty JSON，便于检查，但比 compact JSON 更大。
- 当前 diagnostics 还比较克制。

## 方案建议

只在有证据的地方做针对性验证和小范围硬化，保留文件桥接和 named mutex。先定义性能预算，再决定是否优化。

建议预算：

- hook CLI 正常执行耗时：不含进程启动波动时，尽量低于 100 ms
- 正常使用下状态文件大小：尽量低于 256 KB
- widget 轮询：summary 未变化时，不触发重绘
- mutex wait timeout：默认保持 2000 ms，除非压力测试证明不足

## 备选方案

- 把状态文件替换成本地 HTTP/IPC：延迟更低，但生命周期风险更高，当前不需要。
- 不做压力测试：现在更快，但很容易把并发 bug 带进产品。
- 完成态 task 立即清理：UI 更简单，但会丢失调试证据。

## 实施计划

### Phase 1: 定义压力测试夹具

- 目标：生成可重复的多任务 hook 序列。
- 文件：新增到 `taskbar-widget/scripts/` 的脚本，或配套 checklist 文档
- 任务：模拟多个 Codex 和 Claude session、快速 tool events、stale events、损坏状态文件。
- 预期结果：获得可重复执行的压力场景。

### Phase 2: 测量 Hook 与轮询开销

- 目标：建立基线性能数据。
- 文件：`docs/checklist/` 下的验证文档，必要时增加 diagnostics 字段
- 任务：测量大量 hook CLI 调用和 widget 轮询行为。
- 预期结果：拿到基线数据和明确的 pass/fail 阈值。

### Phase 3: 验证恢复能力

- 目标：确认坏状态和 hook 失败下的安全行为。
- 文件：`agent_state.rs`、状态文件 fixture
- 任务：人为损坏状态文件、制造 mutex 竞争、模拟缺失 session id、模拟 stale event order。
- 预期结果：能触发损坏备份、忽略旧事件，summary 仍保持有效。

### Phase 4: 安装回滚测试

- 目标：确保全局 hook 安装可以撤销。
- 文件：P1 产生的安装脚本
- 任务：在 fixture 用户配置和获得许可后的真实用户配置上执行 apply/restore 测试。
- 预期结果：不会丢失用户原 hooks。

## 验证策略

- `cargo fmt -- --check`
- `cargo check`
- 大量 hook 写入的压力脚本
- P1 完成后做真实并发 Codex 会话测试
- 状态文件损坏测试
- 长时间运行 widget 的人工观察

## 风险与缓解

- Risk: 并发 hooks 在状态文件上竞争。Mitigation: named mutex + 压力测试。
- Risk: task 数量增多后轮询开销升高。Mitigation: 测量状态文件大小和 summary 刷新成本。
- Risk: 没有 event order 时排序不够精确。Mitigation: MVP 先接受 `received_at`；只有在测试暴露问题时，才增加更复杂的序列机制。
- Risk: 日志泄露 payload 信息。Mitigation: diagnostics 保持简短且脱敏。

## 待确认问题

- MVP 需要支持的本地并发 agent 会话上限是多少？
- 如果状态文件变大，是否需要从 pretty JSON 切到 compact JSON？

## 推荐下一步

在 P0 和 P1 完成后，写一个压力脚本，至少模拟 10 个 session、100 条混合 Codex/Claude 事件，并验证 summary 正确性。
