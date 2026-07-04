# Tauri Settings Migration Checklist

日期：2026-07-03

## Checklist Objective

把当前 `taskbar-widget` 里的 settings UI 从 `Slint` 迁移到 `Tauri + React + PNPM`，同时保留现有任务栏 widget、tray、Win32 宿主和检测主循环不变。

目标结果：

- `settings` 改成独立的 `Tauri` 窗口进程，由 tray 或主进程按需拉起。
- `taskbar-widget` 继续作为原生宿主，保留 widget、tray、Win32 message loop、detector 轮询和 taskbar attach 逻辑。
- 配置模型、状态快照、设置读写逻辑下沉到 `shared-core`，避免 UI 迁移时再次复制业务逻辑。
- Tauri UI 在视觉上尽量贴近 [cc_traffic_light_nothing_demo_strict.html](/D:/project/cc-traffic-light/docs/ui/cc_traffic_light_nothing_demo_strict.html) 的结构和节奏，但不为了像素级还原破坏实现稳定性。
- 设置修改需要即时生效；持久化仍沿用现有本地配置行为。

范围：

- 覆盖 `taskbar-widget/` 中与 settings 打开、配置读写、状态导出、IPC、生命周期相关的代码。
- 新增 `Tauri` settings 工程、前端工程、共享 Rust core crate、文档与归档目录。
- 覆盖迁移过程中的验证、回退、归档和交接文档。

非目标：

- 不把任务栏 widget 改成 Tauri。
- 不重写 `taskbar-widget/src/taskbar.rs`。
- 不把 tray 或主循环迁移到 WebView。
- 不追求一次性重构所有 runtime，只迁移 settings UI 及其直接依赖边界。
- 不在第一阶段引入自动更新、云同步、多窗口编排或复杂插件架构。

## Loop Spec

Goal:

- 交付一个可运行的 `Win32 host + Tauri settings` 混合架构。
- 证据是：settings 能从 tray 打开，能显示真实状态，能写入配置，改动能即时影响宿主行为，且 `cargo check` / `cargo build` / `pnpm build` / 手工验证通过。

State:

- 事实源头是代码仓库、`AppConfig`、`AppStatusSnapshot`、共享 core 接口定义、Tauri IPC 契约、验证记录和 reflection 文档。
- 每轮必须保留：当前阶段、已切换的入口、剩余 Slint 依赖、验证结果、阻塞点、回退方案。

Planner:

- 每轮只做一个最小可验证阶段，优先选择“能改变验证状态”的任务。
- 若某阶段验证失败，先判断是 UI 问题、IPC 问题、共享 core 问题，禁止跨三层同时盲改。

Actor:

- 允许的动作：增量重构 Rust 模块、创建 Tauri 工程、抽离 shared-core、建立 IPC、调整 tray 打开逻辑、更新文档。
- 高风险动作：删除旧 Slint 路径、改动配置 schema、改动 settings 打开入口、打包结构调整。

Observer:

- 记录 `cargo check`、`cargo build`、`pnpm install`、`pnpm build`、Tauri 构建输出、tray 打开行为、设置读写行为、即时生效行为。

Verifier:

1. 聚焦编译验证：`cargo check`
2. 共享逻辑验证：相关 Rust 单测
3. 前端验证：`pnpm build`
4. Tauri 验证：`cargo build` 或对应 `tauri build/dev`
5. 运行验证：tray 打开、状态显示、配置写回、即时生效、重复开关窗口

Failure Semantics:

- 环境类问题只重试一次。
- 同一阶段连续两次失败必须回到边界设计或契约层重审，不允许继续堆补丁。
- 若 Tauri 生命周期或 IPC 稳定性无法在保守成本内收敛，允许阶段性停在“并行可用、未切默认入口”状态。

Exit Conditions:

- Success：默认 settings 已切到 Tauri，Slint 不再承担主入口，验证闭环通过。
- Blocked：Tauri 打包/生命周期/IPC 在当前仓库结构下需要额外产品决策或环境前提。
- Risk：若发现迁移会波及 widget/tray 主链路稳定性，则暂停并缩回边界。
- Human takeover：需要你决定 UI fidelity、打包形态或是否彻底删除 Slint 时。

Policy:

- 严守“只迁 settings UI，保留 widget 现状”。
- 优先复用已有配置、状态和 detector 逻辑。
- 默认保留旧 Slint 路径直到 Tauri 路径通过验收。

## Pre-Implementation Checks

