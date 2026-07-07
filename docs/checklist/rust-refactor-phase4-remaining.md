# Rust 重构 Phase 4 剩余项 — 执行清单

## Checklist Objective

完成前一轮评审中暂缓的 Rust 代码重构项，包括类型体系统一和平行枚举合并。

**目标：** 将 6 个枚举/结构体变体合并为 4 个，消除桥接代码，shared-core 6 个测试全部通过。

**范围：**
- 合并 `SourceId` / `AgentId` 平行枚举 → 统一为 `SourceId`
- 合并 `ObservationKind` / `DetectionMethod` 平行枚举 → 统一为 `DetectionMethod`
- 消除 `AgentMonitor` 薄包装 → 直接使用 `HookSummary`
- 优化 `normalize_config` 不必要的 `clone()`

**非目标：**
- 不改变运行时行为
- 不改变 JSON 序列化格式
- 不引入新功能
- 不改动架构分层
- 不改变 `taskbar-settings-tauri` 任何代码（该 crate 当前不引用 `AgentId`）

---

## Pre-Implementation Checks

- [ ] P1.1: 确认目标源文件全部存在且可读
  - `crates/shared-core/src/ui_state.rs` — 定义 `SourceId`、`DetectionMethod`
  - `taskbar-widget/src/agent_state.rs` — 定义 `AgentId`、`AgentMonitor`，多处使用
  - `taskbar-widget/src/detector.rs` — 定义 `ObservationKind`，多处使用
  - `taskbar-widget/src/bin/taskbar_widget_hook.rs` — 使用 `AgentId`
  - `crates/shared-core/src/app_config.rs` — `normalize_config`
- [ ] P1.2: 确认验证命令可运行
  - `cargo test -p shared-core --offline`
  - `cargo check -p taskbar-widget --offline`
- [ ] P1.3: 确认无其他 crate 引用 `AgentId` / `ObservationKind` / `AgentMonitor`
  - `taskbar-settings-tauri` 已确认无引用
- [ ] (执行后自动生成 `docs/reflections/task-RR-P4-PRE-<timestamp>.md`)

---

## Implementation Checklist

### Phase 1: 合并 SourceId / AgentId (预计 8 步)

**目标：** 将 `AgentId` 的所有功能（`parse()`、`Serialize`/`Deserialize`、`#[serde(rename_all = "lowercase")]`）合并到 `SourceId`，全局替换 `AgentId` → `SourceId`，删除 `AgentId`。

- [ ] 1.1: 给 `SourceId` 添加 `#[derive(Serialize, Deserialize)]` 和 `#[serde(rename_all = "lowercase")]`
  - 文件：`crates/shared-core/src/ui_state.rs`
  - 操作：在 `#[derive(...)]` 行追加 `Serialize, Deserialize`，在 `pub enum SourceId` 前添加 `#[serde(rename_all = "lowercase")]`
  - 验证：`cargo check -p shared-core --offline` 通过
- [ ] 1.2: 给 `SourceId` 添加 `parse()` 方法
  - 文件：`crates/shared-core/src/ui_state.rs`
  - 操作：在 `impl SourceId` 块中添加 `pub fn parse(value: &str) -> Option<Self>`（复制 `AgentId::parse` 的逻辑，包括 `"claude" | "claude_code" | "claudecode"` 映射到 `Claude`）
  - 注意：`SourceId` 目前没有 `use serde::{Serialize, Deserialize}` — 该 derive macro 来自 shared-core 的 Cargo.toml 依赖
- [ ] 1.3: 在 `agent_state.rs` 中将所有 `AgentId` 替换为 `SourceId`
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 具体位置：
    - 行 30: `pub enum AgentId` → 删除此枚举定义
    - 行 35-49: `impl AgentId` → 删除整个 impl 块
    - 行 87: `pub agent: AgentId` → `pub agent: SourceId`
    - 行 135: `pub agent: AgentId` → `pub agent: SourceId`
    - 行 421: `for agent in [AgentId::Codex, AgentId::Claude]` → `for agent in [SourceId::Codex, SourceId::Claude]`
    - 行 434: `agent_filter: Option<AgentId>` → `agent_filter: Option<SourceId>`
    - 行 486: `fn parse_task_key(...) -> (AgentId, ...)` → `(SourceId, ...)`
    - 行 488: `AgentId::Claude` → `SourceId::Claude`
    - 行 490: `AgentId::Codex` → `SourceId::Codex`
  - 验证：`cargo check -p taskbar-widget --offline` 检查类型错误
