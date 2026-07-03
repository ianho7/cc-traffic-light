# Slint Settings Migration Checklist

日期：2026-07-02

## Checklist Objective

把现有 [slint-settings-migration-plan.md](/D:/project/cc-traffic-light/docs/plan/slint-settings-migration-plan.md) 转成可执行 checklist，在不破坏当前稳定 Win32 widget / tray 主路径的前提下，把 settings 从纯 Win32 自绘迁移到 `Slint`。

目标结果：

- `settings` 改成 `Slint` 驱动的现代 UI。
- `widget`、`tray`、taskbar attach / retry 主路径继续保留 Win32。
- 所有设置项都进入统一本地持久化模型，关闭软件后重新打开保持一致。
- 所有用户可见字符串都纳入中英国际化，不只覆盖 settings 页面，还包括 tray 菜单、tooltip、状态标签和来源方法标签。
- settings 视觉方向参考 `$nothing-design`：Typography-first、单色基底、三层视觉层级、无阴影。

范围：

- 覆盖 `taskbar-widget/` 内的 settings host、状态桥接、配置 schema、国际化层和相关文档。
- 覆盖 Slint 接入、settings 替换、运行期验证和文档收口。

非目标：

- 不重写 `taskbar-widget/src/taskbar.rs`
- 不把 widget 改成 Slint
- 不把 tray 宿主改成 Slint
- 不引入 WebView、Tauri、WinUI
- 不拆成多进程

## Pre-Implementation Checks

- [x] SSM-PRE-01 阅读 [slint-settings-migration-plan.md](/D:/project/cc-traffic-light/docs/plan/slint-settings-migration-plan.md)，确认 Win32 / Slint 边界、持久化要求和 i18n 范围。
- [x] SSM-PRE-02 阅读 [taskbar-widget/src/main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)、[settings_window.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_window.rs)、[tray_icon.rs](/D:/project/cc-traffic-light/taskbar-widget/src/tray_icon.rs)，确认当前 settings/tray/runtime 入口。
- [x] SSM-PRE-03 阅读 [app_config.rs](/D:/project/cc-traffic-light/taskbar-widget/src/app_config.rs)、[ui_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/ui_state.rs)，确认现有 schema 和统一状态契约。
- [x] SSM-PRE-04 确认 `cargo check`、`cargo fmt -- --check` 仍是最小验证命令，并记录 Slint 接入后可能新增的 build step。
- [x] SSM-PRE-05 确认本 checklist 的反射文件命名规则沿用 `docs/reflections/task-<task-id>-<timestamp>.md`。

当前结论：

- Slint 迁移范围已经固定为 `Win32 widget/tray/main loop + Slint settings`，不包含 `taskbar.rs` 重写，也不包含 tray/widget 宿主迁移。
- 持久化要求已经提升为 schema gate：没有进入 `AppConfig` 的可写项，不能进入正式 settings UI。
- 国际化范围已经明确不是只覆盖 settings 页面，而是同时覆盖 tray 菜单、tooltip、状态标签、来源方法标签和 diagnostics/about 关键文案。
- 当前最小验证命令仍然是 `cargo check` 与 `cargo fmt -- --check`；Slint 接入后预计新增 `build.rs + slint-build` 生成步骤，但不改变最小验收口径。
- reflection 命名规则继续沿用 `docs/reflections/task-<task-id>-<timestamp>.md`，本 checklist 不单独发明新格式。

## Implementation Checklist

### Phase 1: Boundaries and Contracts

- [x] SSM-A-01 梳理当前 `settings_window.rs` 的职责，拆成展示层、状态适配层、命令层、持久化契约层、本地化契约层。
- [x] SSM-A-02 定义 Win32 和 Slint 的明确边界：Win32 保留 tray / main loop / widget host，Slint 只接管 settings presentation。
- [x] SSM-A-03 审计当前所有 settings 项，列出哪些已经进入 `AppConfig`，哪些未来必须补 schema 后才能进入正式 UI。
- [x] SSM-A-04 定义国际化覆盖面清单：settings 文案、tray 菜单、tray tooltip、状态标签、来源方法标签、diagnostics/about 说明文本。
- [x] SSM-A-05 确定语言策略字段进入配置模型，例如 `follow_system` / `zh-CN` / `en` 之类的持久化表示。

Phase 1 结论：

