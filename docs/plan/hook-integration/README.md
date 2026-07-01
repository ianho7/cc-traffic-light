# Hook 集成计划

这套文档用于定义当前仓库里的 Hook 监控 MVP，让 Win11 任务栏红绿灯组件后续可以直接依赖 Claude Code 和 Codex 的状态信号。

计划范围刻意收窄，只做下面几件事：

- 接收 Claude Code 和 Codex 的 hook 回调。
- 把这些回调归一化写入一个共享状态文件，状态模型提前支持多个并发 task。
- 让 Rust 任务栏组件读取该文件并绘制最小状态指示。
- 暂缓安装器、设置界面、进程检测、多 workspace UI 和复杂面板。

## 文档列表

- [01-mvp-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/01-mvp-plan.md)：MVP 优先的实施计划。
- [02-grill-decisions-and-adr.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/02-grill-decisions-and-adr.md)：设计追问记录、ADR 和术语表。
- [03-hook-adjustment-plan.md](/D:/project/cc-traffic-light/docs/plan/hook-integration/03-hook-adjustment-plan.md)：参考 Electron 实现后，对 Codex notify/lifecycle hooks、规则层拆分、SubagentStop 和在线检测边界的调整计划。
- [hook-integration-checklist.md](/D:/project/cc-traffic-light/docs/checklist/hook-integration-checklist.md)：可执行 implementation checklist。
- [hook-payload-sampling.md](/D:/project/cc-traffic-light/docs/checklist/hook-payload-sampling.md)：Phase 0 payload shape 采样记录；当前记录了人工 payload accepted limitation。
- [hook-integration-validation.md](/D:/project/cc-traffic-light/docs/checklist/hook-integration-validation.md)：构建、CLI、并发、stale、corrupt recovery 和待人工 widget 验证记录。
- [codex-lifecycle-hooks-validation.md](/D:/project/cc-traffic-light/docs/checklist/codex-lifecycle-hooks-validation.md)：Codex 正式 lifecycle hooks 的本机验证 checklist；用于替代继续依赖低保真 `notify`。

## 建议执行顺序

1. 先做 Phase 0：用脱敏采样模式确认 Codex lifecycle hooks 和 Claude Code 真实 hook payload 是否包含 `session_id` 和事件时间/序号字段。
2. 实现 task-aware hook 状态模型、Win32 named mutex 和原子 JSON 存储。
3. 增加 Rust hook CLI，支持 hook 接收以及 debug 用的 `set/clear/list`。
4. 在任务栏组件里加入轮询，根据多个 task 聚合后的 `global_summary` 触发重绘。
5. 补充 Codex lifecycle hooks 和 Claude Code 的示例 hook 配置片段。
6. 先用人工构造的单任务、多任务、乱序和 stale hook 调用验证，再接真实 hook。

## MVP 成功信号

只要本地命令可以模拟多个 Codex 或 Claude Code hook，持续更新共享状态文件，并让正在运行的任务栏组件无需重启就切换到预期的 `global_summary` 红绿灯状态，就算 MVP 成功。
