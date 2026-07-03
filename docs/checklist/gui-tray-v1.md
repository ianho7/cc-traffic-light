# GUI / Tray V1 Checklist

日期：2026-07-02

## Checklist Objective

把 [gui-tray-v1-requirements.md](/D:/project/cc-traffic-light/docs/plan/gui-tray-v1-requirements.md) 转成可执行 checklist，在不破坏当前稳定 Win11 taskbar attach 路径的前提下，把现有 PoC 扩展成单进程常驻桌面应用。

目标结果：

- 新增 tray、settings、autostart、detector runtime、diagnostics。
- 保持当前 widget 稳定可见，并扩展为双来源三灯面板。
- 仅通过本地被动观测感知 `Codex` / `Claude Code`。
- 保持纯 Win32 技术栈，不引入第二套 GUI 运行时。

范围：

- 仅覆盖 `taskbar-widget/` 内的 Win32 / Rust 实现和相关文档。
- 覆盖 runtime、UI、配置、状态聚合、诊断、验证与文档收口。

非目标：

- 不引入 WebView、egui、tauri、D2D、DirectComposition。
- 不做系统级服务、多用户安装、多显示器支持。
- 不自动修改用户外部 Codex / Claude 配置。
- 不在 V1 追求 full Nothing implementation。

## Loop Engineering Spec

### Goal

- 交付一个可常驻运行、带 tray 和 settings 的 Win11 本地状态观察器。
- 进展证据来自：runtime 契约文档、配置 schema、关键代码 diff、`cargo check`、桌面人工验证、detector 矩阵验证、降级恢复验证。
- 完成证据不是“窗口能打开”，而是“widget / tray / settings / detector / diagnostics 构成一个闭环，且失败有明确降级与可解释性”。

### State

Source of truth:

- 本 checklist
- [gui-tray-v1-requirements.md](/D:/project/cc-traffic-light/docs/plan/gui-tray-v1-requirements.md)
- `taskbar-widget/src/` 现有实现
- 后续新增的 config / detector / ui 模块

Persistent loop state:

- 当前 phase / task id
- 已确认的配置 schema
- 已确认的 detector 来源优先级与冲突规则
- 最近一次 widget / tray / settings 联调结果
- 最近一次失败分类
- reflection 文档路径

Raw evidence:

- `cargo check` / 运行日志
- 任务栏与托盘人工观察结论
- detector 命中结果
- autostart 和降级恢复验证记录

Discardable state:

- 被后续证据推翻的 UI 布局尝试
- 无法形成结论的临时控件布局
- 一次性调色试验

### Planner

- 默认选择“当前 phase 中依赖已满足且最能改变验证状态的最小任务”。
- 固定顺序为：runtime 契约 -> tray/settings shell -> detector -> widget 扩展 -> diagnostics -> 验证与文档。
- 若桌面行为异常，优先判断是 widget attach、tray 消息循环、detector 输出还是配置加载问题，不混成一个“大 GUI 问题”。
- 若某步需要引入第二套 GUI 栈或重构 taskbar attach 路径，先判定为 scope failure。

### Actor

允许动作：

- 读取和编辑 `taskbar-widget/src/`、`docs/plan/`、`docs/checklist/`
- 运行 `cargo check`
- 在桌面会话运行程序做人工验证
- 新增最小配置、状态与 Win32 UI 代码

中风险动作：

- 调整消息循环或窗口生命周期
- 修改 widget 绘制逻辑
- 新增 detector 读文件 / 读进程的本地观测实现

非默认动作：

- 修改 `taskbar.rs` 中稳定 attach 主路径
- 引入新的 GUI 框架
- 自动写用户外部工具配置

### Observer

- 每次动作后先记录原始观察，再写解释。
- 对桌面验证至少记录：
  - widget 是否可见
  - tray 是否存在
  - settings 是否可打开
  - 当前来源状态是否合理
  - 挂载失败时是否正确降级