- `settings_window.rs` 当前混合了五类职责：
  - 展示层：窗口注册、布局、GDI 绘制、点击命中。
  - 状态适配层：`AppStatusSnapshot` / `AppConfig` 到页面块结构的投影。
  - 命令层：autostart 切换、刷新命令投递、页面切换。
  - 持久化契约层：修改 `AppConfig` 并调用 `save_config`。
  - 本地化契约层：目前缺失，用户可见文本基本为英文硬编码。
- `main.rs` 继续负责主消息循环、widget 更新、tray 回调和 settings 打开入口；Slint 不接管这些宿主职责。
- `tray_icon.rs` 继续保留托盘图标、菜单弹出、tooltip 更新职责，但其用户可见字符串必须改为共用 i18n 层。
- 当前已进入 `AppConfig` 的正式设置项：
  - `general.autostart_enabled`
  - `general.start_minimized_to_tray`
  - `general.close_to_tray`
  - `monitoring.codex_enabled`
  - `monitoring.claude_enabled`
  - `appearance.indicator_style`
  - `appearance.widget_size`
  - `appearance.show_labels`
  - `appearance.reduced_motion`
  - `diagnostics.last_opened_page`
  - `diagnostics.last_manual_refresh_at`
- 当前缺失但必须补 schema 后才能进入正式 UI 的设置项：
  - `language.mode` 或等价字段，至少支持 `follow_system`、`zh-CN`、`en`
  - 如后续 About/Diagnostics 暴露更多用户可写偏好，也必须先进入 `AppConfig`
- 国际化覆盖面已经固定为：
  - settings 全部页面文案
  - tray 菜单
  - tray tooltip
  - 状态标签
  - 来源方法标签
  - diagnostics/about 关键说明文本
- 语言策略的持久化建议先采用枚举型配置值：
  - `follow_system`
  - `zh-CN`
  - `en`
  - 解析失败时回退 `follow_system`

### Phase 2: Slint Infrastructure

- [x] SSM-B-01 在 `taskbar-widget` 中接入 `slint` 依赖和必要的构建脚本。
- [ ] SSM-B-01A 明确 Settings 设计依赖字体及加载方式：`Doto`、`Space Grotesk`、`Space Mono`。
- [x] SSM-B-02 新建 `taskbar-widget/ui/settings.slint`，实现基础静态布局：导航、Overview、General、Diagnostics、About/占位页。
- [x] SSM-B-03 新建 `taskbar-widget/src/settings_slint.rs`，实现最小的 Slint settings window 创建和生命周期管理。
- [x] SSM-B-04 建立 i18n 资源骨架，例如 `taskbar-widget/ui/i18n/en.json` 和 `zh-CN.json`。
- [x] SSM-B-05 约束 Slint 侧不直接硬编码业务字符串，所有用户可见文本都走 key 化资源。
- [x] SSM-B-06 建立第一版 Nothing-inspired 视觉 token：字号角色、单色调色板、间距尺度、边框/圆角规则。

Phase 2 结论：

- `taskbar-widget/Cargo.toml` 已接入 `slint` 与 `slint-build`，并新增 [build.rs](/D:/project/cc-traffic-light/taskbar-widget/build.rs) 编译 `ui/settings.slint`。
- [settings.slint](/D:/project/cc-traffic-light/taskbar-widget/ui/settings.slint) 已提供第一版静态 settings 壳子：
  - 左侧导航
  - 两张顶部 hero/status 卡片
  - Codex / Claude 双来源状态卡片
  - General 三项开关行
  - Diagnostics 卡片与按钮占位
- [settings_slint.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_slint.rs) 已建立最小 host：
  - `create`
  - `show`
  - `hide`
  - `update_snapshot`
  - `update_config`
- i18n 资源骨架已建立在：
  - [en.json](/D:/project/cc-traffic-light/taskbar-widget/ui/i18n/en.json)
  - [zh-CN.json](/D:/project/cc-traffic-light/taskbar-widget/ui/i18n/zh-CN.json)
- 当前 Slint 侧没有把业务文案直接硬编码进 `.slint` 组件；文案入口已集中在 Rust host，可在下一阶段替换为真实 i18n 访问层。
- 第一版 Nothing-inspired token 已在 Slint 骨架中落地为：
  - `Doto` 只用于 hero/state 数值
  - `Space Grotesk` 用于主体标题与正文
  - `Space Mono` 用于导航、标签和开关状态
  - 单色基底 `#000 / #111 / #222 / #333`
  - 无阴影，靠边框、间距、字号建立层级
  - 16px 卡片圆角、999px pill 按钮
