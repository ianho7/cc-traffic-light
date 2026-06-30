# 09. Win11 诊断后 MVP 重计划

## Objective

目标不是重新证明“Rust 能调用 Win32”。

当前要解决的立即问题是：

- `taskbar-widget` 运行时有正常日志
- `SetParent`、`MoveWindow`、子窗口绘制都已经能成功
- 但在当前 Win11 主力机器上，任务栏里仍然看不到稳定可见的模块

最小有用结果：

- 继续使用现有 `taskbar-widget`
- 在当前 Win11 主显示器任务栏中，稳定看到一个固定文本模块，例如 `TASKBAR WIDGET`
- 模块不是普通桌面浮窗
- `cargo run` 可重复复现，关闭后模块消失

MVP 范围内：

- 仅修正当前仓库里的 Win11 主路径
- 仅支持主任务栏
- 仅支持固定文本和 GDI 自绘
- 仅保留一个 Win11 可见宿主路径
- 仅保留最小诊断输出和人工验证

明确不在范围内：

- 经典任务栏支持
- 多显示器
- 透明背景
- D2D / DComposition
- 插件系统
- 通用父窗口候选矩阵
- 通用平台化架构

主要成功信号：

- 肉眼可见模块进入任务栏
- 不再出现“日志成功但任务栏无内容”
- 重复启动结果一致

## MVP Scope Boundary

### Must Have

- Requirement: 复用现有 `taskbar-widget` 项目继续修正  
  MVP Justification: 当前已有可运行 PoC，重新起盘不会增加 MVP 价值。

- Requirement: 为 Win11 主路径固定 `Shell_TrayWnd` 作为父窗口  
  MVP Justification: TrafficMonitor 的 Win11 实现就是这样做的，这直接对应当前“挂进去了但不可见”的核心问题。

- Requirement: 仅把 `TrayNotifyWnd` 和 `Start` 当作定位锚点，而不是父窗口候选  
  MVP Justification: 这能把“宿主选择”和“位置计算”拆开，减少当前错误来源。

- Requirement: 使用最小、保守、可见的窗口样式和 GDI 背景绘制  
  MVP Justification: 先保证看得见，再谈透明、主题和高级渲染。

- Requirement: 保留最小结构化日志与诊断 JSON  
  MVP Justification: 当前问题已经证明只靠肉眼和单条日志不够，需要能回看父窗口、锚点、矩形和样式。

- Requirement: 用人工观察作为主验证手段  
  MVP Justification: 现有 `PrintWindow` 度量已经出现误导，当前 MVP 不能把自动截图当成唯一真相源。

### Must Not Have

- Excluded Item: 继续扩展 `rebar` / `task_switch` / `composition_bridge` 父窗口矩阵  
  Reason for Exclusion: 这属于继续扩大搜索空间，不是收敛 MVP。

- Excluded Item: 现在就做 `Classic / Win11` 双策略框架  
  Reason for Exclusion: 当前只需要救活一条 Win11 主路径。

- Excluded Item: 透明背景与分层窗口优化  
  Reason for Exclusion: 透明会把“嵌入问题”和“渲染问题”重新混在一起。

- Excluded Item: D2D、DirectComposition、主题跟随  
  Reason for Exclusion: 与“先看见模块”无关。

- Excluded Item: 通用任务栏宿主抽象、插件接口、设置系统  
  Reason for Exclusion: 当前只有一个真实用例，做抽象只会放大维护面。

- Excluded Item: 用自动截图脚本直接作为唯一 pass/fail gate  
  Reason for Exclusion: 当前脚本对 Win11 任务栏可见层的观测并不可靠。

### Deferred Until After MVP

- Deferred Item: 经典任务栏支持  
  Why Deferred: 当前只锁定 Win11 主力机器。  
  Signal to Reconsider: Win11 主路径稳定后，需要支持第二个 Windows 任务栏结构。

- Deferred Item: 多显示器和副任务栏支持  
  Why Deferred: 不是当前最小闭环的一部分。  
  Signal to Reconsider: 主任务栏稳定后，需要在副屏展示模块。

