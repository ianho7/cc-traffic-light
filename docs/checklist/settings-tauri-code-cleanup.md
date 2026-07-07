#  checklist: settings-tauri 代码清理

## Checklist Objective

基于上一轮 code review 结果，对 `taskbar-settings-tauri` crate 的 Rust 后端和 TypeScript 前端进行清理重构。

**目标成果：**

- 消除 `pipe_or_fake_error` 的 dead branch
- 将 5 个重复的 command 骨架抽取为公共辅助函数
- 移除 `TauriCommandError` 多余包装
- 添加 React ErrorBoundary
- 清理前端冗余的状态别名
- 优化轮询间隔

**范围内：** `src-tauri/src/lib.rs`、`src/App.tsx`、`src/types.ts`
**范围外：** Phase 4 类型合并（已有独立计划）、i18n 迁移到独立 JSON（工作量较大，暂缓）

---

## Pre-Implementation Checks

- [ ] 确认 `taskbar-settings-tauri` 的 Rust 代码能独立编译（`cargo check -p taskbar-settings-tauri --offline` 失败时可接受，因需要 Tauri 运行时）
- [ ] 确认前端能独立构建（`pnpm build` 通过）
- [ ] 阅读 `lib.rs` 中 5 个 command 的完整 match 骨架，确认差异点
- [ ] 确认 `pipe_or_fake_error` 的所有调用路径

---

## Implementation Checklist

### Phase 1: 修复 dead branch 与重复骨架

- [ ] **1a: 抽取 `call_or_fake` 辅助函数** — 将 `call_pipe → match 5 arm → fake fallback` 模式抽取为泛型辅助函数，统一处理 5 个 command
- [ ] **1b: 修复 `pipe_or_fake_error` dead branch** — 如果 1a 未覆盖，单独删除不可达的 if 分支
- [ ] **1c: 消除 `TauriCommandError`** — 将 command 返回类型改为 `Result<T, String>`，删除 `error()` 辅助函数和 `TauriCommandError` 结构体

### Phase 2: 减少不必要的 clone

- [ ] **2a: `save_settings` 减少 clone** — 将 `call_pipe` 的参数改为引用传递，消除 `settings.clone()` 的第一次拷贝

### Phase 3: 前端清理

- [ ] **3a: 添加 React ErrorBoundary** — 包裹 `App` 组件，捕获渲染异常并显示错误恢复界面
- [ ] **3b: 调整轮询间隔为 5 秒** — 修改 `setInterval` 的毫秒参数
- [ ] **3c: 清理冗余状态别名** — 删除 `stateLabels` / `stateHints` 中 Rust 端不会产生的键（`attention`、`blocking`、`undiscovered` 等），或确认是否真的需要并添加注释说明来源

---

## Validation Checklist

- [ ] **TypeScript 构建** — `cd taskbar-settings-tauri && pnpm build` 无错误
- [ ] **Rust 检查** — `cargo check -p shared-core --offline` 通过
- [ ] **轮询间隔确认** — 修改 1s → 5s 后 `App.tsx:530` 的 `setInterval` 参数已更新
- [ ] **ErrorBoundary 测试** — 在前端 console 中制造一个未捕获异常，确认 ErrorBoundary 显示 fallback UI，而非白屏
- [ ] **状态别名清理确认** — 执行 `grep -r "attention\|blocking\|undiscovered\|attached\|tray_only" taskbar-settings-tauri/src/App.tsx` 检查剩余引用，确认删除后没有 dangling reference

---

## Documentation Checklist

- [ ] 在 `docs/reflections/` 下为每个 Completed phase 生成 reflection 文档

---

## Cleanup Checklist

- [ ] 删除 `TauriCommandError` 和 `error()` 函数（如 Phase 1 已完成）
- [ ] 删除 `pipe_or_fake_error` 函数（如 1a 已消除其调用点）

---

## Completion Criteria

- [ ] 5 个 command 的重复 match 骨架已抽取为 `call_or_fake` 辅助函数
- [ ] `pipe_or_fake_error` 中的 dead branch 已消除
- [ ] `TauriCommandError` 已替换为 `Result<T, String>`
- [ ] 前端有 ErrorBoundary fallback
- [ ] 轮询间隔已改为 5 秒
- [ ] 冗余状态别名已清理
- [ ] 零功能行为变化
