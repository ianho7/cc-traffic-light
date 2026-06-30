# Rust Windows 任务栏自定义模块 MVP 计划包

这是一套可以脱离当前仓库单独使用的文档，目标是指导你在一个全新的 Rust 项目里，实现一个最小可行的 Windows 任务栏自定义模块。

这套文档不是完整产品设计。

这套文档只服务一个目标：

> 用 Rust 在 Windows 任务栏里成功嵌入一个自定义模块，并稳定显示一段自绘内容。

## 当前状态

这套文档最初是“从零启动”的计划包。

现在仓库里已经有一个可运行的 Rust PoC：

- `taskbar-widget`

并且已经完成一轮针对 Win11 主力机器的诊断。

当前最关键的新事实不是“Rust 能不能调用 Win32”，而是：

- 子窗口自身已经能绘制内容
- `SetParent` 和 `MoveWindow` 已经可以成功返回
- 但当前 Win11 路线下，模块仍然没有稳定进入任务栏真实可见层

因此，当前执行时不要只按 `01-08` 的启动前假设推进。

请优先看本目录新增的重计划：

- [09-win11-diagnosis-replan.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/09-win11-diagnosis-replan.md)

## 这套计划解决什么问题

你现在不是要迁移 TrafficMonitor，也不是要做最终架构。

你现在只要验证三件事：

1. Rust 能不能直接操作 Windows 任务栏相关窗口。
2. Rust 能不能把一个自定义窗口挂进任务栏父窗口。
3. Rust 能不能在这个窗口里稳定绘制自己的内容。

只要这三件事成立，这条技术路线就成立。

## MVP 定义

MVP 的最小结果是：

- 一个独立 Rust 可执行程序
- 在 Windows 主任务栏中显示一个小模块
- 模块内容是固定文本，例如 `TASKBAR WIDGET`
- 模块不是普通浮窗，而是嵌入任务栏层级
- 可以反复启动和关闭，结果稳定复现

## 明确不在 MVP 范围内

- 不做动态监控数据
- 不做插件系统
- 不做透明背景
- 不做 D2D / DirectComposition
- 不做多显示器
- 不做 Win10 / Win11 全兼容
- 不做设置界面
- 不做右键菜单
- 不做完整恢复和异常自愈机制

## 文档目录

- [01-mvp-plan.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/01-mvp-plan.md)
- [02-scope-and-decisions.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/02-scope-and-decisions.md)
- [03-project-bootstrap.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/03-project-bootstrap.md)
- [04-implementation-phases.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/04-implementation-phases.md)
- [05-file-layout.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/05-file-layout.md)
- [06-validation-and-debugging.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/06-validation-and-debugging.md)
- [07-risks-and-watchlist.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/07-risks-and-watchlist.md)
- [08-trafficmonitor-reference-map.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/08-trafficmonitor-reference-map.md)
- [09-win11-diagnosis-replan.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/09-win11-diagnosis-replan.md)

## 使用方式

如果你要在全新项目直接开始做，建议按这个顺序看：

1. `01-mvp-plan.md`
2. `02-scope-and-decisions.md`
3. `03-project-bootstrap.md`
4. `04-implementation-phases.md`
5. `06-validation-and-debugging.md`

只有在你想对照 TrafficMonitor 原实现时，再看：

6. `08-trafficmonitor-reference-map.md`

如果你要继续推进当前仓库里已经存在的 `taskbar-widget`，建议按这个顺序看：

1. `09-win11-diagnosis-replan.md`
2. `06-validation-and-debugging.md`
3. `07-risks-and-watchlist.md`
4. `08-trafficmonitor-reference-map.md`

## 成功标准

满足下面 4 条就算 MVP 成功：

- 程序启动后，任务栏中出现自定义模块
- 模块中能显示固定文本
- 模块不是独立浮窗
- 程序关闭后模块消失，重新启动可重复出现

## 推荐执行方式

建议你在一个全新的 Rust 项目里执行这套计划，项目名可以是：

- `taskbar-widget`
- `taskbar-host-poc`
- `win-taskbar-widget-poc`

不要把它一开始就做成大工程。

不要一开始就设计成通用平台。

先跑通一个最小闭环，再扩展。