- Deferred Item: 透明背景和主题适配  
  Why Deferred: 只影响视觉质量，不影响宿主可见性验证。  
  Signal to Reconsider: 模块已经稳定可见，需要更接近产品形态。

- Deferred Item: 自动化诊断脚本精度提升  
  Why Deferred: 这是辅助工具，不是当前主产品结果。  
  Signal to Reconsider: Win11 主路径稳定后，准备把经验固化成可回归的本地脚本。

- Deferred Item: 通用 `Classic / Win11` 策略抽象  
  Why Deferred: 当前只有一个被验证的真实路径。  
  Signal to Reconsider: 第二条系统路径出现重复代码且已证明值得保留。

## Background and Context

当前相关背景已经不是“理论设计”，而是有现成代码和现成诊断结论：

- 现有 Rust PoC 位于 `taskbar-widget`
- 主要代码在：
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/taskbar.rs`
  - `taskbar-widget/src/win32.rs`
- 诊断脚本位于：
  - `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
- 当前诊断结果汇总位于：
  - `taskbar-widget/target/diagnose-taskbar-loop/summary.json`

已确认事实：

- 子窗口自身能绘制，`RenderPass=True`
- `SetParent` 成功，`MoveWindow` 成功
- `WithinParent=True`
- 但当前矩阵下仍然 `VisualPass=False`

外部参考中最关键的新约束来自 TrafficMonitor：

- Win11 不把 `MSTaskSwWClass` 当作最终宿主
- Win11 路线把窗口挂到 `Shell_TrayWnd`
- `TrayNotifyWnd` 和 `Start` 主要用于定位

对应参考：

- [README.md](/D:/project/TrafficMonitor/docs/taskbar-customization/README.md)
- [01-overview-and-principles.md](/D:/project/TrafficMonitor/docs/taskbar-customization/01-overview-and-principles.md)
- [03-reuse-guide.md](/D:/project/TrafficMonitor/docs/taskbar-customization/03-reuse-guide.md)
- [04-implementation-playbook.md](/D:/project/TrafficMonitor/docs/taskbar-customization/04-implementation-playbook.md)
- [Win11TaskbarDlg.cpp](/D:/project/TrafficMonitor/TrafficMonitor/Win11TaskbarDlg.cpp:1)
- [TaskBarDlg.cpp](/D:/project/TrafficMonitor/TrafficMonitor/TaskBarDlg.cpp:976)

Assumption:

- 当前目标环境仍然是 Win11 主力机器，主显示器任务栏，且这是唯一必须支持的 MVP 环境。

## Current State Analysis

### Relevant Files and Roles

- `taskbar-widget/src/main.rs`
  - 创建窗口
  - 进入消息循环
  - 在启动时调用探测、挂载、定位和绘制

- `taskbar-widget/src/taskbar.rs`
  - 当前包含父窗口候选、锚点候选、坐标模式候选
  - 输出诊断日志和 JSON
  - 承载 `SetParent` 和 `MoveWindow` 主逻辑

- `taskbar-widget/src/win32.rs`
  - Win32 工具函数和日志格式化

