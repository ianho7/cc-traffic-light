# Win11 Taskbar Widget Preflight

## Environment Snapshot

- Date: 2026-06-29
- Rust toolchain:
  - `cargo 1.93.0 (083ac5135 2025-12-15)`
  - `rustc 1.93.0 (254b59607 2026-01-19)`
- Target machine evidence:
  - `DisplayVersion: 25H2`
  - `CurrentBuild: 26200`
  - `UBR: 8655`
  - `ProductName: Windows 10 Pro`

## Routing Decision

虽然 `ProductName` 仍显示为 `Windows 10 Pro`，但 `CurrentBuild 26200` 与 `DisplayVersion 25H2` 已足以表明当前机器应按 Win11 路径处理。

因此本 MVP 仍按：

- 单一目标系统：当前主力 Win11
- 单一路径：主任务栏
- 单一图形路径：Win32 + GDI

## Scope Locks

- 不并行展开 Win10 / 经典任务栏分支
- 不并行展开多显示器
- 不引入 GUI 框架、插件系统、主题系统
- 不在 Phase 1 提前写 `SetParent`、任务栏定位和透明绘制

## Taskbar Alignment and DPI

- `TaskbarAl` 注册表项当前未显式存在
- 这通常意味着使用系统默认值；在 Win11 上默认一般为居中
- 该项需要在后续手动验证阶段做一次肉眼确认
- `GetDpiForSystem() = 96`
- 当前系统 DPI 可按 `100%` 处理

## Project Placement

- Rust PoC project path: [taskbar-widget](../../taskbar-widget)
- Chosen layout:
  - `Cargo.toml`
  - `src/main.rs`
  - `src/taskbar.rs`
  - `src/win32.rs`

## Core API Set

- Window creation:
  - `GetModuleHandleW`
  - `RegisterClassW`
  - `CreateWindowExW`
  - `ShowWindow`
- Message loop:
  - `GetMessageW`
  - `TranslateMessage`
  - `DispatchMessageW`
  - `PostQuitMessage`
- Paint path:
  - `BeginPaint`
  - `EndPaint`
  - `GetClientRect`
  - `CreateSolidBrush`
  - `FillRect`
  - `DrawTextW`
  - `SetBkMode`
  - `SetTextColor`

## Immediate Validation Commands

- `cargo check`
- `cargo run`