- `SSM-B-01A` 仍未完成：目前只声明了字体角色并在 Slint 中引用 family name，尚未确定正式字体打包或运行时加载方案。

### Phase 3: Persistence and I18n Foundations

- [x] SSM-C-01 扩展 `AppConfig`，确保所有计划暴露的 settings 项都有明确 schema 落点。
- [x] SSM-C-02 把语言设置纳入 `AppConfig` 持久化，并定义默认值与系统语言跟随策略。
- [x] SSM-C-03 实现统一本地化访问层，供 settings、tray、tooltip、状态标签和方法标签共用。
- [x] SSM-C-04 为状态枚举和来源方法建立用户可见 label 映射，而不是直接显示内部英文 key。
- [x] SSM-C-05 为配置 round-trip 和语言资源完整性补最小测试或校验逻辑。

Phase 3 结论：

- [app_config.rs](/D:/project/cc-traffic-light/taskbar-widget/src/app_config.rs) 已升级到 schema v2，并新增 `localization.language` 持久化字段。
- 语言策略当前采用三态枚举：
  - `follow_system`
  - `zh-CN`
  - `en`
- 缺失 `localization` 字段的旧配置文件会自动回退到 `follow_system`，不破坏旧配置读取。
- 已新增共享 i18n 模块 [i18n.rs](/D:/project/cc-traffic-light/taskbar-widget/src/i18n.rs)，职责包括：
  - 读取 `en.json` / `zh-CN.json`
  - 根据配置推导有效 locale
  - 输出 settings / tray / tooltip / 状态 / 方法 / 可信度标签
- [settings_slint.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_slint.rs) 已不再手写业务文案，而是通过共享 i18n 层填充页面文本和状态详情。
- [tray_icon.rs](/D:/project/cc-traffic-light/taskbar-widget/src/tray_icon.rs) 已改为通过共享 i18n 层输出：
  - 右键菜单文案
  - tooltip 摘要
- 状态枚举、来源方法、可信度和 widget mount 状态都已有用户可见 label 映射，不再直接暴露内部英文 key。
- 已新增最小测试覆盖：
  - 旧配置缺失 `localization` 时的默认回退
  - 语言配置 round-trip
  - 显式 locale 的 localizer 行为
  - 中英资源 key 完整性一致

### Phase 4: Runtime State Bridge

- [x] SSM-D-01 定义 Slint 侧 view model，覆盖 Overview、General、Diagnostics 和语言选择所需字段。
- [x] SSM-D-02 实现 `AppStatusSnapshot -> Slint view model` 的状态推送。
- [x] SSM-D-03 实现 `AppConfig -> Slint UI state` 的状态推送。
- [x] SSM-D-04 让 tray 和 settings 共享统一 i18n 层，而不是各自维护字符串。
- [ ] SSM-D-05 确认 Slint settings 更新不会影响现有 `main.rs` 中 widget / detector 轮询路径。

Phase 4 当前结论：

- [settings.slint](/D:/project/cc-traffic-light/taskbar-widget/ui/settings.slint) 已扩展出第一版真正可消费的 view model 字段，覆盖：
  - Overview hero / widget / codex / claude 状态
  - General 三个布尔设置
  - 语言模式显示字段
  - Diagnostics 摘要、错误和双来源明细
- [settings_slint.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_slint.rs) 已不再只是本地同步壳子，而是具备独立 runtime bridge：
  - `start_runtime`
  - `show`
  - `update`
  - `shutdown_runtime`
- 当前 Slint host 通过独立 UI 线程启动 event loop，并通过 `Weak<SettingsWindow>::upgrade_in_event_loop(...)` 接收主线程推送的 snapshot/config 更新。
- `AppStatusSnapshot -> Slint` 的状态推送已覆盖：
  - `overall_state`
  - `widget_mount_state`
  - `last_widget_attach_at`
  - `last_detection_refresh_at`
  - `last_error_summary`
  - Codex / Claude 的 state / method / confidence / message
- `AppConfig -> Slint` 的状态推送已覆盖：
  - `general.autostart_enabled`
  - `general.start_minimized_to_tray`
  - `general.close_to_tray`
  - `localization.language`
- tray 与 settings 现在确实共享同一个 i18n 模块；不再各自维护独立文案逻辑。
- `SSM-D-05` 暂不打勾：虽然 `cargo check` / `cargo test --lib` 已通过，且桥接层没有改动 taskbar attach 算法，但仍缺一次实际运行层面的人工验证。

### Phase 5: Commands and Interaction