- [ ] 1.4: 删除 `agent_state.rs` 中 `AgentId` 枚举定义和 `impl AgentId` 块
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 操作：删除行 28-50（整个 enum AgentId + impl AgentId）
  - 风险：如果有遗漏引用，编译会报错，逐一修复即可
- [ ] 1.5: 更新 `taskbar_widget_hook.rs` 中的 `AgentId` 引用为 `SourceId`
  - 文件：`taskbar-widget/src/bin/taskbar_widget_hook.rs`
  - 行 8: `use taskbar_widget::agent_state::{self, AgentId, AgentState, HookEventUpdate};` → 移除 `AgentId` 引入，或改为引入 `SourceId`
  - 行 90: `AgentId::parse(...)` → `SourceId::parse(...)`
  - 行 128: `AgentId::Codex` → `SourceId::Codex`
  - 注意：`SourceId` 定义在 `shared-core::ui_state` 中，需要确认 `taskbar_widget_hook.rs` 的引入路径
- [ ] 1.6: 更新 `detector.rs` 中的隐式 `AgentId` 引用（如果有）
  - 文件：`taskbar-widget/src/detector.rs`
  - 操作：检查是否有 `AgentId::Codex` / `AgentId::Claude` 或 `use ... AgentId` 引用
- [ ] 1.7: 检查 `ui_state.rs` 中测试代码是否用到 `AgentId`
  - 文件：`crates/shared-core/src/ui_state.rs`
  - 操作：目前测试仅用 `SourceId::Codex`，确认无问题
- [ ] 1.8: 验证 Phase 1 编译通过
  - 命令：`cargo test -p shared-core --offline`
  - 命令：`cargo check -p taskbar-widget --offline`
  - (执行后自动生成 `docs/reflections/task-RR-P4-P1-<timestamp>.md`)

### Phase 2: 合并 ObservationKind / DetectionMethod (预计 7 步)

**目标：** 用 `DetectionMethod`（来自 shared-core）替换 `taskbar-widget` 中本地的 `ObservationKind` 枚举，删除 `ObservationKind` 及 `method()` 桥接函数。

- [ ] 2.1: 将 `detector.rs` 中所有 `ObservationKind` 引用替换为 `DetectionMethod`
  - 文件：`taskbar-widget/src/detector.rs`
  - 具体位置：
    - 行 16-44: `pub enum ObservationKind` + `impl ObservationKind` → 删除
    - 行 49: `pub kind: ObservationKind` → `pub kind: DetectionMethod`
    - 行 107: `ObservationKind::StateFile` → `DetectionMethod::StateFile`
    - 行 204: `fn source_priority(_source_id: SourceId, kind: ObservationKind)` → `kind: DetectionMethod`
    - 行 206-210: match 分支用 `DetectionMethod::...` 替换，注意 `DetectionMethod::Unknown` 需要添加对应优先级
    - 行 227: `kind: ObservationKind` → `kind: DetectionMethod`
    - 行 258: `kind: ObservationKind::Process` → `kind: DetectionMethod::Process`
  - 注意：`detector.rs` 的引入语句（行 7）已有 `DetectionMethod` — 确认 `use crate::ui_state::{... DetectionMethod ...}` 存在
- [ ] 2.2: 删除 `ObservationKind` 枚举定义及整个 `impl ObservationKind` 块
  - 文件：`taskbar-widget/src/detector.rs`
  - 操作：删除行 15-44
- [ ] 2.3: 处理 `Unknown` 变体 — `source_priority` 需要为 `DetectionMethod::Unknown` 赋值优先级
  - 文件：`taskbar-widget/src/detector.rs`
  - 操作：在 match 中添加 `DetectionMethod::Unknown => 0`（优先级最低）
