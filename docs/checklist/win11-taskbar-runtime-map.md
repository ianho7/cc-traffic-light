# Win11 Taskbar Runtime Map

## Purpose

记录当前主力机器上实际观察到的任务栏窗口结构、选定的父窗口与锚点，以及当前最小定位规则。

这份文档服务于：

- `P2-02`
- `P2-03`
- `P2-06`
- `P6-01`
- `DOC-03`
- `DOC-04`

## Observed Runtime Snapshot

观测时间：`2026-06-29`

根窗口：

- `Shell_TrayWnd` `0x10102` `Rect=0,1104,2048,1152`

直接相关子窗口：

- `Start` `0x10106` `Rect=583,1104,628,1152`
- `TrayDummySearchControl` `0x10108` `Rect=0,1104,0,1104`
- `TrayNotifyWnd` `0x1010A` `Rect=1802,1104,2048,1152`
- `ReBarWindow32` `0x1010E` `Rect=628,1104,1288,1152`
- `MSTaskSwWClass` `0x10110` `Rect=628,1104,1288,1152`
- `MSTaskListWClass` `0x10112` `Rect=628,1104,1288,1152`
- `Windows.UI.Composition.DesktopWindowContentBridge` `0x1022A` `Rect=0,1104,2048,1152`
- `Windows.UI.Input.InputSite.WindowClass` `0x1022C` `Rect=0,1104,0,1104`

## Chosen Runtime Decisions

当前实现采用：

- 候选父窗口：`Shell_TrayWnd`
- 位置锚点：`TrayNotifyWnd`
- 子窗口样式：`WS_CHILD | WS_VISIBLE`

选择理由：

- `Shell_TrayWnd` 是当前机器上可稳定探测到的根宿主
- `TrayNotifyWnd` 在右侧边界稳定，可用于反推出一个保守模块区域
- `ReBarWindow32` / `MSTaskSwWClass` 已被记录，但当前先不把布局耦合到它们的内部策略

## Current Layout Rule

当前最小定位规则：

- 读取 `Shell_TrayWnd` 矩形作为父矩形
- 读取 `TrayNotifyWnd` 矩形作为右侧锚点
- 使用固定模块宽度与固定 margin
- 把模块放到 `TrayNotifyWnd` 左侧

当前一次实测结果：

- `parent_rect=0,1104,2048,1152`
- `anchor_rect=1802,1104,2048,1152`
- `module_rect=1307,1104,1435,1142`

## Visual Evidence

当前任务栏捕获图：

- [taskbar-phase4-printwindow.png](./taskbar-phase4-printwindow.png)

这张图中可见任务栏右侧出现 `TASKBAR WIDGET` 文本模块，说明：

- 子窗口已嵌入任务栏层级
- 子窗口已移动到任务栏矩形带内
- 固定文本已可见

## Repeatability Evidence

当前已验证至少两轮：

- 启动
- 探测 `Shell_TrayWnd`
- `SetParent`
- `MoveWindow`
- 通过 `WM_CLOSE` 优雅关闭
- 再次启动重复成功

运行信号：

- `taskbar attach result success=true`
- `taskbar layout result moved=true`
- `WM_DESTROY received`
- `message loop exited cleanly`