- 对 detector 验证至少记录：
  - 命中来源类型
  - 更新时间
  - 可信度
  - 是否触发冲突降级

### Verifier

Verifier order:

1. code review：确认改动仍在纯 Win32 + Rust 范围内
2. `cargo check`
3. tray / settings shell 人工 smoke test
4. detector 来源矩阵验证
5. widget 双来源三灯可见性验证
6. 挂载失败降级与自动恢复验证
7. autostart 与设置持久化验证

Actor 不能自证完成；必须至少有命令结果和人工桌面观察作为独立证据。

### Failure Semantics

- Transient failure: 一次性构建失败、桌面刷新偶发异常，可重试 1 次。
- Runtime failure: tray、settings、widget 生命周期互相打架，必须先减小耦合再继续。
- Detector failure: 本地数据源不稳定或命中错误，必须先补来源证据与可信度规则。
- Scope failure: 为解决样式或布局问题开始引入新 GUI 栈或重写 attach 路径，立即停止。
- Evidence failure: 没有形成桌面观察或来源矩阵结论，不得判定完成。

### Exit Conditions

- Success exit: Completion Criteria 满足。
- Blocked exit: 当前机器或工具环境无法提供稳定本地来源，且继续编码无法新增证据。
- Budget exit: 连续多轮只在 UI 细节上低收益试错，没有新增验证信号。
- Risk exit: 下一步必须越过纯 Win32 或当前 taskbar 路径边界。
- Human takeover exit: 需要用户在产品方向上放弃现有边界，例如接受 WebView 或关闭 widget 强制显示。

### Policy

- V1 统一保持纯 Win32。
- 默认不改 `taskbar.rs` 稳定 attach 路径，除非有阻塞证据。
- 默认不改外部工具配置。
- 先保证行为闭环，再考虑更高保真视觉语言。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、需求文档、相关代码和最近验证记录。
2. Choose：选择一个能直接改变验证状态的最小 task。
3. Act：做最小实现或文档更新。
4. Observe：记录命令结果与桌面观察。
5. Verify：运行该 task 对应的最小 verifier。
6. Reflect：为完成、阻塞或跳过任务生成 reflection。
7. Decide：继续下一 task、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 phase 还有未完成 task，且上一轮拿到了新的结构化证据。

停止条件：

- Completion Criteria 满足。
- 下一步需要引入范围外技术或环境假设。
- 连续多轮只剩低收益样式试错。

## Pre-Implementation Checks

- [x] GTV1-PRE-01 阅读 [gui-tray-v1-requirements.md](/D:/project/cc-traffic-light/docs/plan/gui-tray-v1-requirements.md)，确认产品边界、来源优先级和 GUI 技术决策。
- [x] GTV1-PRE-02 阅读 `taskbar-widget/src/main.rs`、`taskbar-widget/src/taskbar.rs`、`taskbar-widget/src/agent_state.rs`，标出可复用的窗口生命周期、绘制和状态读取路径。
- [x] GTV1-PRE-03 确认现有 P3 / P4 文档中哪些结论必须保留，例如稳定 attach 路径、global summary 消费方式和 runtime 风险。
- [x] GTV1-PRE-04 确认计划中的新增模块命名和目标文件位置，避免边做边漂。
- [x] GTV1-PRE-05 确认最小验证命令与人工验证清单。

## Implementation Checklist

### Phase 1: Runtime Contract 与模块切分

- [x] GTV1-A-01 定义 V1 的运行时模块边界：widget host、tray host、settings host、detector runtime、settings store、status aggregator。
- [x] GTV1-A-02 定义用户级配置 schema，覆盖 autostart、tray 行为、detector enable flags、widget style、diagnostics metadata。
- [x] GTV1-A-03 定义统一聚合状态结构，明确来源状态、总状态、可信度、最近更新时间和错误摘要字段。
- [x] GTV1-A-04 明确 settings、detector 和 widget 之间的消息 / 刷新模型，避免 UI 直接读取底层来源文件。
- [x] GTV1-A-05 记录哪些现有代码保留在 `main.rs`，哪些需要拆出新模块。

