# Taskbar Widget 视觉改造 Phase 1 Checklist

日期：2026-07-04

## Current Status

状态更新时间：2026-07-04 23:28

- 已完成：
  - host 侧默认黑底 `GDI Ellipse` 已替换为独立 `widget_render` 模块输出的逐像素透明默认圆灯。
  - widget 已改为按组热区命中，透明空白区域走 `HTTRANSPARENT`，点击灯组会复用现有设置入口。
  - 第一阶段 `widget_visual.palette` 配色模型、Tauri 配色页、宿主实时重绘、配置落盘链路已打通。
  - `cargo check -p taskbar-widget --offline`
  - `cargo test --workspace --offline`
  - `pnpm run build:frontend`
  - `cargo build -p taskbar-settings-tauri --offline`
  - `cargo build -p taskbar-widget --offline`
- 待人工桌面验证：
  - 透明背景在真实任务栏挂载场景下的最终观感
  - 单组/双组热区体感
  - 0/1/2 组显隐与 tray/detector 主链路的桌面回归

## Checklist Objective

目标是把 [taskbar-widget-visual-roadmap.md](/D:/project/cc-traffic-light/docs/plan/taskbar-widget-visual-roadmap.md) 中的第一阶段 `MVP` 转成可执行 checklist，并在不改动 widget 宿主基本架构的前提下，完成以下结果：

- widget 背景默认透明，不再依赖黑色背景承载灯组。
- 默认红黄绿圆灯具备明显优于当前 `GDI Ellipse` 的边缘观感。
- 透明背景下仍然保持按组易点击，不要求用户精确点中灯本体。
- 保持当前 0/1/2 组宽度切换、tray、detector、taskbar attach 主链路不回归。
- 第一阶段支持“真正有效”的自定义配色，但不引入图片上传。

范围：

- 覆盖 `taskbar-widget/` 中与 widget 渲染、命中、显隐、布局同步、外观配置相关的代码路径。
- 覆盖 `shared-core` 中与第一阶段视觉配置直接相关的最小配置模型调整。
- 覆盖 Tauri settings 中第一阶段最小配色 UI 和即时生效链路。

非目标：

- 不支持图片上传。
- 不支持 `GIF`。
- 不支持背景颜色或背景透明度自定义。
- 不支持按 `Codex` / `Claude Code` 分别配置视觉方案。
- 不把 widget 改写成 Tauri / WebView / 新宿主进程。
- 不把这轮工作扩展成新的状态模型改造。

## Loop Engineering Spec

### Goal

- 交付一个透明背景、抗锯齿默认圆灯、按组热区、自定义配色的第一阶段 widget。
- 进度证据来自：透明渲染探针结果、代码 diff、`cargo check` / `cargo test` / `pnpm` 构建结果、手工桌面验证记录。
- 完成证据不是“代码看起来更合理”，而是“桌面上实际显示效果正确、交互可用、配置生效、主链路无明显回归”。

### State

