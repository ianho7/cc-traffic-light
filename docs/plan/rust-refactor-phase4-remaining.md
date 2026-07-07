# Rust 代码重构 — 剩余项实施计划

## Objective

完成前一轮评审中暂缓的 Rust 代码重构项，包括类型体系统一和平行枚举合并。

**解决的问题：**
- `SourceId` 与 `AgentId` 是完全相同的枚举，平行存在导致转换桥接代码散落
- `ObservationKind` 与 `DetectionMethod` 等价，通过多余的 `method()` 桥接
- `AgentMonitor` 是仅包裹一个字段的薄包装，增加间接层

**预期结果：** 6 个枚举/结构体变体合并为 4 个，消除桥接代码，shared-core 6 个测试全通过。

**范围内：** 纯重构，不改变运行时行为，不改变 JSON 序列化格式。
**范围外：** 不引入新功能，不改动架构分层。

---

## Background and Context

项目有三个 Rust crate：shared-core、taskbar-widget（bin）、taskbar-settings-tauri。

已完成的 Phase 1-5 覆盖了 P0-P3 的大部分问题。剩余项集中在**类型体系统一**——这是 Phase 4 的主体内容，因涉及枚举合并和跨文件修改，拆分到独立 session 处理。

### 已有经验

上一次 session 中已成功完成相似的重构：
- `changed_keys` 从手动 17 字段对比改为 serde_json Value diff
- `source_priority` 中 Codex/Claude 重复分支合并
- `infer_state` 删除两个未用参数

---

## Current State Analysis

### 问题 1: SourceId / AgentId 平行枚举

```rust
// shared-core/ui_state.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceId { Codex, Claude }
impl SourceId {
    pub fn as_str(self) -> &'static str { ... }
}

// agent_state.rs
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentId { Codex, Claude }
impl AgentId {
    pub fn parse(value: &str) -> Option<Self> { ... }
    pub fn as_str(&self) -> &'static str { ... }
}
```

差异点：`AgentId` 有 `parse()` 方法 + 序列化派生。`SourceId` 无序列化、无 parse。两者 `as_str` 输出完全一致（"codex"/"claude"）。

**涉及的源文件：** `agent_state.rs`（定义 AgentId + 多处使用）、`hook_rules.rs`（转为 `parse_task_key` 返回 `AgentId`）、`i18n.rs`（使用 `SourceId`）

### 问题 2: ObservationKind / DetectionMethod 平行枚举

```rust
// detector.rs
pub enum ObservationKind { LogFile, StateFile, SessionFile, Process, HookState }
impl ObservationKind {
    pub fn method(self) -> DetectionMethod { ... }
}

// shared-core/ui_state.rs
pub enum DetectionMethod { LogFile, StateFile, SessionFile, Process, HookState, Unknown }
```

一一对应（差异：DetectionMethod 多一个 `Unknown`）。ObservationKind 的 `method()` 是纯转换函数，全部调用点可以直接使用 DetectionMethod。

**涉及的源文件：** `detector.rs`（定义 ObservationKind + 使用）、`i18n.rs`（使用 DetectionMethod）、`ui_state.rs`（定义 DetectionMethod）

### 问题 3: AgentMonitor 薄包装

```rust
pub struct AgentMonitor {
    pub summary: HookSummary,
}
// HookMonitorState
pub agents: BTreeMap<String, AgentMonitor>,
```

只包裹一个 `HookSummary`，没有任何额外行为或数据。

**涉及的源文件：** `agent_state.rs`（定义 + 使用）、任何读取 `agents` 的代码。

### 问题 4: normalize_config 不必要的 clone（杂项）

```rust
// shared-core/app_config.rs:299
write_config(&path, &normalize_config(config.clone()))
```

`normalize_config` 以值接收参数（`fn normalize_config(mut config: AppConfig)`），但调用处已持有 `config` 所有权。如果改成接受引用并返回新值，调用方可避免 `clone`。

---

## Proposed Solution

三个阶段依次执行，每阶段都是独立的机械重构。

**核心设计原则：**
1. 保留 serde rename 一致性 —— `#[serde(rename_all = "lowercase")]` 在所有合并后的枚举上保持
2. `AgentId` 功能合并到 `SourceId`，删除 `AgentId`
3. `ObservationKind` 替换为 `DetectionMethod`，删除 `ObservationKind`
4. `AgentMonitor` 扁平化为 `HookSummary`

---

## Alternatives Considered

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| 保留 AgentId 别名（type AgentId = SourceId） | 改动最小 | 遗留别名混淆 | 不选 |
| 给 SourceId 加 Serialize/Deserialize 再别名 | 保持类型分离 | 两个类型仍然平行存在 | 不选，直接合并 |
| BTreeMap<String, HookSummary> 替换 AgentMonitor | 最直接 | 需要调整所有引用点和测试 | 选用 |

---

## Implementation Plan

### Phase 1: 合并 SourceId / AgentId

**Goal:** 将 `AgentId` 的功能纳入 `SourceId`，删除 `AgentId`。

**Files:**
- `taskbar-widget/src/agent_state.rs`
- `crates/shared-core/src/ui_state.rs`

