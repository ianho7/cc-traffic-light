# Plans

这里存放后续实施计划。按优先级拆成独立文件夹，方便后续分别转成 checklist 并单独执行。

## 优先级顺序

1. [p0-codex-state-write](./p0-codex-state-write/README.md)：把已验证的 Codex lifecycle hooks 从 dump 模式切到真实状态写入。
2. [p1-global-hook-install](./p1-global-hook-install/README.md)：安装用户级全局 hooks，一次 trust 后让所有本地 Codex 会话静默触发回调。
3. [p2-claude-code-hook-validation](./p2-claude-code-hook-validation/README.md)：采样并验证真实 Claude Code payload。
4. [p3-taskbar-traffic-light-ui](./p3-taskbar-traffic-light-ui/README.md)：把当前文本 widget 演进为任务栏红绿灯组件。
5. [p4-runtime-hardening](./p4-runtime-hardening/README.md)：验证并发、性能、stale 恢复、安装回滚和长时间运行稳定性。
6. [gui-tray-v1-requirements.md](./gui-tray-v1-requirements.md)：为纯 Win32 GUI、tray、diagnostics、autostart 和零配置 detector 定义 V1 需求。
7. [slint-settings-migration-plan.md](./slint-settings-migration-plan.md)：为“保留 Win32 widget/tray、仅把 settings 迁移到 Slint”的混合方案定义实施计划。

## 当前依赖链

Codex lifecycle hook payload 验证已经通过。当前最先要做的是 P0：证明真实 Codex hooks 能写入 `state.json`，并驱动正在运行的任务栏 widget 更新。