- `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
  - 跑父窗口/锚点/坐标模式组合
  - 采集 before/after/child 图像
  - 汇总 JSON 和表格结果

### Existing Implementation Details

- 当前代码默认把“找哪个父窗口”和“怎么定位”放进同一个实验矩阵
- 当前窗口先以普通顶层窗口样式创建，再在后面切成 `WS_CHILD`
- 诊断脚本会把 `PrintWindow` 结果当成父层是否变化的重要依据

### Known Limitations That Affect The MVP

- 运行路径没有收敛到一个明确的 Win11 宿主策略
- 当前默认路径仍把 `task_switch` / `rebar` 当成强候选，这与 TrafficMonitor 的 Win11 做法不一致
- 当前自动诊断度量可能把“截图方法问题”误判成“产品实现失败”
- 当前验证方式没有把“人工可见性”明确设为主真相源

### Known Bugs or Risks That Affect The MVP

- 现有总结里已经出现“截图肉眼有变化，但 `MeanDelta=0`”的度量异常
- 现有 `summary.json` 说明子窗口在画，但宿主可见层未被可靠验证
- 如果继续扩大候选矩阵，诊断成本会持续上升，但 MVP 不一定更接近成功

### Dependencies or External Systems Involved

- `windows` crate
- 当前 Win11 Explorer 任务栏窗口树
- 本机 PowerShell 诊断脚本

## MVP Decision Gate

- Does this solve the user's immediate problem?
  - 是。当前问题不是“怎么做架构”，而是“为什么任务栏里看不到模块”。

- Can the MVP be validated without this item?
  - 经典任务栏、多屏、透明、D2D 都可以不做。

- Is there an existing project pattern that avoids new design?
  - 有。TrafficMonitor 的 Win11 路线已经给出更窄的父窗口和定位模型。

- Is any part of the plan preparing for a hypothetical future?
  - 如果现在继续保留多父窗口矩阵、双策略框架、自动化视觉 gating，就属于面向未来而不是面向 MVP。

- Are any new dependencies, services, abstractions, workflows, or data sources truly necessary?
  - 不需要。现有 `windows` crate 和脚本足够。

- What is the simplest acceptable implementation?
  - 一条 Win11 路径：`Shell_TrayWnd` 作为父窗口，`Start` / `TrayNotifyWnd` 只做定位，GDI 固定文本，人工肉眼验证。

- Keep:
  - 现有 `taskbar-widget`
  - `windows` crate
  - GDI 固定文本绘制
  - 结构化日志和诊断 JSON

- Remove:
  - Win11 运行时父窗口候选矩阵
  - 把截图脚本结果当成唯一验证依据

- Defer:
  - 经典任务栏
  - 多显示器
  - 透明和主题
  - 通用策略抽象

- Simplify:
  - 把当前 MVP 收敛成一个 Win11 可见宿主路径
  - 把自动诊断降级为辅助证据

## Proposed MVP Solution

### Decision: 继续修现有 PoC，而不是重建新项目

- Choice: 直接在 `taskbar-widget` 上收敛 Win11 主路径
- MVP Justification: 当前已有足够多的真实诊断数据，重建只会丢掉上下文
- Simpler Alternative Considered: 从零新建更小的 PoC
- Why Not More Complex: 当前不是代码规模问题，而是宿主策略问题

### Decision: Win11 上固定使用 `Shell_TrayWnd` 作为父窗口

- Choice: 不再把 `rebar`、`task_switch`、`composition_bridge` 作为 Win11 主线宿主
- MVP Justification: 这直接对齐 TrafficMonitor 的 Win11 路线，也最贴近当前失败原因
- Simpler Alternative Considered: 继续保留矩阵，让脚本自动找正确宿主
- Why Not More Complex: 当前矩阵已经证明“搜索更多候选”不会自然得到可见结果

### Decision: `TrayNotifyWnd` / `Start` 只用于位置推算

- Choice: 定位锚点保留，但不参与 `SetParent`
- MVP Justification: 这让“可见层宿主”和“模块摆放”职责清晰分离
- Simpler Alternative Considered: 只用任务栏矩形做固定偏移
- Why Not More Complex: 继续把锚点当父窗口会扩大错误空间

### Decision: 保持 GDI、非透明、醒目背景

- Choice: 保持固定文本和明显背景色，必要时补 `WS_EX_TOOLWINDOW` 与更保守的窗口样式
- MVP Justification: 当前第一目标是可见，不是接近最终视觉
- Simpler Alternative Considered: 只画文字，不画背景
- Why Not More Complex: 透明和高级渲染会再次引入额外变量

### Decision: 最小重定位循环可以纳入 MVP，但只服务 Win11 可见性

- Choice: 如果启动时一次 `MoveWindow` 不稳定，则加入轻量位置刷新
- MVP Justification: TrafficMonitor 的经验表明 Win11 任务栏不是一次定位就结束
- Simpler Alternative Considered: 只在启动时定位一次
- Why Not More Complex: 不做完整控制器，只做服务当前可见性的最小重算

### Decision: 人工验证为主，脚本验证为辅

- Choice: 把“用户肉眼能看到模块”设为主成功标准
- MVP Justification: 当前自动截图链路已被证明存在观测偏差
- Simpler Alternative Considered: 只看日志和 `summary.json`
- Why Not More Complex: 现阶段构建更强视觉自动化工具并不能直接交付 MVP

## Alternatives Considered

- Description: 保持当前父窗口矩阵，继续试 `rebar` / `task_switch` / `composition_bridge`  
  Advantages: 不需要立刻删代码  
  Disadvantages: 继续扩大搜索空间，难以形成清晰主线  
  MVP Fit: 差  
  Reason Not Selected: 当前需要收敛，不需要继续横向发散

- Description: 直接移植 TrafficMonitor 的完整任务栏层级和刷新体系  
  Advantages: 更贴近已验证产品  
  Disadvantages: 体量过大，且会把透明、主题、渲染一起带进来  
  MVP Fit: 差  
  Reason Not Selected: 当前只需要最小可见闭环

- Description: 回退到普通桌面浮窗来证明 Rust 绘制没问题  
  Advantages: 易实现  
  Disadvantages: 不能解决“任务栏里为什么看不到”  
  MVP Fit: 不合格  
  Reason Not Selected: 无法回答当前核心问题

- Description: 先做 `Classic / Win11` 双策略抽象再改实现  
  Advantages: 看起来更整洁  
  Disadvantages: 在只有一个真实路径时属于提前抽象  
  MVP Fit: 差  
  Reason Not Selected: 当前没有第二个已确认宿主路径可复用

## Implementation Plan

### Phase 1: 收敛 Win11 宿主路径

- Goal:
  - 把运行时主路径缩成一个明确的 Win11 宿主和定位模型
- Files:
  - `taskbar-widget/src/taskbar.rs`
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/win32.rs`
- Tasks:
  - 将 Win11 主路径父窗口固定为 `Shell_TrayWnd`
  - 保留 `TrayNotifyWnd` / `Start` 作为定位信息输入
  - 简化默认配置，避免启动后仍优先走实验矩阵思路
  - 记录父窗口、锚点、最终矩形和窗口样式
