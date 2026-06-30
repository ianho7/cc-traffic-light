# 08. TrafficMonitor 参考映射

这份文档只用来告诉你：如果你需要对照当前 TrafficMonitor 的 C++ 实现，应该看哪里。

这不是新 Rust 项目的直接依赖。

## 1. 任务栏窗口生命周期

Rust MVP 里对应问题：

- 什么时候创建窗口
- 什么时候展示
- 什么时候关闭

TrafficMonitor 参考点：

- `OpenTaskBarWnd()` 负责创建任务栏窗口
- `CloseTaskBarWnd()` 负责关闭任务栏窗口

参考文档：

- `docs/taskbar-customization/02-architecture-deep-dive.md`

## 2. 子窗口挂载

Rust MVP 里对应问题：

- 如何把自己的窗口挂到任务栏父窗口下

TrafficMonitor 核心点：

- `SetParent(this->m_hWnd, GetParentHwnd())`

参考文档：

- `docs/taskbar-customization/01-overview-and-principles.md`
- `docs/taskbar-customization/03-reuse-guide.md`

## 3. 经典任务栏定位

Rust MVP 里对应问题：

- 如何在经典任务栏里找到宿主内部窗口
- 如何给自己的窗口腾出空位

TrafficMonitor 关键窗口：

- `Shell_TrayWnd`
- `ReBarWindow32`
- `WorkerW`
- `MSTaskSwWClass`
- `MSTaskListWClass`

参考文档：

- `docs/taskbar-customization/02-architecture-deep-dive.md`
- `docs/taskbar-customization/04-implementation-playbook.md`

## 4. Win11 定位

Rust MVP 里对应问题：

- 如何根据通知区和开始按钮反推位置

TrafficMonitor 关键窗口：

- `TrayNotifyWnd`
- `Start`

参考文档：

- `docs/taskbar-customization/02-architecture-deep-dive.md`
- `docs/taskbar-customization/06-glossary-and-pitfalls.md`

## 5. GDI 绘制路径

Rust MVP 里对应问题：

- 如何在子窗口中绘制最小内容

TrafficMonitor 参考：

- `OnPaint()`
- `ShowInfo()`
- `DrawDisplayItem()`

在 Rust MVP 里你不需要复刻它的完整绘制系统。

你只需要借用它的思路：

- 接收 `WM_PAINT`
- 建立绘图上下文
- 画背景
- 画文字

## 6. 目前最值得借鉴的不是代码，而是边界

TrafficMonitor 真正最值得借鉴的是：

- 它把任务栏窗口单独抽出来
- 它把平台差异拆成不同策略类
- 它把绘制和业务数据分开

但 Rust MVP 不应该现在就完整复刻这些抽象。

MVP 阶段只应该借鉴：

- 最小窗口嵌入路径
- 最小定位路径
- 最小绘制路径

## 7. 你在新项目里真正要做的事

不是“翻译 TrafficMonitor”。

而是：

1. 用 TrafficMonitor 证明这条路线在 C++ 里成立
2. 用 Rust 重新做一个更小的验证版
3. 等 MVP 成功后，再决定哪些抽象值得迁移

## 8. 建议的使用方式

如果你卡住了，可以按这个顺序回查旧文档：

1. 先看 `docs/taskbar-customization/01-overview-and-principles.md`
2. 再看 `docs/taskbar-customization/03-reuse-guide.md`
3. 需要模板时看 `docs/taskbar-customization/04-implementation-playbook.md`
4. 遇到兼容性问题时看 `docs/taskbar-customization/06-glossary-and-pitfalls.md`

但新项目执行时，仍以本目录下的计划文件为主。
