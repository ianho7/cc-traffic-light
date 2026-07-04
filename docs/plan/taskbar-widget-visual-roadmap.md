# Taskbar Widget 视觉改造计划

## Objective

本计划用于解决当前任务栏 widget 的视觉与交互问题，并固定后续图片支持的扩展方向。

要解决的问题：

- widget 目前依赖黑色背景承载圆灯，观感粗糙。
- 当前 `GDI` 直接画圆存在明显锯齿。
- 背景如果改为透明，现有点击命中模型也需要一起调整。
- 后续希望支持图片替换，但不希望到时再重新讨论需求边界。

预期结果：

- 第一阶段交付一个透明背景、抗锯齿、更精致、仍然易于点击的默认圆灯 widget。
- 第二阶段在不推翻第一阶段渲染底座的前提下，扩展到静态图片资源。

范围内：

- `Win32` widget 渲染路径
- widget 命中区域与交互模型
- 第一阶段配色扩展能力
- 第二阶段图片支持的技术边界

范围外：

- detector 逻辑
- hook 语义与状态聚合规则
- Tauri settings 主入口迁移
- `GIF`
- 自定义 widget 布局
- 多套预设主题系统

## Background and Context

当前仓库已经完成了状态模型简化与 Tauri settings 主链路切换，widget 仍由 `taskbar-widget` 这个 `Win32` 宿主负责绘制与挂载。

与本计划直接相关的代码事实：

- [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs) 负责创建 widget 窗口、处理 `WM_PAINT`、按状态绘制灯组。
- [taskbar-widget/src/taskbar.rs](D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs) 负责 `SetParent`、layered style、任务栏定位、宽度 0/1/2 组切换与刷新。
- [taskbar-widget/src/settings_bridge.rs](D:/project/cc-traffic-light/taskbar-widget/src/settings_bridge.rs) 提供运行时配置读取，当前已经驱动 `codex_enabled` / `claude_enabled` 的实时显示。
- [crates/shared-core/src/app_config.rs](D:/project/cc-traffic-light/crates/shared-core/src/app_config.rs) 仍然保留 `appearance` 配置结构，但 Tauri UI 已经不再暴露过去那批无效外观项。
- [taskbar-settings-tauri/src/App.tsx](D:/project/cc-traffic-light/taskbar-settings-tauri/src/App.tsx) 当前只保留监控与基础设置页，适合后续承接“第一阶段配色”与“第二阶段图片导入”。

前置决策，已在讨论中确认：

- widget 继续使用 `Win32` 宿主，不更换整套宿主技术栈。
- 第一阶段只做默认圆灯的透明背景和精细化，不做图片上传。
- 第二阶段支持静态图片，但不支持 `GIF`。
- 第二阶段图片方案需要标准化输出、受管目录、灰态派生和稳定命名。

未验证但需要在实施前确认的假设：

- `SetParent` 到任务栏宿主后，目标窗口仍可稳定使用真正的逐像素 `alpha` 绘制路径。
- 透明背景与当前 `StyleMode::PopupParented` / `WS_EX_LAYERED` 组合不会引入新的点击或刷新异常。

## Current State Analysis

### 现有实现

- `main.rs` 中的 `paint_window()` 仍然走 `BeginPaint -> FillRect -> Ellipse`。
- `paint_style()` 目前直接返回黑底、分隔线、文本色和 3 个圆灯的固定 `RECT`。
- `draw_light()` 只是用 `CreateSolidBrush + Ellipse` 画实心圆，没有抗锯齿，也没有逐像素透明。
- `lamp_fill()` 只支持三色圆灯语义：绿/黄/红，对应 `Idle|Working`、`Completed|NeedsAttention`、`Error`。

### 当前挂载与窗口风格

- `taskbar.rs` 中 `LayeredMode` 目前只有 `Off` 和 `Opaque` 两种模式。
- `Opaque` 的实现不是逐像素透明，而是：
  - 给窗口加 `WS_EX_LAYERED`
  - 调用 `SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA)`
- 这说明当前“layered”只是开启了分层窗口能力，并没有真正输出透明背景位图。

### 当前布局与交互

- `position_in_taskbar()` 已经根据启用组数切换宽度：
  - 0 组：宽度 0，不显示
  - 1 组：宽度 80
  - 2 组：宽度 160
- 当前没有独立的热区系统，命中行为仍然隐含在整个窗口矩形上。
- 这与“背景透明但仍然容易点”之间还存在设计空缺。

### 当前配置现状

