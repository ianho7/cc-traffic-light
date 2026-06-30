# Taskbar Visibility Diagnosis Loop

## Purpose

针对当前问题建立一套可重复执行的 debug loop：

> 日志显示 `SetParent` / `MoveWindow` 成功，但用户在桌面上看不到普通窗口，也看不到任务栏模块。

核心原则不是继续猜，而是把每个猜想变成一条有 pass/fail 的实验。

## Ranked Hypotheses

1. `Shell_TrayWnd` 不是当前 Win11 上真正可见的宿主层；如果切换到 `ReBarWindow32` 或 `MSTaskSwWClass`，模块会出现。
2. 当前定位坐标虽然“数学上成立”，但不是父客户区坐标；如果改成 `screen_to_client` 模式，模块会出现。
3. 当前子窗口已经挂到任务栏树里，但被更上层的合成内容遮挡；如果比较任务栏截图前后，目标区域不会发生预期变化。
4. 当前窗口样式切换与显示时机组合不稳定；不同父窗口/坐标组合会出现明显不同的可见性结果。

## Feedback Loop

Loop 入口脚本：

- [diagnose-taskbar-loop.ps1](../../taskbar-widget/scripts/diagnose-taskbar-loop.ps1)

Loop 的输入变量：

- `TASKBAR_MVP_PARENT`
  - `shell`
  - `rebar`
  - `task_switch`
  - `composition_bridge`
- `TASKBAR_MVP_ANCHOR`
  - `tray_notify`
  - `task_switch`
  - `start`
  - `shell`
- `TASKBAR_MVP_COORD_MODE`
  - `rect_delta`
  - `screen_to_client`

应用会把结构化诊断写到：

- `TASKBAR_MVP_DIAG_FILE`

## What The Loop Measures

每个变体都会自动执行：

1. 启动前清理残留 `TaskbarWidgetWindow` 子窗口与残留进程
2. 按所选 parent 策略抓取一次候选父窗口图像
2. 启动应用并注入 parent/anchor/coord 变量
3. 等待诊断 JSON 落盘
4. 再抓一次候选父窗口图像
5. 直接抓一次应用子窗口图像
6. 根据 `module_rect` 裁剪前后父窗口图像同一区域
6. 计算：
   - `AttachSuccess`
   - `LayoutMoved`
   - `WithinParent`
   - `MeanDelta`
   - `BrightPixels`
   - `ChildBrightPixels`
7. 通过子窗口句柄发 `WM_CLOSE`
8. 输出每个变体的 `Pass/Fail`

## Pass Signal

当前 loop 分两层信号：

- `RenderPass`
  - 子窗口自身图像里有足够亮色像素，说明 `WM_PAINT` 本身是活的
- `VisualPass`
  - `attach.success == true`
  - `layout.moved == true`
  - `WithinParent == true`
  - 父窗口裁剪区域有效
  - `MeanDelta > 8`
  - `BrightPixels > 20`

当前总 `Pass` 判定是：

- `RenderPass == true`
- `VisualPass == true`

这表示：

- 子窗口自己确实画出来了
- 父窗口目标区域也发生了预期视觉变化

## How To Run

在 [taskbar-widget](/D:/project/cc-traffic-light/taskbar-widget) 下运行：

```powershell
./scripts/diagnose-taskbar-loop.ps1
```

如只想测试一组更聚焦的猜想：

```powershell
./scripts/diagnose-taskbar-loop.ps1 -Parents rebar,task_switch -Anchors task_switch,start -CoordModes rect_delta,screen_to_client
```

输出目录默认在：

```text
taskbar-widget/target/diagnose-taskbar-loop/
```

关键产物：

- `summary.json`
- `<variant>.json`
- `<variant>.before.png`
- `<variant>.after.png`
- `<variant>.child.png`
- `<variant>.stdout.log`

## How To Interpret Results

如果出现：

- `RenderPass=true` 但 `VisualPass=false`
  说明子窗口自己画出来了，但没有正确进入父窗口可见层

- `LayoutMoved=true` 但 `WithinParent=false`
  说明算出来的模块矩形已经跑到父窗口边界之外，锚点或坐标系不对

- `LayoutMoved=true` 且 `WithinParent=true`，但 `MeanDelta` 很低
  说明位置看似合理，但父窗口可见层没有呈现出这块变化，宿主层选择大概率不对

- `MeanDelta` 高且 `BrightPixels` 高
  说明这个变体更接近真实可见解

## Current Use

这套 loop 不是最终实现。

它服务于当前诊断阶段：

- 合理猜想
- 快速验证
- 记录哪条路径真正改变了可见结果