**Tasks:**

1. 给 `SourceId` 添加 `#[derive(Serialize, Deserialize)]` 和 `#[serde(rename_all = "lowercase")]`
2. 给 `SourceId` 添加 `parse()` 方法（复制 AgentId 的匹配逻辑）
3. 在 `agent_state.rs` 中，将所有 `AgentId` 替换为 `SourceId`
4. 删除 `agent_state.rs` 中的 `AgentId` 枚举及 `impl AgentId` 块
5. 更新 `parse_task_key()` 的返回类型从 `(AgentId, ...)` 改为 `(SourceId, ...)`
6. 更新所有引用 `AgentId` 的测试代码

**Expected Result:** `SourceId` 承担全部职责，`AgentId` 删除，`cargo test -p shared-core --offline` 通过。

---

### Phase 2: 合并 ObservationKind / DetectionMethod

**Goal:** 用 `DetectionMethod` 替换 `ObservationKind`。

**Files:**
- `taskbar-widget/src/detector.rs`
- `taskbar-widget/src/ui_state.rs`（仅在调用处）

**Tasks:**

1. 在 `detector.rs` 中，将所有 `ObservationKind` 替换为 `DetectionMethod`
2. 更新 `SourceObservation.kind` 字段类型从 `ObservationKind` 改为 `DetectionMethod`
3. 更新 `source_priority()` 函数参数类型
4. 更新所有 `ObservationKind::Foo` 引用为 `DetectionMethod::Foo`
5. 删除 `ObservationKind` 枚举及 `impl ObservationKind` 块
6. 删除 `ObservationKind::method()` 转换函数（不再需要）
7. 确认 `ObservationKind::as_str()` 与 `DetectionMethod::as_str()` 输出一致（两者都是蛇形命名，只有 `DetectionMethod` 多一个 `Unknown => "unknown"`）

**Expected Result:** `ObservationKind` 删除，`source_priority` 直接使用 `DetectionMethod`，`cargo test -p shared-core --offline` 通过。

---

### Phase 3: AgentMonitor 消除 + normalize_config clone

**Goal:** 消除 `AgentMonitor` 薄包装，优化 `normalize_config` clone。

**Files:**
- `taskbar-widget/src/agent_state.rs`
- `crates/shared-core/src/app_config.rs`

**Tasks (AgentMonitor):**

1. 删除 `AgentMonitor` 结构体
2. `HookMonitorState.agents` 从 `BTreeMap<String, AgentMonitor>` 改为 `BTreeMap<String, HookSummary>`
3. 更新 `HookMonitorState::default_at()` 的初始化代码
4. 更新 `refresh_summaries()` 中的 agents 插入代码
5. 检查任何读取 `agents[...].summary` 的代码，改为直接读取 `agents[...]`

**Tasks (normalize_config clone):**

1. 修改 `normalize_config` 签名为 `fn normalize_config(config: &AppConfig) -> AppConfig`
2. 在函数体内先克隆参数再修改（move 语义变更为 clone-then-return）
3. 调用点改为 `normalize_config(config)` 而非 `normalize_config(config.clone())`

**Expected Result:** AgentMonitor 删除，normalize_config 少一次 clone，`cargo test -p shared-core --offline` 通过。

---

## Validation Strategy

每个 Phase 独立验证：

```powershell
cargo test -p shared-core --offline
# 6 个测试全部通过
```

```powershell
cargo check -p taskbar-widget --offline
# 由于资源文件缺失可能失败，但类型错误会被检出
```

核心验证点：
- Phase 1: `AgentId::parse("codex")` → `SourceId::parse("codex")` 行为一致
- Phase 2: `ObservationKind::LogFile.method()` → `DetectionMethod::LogFile` 直接使用
- Phase 2: `as_str()` 输出验证（LogFile → "log_file"，与之前一致）
- Phase 3: `agents["codex"].summary.state` → `agents["codex"].state`

---

## Risks and Mitigations

| 风险 | 影响 | 可能性 | 缓解 |
|------|------|--------|------|
| Phase 1: `#[serde(rename_all)]` 添加后改变 JSON 格式 | `state.json` 反序列化失败 | 低 | `AgentId` 和 `SourceId` 都用 `rename_all = "lowercase"`，当前 `AgentId` 已有该标注，`SourceId` 新增后效果相同 |
| Phase 3: `agents` 从 `AgentMonitor` 改为 `HookSummary` 改变 state.json 结构 | 已有状态文件无法读取 | 中 | `HookMonitorState` 使用 `#[serde(default)]` 处理缺失字段；可考虑保留兼容性的反序列化路径（先用旧格式读取） |
| Phase 1: 测试中使用了 `AgentId::Codex` 的代码 | 编译失败 | 低 | 逐一替换为 `SourceId::Codex` |

---

## Open Questions

无。所有设计决策已在方案中明确。

---

## Recommended Next Step

从 **Phase 1** 开始执行：先给 `SourceId` 添加 `parse()` + 序列化派生，然后全局替换 `AgentId` → `SourceId`，最后删除 `AgentId` 定义。
