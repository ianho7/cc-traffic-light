# taskbar-widget 项目树

说明：

- 本清单聚焦源码、配置、UI 资源和辅助脚本。
- `target/` 属于构建产物目录，故不纳入正文。
- 每条摘要只描述“这个文件是做什么的”。

```text
taskbar-widget/
|-- .gitignore
|-- build.rs
|-- Cargo.toml
|-- README.md
|-- examples.claude-hooks.json
|-- examples.codex-hooks.toml
|-- FILETREE.md
|-- scripts/
|   |-- codex-lifecycle-hook-dump.ps1
|   |-- codex-notify-probe-config.ps1
|   |-- codex-notify-probe-wrapper.ps1
|   |-- diagnose-taskbar-loop.ps1
|   |-- diagnose-widget-liveness.ps1
|   |-- validate-tauri-settings-read-model.ps1
|   `-- install-codex-hooks.ps1
|-- src/
|   |-- agent_state.rs
|   |-- app_config.rs
|   |-- autostart.rs
|   |-- detector.rs
|   |-- hook_rules.rs
|   |-- i18n.rs
|   |-- lib.rs
|   |-- main.rs
|   |-- runtime_contract.rs
|   |-- settings_window.rs
|   |-- taskbar.rs
|   |-- tray_icon.rs
|   |-- ui_state.rs
|   |-- win32.rs
|   `-- bin/
|       `-- taskbar_widget_hook.rs
`-- ui/
    `-- i18n/
        |-- en.json
        `-- zh-CN.json
```

## 顶层文件

- `.gitignore`：忽略 Rust 构建输出和本地环境噪音文件。
- `build.rs`：在编译阶段为宿主嵌入 Windows manifest。
- `Cargo.toml`：定义 Rust 包信息、依赖和构建脚本入口。
- `README.md`：说明任务栏挂载验证目标、稳定路径和调试方法。
- `examples.claude-hooks.json`：提供 Claude Code hook 的示例配置文件。
- `examples.codex-hooks.toml`：提供 Codex hook 的示例配置文件。
- `FILETREE.md`：记录 `taskbar-widget` 的项目结构与文件用途。

## scripts

- `scripts/codex-lifecycle-hook-dump.ps1`：抓取并落盘 Codex 生命周期 hook 负载样本。
- `scripts/codex-notify-probe-config.ps1`：集中定义 Codex 通知探针的测试配置参数。
- `scripts/codex-notify-probe-wrapper.ps1`：包装 Codex 通知探针执行流程并输出诊断结果。
- `scripts/diagnose-taskbar-loop.ps1`：批量运行任务栏挂载诊断组合并收集结果。
- `scripts/diagnose-widget-liveness.ps1`：检查任务栏组件显示、刷新和存活状态。
- `scripts/validate-tauri-settings-read-model.ps1`：启动宿主并验证 live named-pipe settings read/write 基线。
- `scripts/install-codex-hooks.ps1`：安装或更新本项目使用的 Codex hook 配置。

## src

- `src/agent_state.rs`：管理 hook 状态文件、互斥锁、TTL 和汇总快照。
- `src/app_config.rs`：定义应用配置模型、默认值和配置文件读写。
- `src/autostart.rs`：处理开机自启状态的查询、同步和切换。
- `src/detector.rs`：把状态文件和进程观测聚合成界面可用状态快照。
- `src/hook_rules.rs`：解析 hook 负载字段并提取会话与状态线索。
- `src/i18n.rs`：加载本地化文案并提供界面显示用翻译接口。
- `src/lib.rs`：导出共享状态、配置、检测和本地化模块。
- `src/main.rs`：负责进程启动、窗口创建、托盘、轮询和消息循环。
- `src/runtime_contract.rs`：定义运行时约定的模块名与信号名集合。
- `src/settings_window.rs`：提供原生 GDI 设置窗口的极限后备实现与交互绑定。
- `src/taskbar.rs`：探测任务栏窗口、挂载子窗口并计算布局位置。
- `src/tray_icon.rs`：创建托盘图标菜单并分发用户命令。
- `src/ui_state.rs`：定义设置界面和组件绘制共享的状态结构。
- `src/win32.rs`：封装常用 Win32 辅助函数、日志和格式化工具。

## src/bin

- `src/bin/taskbar_widget_hook.rs`：接收外部 hook 事件并写入统一状态文件。

## ui/i18n

- `ui/i18n/en.json`：提供设置界面和状态文本的英文文案资源。
- `ui/i18n/zh-CN.json`：提供设置界面和状态文本的简体中文文案资源。

## Archive

- `../archive/slint-settings/`：保存已从主链路退场的 Slint settings host 与 UI 结构，供迁移回溯使用。