- [x] TSM-PRE-01 阅读 [taskbar-widget/src/main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)、[settings_slint.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_slint.rs)、[settings_window.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_window.rs)，确认当前 settings 打开链路和 fallback 链路。
- [x] TSM-PRE-02 阅读 [taskbar-widget/src/app_config.rs](/D:/project/cc-traffic-light/taskbar-widget/src/app_config.rs)、[ui_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/ui_state.rs)、[runtime_contract.rs](/D:/project/cc-traffic-light/taskbar-widget/src/runtime_contract.rs)，确认共享 core 的候选边界。
- [x] TSM-PRE-03 对照 [cc_traffic_light_nothing_demo_strict.html](/D:/project/cc-traffic-light/docs/ui/cc_traffic_light_nothing_demo_strict.html)、[nothing-signal-console-spec.md](/D:/project/cc-traffic-light/docs/ui/nothing-signal-console-spec.md)、[nothing-signal-console-checklist.md](/D:/project/cc-traffic-light/docs/ui/nothing-signal-console-checklist.md)，冻结 UI fidelity 基线。
- [x] TSM-PRE-04 确认当前最小验证命令集合：`cargo check`、相关 Rust tests、`pnpm build`、必要时 `cargo build`。
- [x] TSM-PRE-05 确认仓库目前不是 Cargo workspace；若引入 `shared-core` / Tauri crate，需要先设计根级 workspace 改造顺序。
- [x] TSM-PRE-06 确认归档策略：旧 Slint settings 逻辑在最终阶段迁移到 `archive/`，不是直接删除。
- [x] TSM-PRE-07 确认 reflection 命名仍使用 `docs/reflections/task-<task-id>-<timestamp>.md`。

## Implementation Checklist

### Phase 1: Scope Freeze and Architecture Baseline

- [x] TSM-A-01 固化迁移边界文档：`Win32 host + Tauri settings + shared-core`，明确哪些模块保留原地不动。
- [x] TSM-A-02 盘点当前 settings 相关职责，拆成：配置 schema、状态快照、设置命令、窗口入口、字符串/i18n、UI 渲染。
- [x] TSM-A-03 列出所有当前 settings 页面的真实字段、写入行为、立即生效要求、是否需要轮询刷新。
- [x] TSM-A-04 确定共享 core 的第一批内容：`AppConfig`、`AppStatusSnapshot` 投影、settings service、只读/读写 DTO、错误类型。
- [x] TSM-A-05 明确迁移阶段的回退策略：默认保留 Slint host，直到 Tauri 完成“显示真实状态 + 写回配置 + 即时生效”三项门槛。

### Phase 2: Shared Core Extraction

- [x] TSM-B-01 在根目录建立 Cargo workspace 方案，保证 `taskbar-widget` 与未来 `shared-core` / Tauri crate 能并存。
- [x] TSM-B-02 新建 `crates/shared-core/`，迁移纯业务层类型和配置读写逻辑，不把 Win32 句柄或 UI 框架依赖带进去。
- [x] TSM-B-03 在 shared-core 中建立 settings service 边界，例如 `load_settings`、`save_settings`、`read_status_snapshot`、`apply_settings_change`。
- [x] TSM-B-04 抽出 settings DTO / view model 投影，避免 Tauri 直接消费内部 Rust runtime 结构。
- [x] TSM-B-05 为 shared-core 补最小测试：配置 round-trip、默认值回退、字段兼容、状态投影正确性。

### Phase 3: Native Host Boundary Refactor

- [x] TSM-C-01 在 `taskbar-widget` 中把当前 `settings_slint.rs` 依赖的业务逻辑改成调用 shared-core 或统一 settings service。
- [x] TSM-C-02 为宿主进程补一个独立的 settings bridge 模块，负责对外暴露“取状态、取配置、写配置、触发刷新、通知即时生效”。
- [x] TSM-C-03 设计宿主与 Tauri 的通信协议，至少覆盖：`get_snapshot`、`get_settings`、`save_settings`、`request_refresh`、`notify_settings_applied`。
- [x] TSM-C-04 选择并落地保守通信路径：本地 IPC 负责命令与即时生效，配置文件仍作为持久化事实源。
- [x] TSM-C-05 明确宿主只负责单一 settings 进程生命周期，避免重复点开 tray 菜单时拉起多个 Tauri 实例。

### Phase 4: Tauri App Scaffold

- [x] TSM-D-01 新建 `taskbar-settings-tauri/` 工程，包管理器固定为 `PNPM`。
- [x] TSM-D-02 选择 `React` 作为前端层，并建立最小路由/页面壳子，对齐现有 6 个 settings 页面结构。
- [x] TSM-D-03 建立 `src-tauri/` 与 Rust 后端最小命令通道，先用假数据打通窗口创建与命令回路。
- [x] TSM-D-04 设计并落地根级脚本与工作区文件，例如 `pnpm-workspace.yaml`、根级 `package.json`、必要的 ignore 规则。
- [x] TSM-D-05 确认 Tauri 构建产物形态符合“用户视角仍是一套应用，多文件安装可接受”的约束。