当前实现（2026-07-02）：

- 新增 `taskbar-widget/src/app_config.rs`，定义 V1 用户级配置 schema、默认值、配置路径和加载 / 保存契约。
- 新增 `taskbar-widget/src/runtime_contract.rs`，把 `widget_host`、`tray_host`、`settings_host`、`detector_runtime`、`settings_store`、`status_aggregator` 固化为运行时模块边界，并定义配置变更、状态更新、手动刷新和 shutdown 等信号。
- 新增 `taskbar-widget/src/ui_state.rs`，定义来源级状态、可信度、检测方法、widget 挂载状态和统一快照结构，为后续 settings / tray / widget 共享同一状态契约做准备。
- `main.rs` 仍保留现有 Win32 启动、taskbar attach、GDI paint 和 timer 轮询主路径；新增启动时配置加载与 runtime contract 日志，并预留 `APP_STATUS_SNAPSHOT` 作为后续统一状态入口。
- `lib.rs` 已导出 `app_config`、`runtime_contract`、`ui_state`，当前 Phase 1 仅做编译通过的骨架接线，不提前实现 tray/settings/detector UI。

### Phase 2: Tray 与 Settings Shell

- [x] GTV1-B-01 实现托盘图标与基础菜单：打开设置、立即重新探测、退出。
- [x] GTV1-B-02 实现设置窗口框架与页面导航，保持纯 Win32 控件方案。
- [x] GTV1-B-03 实现首页总览区域：总状态、widget 挂载状态、最近成功观测时间。
- [x] GTV1-B-04 实现来源卡片骨架：状态、识别方式、最近更新时间、可信度。
- [x] GTV1-B-05 实现通用设置页：开机启动、启动后最小化到托盘、关闭窗口时隐藏到托盘。

当前实现（2026-07-02）：

- 新增 `taskbar-widget/src/tray_icon.rs`，实现基于 `Shell_NotifyIconW` 的最小 tray host。
- tray 当前支持：
  - 左键打开 settings
  - 右键菜单
  - 菜单项：`Open Settings`、`Refresh Detection`、`Exit`
- 新增 `taskbar-widget/src/settings_window.rs`，实现独立 Win32 settings 窗口类与自绘 overview shell。
- settings 当前采用“左侧导航占位 + 右侧 overview 面板”的结构，导航项已固定为：
  - `OVERVIEW`
  - `GENERAL`
  - `MONITORING`
  - `APPEARANCE`
  - `DIAGNOSTICS`
  - `ABOUT`
- overview 当前已显示：
  - 总状态
  - widget 挂载状态
  - `Last Refresh`
  - `Codex` 卡片
  - `Claude` 卡片
- 当前来源卡片骨架已包含：
  - `State`
  - `Method`
  - `Confidence`
- settings 当前支持点击左侧导航切换到占位页。
- `GENERAL` 页当前支持三个最小交互开关：
  - `Enable autostart`
  - `Start minimized to tray`
  - `Close window to tray`
- 上述三个开关会直接写回 `config.json`，当前只完成本地配置持久化，尚未接上真实 autostart 安装逻辑。

### Phase 3: Detector 与状态聚合

- [x] GTV1-C-01 为 `Codex` 实现固定来源优先级 detector 骨架。
- [x] GTV1-C-02 为 `Claude Code` 实现固定来源优先级 detector 骨架。
- [x] GTV1-C-03 实现来源优先级、同级更新时间比较和冲突降级为 `状态不可信` 的规则。
- [x] GTV1-C-04 明确进程检测只作为兜底来源，不单独判定高置信度 `工作中`。
- [x] GTV1-C-05 将 detector 输出接入统一状态聚合层，并同步到 settings 首页与 tray tooltip。

当前实现（2026-07-02）：

- 新增 `taskbar-widget/src/detector.rs`，把来源探测与聚合从 `ui_state` 中独立出来。
- 当前 detector 骨架已固定 `Codex` / `Claude` 的来源优先级框架：
  - `log_file`
  - `state_file`
  - `session_file`
  - `process`
  - `hook_state`