- [x] SSM-E-01 把 General 页开关事件回调接到现有 config/autostart backend。
- [x] SSM-E-02 把 Diagnostics 的 `Refresh Now` 接到现有主窗口刷新命令。
- [x] SSM-E-03 把语言切换事件接到持久化配置，并保证重启后仍保持上次语言。
- [x] SSM-E-04 把 tray 菜单文案改为通过 i18n 层输出。
- [x] SSM-E-05 把 tray tooltip 改为通过 i18n 层输出，并保持聚合状态摘要语义不变。

Phase 5 当前结论：

- [settings.slint](/D:/project/cc-traffic-light/taskbar-widget/ui/settings.slint) 已为以下交互暴露 callback：
  - `toggle_autostart`
  - `toggle_start_minimized`
  - `toggle_close_to_tray`
  - `cycle_language`
  - `request_refresh`
- [settings_slint.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_slint.rs) 已把这些 callback 绑定到现有 backend：
  - autostart 仍走 `autostart::set_enabled`
  - 普通布尔设置仍写回 `AppConfig`
  - 语言设置写回 `localization.language`
  - 刷新按钮仍通过 `WM_COMMAND + TRAY_CMD_REFRESH` 投递到主窗口
- [settings_window.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_window.rs) 已抽出可复用 backend：
  - `update_config`
  - `toggle_autostart_setting`
  - `cycle_language_setting`
  - `request_manual_refresh_command`
  - `current_snapshot`
- 当前语言切换会立即刷新 Slint 界面自身文案；tray 菜单读取同一配置，因此后续打开菜单时会立即使用新语言。
- “重启后保持语言一致”的静态证据仍来自 `AppConfig` round-trip 单测；本轮未新增运行期人工验证。

### Phase 6: Host Swap

- [x] SSM-F-01 修改 `Open Settings` 流程，默认打开 Slint settings。
- [x] SSM-F-02 保留旧 `settings_window.rs` 作为短期 fallback，直到 Slint 联调稳定。
- [x] SSM-F-03 确认 `settings_window.rs` 中不再承担主入口职责，只保留 fallback 或待删状态。
- [ ] SSM-F-04 验证 Slint settings 打开/关闭多次时不会破坏主消息循环。

Phase 6 当前结论：

- [main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs) 的 `Open Settings` 路径已经改为：
  - 优先尝试 Slint runtime
  - 启动或显示失败时回退到旧 Win32 settings
- 旧 [settings_window.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_window.rs) 仍然保留：
  - 窗口创建
  - snapshot/config 存储
  - 原有点击交互
  - fallback 展示职责
- 新的主入口职责已经转移给 Slint runtime bridge；Win32 settings 不再是默认路径，只是安全网。

### Phase 7: Visual and Documentation Closeout

- [ ] SSM-G-01 统一 Slint 主题 token：间距、字号层级、圆角、中性色、语义色。
- [ ] SSM-G-01A 把 Nothing-inspired 设计规则落实到 settings：三层视觉层级、单色基底、状态色事件化、无阴影。
- [ ] SSM-G-02 把 [gui-tray-v1-requirements.md](/D:/project/cc-traffic-light/docs/plan/gui-tray-v1-requirements.md) 中关于 settings 技术栈和国际化的描述更新为最新结论。
- [ ] SSM-G-03 根据实施结果更新 [gui-tray-v1.md](/D:/project/cc-traffic-light/docs/checklist/gui-tray-v1.md) 或新增交叉引用，避免文档漂移。
- [ ] SSM-G-04 更新 `docs/handoff/`，说明当前混合栈结构、已知限制和下一步建议。
- [ ] SSM-G-05 审计是否仍保持“Win32 widget/tray + Slint settings”的范围，没有漂移成全量 UI 重写。

## Validation Checklist

