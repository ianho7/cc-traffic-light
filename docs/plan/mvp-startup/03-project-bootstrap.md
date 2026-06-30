# 03. 项目初始化指南

## 1. 新项目建议

建议新建一个独立 Rust 项目：

```powershell
cargo new taskbar-widget --bin
```

建议项目目录结构保持极小：

```text
taskbar-widget/
  Cargo.toml
  src/
    main.rs
    taskbar.rs
    win32.rs
```

## 2. Cargo 依赖建议

MVP 只建议引入一个核心依赖：

```toml
[dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_System_LibraryLoader",
  "Win32_UI_WindowsAndMessaging",
] }
```

如果你后续需要更细分的 API，再增 feature。

MVP 不建议现在就加：

- GUI 框架
- 日志框架
- 序列化框架
- 配置库

## 3. 第一批必须接触的 Win32 API

### 窗口创建

- `RegisterClassW`
- `CreateWindowExW`
- `DefWindowProcW`
- `ShowWindow`
- `UpdateWindow`

### 消息循环

- `GetMessageW`
- `TranslateMessage`
- `DispatchMessageW`
- `PostQuitMessage`

### 绘制

- `BeginPaint`
- `EndPaint`
- `DrawTextW`
- `FillRect`
- `SetBkMode`
- `SetTextColor`

### 任务栏宿主

- `FindWindowW`
- `FindWindowExW`
- `SetParent`
- `GetWindowRect`
- `MoveWindow`

### 调试

- `GetLastError`

## 4. 第一版最小窗口行为

建议第一版先做到：

- 窗口标题固定
- 固定尺寸，例如 `120 x 24`
- 黑底白字
- 文本居中显示

先不要：

- 动态改变窗口大小
- 响应鼠标
- 透明背景

## 5. 目标系统选择建议

如果你能选环境，优先建议：

- 一台你能反复调试的本机
- 不要远程环境
- 不要虚拟机里先做

如果你有 Win10 和 Win11 都可选：

- 从你最容易拿到 Explorer 窗口结构信息的那台开始

如果你当前主力机器就是 Win11：

- 就直接做 Win11 的最小路径

但原则仍然不变：

- 只支持一个路径

## 6. 第一版日志建议

MVP 只需要最小日志：

- 找到的任务栏句柄
- 找到的辅助窗口句柄
- `SetParent` 是否成功
- `GetLastError` 的值
- 最终 `MoveWindow` 的坐标

不要上完整日志系统。

直接：

- `println!`
- 或调试输出

就够了。

## 7. 第一个可运行里程碑

你应该先实现一个不嵌任务栏的普通窗口版本。

目标：

- 程序能跑
- 能收到 `WM_PAINT`
- 能画出文本

只有这个最小窗口稳定以后，再加任务栏嵌入。

原因很简单：

- 否则你分不清是“Win32 窗口没写对”
- 还是“任务栏嵌入逻辑有问题”