- 当前实际已接线的来源只有 `state_file`，来源于现有 `state.json` 的 agent summary；其余来源目前仍是骨架优先级位点。
- 当前已新增 `process` 兜底来源：
  - `Codex` 匹配 `codex(.exe)`
  - `Claude` 匹配 `claude(.exe)` / `claude-code(.exe)` / `claudecode(.exe)`
  - 仅在检测到进程存在时输出低置信度 `Idle`
  - 不会单独把状态抬成高置信度 `Working`
- 已实现同级来源冲突规则：
  - 先比较来源优先级
  - 同级比较 `updated_at`
  - 同级且同样新鲜但状态矛盾时，降级为 `Untrusted`
- 已实现总聚合状态规则，并改由 detector 统一生成 `AppStatusSnapshot`。
- `main.rs` 当前启动初始化和轮询刷新都改为通过 detector 生成 snapshot，再同步给 settings。
- tray 的 `Refresh Detection` 当前已走统一 snapshot 刷新路径，tooltip 也会随统一 snapshot 输出 `overall/codex/claude` 摘要。

### Phase 4: Widget 扩展

- [x] GTV1-D-01 把当前 widget 扩展为双来源并列布局：`Codex`、`Claude`。
- [x] GTV1-D-02 为每个来源实现固定三灯映射：绿 / 黄 / 红。
- [x] GTV1-D-03 落实 `未发现` 与 `状态不可信` 的视觉区分。
- [x] GTV1-D-04 保持当前 taskbar attach / layered / positioning 主路径不变。
- [x] GTV1-D-05 把总聚合状态同步到极简 tray 图标。

当前实现（2026-07-02）：

- `main.rs` 的 widget paint 输入已从单个 `HookSummary` 改为统一 `AppStatusSnapshot`。
- 当前 widget 已切成双来源并列布局：
  - 左半区 `Codex`
  - 右半区 `Claude`
- 每个来源当前固定三灯：
  - 绿灯
  - 黄灯
  - 红灯
- 当前灯语实现为：
  - `Idle` / `Working` -> 绿灯
  - `Attention` -> 黄灯
  - `Blocking` -> 红灯
  - `Undiscovered` -> 三灯暗置
  - `Untrusted` -> 三灯统一灰亮
- 本轮只改了 paint path 和 snapshot 比较逻辑，没有修改 `taskbar.rs` 的 attach / layered / positioning 主路径。
- tray 当前已改为“总聚合状态 -> 极简单灯图标”：
  - `Idle` / `Working` -> 绿灯
  - `Attention` -> 黄灯
  - `Blocking` -> 红灯
  - `Undiscovered` -> 深灰灯
  - `Untrusted` -> 浅灰灯
- 当前 tray 仍保持极简聚合语义，不展开到双来源六灯；来源细节继续放在 widget 与 settings 中。

### Phase 5: 诊断、持久化与降级恢复

- [x] GTV1-E-01 实现用户级配置持久化与加载。
- [x] GTV1-E-02 实现当前用户级 autostart。
- [x] GTV1-E-03 实现只读诊断页：来源类型、更新时间、可信度、错误摘要、手动刷新。
- [x] GTV1-E-04 实现 widget 挂载失败后的托盘降级逻辑。
- [x] GTV1-E-05 实现后台周期性重试 widget 恢复逻辑。

当前实现（2026-07-02）：

- `taskbar-widget/src/app_config.rs` 已提供用户级配置文件路径、默认值、加载诊断和保存逻辑，配置当前保存在 `%APPDATA%\\CcTrafficLight\\config.json`。
- 当前已持久化的设置包括：
  - `autostart_enabled`
  - `start_minimized_to_tray`
  - `close_to_tray`
  - 监控启用位
  - 外观占位设置
  - settings 最近打开页