- [ ] SSM-VAL-01 `cargo check` 通过。
- [ ] SSM-VAL-02 `cargo fmt -- --check` 通过。
- [ ] SSM-VAL-03 Slint settings 可从 tray 正常打开，且默认入口不再是旧 Win32 自绘页面。
- [ ] SSM-VAL-04 Overview 中总状态、widget mount、Codex/Claude 卡片与实际运行一致。
- [ ] SSM-VAL-05 General 中所有设置项修改后会写入本地配置，并在关闭软件后重新打开保持一致。
- [ ] SSM-VAL-06 语言设置修改后能立即影响 settings 文案，并在重启后保持一致。
- [ ] SSM-VAL-07 tray 菜单支持中英切换，文案与当前语言一致。
- [ ] SSM-VAL-08 tray tooltip 支持中英切换，且聚合状态摘要无明显回归。
- [ ] SSM-VAL-09 用户可见状态标签与来源方法标签支持中英显示，而不是直出英文 key。
- [ ] SSM-VAL-10 Diagnostics 页面显示真实 detector 数据，且手动刷新有效。
- [ ] SSM-VAL-11 widget / tray / attach / retry 主路径在迁移后无明显回归。
- [ ] SSM-VAL-12 Slint settings 打开/关闭多次后，无明显句柄泄漏、卡死或状态不同步。
- [ ] SSM-VAL-13 在 Slint settings 创建失败时，存在明确 fallback 或可诊断行为。
- [ ] SSM-VAL-14 Settings 的视觉层级、字体角色、单色基底和状态色使用符合 Nothing-inspired 方向，而不是默认工具风格。

## Documentation Checklist

- [ ] SSM-DOC-01 更新计划文档，确保实施结论与 [slint-settings-migration-plan.md](/D:/project/cc-traffic-light/docs/plan/slint-settings-migration-plan.md) 一致。
- [ ] SSM-DOC-02 更新 GUI / Tray V1 需求文档中关于 settings 技术栈、持久化和国际化的描述。
- [ ] SSM-DOC-03 更新 handoff，记录当前混合栈边界、已知限制和后续建议。
- [ ] SSM-DOC-04 对每个完成、跳过或阻塞任务生成 reflection。
- [ ] SSM-DOC-05 明确记录国际化覆盖范围不是只限于 settings 页面。
- [ ] SSM-DOC-06 明确记录 Nothing-inspired 只作为 settings 视觉方向，不自动扩展到 widget/tray 主路径。

## Cleanup Checklist

- [ ] SSM-CLN-01 确认没有遗留未使用的 Win32 settings 绘制路径或实验性 Slint 接线。
- [ ] SSM-CLN-02 确认没有把可写设置遗漏在 `AppConfig` 之外。
- [ ] SSM-CLN-03 确认没有把英文硬编码继续留在用户可见区域。
- [ ] SSM-CLN-04 确认命名与现有 `taskbar-widget` 模块职责一致。
- [ ] SSM-CLN-05 确认没有提交临时日志、临时 i18n 资源或本地路径。
- [ ] SSM-CLN-06 确认没有把 Nothing 风格误实现成纯装饰性黑白皮肤，而忽略层级和信息密度规则。

## Completion Criteria

以下条件满足时，Slint settings migration 可判定完成：

- 默认 settings host 已切换为 Slint。
- widget / tray / taskbar attach / recovery 主路径保持 Win32 且无明显回归。
- 所有正式暴露的设置项都有明确本地持久化，并在软件重启后保持一致。
- 中英国际化已覆盖：
  - settings 全部文案
  - tray 菜单
  - tray tooltip
  - 状态标签
  - 来源方法标签
  - diagnostics / about 等关键运行时说明文本
- Settings 视觉实现符合约定的 Nothing-inspired 方向：
  - `Doto` / `Space Grotesk` / `Space Mono` 角色明确
  - 三层视觉层级清晰
  - 单色基底 + 状态色事件化
  - 无阴影、以排版和结构建立层次
- `cargo check`、`cargo fmt -- --check` 和人工联调通过。
- 文档、handoff 和 reflections 已回写。

可接受的已知限制：

- 初期可以保留旧 `settings_window.rs` 作为 fallback，但不能继续作为默认 settings host。
- `Monitoring` / `Appearance` 页面如果暂时未补齐全部真实交互，可以先保持结构化占位，但对应设置项不能伪装成已持久化。

## Reflection / Task Summary Generation

每完成一个 checklist item，自动生成：

```text
docs/reflections/task-<task-id>-<timestamp>.md
```

模板：

```markdown
- Task: <task name>
- Encountered Problem: <problem description>
- Thought Process: <how problem was analyzed>
- Options Considered: <list of solutions considered>
- Chosen Solution: <final decision>
- Rationale: <reason for choosing this solution>
```

规则：

- task id 必须对应本 checklist 条目，例如 `SSM-C-03`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及持久化的任务必须记录 schema 设计和重启后行为结论。
- 涉及国际化的任务必须记录覆盖范围、字符串来源和 fallback 规则。
- 涉及运行期桌面行为的任务必须记录 settings / tray / widget 观察结论。
- 涉及视觉实现的任务必须记录字体选择、层级规则和是否符合 Nothing-inspired 设计约束。