- [ ] 2.4: 删除 `kind.method()` 调用点，改为直接使用 `DetectionMethod`
  - 文件：`taskbar-widget/src/detector.rs`
  - 位置：行 171 (`best.kind.method()`) 和行 181 (`best.kind.method()`) — `kind` 现已是 `DetectionMethod`，直接使用 `best.kind` 即可，无需 `.method()` 调用
- [ ] 2.5: 检查 `detector.rs` 中 `use` 语句是否需要补充 `DetectionMethod`
  - 确认行 6-9 的 `use crate::ui_state::{}` 已包含 `DetectionMethod`
- [ ] 2.6: 验证 `as_str()` 输出一致性
  - `ObservationKind::as_str()` 输出：`"log_file"`, `"state_file"`, `"session_file"`, `"process"`, `"hook_state"`
  - `DetectionMethod::as_str()` 输出完全一致（多一个 `"unknown"`）
  - ✓ 替换安全
- [ ] 2.7: 验证 Phase 2 编译通过
  - 命令：`cargo test -p shared-core --offline`
  - 命令：`cargo check -p taskbar-widget --offline`
  - (执行后自动生成 `docs/reflections/task-RR-P4-P2-<timestamp>.md`)

### Phase 3: AgentMonitor 消除 & normalize_config 优化 (预计 7 步)

**目标：** 消除 `AgentMonitor` 薄包装结构体，直接在 `BTreeMap` 中存储 `HookSummary`；优化 `normalize_config` 不必要的 clone。

- [ ] 3.1: 删除 `AgentMonitor` 结构体定义
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 操作：删除行 111-114 (`pub struct AgentMonitor { pub summary: HookSummary }`)
- [ ] 3.2: 将 `HookMonitorState.agents` 字段类型从 `BTreeMap<String, AgentMonitor>` 改为 `BTreeMap<String, HookSummary>`
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 位置：行 129: `pub agents: BTreeMap<String, AgentMonitor>` → `pub agents: BTreeMap<String, HookSummary>`
- [ ] 3.3: 更新 `HookMonitorState::default_at()` 初始化代码
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 位置：行 182-193
  - 操作：改为 `agents.insert("claude".to_string(), idle.clone())` 和 `agents.insert("codex".to_string(), idle.clone())`（直接存 `HookSummary`，不再包 `AgentMonitor { summary: ... }`）
- [ ] 3.4: 更新 `refresh_summaries()` 中的 agents 插入代码
  - 文件：`taskbar-widget/src/agent_state.rs`
  - 位置：行 421-428
  - 操作：改为 `state.agents.insert(agent.as_str().to_string(), summarize_tasks(...))`，去掉 `AgentMonitor { summary: ... }` 包装
- [ ] 3.5: 更新所有读取 `agents[...].summary` 的代码
  - 文件：`taskbar-widget/src/detector.rs`
  - 位置：行 104: `result.state.agents.get(key).map(|monitor| &monitor.summary)` → `result.state.agents.get(key)`（直接返回 `&HookSummary`）
- [ ] 3.6: 优化 `normalize_config` clone
  - 文件：`crates/shared-core/src/app_config.rs`
  - 行 389: `fn normalize_config(mut config: AppConfig) -> AppConfig` → `fn normalize_config(config: &AppConfig) -> AppConfig`
  - 行 389-397: 函数体内改为 `let mut config = config.clone();` 开头（将 clone 移到函数内部），然后修改并返回
  - 行 299: `normalize_config(config.clone())` → `normalize_config(&config)`
- [ ] 3.7: 验证 Phase 3 编译通过
  - 命令：`cargo test -p shared-core --offline`
  - 命令：`cargo check -p taskbar-widget --offline`
  - (执行后自动生成 `docs/reflections/task-RR-P4-P3-<timestamp>.md`)

---

## Validation Checklist

- [ ] V.1: `cargo test -p shared-core --offline` (6 个测试全通过)
  - 预期：6/6 通过
  - 如果失败：检查测试中是否使用了 `AgentId` / `ObservationKind` / `AgentMonitor`
- [ ] V.2: `cargo check -p taskbar-widget --offline` (类型检查通过)
  - 预期：0 错误，仅可能有 warning
  - 如果失败：检查遗漏的引用替换