- 新增 `taskbar-widget/src/autostart.rs`，通过 `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run` 实现当前用户级自启动。
- settings 的 `Enable autostart` 开关现在会同时：
  - 更新注册表自启动项
  - 更新内存配置
  - 保存回 `config.json`
- 启动时会先读取配置文件，再用注册表实际状态回填 `autostart_enabled`，减少 UI 展示和系统真实状态不一致的情况。
- `DIAGNOSTICS` 页当前已接上真实只读数据：
  - `Widget Mount`
  - `Last Detection Refresh`
  - `Last Error`
  - `Codex` / `Claude` 的状态、来源方法、可信度、更新时间和消息摘要
- `DIAGNOSTICS` 页的 `REFRESH NOW` 当前会：
  - 记录 `last_manual_refresh_at`
  - 向主窗口发送刷新命令
  - 立即走一次统一 detector 刷新路径
- tray 菜单里的 `Refresh Detection` 也已从“只刷新显示”改为“真正重新探测并同步 widget / tray / settings”。 
- 主循环当前新增 widget 运行时状态层，按以下语义工作：
  - 挂载成功且定位成功 -> `Attached`
  - 初次挂载失败 -> `TrayOnly`
  - 进入后台恢复期 -> `Retrying`
- 当初始 attach / layout 不成立时，主 widget 窗口会隐藏，只保留 tray 与 settings 能力继续运行。
- 后台每 `5s` 会发起一次最小恢复尝试：
  - 重新 probe taskbar
  - 重新 attach
  - 重新 layout
  - 成功则恢复 widget 显示并刷新可见性
  - 失败则继续保持隐藏并保留 `Retrying`
- detector 聚合快照现在会消费真实 `widget_mount_state` 和 `last_widget_attach_at`，因此 overview / diagnostics 能看到降级与恢复中的状态。 

### Phase 6: 收口与文档

- [ ] GTV1-F-01 回写需求与 checklist 中已确认的实现结论，避免后续重复决策。
- [x] GTV1-F-02 更新相关 README / handoff，说明 V1 现状、已知限制和下一步建议。
- [x] GTV1-F-03 为完成、阻塞或跳过的任务生成 reflection。
- [ ] GTV1-F-04 确认没有把 V1 漂移成多 GUI 栈、多服务或自动配置修改方案。

## Validation Checklist

- [x] GTV1-VAL-01 `cargo check` 通过。
- [x] GTV1-VAL-02 tray 图标存在，菜单可正常打开设置、手动刷新和退出。
- [x] GTV1-VAL-03 settings 首页能显示总状态、widget 挂载状态和两张来源卡片。
- [x] GTV1-VAL-04 通用设置项可保存并重新加载。
- [x] GTV1-VAL-05 当前用户级 autostart 可启用 / 禁用。
- [x] GTV1-VAL-06 `Codex` detector 至少命中一条有效来源，并给出合理可信度。
- [x] GTV1-VAL-07 `Claude Code` detector 至少命中一条有效来源，并给出合理可信度。
- [ ] GTV1-VAL-08 同级来源冲突时正确降级到 `状态不可信`。
- [x] GTV1-VAL-09 widget 双来源三灯在任务栏中可辨认。
- [x] GTV1-VAL-10 `未发现` 与 `状态不可信` 在 widget 上能视觉区分。
- [x] GTV1-VAL-11 widget 挂载失败时软件保留 tray 并后台重试恢复。
- [x] GTV1-VAL-12 tray 聚合状态、settings 总状态和 widget 来源状态之间无明显矛盾。

当前验证（2026-07-02）：

- `cargo check` 通过。
- `cargo fmt -- --check` 通过。
- 用户已在真实 Win11 桌面会话中按人工验证清单完成一次完整联调，并反馈“都是正常的”。
- 本次已确认通过的人工验证范围包括：
  - tray 图标存在，左键打开 settings、右键菜单、手动刷新和退出正常
  - settings 首页、General、Diagnostics 页面显示正常
  - 通用设置项可保存并重新加载
  - 当前用户级 autostart 可启用 / 禁用
  - `Codex` / `Claude Code` 均能命中至少一条有效来源，并给出合理可信度
  - widget 双来源三灯可辨认，`未发现` 与 `状态不可信` 可区分
  - widget 挂载失败时可退化为 tray-only，并在后台重试恢复
  - tray / settings / widget 三处状态无明显矛盾