- Source of truth:
  - 本 checklist
  - [taskbar-widget-visual-roadmap.md](/D:/project/cc-traffic-light/docs/plan/taskbar-widget-visual-roadmap.md)
  - [taskbar-widget/src/main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - [taskbar-widget/src/taskbar.rs](/D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs)
  - [crates/shared-core/src/app_config.rs](/D:/project/cc-traffic-light/crates/shared-core/src/app_config.rs)
  - [taskbar-settings-tauri/src/App.tsx](/D:/project/cc-traffic-light/taskbar-settings-tauri/src/App.tsx)
- Persistent loop state:
  - 当前 phase / task id
  - 透明渲染路线是否跑通
  - 当前渲染实现是否已脱离黑底 `GDI` 方案
  - 热区策略验证结论
  - 第一阶段配色配置是否已全链路打通
- Raw evidence:
  - 编译输出
  - 运行日志
  - 手工桌面观察
  - 配置落盘结果
  - reflection 文档

### Planner

- 每轮优先选择“最能改变验证状态的最小任务”。
- 固定顺序为：
  - 先验证透明渲染路线可行
  - 再替换默认渲染实现
  - 再接命中热区
  - 最后接第一阶段配色配置
- 若透明渲染路线未验证通过，禁止提前展开设置页配色 UI。
- 若出现主链路回归，先回退到最近可验证阶段，不并行改三层以上。

### Actor

- 允许动作：
  - 读取并重构 `taskbar-widget` 渲染代码
  - 新增 widget render / hit-test / visual-config 模块
  - 调整 `shared-core` 最小配置模型
  - 调整 Tauri 设置页第一阶段配色 UI
  - 运行构建和手工验证
- 中风险动作：
  - 调整窗口风格或 layered 相关逻辑
  - 切换 `WM_PAINT` 渲染路径
  - 修改任务栏挂载后显示行为
- 非默认动作：
  - 修改 detector 聚合逻辑
  - 扩展到第二阶段图片支持
  - 引入更重的全新渲染栈

### Observer

- 先记录“发生了什么”，再解释“这意味着什么”。
- 对每次桌面验证至少记录：
  - 是否仍有黑底
  - 边缘是否明显更平滑
  - 1 组 / 2 组宽度是否正确
  - 热区是否易点
  - 配色修改是否实时生效
- 若失败，记录失败类别：
  - 透明显示失败
  - 渲染质量失败
  - 命中失败
  - 配置链路失败
  - 主链路回归

### Verifier

- Verifier order:
  1. focused local check：局部代码审阅和最小运行验证
  2. `cargo check -p taskbar-widget --offline`
  3. `cargo test --workspace --offline`
  4. `pnpm run build:frontend`
  5. `cargo build -p taskbar-settings-tauri --offline`
  6. `cargo build -p taskbar-widget --offline`
  7. 桌面人工验证

### Failure Semantics

- Transient failure：
  - 单次构建失败、窗口未刷新、任务栏未及时重绘，只重试 1 次。
- Strategy failure：
  - 当前透明渲染方案在任务栏挂载场景下不可用，必须回到渲染路线选择，不继续堆补丁。
- Scope failure：
  - 为解决第一阶段问题开始引入图片、主题包或 per-agent 外观，立即停止并收回范围。
- Regression failure：
  - 一旦影响 tray、detector、0/1/2 组宽度逻辑或主链路显隐，优先回退到上一阶段可验证状态。

### Exit Conditions

- Success exit：
  - Completion Criteria 全部满足。
- Blocked exit：
  - 逐像素透明在当前任务栏宿主模型下需要额外产品决策或更换路线。
- Risk exit：
  - 下一步必须改 detector、状态模型或大范围改宿主架构。
- Human takeover exit：
  - 需要你决定是否接受某个透明/命中权衡，且本地证据无法继续收敛。

### Policy

- 保持 `Win32` 宿主不变。
- 第一阶段不引入图片资源。
- 先解决透明背景和渲染质量，再接设置页配色。
- 禁止把“视觉问题”误扩展成“状态语义问题”。

## Pre-Implementation Checks

- [x] TWV1-PRE-01 阅读 [taskbar-widget-visual-roadmap.md](/D:/project/cc-traffic-light/docs/plan/taskbar-widget-visual-roadmap.md)，确认第一阶段范围与第二阶段边界。
- [x] TWV1-PRE-02 阅读 [taskbar-widget/src/main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)，定位当前 `WM_PAINT`、`paint_style()`、`draw_light()` 和显隐逻辑。
- [x] TWV1-PRE-03 阅读 [taskbar-widget/src/taskbar.rs](/D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs)，确认当前 `LayeredMode`、`SetLayeredWindowAttributes()`、宽度 0/1/2 组切换实现。
- [x] TWV1-PRE-04 阅读 [crates/shared-core/src/app_config.rs](/D:/project/cc-traffic-light/crates/shared-core/src/app_config.rs)，确认现有 `appearance` 字段与第一阶段新增视觉配置的兼容边界。
- [x] TWV1-PRE-05 阅读 [taskbar-settings-tauri/src/App.tsx](/D:/project/cc-traffic-light/taskbar-settings-tauri/src/App.tsx) 和 [taskbar-settings-tauri/src-tauri/src/lib.rs](/D:/project/cc-traffic-light/taskbar-settings-tauri/src-tauri/src/lib.rs)，确认第一阶段设置页落点。
- [x] TWV1-PRE-06 确认最小验证命令集合：`cargo check -p taskbar-widget --offline`、`cargo test --workspace --offline`、`pnpm run build:frontend`、`cargo build -p taskbar-settings-tauri --offline`、`cargo build -p taskbar-widget --offline`。
- [x] TWV1-PRE-07 确认 reflection 文档命名仍使用 `docs/reflections/task-<task-id>-<timestamp>.md`。

## Implementation Checklist

### Phase 1: 透明渲染路线验证

- [x] TWV1-A-01 抽出当前 widget 绘制路径中的背景、灯组布局和状态映射，避免透明探针直接耦合全部旧实现。
- [x] TWV1-A-02 在 `taskbar-widget` 中建立最小透明渲染探针，验证任务栏挂载场景下是否能显示透明背景内容。
- [ ] TWV1-A-03 记录透明渲染探针在 0 组 / 1 组 / 2 组场景下的可见性、刷新、显隐表现。
- [x] TWV1-A-04 明确第一阶段采用的窗口风格 / layered 组合，并把未采用的路径写回 plan 或 reflection。
- [ ] TWV1-A-05 若透明探针失败，先分类为 API 路线失败或挂载兼容失败，不继续盲改默认 renderer。

### Phase 2: 默认圆灯渲染层替换

- [x] TWV1-B-01 新增独立的 widget render 模块，承接默认圆灯布局与透明背景输出。
- [x] TWV1-B-02 让 `main.rs` 从“直接画黑底 + `Ellipse`”切换到“调用 render 模块提交默认灯组”。
- [x] TWV1-B-03 保持现有状态语义映射不变：`Idle|Working -> 绿`，`Completed|NeedsAttention -> 黄`，`Error -> 红`。
- [x] TWV1-B-04 移除默认黑底依赖，确认 widget 在透明背景下仍可读。
- [x] TWV1-B-05 为新的渲染结构补最小注释，说明布局与透明策略的工程理由。

### Phase 3: 按组热区与交互同步

- [x] TWV1-C-01 定义每个灯组的矩形热区数据结构，不再把命中语义隐含在整块窗口矩形里。
- [ ] TWV1-C-02 在单组可见场景下验证只有一个热区，且用户无需精确点中灯本体。
- [ ] TWV1-C-03 在双组可见场景下验证左右两组分别拥有独立热区，不把整块宽度做成统一点击区。
- [ ] TWV1-C-04 验证禁用某个来源后，宽度、可见灯组和热区同步更新，不需重启。
- [ ] TWV1-C-05 验证透明背景区域不会出现明显“不可见但乱拦截点击”的问题。

### Phase 4: 第一阶段配色配置

- [x] TWV1-D-01 设计第一阶段最小视觉配置模型，只覆盖默认圆灯配色，不复用旧的无效 `indicator_style` / `widget_size` 语义。
- [x] TWV1-D-02 在 `shared-core` 中实现默认值、序列化和兼容回退。
- [x] TWV1-D-03 在 Tauri 设置页中增加第一阶段配色 UI，范围只限默认圆灯颜色。
- [x] TWV1-D-04 打通设置修改到宿主实时重绘链路，确认变更无需重启即可生效。
- [x] TWV1-D-05 验证配色修改只影响默认圆灯视觉，不影响监控开关、状态聚合或其他设置页逻辑。

### Phase 5: 主链路回归与收口

- [ ] TWV1-E-01 验证 0 组隐藏、1 组宽度 1、2 组宽度 2 的既有行为保持不变。
- [ ] TWV1-E-02 验证 tray 打开设置、手动刷新、状态轮询和 widget 重绘仍然正常。
- [x] TWV1-E-03 审计并移除第一阶段过程中产生的实验性分支、死代码和临时日志。
- [x] TWV1-E-04 回写文档：plan、checklist、必要的 README 或 handoff。
- [x] TWV1-E-05 为每个完成、跳过或阻塞任务生成 reflection。

## Validation Checklist

- [x] TWV1-VAL-01 `cargo check -p taskbar-widget --offline` 通过。
- [x] TWV1-VAL-02 `cargo test --workspace --offline` 通过。
- [x] TWV1-VAL-03 `pnpm run build:frontend` 通过。
- [x] TWV1-VAL-04 `cargo build -p taskbar-settings-tauri --offline` 通过。
- [x] TWV1-VAL-05 `cargo build -p taskbar-widget --offline` 通过。
- [ ] TWV1-VAL-06 启动 `target\debug\taskbar-widget.exe` 后，默认背景透明，不再出现黑底块。
- [ ] TWV1-VAL-07 默认圆灯边缘观感明显优于当前 `GDI Ellipse` 版本。
- [ ] TWV1-VAL-08 只有 `Codex`、只有 `Claude Code`、两者都开、两者都关四种场景下，显示宽度与可见性都正确。
- [ ] TWV1-VAL-09 单组和双组场景下，点击热区都足够友好，不要求精确点灯。
- [ ] TWV1-VAL-10 切换监控开关后，widget 可见组、宽度和热区实时更新。
- [ ] TWV1-VAL-11 修改第一阶段配色后，widget 视觉实时更新且配置正确落盘。
- [ ] TWV1-VAL-12 tray、detector、状态轮询、显隐、任务栏挂载主链路无明显回归。

## Documentation Checklist

- [x] TWV1-DOC-01 更新 [taskbar-widget-visual-roadmap.md](/D:/project/cc-traffic-light/docs/plan/taskbar-widget-visual-roadmap.md) 中第一阶段的技术结论。
- [ ] TWV1-DOC-02 若第一阶段最终确定了新的模块拆分，补到 plan 或 handoff。
- [ ] TWV1-DOC-03 如新增有效视觉配置字段，补充到相关配置说明文档。
- [x] TWV1-DOC-04 对每个已完成任务生成 reflection。

## Cleanup Checklist

- [x] TWV1-CLN-01 删除已废弃的黑底默认绘制分支或无效兼容代码。
- [ ] TWV1-CLN-02 删除透明渲染探针留下的临时日志和实验常量。
- [ ] TWV1-CLN-03 确保命名一致，避免同时出现多个含义重叠的“render / paint / visual”结构。
- [ ] TWV1-CLN-04 确保没有把第二阶段图片字段提前半实现到第一阶段代码里。
- [ ] TWV1-CLN-05 确保没有提交本地路径、临时截图或机器特定调试产物。

## Completion Criteria

以下条件满足时，第一阶段视觉改造才算完成：

- widget 背景默认透明。
- 默认红黄绿圆灯的边缘观感明显优于当前版本。
- 当前 0/1/2 组显示逻辑与监控开关联动保持正确。
- 透明背景下的点击交互仍然易用，热区按组独立。
- 第一阶段自定义配色已全链路打通：UI 修改、宿主重绘、配置落盘、重启保持一致。
- 第一阶段不包含图片上传、灰度派生、受管资源目录和 `GIF` 支持。
- `cargo check`、`cargo test`、前端构建、目标构建和人工桌面验证通过。
- 所有完成、跳过或阻塞任务都已写入 reflection 或明确记录原因。

## Reflection / Task Summary Generation

每个完成的 checklist 任务都应自动生成：

```text
docs/reflections/task-<task-id>-<timestamp>.md
```

内容模板：

- Task: `<task name>`
- Encountered Problem: `<problem description>`
- Thought Process: `<how problem was analyzed>`
- Options Considered: `<list of solutions considered>`
- Chosen Solution: `<final decision>`
- Rationale: `<reason for choosing this solution>`