- Expected Result:
  - 代码路径不再围绕“猜哪一个父窗口是对的”运转
- MVP Check:
  - Why this phase is necessary:
    - 当前问题就是宿主选择没有收敛
  - What is intentionally not included:
    - 不做经典任务栏支持
    - 不做多显示器
    - 不做透明

### Phase 2: 修正窗口样式和可见定位

- Goal:
  - 让模块在 Win11 任务栏中真实可见
- Files:
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/taskbar.rs`
- Tasks:
  - 把窗口创建样式改成更适合嵌入的保守样式
  - 增加 `WS_EX_TOOLWINDOW` 等与任务栏内嵌一致的最小 ex-style
  - 使用 `TrayNotifyWnd` / `Start` 重新计算 X/Y
  - 必要时增加最小位置刷新
  - 保持深色不透明背景和固定文本
- Expected Result:
  - 启动后，用户能在任务栏中直接看到 `TASKBAR WIDGET`
- MVP Check:
  - Why this phase is necessary:
    - “挂进去但看不见”不算 MVP
  - What is intentionally not included:
    - 不做主题跟随
    - 不做透明
    - 不做高级渲染

### Phase 3: 收敛验证方式

- Goal:
  - 把验证机制改成对当前 Win11 路径真正有用的证据组合
- Files:
  - `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
  - `docs/checklist/taskbar-visibility-diagnosis-loop.md`
  - 可选 `taskbar-widget/README.md`
- Tasks:
  - 将“人工可见性”写成主成功标准
  - 让脚本输出把事实和推断分开
  - 保留 `child render`、`attach success`、`module rect` 等硬证据
  - 降低对 `PrintWindow` ROI delta 的唯一依赖
- Expected Result:
  - 调试时能快速判断是“产品没显示”还是“观测工具不可信”
- MVP Check:
  - Why this phase is necessary:
    - 当前已经出现工具误导，会直接影响后续判断
  - What is intentionally not included:
    - 不做完整自动回归平台
    - 不做复杂图像分析

## Validation Strategy

最小验证不要追求自动化完美，而要直接回答“用户现在能不能看到模块”。

- The minimum tests needed:
  - `cargo check`
  - `cargo run`
  - 至少两次重复启动与关闭

