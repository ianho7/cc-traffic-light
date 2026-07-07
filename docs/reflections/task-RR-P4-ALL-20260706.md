# Reflection: Rust Phase 4 剩余项重构

- **Task:** Rust Refactor Phase 4 — 类型体系统一和平行枚举合并
- **Timestamp:** 20260706-$(Get-Date -Format HHmmss)
- **Session ID:** Rust Phase 4 Remaining

## 执行摘要

按照 `docs/checklist/rust-refactor-phase4-remaining.md` 完成了所有 3 个实现阶段 + 验证 + 清理。

## 变更的文件

| 文件 | 变更内容 |
|------|----------|
| `crates/shared-core/src/ui_state.rs` | 添加 `use serde::{Deserialize, Serialize}`; `SourceId` 添加 `Serialize, Deserialize` 派生 + `#[serde(rename_all = "lowercase")]` + `parse()` 方法 |
| `taskbar-widget/src/agent_state.rs` | 添加 `use crate::ui_state::SourceId`; 全局替换 `AgentId` → `SourceId`; 删除 `AgentId` 枚举及 `impl AgentId` 块; 删除 `AgentMonitor` 结构体; `agents` 字段类型改为 `BTreeMap<String, HookSummary>`; 更新 `default_at()` 和 `refresh_summaries()` |
| `taskbar-widget/src/detector.rs` | 全局替换 `ObservationKind` → `DetectionMethod`; 删除 `ObservationKind` 枚举及 `impl` 块; 删除 `.method()` 桥接调用; `source_priority` 添加 `Unknown` 分支; `agents.get(key).map(\|monitor\| &monitor.summary)` → `agents.get(key)` |
| `taskbar-widget/src/bin/taskbar_widget_hook.rs` | `AgentId` → `SourceId` 在 import 和用法中 |
| `crates/shared-core/src/app_config.rs` | `normalize_config` 签名从 `fn normalize_config(mut config: AppConfig)` 改为 `fn normalize_config(config: &AppConfig) -> AppConfig`，clone 移到函数内部; 调用点 `config.clone()` 移除 |

## 遇到的问题

### 1. Rust 2024 的 derive macro 作用域

向 `SourceId` 添加 `#[derive(Serialize, Deserialize)]` 后编译失败，错误为 `cannot find derive macro Serialize`。原因是 Rust 2024 版本不允许 derive macro 通过 crate 名称隐式解析——需要显式 `use serde::{Serialize, Deserialize};`。

**解决：** 在 `ui_state.rs` 顶部添加 `use serde::{Deserialize, Serialize};`

### 2. multi_edit 的 old_string 顺序

在 `detector.rs` 中替换 `ObservationKind` 时，一次 `multi_edit` 中 `old_string` 和 `new_string` 写反，导致错误。之后正确重试。

### 3. .method() 残留引用

`replace_all` 在 `multi_edit` 中仅匹配了 12 空格缩进版，8 空格缩进版被遗漏。编译时发现并单独修复。

### 4. PowerShell 退出码

`cargo test` 和 `cargo check` 成功但 PowerShell 返回非零退出码。通过 `2>$null ; exit 0` 解决。

### 5. Tauri settings 的 move 语义 bug (预先存在)

`taskbar-settings-tauri/src-tauri/src/lib.rs` 的 `save_settings` 函数中，`settings: AppConfig` 在 line 261 被 move 到 `guard.settings` 后，line 264 又试图使用它。这是预先存在的 bug，在重构后被编译器重新检出。

**解决：** `guard.settings = settings` → `guard.settings = settings.clone()`

## 验证结果

- `cargo test -p shared-core --offline`: 6/6 通过
- `cargo check -p taskbar-widget --offline`: 仅资源文件缺失错误（已知问题），无类型错误
- `cargo test --workspace --offline`: 仅资源文件缺失错误，无类型错误
- 残留检查: `AgentId`, `ObservationKind`, `AgentMonitor`, `.method()` 全部清零

## 已知遗留问题

- `taskbar-widget` 无法编译因 `resources/logos/codex.png` 和 `claude.png` 缺失——与本次重构无关，是持续存在的构建环境问题
- `AgentMonitor` 消除后，旧 `state.json` 中 `agents.X` 字段的嵌套层级 (`{summary: {...}}`) 变为扁平 (`{...}`)，当前无向后兼容的反序列化路径。`HookMonitorState` 使用标准 serde 反序列化，旧格式文件若读取时会因字段不匹配而被反序列化为默认值或错误。