- 当前仍缺一条单独造出的来源冲突场景证据，因此 `GTV1-VAL-08` 继续保留未完成。

## Documentation Checklist

- [ ] GTV1-DOC-01 保持 [gui-tray-v1-requirements.md](/D:/project/cc-traffic-light/docs/plan/gui-tray-v1-requirements.md) 与实现结论一致。
- [x] GTV1-DOC-02 在 checklist 中持续记录默认决策和验证结果。
- [x] GTV1-DOC-03 如运行期诊断或下一步建议发生变化，更新 `docs/handoff/`。
- [x] GTV1-DOC-04 每个完成、跳过或阻塞任务都生成 reflection。
- [ ] GTV1-DOC-05 明确记录 V1 继续维持纯 Win32，而不是 full Nothing implementation。

## Cleanup Checklist

- [ ] GTV1-CLN-01 确认没有无意引入第二套 GUI 运行时或额外大依赖。
- [ ] GTV1-CLN-02 确认没有无意扩大到系统级服务、管理员提权或自动改外部配置。
- [ ] GTV1-CLN-03 确认没有留下临时日志、硬编码本地路径或实验性探测代码。
- [ ] GTV1-CLN-04 确认命名与现有 `taskbar-widget` 模块职责一致。
- [ ] GTV1-CLN-05 确认注释和文档都保持 MVP 边界。

## Completion Criteria

以下条件满足时，GUI / Tray V1 可判定完成：

- 单进程常驻后台应用可运行，统一管理 widget、tray、settings。
- settings 能展示总状态、来源卡片、诊断和基本设置。
- 当前用户级 autostart 可用。
- `Codex` 与 `Claude Code` 至少各有一条零配置本地来源被稳定识别。
- 来源优先级、冲突降级和可信度规则已实现。
- widget 已扩展为双来源并列三灯面板。
- widget 挂载失败时能自动降级到仅托盘并后台重试恢复。
- `cargo check` 和桌面人工验证通过。
- 已知限制、下一步建议和 reflections 已写回文档。

可接受的已知限制：

- V1 仍保持纯 Win32，视觉只做到 Nothing-inspired，而非高保真复刻。
- detector 可以先覆盖有限来源集合，但必须对来源类型和可信度保持可解释。
- 若某些来源只做到 `未发现`，可接受，但不能伪造高置信度状态。

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

- task id 必须对应本 checklist 条目，例如 `GTV1-C-03`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及 detector 的任务必须记录来源类型、可信度和冲突处理结论。
- 涉及桌面行为的任务必须记录 widget / tray / settings 观察结论。

## Goal Usage Recommendation

GUI / Tray V1 适合在真正开始编码时启用 `/goal`，因为这是一个明显的长链路运行时闭环任务：runtime 契约、tray / settings shell、detector、多来源聚合、widget 扩展、降级恢复和人工验证都需要跨多轮推进。

建议 objective：

```text
Implement a pure Win32 GUI and tray V1 for the taskbar widget app, including zero-config local detectors for Codex and Claude Code, dual-source widget rendering, settings persistence, diagnostics, and widget-to-tray degradation recovery.
```

Continue condition：

- 当前 phase 还有未完成任务，且上一轮拿到了新的代码或验证证据。

Completion condition：

- Completion Criteria 全部满足，且最新验证结果已回写 checklist / handoff / reflection。

Blocked condition：

- 当前机器或工具环境无法提供稳定本地来源，且进一步编码无法形成新证据。

Budget boundary：

- 同一 phase 连续 3 次只有低收益 UI / detector 试错而没有新增验证证据时，停止并转入 handoff。
