# Tauri Settings IPC Contract

日期：2026-07-03

## Objective

定义 `taskbar-widget` 宿主与 `taskbar-settings-tauri` 之间的最小通信协议，先保证：

- 命令名固定
- DTO 固定
- 传输路径固定
- 假数据 backend 与未来真实宿主 adapter 共享同一协议面

当前不追求：

- 本轮就把真实宿主 IPC server 全部接通
- 本轮就切默认 settings 主入口

## Transport Choice

本次迁移选择的保守通信路径是：

- 传输：Windows 本地 named pipe
- pipe 名称：`\\\\.\\pipe\\cc-traffic-light-settings-v1`
- 编码：UTF-8 JSON envelope，单请求/单响应
- 持久化事实源：仍然是 `%APPDATA%\\CcTrafficLight\\config.json`

理由：

- 项目本身就是 Windows-only host
- named pipe 不需要开放端口
- Tauri Rust backend 可以安全地承担 pipe client
- 前端 WebView 不直接碰宿主 pipe，只调用 Tauri commands

## Protocol Version

- 协议版本：`cc_traffic_light.settings.v1`

任何后续 breaking change 都应提升版本，而不是复用现有 envelope。

## Required Commands

### `get_snapshot`

用途：

- 读取 settings 展示所需的当前只读状态快照

返回：

- `StatusSnapshotView`

### `get_settings`

用途：

- 读取当前正式配置

返回：

- `AppConfig`

### `save_settings`

用途：

- 提交完整 settings 写回

输入：

- `AppConfig`

返回：

- `SettingsSaveResultDto`

约束：

- 配置文件仍是事实源
- 未来宿主收到此命令后，需要先落盘，再触发即时应用

### `request_refresh`

用途：

- 请求宿主立即刷新 detector / runtime snapshot

返回：

- `SettingsRefreshResultDto`

### `notify_settings_applied`

用途：

- settings 侧把已应用字段回传给宿主，用于日志、后续 toast 或 apply bookkeeping

输入：

- `SettingsAppliedNotificationDto`

## Envelope Shape

```json
{
  "protocol_version": "cc_traffic_light.settings.v1",
  "request_id": "req-123",
  "command": {
    "save_settings": {
      "settings": {}
    }
  }
}
```

当前实现状态：

- 协议常量和 DTO 已进入 `shared-core`
- `taskbar-widget` 已启动 named pipe server，并由 `settings_bridge` 承接请求
- Tauri backend 默认要求 named pipe 可用；仅在显式设置 `CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND=1` 时才允许回退 fake backend
- 持久化事实源仍然是 `%APPDATA%\\CcTrafficLight\\config.json`

## Layering Rules

- 前端只认识 Tauri command 名，不认识 named pipe 细节
- Tauri Rust backend 只认识 shared-core 协议 DTO
- 宿主 adapter 负责把 shared-core 命令落到 Win32 / registry / message loop
- config file 继续作为持久化事实源，不把 named pipe 变成第二事实源
