# 素材灯组 MVP 交付与运行验证

## 已交付

- 本地素材灯组：固定绿、黄、红三槽位；Canvas 裁剪后保存为 64×64 PNG。
- 灯组可复用，并可分别应用至 Codex、Claude Code；可恢复内建灯组。
- taskbar host 保留 Agent 标识、既有布局与整组点击热区；自定义素材复用原状态 alpha、常亮和闪烁规则。
- 资源丢失或损坏时，对应 Agent 回退内建灯组；使用中的灯组不可删除。
- 配置 schema 升至 5，旧配置安全回退。

## 自动验证

- `cargo test --workspace --offline`：通过。
- `cargo check -p taskbar-widget --offline`：通过。
- `pnpm -C taskbar-settings-tauri build`：通过。
- `cargo build -p taskbar-settings-tauri --offline` 后 `cargo build -p taskbar-widget --offline`：通过。
- 最终 host 验证路径：`D:\project\cc-traffic-light\target\debug\taskbar-widget.exe`。

## Windows 人工验收

用户确认：

- 三图上传、裁剪和保存可用。
- 应用至任务栏的自定义素材可用。
- Claude Code 状态和 hook 正常。
- 素材缺失会回退内建灯组。
- 窄窗口布局可用。

初始 Codex 空闲/不闪烁并非素材渲染问题。诊断发现 `~/.codex/hooks.json` 与 Windows wrapper 均指向已安装 hook，但安装器重写 hook 定义后，Codex Desktop 0.144.2 的用户级 hooks 需要重新 trust。

在 Codex Desktop 运行 `/hooks` 并重新 Review/Trust `C:\Users\admin\.codex\hooks.json` 中的 CcTrafficLight hooks 后，Codex 事件开始写入 `state.json`，状态和素材闪烁恢复。

## 后续安装注意事项

- Codex 的命令 hook 有定义哈希信任机制；升级/重装后定义变更可能需要用户在 `/hooks` 中重新信任。
- Claude Code 在本次环境中无需该额外 trust 步骤；其 settings hook 在安装后直接生效。
- 不要启用项目本地 `D:\project\cc-traffic-light\.codex\hooks.json` 的禁用条目；本功能使用的是用户级 `C:\Users\admin\.codex\hooks.json`。
