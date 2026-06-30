# 01. MVP 计划

> 注：本文件是“从零启动”时的基线计划。基于当前仓库已经存在的 `taskbar-widget` 和最新 Win11 诊断结论，执行时请优先阅读 [09-win11-diagnosis-replan.md](/D:/project/cc-traffic-light/docs/plan/mvp-startup/09-win11-diagnosis-replan.md)。

## Objective

目标是实现一个最小 Rust PoC，在 Windows 任务栏里显示一个自定义模块，用来证明“Rust + Win32 + 子窗口嵌入 + GDI 自绘”是可行路线。

最小有用结果：

- 程序可运行
- 自定义窗口被挂到任务栏宿主内
- 在该窗口中显示固定文本

MVP 范围内：

- 独立 Rust 程序
- 单目标 Windows 环境
- 主任务栏
- GDI 文本绘制
- 启动、显示、关闭、复现

MVP 范围外：

- 全版本兼容
- 动态数据
- 透明与 D2D
- 多屏
- 插件系统

主要成功信号：

- 自定义模块稳定出现在任务栏中，而不是普通桌面窗口

## MVP Scope Boundary

### Must Have

- Requirement: 通过 Rust 创建原生 Win32 窗口  
  MVP Justification: 没有原生窗口就没有任务栏子窗口。

- Requirement: 通过 Win32 API 查找任务栏宿主窗口  
  MVP Justification: 这是嵌入任务栏的必要前提。

- Requirement: 调用 `SetParent` 把窗口挂到任务栏父窗口  
  MVP Justification: 这是 MVP 的核心验证点。

- Requirement: 通过 GDI 绘制固定文本  
  MVP Justification: 证明嵌入后的窗口是可见且可控的。

- Requirement: 具备最小位置计算与 `MoveWindow` 调整  
  MVP Justification: 不定位就无法证明“任务栏模块”成立。

- Requirement: 提供最小调试输出  
  MVP Justification: `SetParent`、查找句柄、定位失败时需要快速判断。

### Must Not Have

- Excluded Item: D2D / DComposition  
  Reason for Exclusion: 与核心验证无关，只会扩大变量。

- Excluded Item: 透明背景  
  Reason for Exclusion: 会引入分层窗口和额外重绘问题。

- Excluded Item: 动态刷新数据  
  Reason for Exclusion: 固定文本已经足够验证宿主模型。

- Excluded Item: 多显示器  
  Reason for Exclusion: 会显著增加任务栏枚举和位置分支。

- Excluded Item: 插件化模块  
  Reason for Exclusion: 当前只有一个模块，没有第二个真实用例。

- Excluded Item: GUI 框架  
  Reason for Exclusion: 任务栏嵌入依赖底层 Win32 控制，框架反而增加不确定性。

### Deferred Until After MVP

- Deferred Item: Win11 与经典任务栏双策略  
  Why Deferred: MVP 只需要单目标系统打通。  
  Signal to Reconsider: 当前目标环境已经稳定，准备扩展第二个 Windows 版本。

- Deferred Item: 多显示器支持  
  Why Deferred: 不影响路线验证。  
  Signal to Reconsider: 需要在副屏任务栏上显示模块。

- Deferred Item: 实时数据与定时刷新  
  Why Deferred: 不是当前最小闭环的一部分。  
  Signal to Reconsider: 需要承载 CPU/网速等动态内容。

- Deferred Item: 透明和主题跟随  
  Why Deferred: 只影响视觉，不影响路线验证。  
  Signal to Reconsider: 需要更接近产品形态。

## Background and Context

当前参考实现来源于 TrafficMonitor，但新 MVP 将脱离该仓库单独运行。

关键背景：

- 原方案不是官方 DeskBand，而是自建窗口嵌入任务栏
- 原方案对 Explorer 内部窗口结构有依赖
- Rust MVP 的目标不是迁移现有代码，而是验证核心技术路线

关键假设：

- 先只验证一台目标机器上的一个 Windows 版本
- 先接受保守位置与固定文本
- 先使用 `windows` crate 调 Win32 API

## Current State Analysis

当前没有 Rust 代码，也没有现成的新项目结构。

已有可用输入只有两类：

- TrafficMonitor 架构文档
- TrafficMonitor C++ 源码中的实现思路

当前限制：

- 没有已经验证过的 Rust 任务栏嵌入样例
- 目标系统版本未被固定
- 新项目尚未建立

## MVP Decision Gate

- Does this solve the immediate problem? 是。
- Can the MVP be validated without dynamic data? 可以。
- Can the MVP be validated without D2D? 可以。
- Is any part preparing for future scale? 如果加入插件、透明、多屏，就是。
- Are new dependencies necessary? 仅 `windows` crate 是必要的。
- What is the simplest acceptable implementation? 单窗口、固定文本、GDI、单系统。

- Keep: 原生 Win32、`SetParent`、GDI、自定义窗口、最小定位
- Remove: GUI 框架、插件、透明、D2D、多显示器
- Defer: 第二种任务栏策略、动态内容、主题适配
- Simplify: 先不要抽象成完整平台，只做最短路径

## Proposed MVP Solution

### Decision: 独立可执行程序

- Choice: 先做一个新 Rust 二进制项目
- MVP Justification: 最快验证核心闭环
- Simpler Alternative Considered: 脚本或现有 C++ 集成
- Why Not More Complex: 当前不是集成问题，是路线验证问题

### Decision: 原生 Win32 API

- Choice: 使用 `windows` crate
- MVP Justification: 任务栏嵌入依赖底层句柄与消息循环
- Simpler Alternative Considered: GUI 框架
- Why Not More Complex: GUI 框架并不针对任务栏宿主嵌入

### Decision: 单一目标系统路径

- Choice: 只实现一套最小任务栏定位逻辑
- MVP Justification: 可以快速拿到可验证结果
- Simpler Alternative Considered: 只显示普通窗口
- Why Not More Complex: 同时支持多个 Windows 版本会破坏 MVP 边界

### Decision: GDI 固定文本绘制

- Choice: `WM_PAINT` 中画固定文本
- MVP Justification: 足够证明窗口存在、可控、可见
- Simpler Alternative Considered: 不绘制内容
- Why Not More Complex: 复杂绘制对当前验证没有额外价值

## Alternatives Considered

- Description: 直接实现最终架构  
  Advantages: 后续少重写  
  Disadvantages: 当前风险和范围都会失控  
  MVP Fit: 差  
  Reason Not Selected: 当前目标是验证，不是完备设计

- Description: 直接做 Win11 完整兼容  
  Advantages: 面向最新系统  
  Disadvantages: 定位风险最高  
  MVP Fit: 差  
  Reason Not Selected: 不是最快闭环

- Description: 只做桌面浮窗模拟  
  Advantages: 容易实现  
  Disadvantages: 不能证明任务栏嵌入成立  
  MVP Fit: 不合格  
  Reason Not Selected: 无法回答核心问题

## Recommended Next Step

创建一个新的 Rust 可执行项目，并先完成普通 Win32 窗口 + 消息循环 + GDI 固定文本显示。