- Manual checks:
  - 启动后，任务栏中能看到 `TASKBAR WIDGET`
  - 桌面中没有单独普通浮窗
  - 关闭程序后模块消失
  - 第二次启动仍能在同一区域看到模块

- Commands to run:
  - `cargo check`
  - `cargo run`
  - 必要时：`.\scripts\diagnose-taskbar-loop.ps1 -Parents shell -Anchors start,task_switch -CoordModes rect_delta,screen_to_client`

- Expected outputs:
  - 日志中可见非空 `Shell_TrayWnd`
  - 结构化诊断里 `attach.success=true`
  - 子窗口自身可绘制
  - 用户肉眼能在任务栏内看到模块

- Failure cases that matter for the MVP:
  - 任务栏里仍然看不到模块
  - 模块仍表现为普通浮窗
  - 模块只偶尔出现，无法重复
  - 一旦加入轻量重定位仍无法稳定显示

- What does not need to be tested yet:
  - 多显示器
  - 透明背景
  - 主题切换
  - Explorer 重启恢复
  - 经典任务栏

## Risks and Mitigations

- Risk: 当前 Win11 构建下即使挂到 `Shell_TrayWnd` 仍不可见  
  Impact: 当前 MVP 主路径失效  
  Likelihood if known: 中  
  MVP Mitigation: 先严格对齐 TrafficMonitor 的父窗口、定位锚点和窗口样式  
  Fallback Plan: 只在这一条路径失败后，再做一次最小差异对照，不立即恢复全矩阵

- Risk: 当前窗口样式本身不适合任务栏内嵌  
  Impact: `SetParent` 成功但最终不可见或行为异常  
  Likelihood if known: 高  
  MVP Mitigation: 改成更保守的无边框/工具窗口风格，先保留不透明背景  
  Fallback Plan: 用更简单的窗口样式继续缩小变量

- Risk: Win11 位置需要持续重算，启动一次定位不足  
  Impact: 启动时短暂出现或根本看不到  
  Likelihood if known: 中  
  MVP Mitigation: 增加最小位置刷新，而不是完整控制器  
  Fallback Plan: 先用固定周期刷新证明可见性，再决定是否保留

- Risk: 自动诊断脚本继续误导实现判断  
  Impact: 走错排障方向，浪费时间  
  Likelihood if known: 高  
  MVP Mitigation: 人工观察优先，脚本只提供辅助事实  
  Fallback Plan: 暂时停用视觉 delta 作为 gate，只保留 child render 和 attach facts

## Over-Engineering Watchlist

- 不要现在做 `Classic / Win11` 双策略框架
- 不要继续给 Win11 主路径添加新的父窗口候选
- 不要引入 D2D、DirectComposition、透明背景
- 不要创建插件系统或通用模块系统
- 不要为了“更优雅”重写整个 `taskbar-widget`
- 不要把诊断脚本扩成完整视觉自动化平台
- 不要因为 README 文档更新而顺手重构全部计划文档
- 不要在没有第二个真实用例时抽象通用宿主接口

## Add-on: UI / Product Feature Projects

- Primary User Flow:
  - 用户运行 `cargo run`，然后直接在主任务栏里看到固定文本模块。

- Minimum Acceptable UX:
  - 看得见、不是浮窗、能重复出现即可。

- Existing Components to Reuse:
  - 现有 `taskbar-widget` 窗口创建、GDI 绘制和诊断日志。

- States Required for MVP:
  - 启动中
  - 已嵌入且可见
  - 关闭后消失

- States Deferred:
  - 透明主题适配
  - 动态数据
  - 右键菜单
  - 错误提示 UI

- User Validation Signal:
  - 用户肉眼确认任务栏内出现固定文本模块。

## Open Questions

> No blocking open questions. Proceed with the simplest MVP assumptions listed above.

## Recommended Next Step

先修改 `taskbar-widget/src/taskbar.rs` 和 `taskbar-widget/src/main.rs`，把 Win11 主路径收敛成：

- `Shell_TrayWnd` 作为父窗口
- `Start` / `TrayNotifyWnd` 只做定位
- 更保守的嵌入窗口样式

然后立即用本机肉眼验证一次 `cargo run` 的可见性。