- [ ] V.3: `cargo test --workspace --offline` (工作空间测试)
  - 预期：全部通过
- [ ] V.4: `cargo build -p taskbar-widget --offline` (完整构建)
  - 预期：构建成功
- [ ] V.5: Phase 1 核心行为验证
  - `SourceId::parse("codex")` 返回 `Some(SourceId::Codex)`
  - `SourceId::parse("claude_code")` 返回 `Some(SourceId::Claude)`
  - `SourceId::parse("invalid")` 返回 `None`
  - JSON 序列化 `{"codex": ...}` 格式不变
- [ ] V.6: Phase 2 核心行为验证
  - `source_priority(DetectionMethod::LogFile)` 返回 0 (最低)
  - `source_priority(DetectionMethod::HookState)` 返回 4 (最高)
  - `DetectionMethod::as_str()` 在全部变体上输出正确
- [ ] V.7: Phase 3 核心行为验证
  - `agents["codex"].state` 替代 `agents["codex"].summary.state`
  - `normalize_config(&config)` 不产生额外 clone
- [ ] V.8: 序列化兼容性验证
  - `AgentId` 和 `SourceId` 的 JSON 输出在 serde 下格式一致（两者都是 `rename_all = "lowercase"`）
  - `state.json` 反序列化不受 `AgentMonitor` → `HookSummary` 变化影响（差一个包装层，但字段名 `agents` 不变，值变成扁平 `HookSummary`）
  - `#[serde(default)]` 已在 `HookMonitorState` 上使用？如无，需确保旧 `state.json` 兼容
  (执行后自动生成 `docs/reflections/task-RR-P4-VAL-<timestamp>.md`)

---

## Documentation Checklist

- [ ] D.1: 更新 `docs/plan/rust-refactor-phase4-remaining.md` 状态（标记为已完成）
- [ ] D.2: 在 `docs/reflections/` 中为每个 Phase 生成反思文档
- [ ] D.3: 如果影响了 `AGENTS.md` 中描述的模块边界，更新相关描述

---

## Cleanup Checklist

- [ ] C.1: 确认无 `AgentId`、`ObservationKind`、`AgentMonitor` 残留引用
  - 命令：`rg "AgentId" taskbar-widget/src/` (预期 0 结果)
  - 命令：`rg "ObservationKind" taskbar-widget/src/` (预期 0 结果)
  - 命令：`rg "AgentMonitor" taskbar-widget/src/` (预期 0 结果)
- [ ] C.2: 确认 `ObservationKind::method()` 桥接函数全局删除
- [ ] C.3: 确认 `kind.method()` 调用全部改为直接使用 `kind`
- [ ] C.4: 确保无多余的空 `impl` 块或未使用的 use 导入

---

## Completion Criteria

| 标准 | 要求 |
|------|------|
| 行为 | 6 个枚举/结构体变体合并为 4 个，桥接代码消除 |
| 编译 | `cargo test -p shared-core --offline` + `cargo check -p taskbar-widget --offline` 通过 |
| 序列化 | JSON 格式不变，旧 state.json 可反序列化 |
| 文档 | `docs/reflections/` 中每 Phase 有反思文档 |
| 已知限制 | `AgentMonitor` 消除后，直接反序列化旧 `state.json`（含 `agents.X.summary` 层级）会失败。`HookMonitorState` 当前使用默认反序列化，旧格式含多余包装层级时可能被 `#[serde(default)]` 吞掉或解析失败。如果需要向后兼容，需添加自定义反序列化路径或手动处理。 |

---

## 任务 ID 前缀

- Phase 1 任务: `RR-P4-P1-01` ~ `RR-P4-P1-08`
- Phase 2 任务: `RR-P4-P2-01` ~ `RR-P4-P2-07`
- Phase 3 任务: `RR-P4-P3-01` ~ `RR-P4-P3-07`
- 验证任务: `RR-P4-VAL-01` ~ `RR-P4-VAL-08`
- 文档任务: `RR-P4-DOC-01` ~ `RR-P4-DOC-03`
- 清理任务: `RR-P4-CLN-01` ~ `RR-P4-CLN-04`

每个任务完成时自动生成 `docs/reflections/task-<task-id>-<timestamp>.md`。