- `shared-core` 仍有 `appearance.ui_theme`、`appearance.indicator_style`、`appearance.widget_size` 等旧字段。
- Tauri UI 已经移除了这些无实际效果的项，但底层配置模型和变更键还在。
- 这给第一阶段新增“真正有效的 widget 外观配置”提供了存量位置，但也意味着要避免继续沿用无效字段语义。

### 已知限制与风险

- 现有 `WM_PAINT` 管线和黑底是绑定的，不能直接满足“背景透明 + 精细边缘”。
- 现有 `GDI Ellipse` 无法从根上解决锯齿观感。
- 现有 `SetLayeredWindowAttributes` 只够做整窗 `alpha`，不够做“背景透明、灯本体可见”的逐像素输出。

## Proposed Solution

### 高层方案

保留 `Win32` 宿主、任务栏挂载和 0/1/2 组宽度逻辑不变，只替换 widget 的渲染实现。

推荐路线：

- 继续由 `taskbar-widget` 创建宿主窗口。
- 把当前 `WM_PAINT` 里“直接画黑底和圆灯”的实现，改成“离屏生成 32-bit ARGB 位图，再提交到窗口”的渲染模型。
- 第一阶段先让离屏位图只承载默认圆灯和透明背景。
- 第二阶段再把同一渲染接口扩展到静态图片资源。

### 技术路线选择

推荐的目标路线：

- 宿主：继续 `Win32`
- 渲染模型：离屏 `32-bit ARGB` 位图
- 透明能力：逐像素 `alpha`
- 圆灯绘制：不要再用裸 `Ellipse`，改为在离屏位图上输出抗锯齿结果
- 命中模型：独立于可见像素，按组定义矩形热区

更具体地说，第一阶段需要把“窗口怎么挂到任务栏”与“内容怎么被绘制”解耦：

- 挂载仍由 `taskbar.rs` 负责
- 渲染由新的 widget render 层负责
- `main.rs` 只负责把当前状态快照和可见组布局喂给 render 层

### 为什么优先选这条路

- 它最符合当前用户问题：黑底、锯齿、粗糙感。
- 它不要求替换掉现有 `Win32` 技术栈。
- 它能为第二阶段图片支持直接复用，不会出现“第一阶段做完还要推翻”。
- 它把风险集中在一个清晰问题上：透明 alpha 渲染能否在任务栏挂载场景下稳定工作。

### 预期实现后的行为

第一阶段完成后：

- widget 视觉上不再有黑底块。
- 用户看到的是悬浮在任务栏上的圆灯组。
- 点击不要求精确点中灯本体，按组热区仍然足够友好。
- 关闭某一来源时，宽度/布局与命中区保持同步。

第二阶段完成后：

- 用户可以在 Tauri 中导入静态图片替代默认圆灯。
- 应用自动生成 `active` / `inactive` 两份标准化资源。
- widget 渲染层只消费受管资源，不直接依赖用户原始文件。

## Alternatives Considered

### 方案 A：继续用现有 `GDI` 直接绘制，只微调颜色和尺寸

优点：

- 改动最小
- 不需要新的渲染抽象

缺点：

- 不能根本解决锯齿
- 不能自然支持透明背景
- 第二阶段图片支持仍然会推倒重来

风险：

- 第一阶段看似快，第二阶段返工大

未选原因：

- 无法同时满足透明背景和后续可扩展性

### 方案 B：继续 `Win32` 宿主，但切到离屏 `32-bit ARGB` 位图渲染

优点：

- 满足透明背景
- 满足抗锯齿目标
- 第二阶段可复用

缺点：

- 需要新增渲染层抽象
- 需要验证与任务栏宿主场景的兼容性

风险：

- 如果逐像素 `alpha` 在父子关系下表现不稳定，需要额外兼容分支

选择原因：

- 在当前范围内，这是收益最高且最可延展的方案

### 方案 C：直接引入 `Direct2D` 或更完整的新渲染后端

优点：

- 理论上渲染质量最好
- 后续扩展空间大

缺点：

- 引入复杂度明显上升
- 需要额外处理初始化、资源生命周期和 Windows 版本差异
- 当前需求规模下可能过重

风险：

- 为解决“圆灯黑底”问题而过度设计

未选原因：

- 当前第一阶段目标不需要这么重的方案

## Implementation Plan

### Phase 0: 透明渲染可行性探针

- Goal:
  - 验证任务栏挂载场景下，逐像素 `alpha` 渲染是否可行