### Phase 5: Static UI Fidelity

- [x] TSM-E-01 按 [cc_traffic_light_nothing_demo_strict.html](/D:/project/cc-traffic-light/docs/ui/cc_traffic_light_nothing_demo_strict.html) 建立 Tauri 前端静态布局，不接业务前先把结构、节奏、信息层级做准。
- [x] TSM-E-02 建立全局 design tokens：字体、字号层级、间距、描边、圆角、状态色、surface 层级。
- [x] TSM-E-03 复刻关键交互形态：顶部概览、侧边导航、状态卡片、列表行、开关/选择器、诊断卡。
- [x] TSM-E-04 明确哪些地方要求“近似还原”，哪些地方允许为了工程稳定做实现层调整。
- [x] TSM-E-05 产出一轮视觉对照记录，逐项标记与 HTML demo 的已对齐项和未对齐项。

### Phase 6: Read-Only Integration

- [x] TSM-F-01 让 Tauri UI 读取真实配置与真实状态，但先不开放写入。
- [x] TSM-F-02 接入 1 秒轮询刷新策略，用于更新 detector 状态与运行摘要。
- [x] TSM-F-03 确认 Overview、Diagnostics、About、各来源状态卡都显示真实数据，不再依赖假数据。
- [ ] TSM-F-04 验证宿主轮询与 Tauri 轮询之间没有明显重复抖动、卡顿或日志刷屏。
- [ ] TSM-F-05 在 read-only 阶段保留 Slint 为默认入口，Tauri 只作为灰度验证路径。

### Phase 7: Write Path and Immediate Apply

- [x] TSM-G-01 接通 General / Monitoring / Appearance / Diagnostics 中所有当前真实可写项。
- [x] TSM-G-02 把设置修改写回 shared-core，再由宿主完成真正应用动作。
- [x] TSM-G-03 建立“保存成功后立即通知宿主刷新或重载”的路径，满足即时生效要求。
- [ ] TSM-G-04 验证每个设置项的三件事：UI 更新、配置落盘、宿主行为变化。
- [ ] TSM-G-05 对无法即时生效的项单独标注，并明确是否允许保留为“下次生效”。

### Phase 8: Entry Switch and Lifecycle Hardening

- [x] TSM-H-01 把 tray 的 `Open Settings` 主入口切到 Tauri。
- [x] TSM-H-02 保留 Win32 fallback 开关，直到 Tauri 完成稳定性验收。
- [x] TSM-H-03 验证重复打开、关闭、隐藏、再打开 settings 时，不会留下孤儿进程或僵尸窗口。
- [x] TSM-H-04 验证主进程退出时能正确回收或通知 Tauri settings 进程。
- [x] TSM-H-05 验证 Tauri 异常退出时，宿主侧有清晰可诊断日志，并允许再次打开。

### Phase 9: Slint Retirement and Archive

- [x] TSM-I-01 确认 `settings_slint.rs` 已归档且不再承担默认主入口职责。
- [x] TSM-I-02 评估 `settings_window.rs` 仍保留为极限 fallback，仅在 Tauri 不可用时显示。
- [x] TSM-I-03 将旧 Slint settings 相关 UI 资源和说明文档归档到 `archive/slint-settings/`，保留迁移参考价值。
- [x] TSM-I-04 清理 `taskbar-widget/Cargo.toml` 中仅用于 Slint settings 的依赖与 build 脚本。
- [x] TSM-I-05 审计仓库，确认没有遗留“看起来还在用，实际上已废弃”的双轨 settings 代码。

### Phase 10: Packaging, Docs, and Handoff

- [x] TSM-J-01 更新架构文档，明确项目从“Win32 + Slint settings”演进为“Win32 host + Tauri settings + shared-core”。
- [x] TSM-J-02 更新运行和构建文档，加入 `PNPM`、Tauri、workspace 相关命令。
- [x] TSM-J-03 更新 handoff 文档，记录进程模型、IPC 策略、即时生效路径、已知限制和后续建议。
- [x] TSM-J-04 为每个完成、跳过或阻塞任务自动补 reflection。
- [x] TSM-J-05 在最终文档中明确：任务栏 widget 仍保持现状，不属于这次 Tauri UI 迁移范围。

## Validation Checklist

