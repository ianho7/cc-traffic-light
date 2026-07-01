# P3: 任务栏红绿灯 UI

## 目标

把当前任务栏中的文字 widget，演进成一个能直观表达 hook 状态的最小红绿灯组件。预期结果是：在不破坏现有稳定 Win11 attach/render 路径的前提下，能清晰表示 `idle`、`working`、`waiting`、`done`、`error`。

范围内：现有 Win32 widget 内的 GDI 绘制修改、状态到视觉样式的映射、人工可见性验证。范围外：D2D、DirectComposition、多显示器、Explorer 重启恢复、设置界面和通用主题系统。

## 背景与上下文

当前 widget 会在彩色背景上渲染类似 `WORKING 1` 的文本。稳定任务栏路径是 `popup_parented + WS_EX_LAYERED + SetLayeredWindowAttributes(alpha=255, LWA_ALPHA)`。现有仓库规则明确要求：除非必要，不要动 taskbar probing、`SetParent` 或 positioning。

## 当前状态分析

相关文件：

- `taskbar-widget/src/main.rs`：轮询、paint state、`paint_window`、`paint_style`
- `taskbar-widget/src/taskbar.rs`：任务栏探测和定位，当前应保持不动
- `taskbar-widget/src/agent_state.rs`：提供 `global_summary` 和按 agent summary

当前 MVP 只消费 `global_summary`。状态文件已经跟踪多任务，但还没有逐个渲染这些任务。

## 方案建议

保持当前单 widget 区域不变，把“以文字为主”的渲染替换为更直观的状态指示：

- 用彩色圆点或 pill 表示状态
- 可选显示紧凑计数文本
- 可选显示 stale 标记
- MVP 不引入图片，除非后续视觉需求明确要求

第一版 UI 仍然只使用 GDI，并保持当前窗口尺寸不变，避免影响 Win11 任务栏可见性。

## 备选方案

- 渲染图片或图标：视觉更丰富，但会增加资源加载和 DPI 复杂度。
- 立即显示按 agent 分列：后续有价值，但布局成本更高，而且不应早于全局状态链路稳定。
- 切到 D2D/DirectComposition：扩展性更好，但违背当前 MVP 的稳定性边界。

## 实施计划

### Phase 1: 定义视觉契约

- 目标：明确每种状态的颜色、标签、计数和 stale 行为。
- 文件：`taskbar-widget/src/main.rs`、相关文档
- 任务：定义 `idle`、`working`、`waiting`、`done`、`error` 的视觉映射。
- 预期结果：渲染行为无歧义。

### Phase 2: 实现最小状态指示器

- 目标：用 GDI 绘制红绿灯 UI。
- 文件：`taskbar-widget/src/main.rs`
- 任务：重构 `paint_style`，绘制圆点或 pill，并保留当前背景色 fallback 逻辑。
- 预期结果：在任务栏尺寸下，状态变化一眼可见。

### Phase 3: 用 Debug CLI 验证

- 目标：在不依赖真实 hooks 的情况下验证所有状态。
- 文件：状态文件即可
- 任务：用 `taskbar_widget_hook.exe set codex_123 <state>` 触发不同状态，观察任务栏。
- 预期结果：所有状态都能清晰渲染。

### Phase 4: 用真实 Codex Hooks 验证

- 目标：确认真实 hook 事件能驱动 UI。
- 文件：除非必要，不改 taskbar host 逻辑
- 任务：运行 widget，触发 Codex lifecycle events，观察视觉变化。
- 预期结果：真实 lifecycle events 能在 1000 ms 内反映到 UI。

## 验证策略

- `cargo fmt -- --check`
- `cargo check`
- 人工状态矩阵测试：idle、working、waiting、done、error、stale
- 在 P0 之后做真实 Codex hook smoke test
- 在 Win11 任务栏做人工可见性检查

## 风险与缓解

- Risk: 组件太小，状态难以读懂。Mitigation: 先按真实任务栏尺寸测试，再决定是否增加复杂度。
- Risk: GDI 改动影响可见性。Mitigation: 不改变窗口样式、父子关系、layered 模式、锚点或位置。
- Risk: 前景和背景对比不足。Mitigation: 第一版先用高对比固定颜色。

## 待确认问题

- MVP 是显示“圆点 + 文本”，还是先只显示圆点？
- 在 `global_summary` 稳定前，是否应该提前展示按 agent 状态？

## 推荐下一步

等 P0 证明真实状态写入链路打通后，在 `taskbar-widget/src/main.rs` 中实现基于 GDI 的圆点或 pill 指示器，不动 `taskbar.rs`。