- Files:
  - [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - [taskbar-widget/src/taskbar.rs](D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs)
  - 可新增 `taskbar-widget/src/widget_render.rs`
- Tasks:
  - 抽出最小的离屏位图绘制试验路径
  - 验证透明背景下是否仍能正常显示在任务栏中
  - 验证 `SetParent` 后窗口刷新、显隐、宽度切换是否稳定
  - 记录推荐窗口风格与 layered 策略，决定是否继续沿用 `StyleMode::PopupParented`
- Expected Result:
  - 得到一个“这条渲染路线能否跑通”的明确结论

### Phase 1: 渲染层抽象与默认圆灯重做

- Goal:
  - 用新的渲染层替换当前 `GDI` 黑底圆灯实现
- Files:
  - [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - [taskbar-widget/src/taskbar.rs](D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs)
  - 新增 `taskbar-widget/src/widget_render.rs`
- Tasks:
  - 抽离“布局数据”和“渲染数据”结构
  - 让 `paint_style()` 不再直接携带黑底色，而是输出灯组布局信息
  - 新增默认圆灯 renderer，负责透明背景和抗锯齿输出
  - 保留现有状态到红/黄/绿的语义映射
- Expected Result:
  - 默认圆灯在视觉上更精致，且背景透明

### Phase 2: 按组热区与交互重构

- Goal:
  - 在背景透明前提下，保留易点击交互
- Files:
  - [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - 如有必要，新增命中测试辅助模块
- Tasks:
  - 定义每个灯组的矩形热区
  - 使 1 组和 2 组场景下的热区与可见布局同步
  - 避免让整块不可见区域无限拦截任务栏点击
  - 为后续不同组独立交互保留接口
- Expected Result:
  - 用户无需精确点中灯像素，透明背景也不牺牲可用性

### Phase 3: 第一阶段配色配置接入

- Goal:
  - 让默认圆灯支持真正有效的自定义配色
- Files:
  - [crates/shared-core/src/app_config.rs](D:/project/cc-traffic-light/crates/shared-core/src/app_config.rs)
  - [taskbar-widget/src/settings_bridge.rs](D:/project/cc-traffic-light/taskbar-widget/src/settings_bridge.rs)
  - [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - [taskbar-settings-tauri/src/App.tsx](D:/project/cc-traffic-light/taskbar-settings-tauri/src/App.tsx)
  - [taskbar-settings-tauri/src/types.ts](D:/project/cc-traffic-light/taskbar-settings-tauri/src/types.ts)
  - [taskbar-settings-tauri/src-tauri/src/lib.rs](D:/project/cc-traffic-light/taskbar-settings-tauri/src-tauri/src/lib.rs)
- Tasks:
  - 设计新的最小 widget 视觉配置字段，不复用旧的无效 `indicator_style` / `widget_size`
  - 在 Tauri 中提供有限的颜色配置 UI
  - 让设置变更实时驱动 widget 重绘
  - 明确默认值与回退逻辑
- Expected Result:
  - 第一阶段完成，用户可在透明背景 + 精致圆灯基础上调整配色

### Phase 4: 第二阶段图片支持预埋

- Goal:
  - 把第二阶段计划转化为可以直接接续开发的技术边界
- Files:
  - [crates/shared-core/src/app_config.rs](D:/project/cc-traffic-light/crates/shared-core/src/app_config.rs)
  - [taskbar-settings-tauri/src/App.tsx](D:/project/cc-traffic-light/taskbar-settings-tauri/src/App.tsx)
  - 可新增资源处理模块与受管目录约定文档
- Tasks:
  - 固定状态槽位与 `active` / `inactive` 资源模型
  - 固定受管目录和稳定文件命名约定
  - 固定图片导入、裁切、标准化、灰度派生的接口边界
  - 不在本阶段实现图片功能，但避免第一阶段字段设计与第二阶段冲突
- Expected Result:
  - 第二阶段可以直接开工，不需要再重新定义需求

## Validation Strategy

### 单元与模块验证

- 为新的配置字段与默认值补充 `shared-core` 反序列化测试
- 为颜色配置与回退逻辑补充 Rust 单元测试
- 如果新增 render 参数结构，至少补充布局/状态映射测试

### 集成验证

建议运行：

```powershell
cargo check -p taskbar-widget --offline
cargo test --workspace --offline
pnpm run build:frontend
cargo build -p taskbar-settings-tauri --offline
cargo build -p taskbar-widget --offline
```

### 手工验证

- 启动 `target\debug\taskbar-widget.exe`
- 验证两组可见时宽度为 2 组、单组可见时宽度为 1 组、全关时隐藏
- 验证透明背景下没有黑底残留
- 验证圆灯边缘明显优于当前版本
- 验证点击灯附近仍能触发，不要求精确点中灯本体
- 验证设置页切换监控开关和配色时，widget 能实时更新

### 失败场景验证

- 任务栏挂载失败时是否仍能退回 tray only
- 单组/双组切换时透明区域是否出现脏刷新
- 高频状态变化时是否出现闪烁
- 透明背景下是否出现鼠标命中异常

## Risks and Mitigations

- Risk:
  - 逐像素 `alpha` 在 `SetParent` 到任务栏后的兼容性不稳定
  - Impact:
    - 会影响第一阶段核心目标
  - Likelihood:
    - 中
  - Mitigation:
    - 先做 Phase 0 技术探针，明确最小可行路径
  - Fallback plan:
    - 如果逐像素路径失败，再评估次优透明方案，但不先把 fallback 当主路线

- Risk:
  - 第一阶段继续沿用旧 `appearance` 字段语义，导致配置债务扩大
  - Impact:
    - 第二阶段图片支持时配置模型会混乱
  - Likelihood:
    - 中
  - Mitigation:
    - 第一阶段新增专用 widget 视觉配置字段或明确弃用旧字段
  - Fallback plan:
    - 至少在 Tauri 与 Rust 两侧把旧字段标成兼容保留，不再扩展

- Risk:
  - 热区过大，透明背景下干扰任务栏原生点击
  - Impact:
    - 用户交互体验下降
  - Likelihood:
    - 中
  - Mitigation:
    - 热区按组收敛，不做整块宽度统一命中
  - Fallback plan:
    - 增加调试日志和可视化诊断开关，微调热区尺寸

- Risk:
  - 图片支持阶段与第一阶段渲染接口不兼容
  - Impact:
    - 第二阶段返工
  - Likelihood:
    - 低到中
  - Mitigation:
    - 第一阶段 render 层按“状态槽位 -> active/inactive 视觉资源”抽象
  - Fallback plan:
    - 在 Phase 4 先把资源接口文档化，再进入实现

## Open Questions

- `Win32` 任务栏父子窗口场景下，推荐的逐像素透明提交方式具体选哪条 API 路径，仍需通过 Phase 0 验证后定案。
- 第一阶段配色配置是挂到现有 `appearance` 结构下，还是新增独立的 `widget_visual` 配置块，需要在实现前确认一次。

## Recommended Next Step

先执行 Phase 0：在 [taskbar-widget/src/main.rs](D:/project/cc-traffic-light/taskbar-widget/src/main.rs) 和 [taskbar-widget/src/taskbar.rs](D:/project/cc-traffic-light/taskbar-widget/src/taskbar.rs) 上做一个最小透明渲染探针，验证任务栏挂载场景下的逐像素 `alpha` 路径是否稳定可用。

## Phase 1 Technical Conclusion

日期：2026-07-04 23:28

第一阶段当前实现已经落在以下技术结论上：

- widget 继续保持 `Win32` 宿主，不引入 `Tauri/WebView` 渲染承载。
- 透明路线采用：
  - `StyleMode::PopupParented`
  - `WS_EX_LAYERED`
  - `LayeredMode::PerPixel`
  - `UpdateLayeredWindow()` 提交 32-bit ARGB 离屏缓冲
- 默认灯组不再依赖 `FillRect + Ellipse` 黑底直画，而是改由 [widget_render.rs](D:/project/cc-traffic-light/taskbar-widget/src/widget_render.rs) 负责：
  - 布局
  - 状态到红黄绿语义映射
  - 简单超采样圆灯边缘
  - 逐像素透明输出
- 透明背景下的交互不再让整块窗口持续吃点击：
  - 每个来源拥有独立矩形热区
  - `WM_NCHITTEST` 在热区外返回 `HTTRANSPARENT`
  - 热区内点击复用现有“打开设置”行为
- 第一阶段有效配置字段定为 `widget_visual.palette.{green,yellow,red,off}`：
  - 不复用旧 `indicator_style` / `widget_size` 语义
  - 通过 named-pipe settings 保存
  - 宿主侧收到应用通知后即时重绘

未完成的仍是桌面侧最终证据，而不是代码链路本身：

- 真实任务栏挂载下的最终透明观感
- 单组/双组热区体感
- 0/1/2 组显隐与 tray/detector 主链路的人工回归确认