- [x] TSM-VAL-01 `cargo check` 通过。
- [x] TSM-VAL-02 shared-core 相关 Rust tests 通过。
- [x] TSM-VAL-03 `pnpm build` 通过。
- [x] TSM-VAL-04 Tauri 工程可构建并正常打开 settings 窗口。
- [x] TSM-VAL-05 从 tray 打开 settings 时，默认进入 Tauri 路径。
- [x] TSM-VAL-06 Overview 和 Diagnostics 显示真实运行状态，而不是静态演示数据。
- [x] TSM-VAL-07 当前 6 个页面结构都存在，字段与现有配置模型一一对应。
- [ ] TSM-VAL-08 每个可写设置项都完成：改值、落盘、重启保持一致、即时影响宿主行为。
- [ ] TSM-VAL-09 1 秒轮询不会造成明显 CPU 抖动、日志洪泛或窗口卡顿。
- [x] TSM-VAL-10 连续多次打开/关闭 settings 后，无重复进程、僵尸窗口或状态不同步。
- [x] TSM-VAL-11 Tauri 异常退出后，tray 再次打开 settings 能恢复。
- [ ] TSM-VAL-12 widget、tray、detector、taskbar attach 主链路无明显回归。
- [ ] TSM-VAL-13 与 HTML demo 的关键视觉基线一致：层级、版式、字体角色、状态卡气质、信息密度。
- [x] TSM-VAL-14 旧 Slint 代码已归档或明确降级为 fallback，不再形成真实双主入口。

## Documentation Checklist

- [x] TSM-DOC-01 新增或更新迁移架构说明，包含进程边界图和通信方向说明。
- [x] TSM-DOC-02 更新构建命令说明，区分 Rust 宿主与 Tauri 前端。
- [x] TSM-DOC-03 更新 settings 设计文档，记录与 HTML demo 的差异点和理由。
- [x] TSM-DOC-04 更新 handoff，明确剩余风险、已知限制、回退方式。
- [x] TSM-DOC-05 对每个已完成任务生成 reflection。

## Cleanup Checklist

- [x] TSM-CLN-01 删除不再使用的 Slint-only 资源引用和构建残留。
- [ ] TSM-CLN-02 清理临时 IPC 调试日志、假数据、实验页面和无效脚本。
- [ ] TSM-CLN-03 确保命名一致：`shared-core`、宿主 bridge、Tauri commands、前端 DTO 不混用旧术语。
- [ ] TSM-CLN-04 确保没有把 Win32 句柄、UI 框架类型或 Tauri runtime 细节泄漏进 shared-core。
- [ ] TSM-CLN-05 确保仓库里没有提交本地路径、临时截图、调试产物或机器特定配置。

## Completion Criteria

以下条件满足时，Tauri settings migration 才能算完成：

- 默认 settings 主入口已经从 Slint 切到 Tauri。
- `taskbar-widget` 仍保留 widget、tray、Win32 主循环和 detector 主链路，且无明显回归。
- 共享 core 已承接配置模型和 settings 业务边界，Tauri 不直接绕过它写业务逻辑。
- 当前 6 个 settings 页面都能显示真实数据。
- 所有正式暴露的可写项都能完成：修改、落盘、重启保持、即时生效。
- Tauri 进程生命周期可控，重复打开/关闭稳定。
- 旧 Slint settings 已归档到 `archive/` 或明确降级为仅开发期 fallback。
- `cargo check`、相关 Rust tests、`pnpm build`、必要的 Tauri 构建和人工验证全部通过。
- 文档、handoff、reflection 已补齐。

可接受的已知限制：

- 第一阶段可以保留 Win32 fallback，但不能长期保留双主入口。
- 可以接受多文件安装形态，只要用户认知上仍是一套应用。
- 与 HTML demo 的差异允许存在，但必须有明确记录，且不能破坏整体 Nothing 风格层级。

## Reflection / Task Summary Generation

每完成一个 checklist item，自动生成：

```text
docs/reflections/task-<task-id>-<timestamp>.md
```

模板：

```markdown
- Task: <task name>
- Encountered Problem: <problem description>
- Thought Process: <how problem was analyzed>
- Options Considered: <list of solutions considered>
- Chosen Solution: <final decision>
- Rationale: <reason for choosing this solution>
```

规则：

- task id 必须对应本 checklist 条目，例如 `TSM-C-03`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及共享 core 的任务，必须记录边界为何这样切、哪些类型被保留在宿主侧。
- 涉及 IPC 的任务，必须记录协议、失败处理、为何不用更重的双向同步方案。
- 涉及 UI fidelity 的任务，必须记录哪些地方对齐了 HTML demo，哪些地方故意没硬抄。
- 涉及 Slint 退场的任务，必须记录归档范围、fallback 保留条件和最终删改依据。
