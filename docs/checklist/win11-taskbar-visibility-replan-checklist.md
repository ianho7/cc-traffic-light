# Win11 Taskbar Visibility Replan Checklist

## Checklist Objective

目标：

- 按 [09-win11-diagnosis-replan.md](../plan/mvp-startup/09-win11-diagnosis-replan.md) 的约束，修正当前 `taskbar-widget`
- 让 `cargo run` 在当前主力 Win11 机器上稳定显示一个任务栏内嵌模块，例如 `TASKBAR WIDGET`
- 把后续执行收敛到一条 Win11 主路径，而不是继续扩大父窗口实验矩阵

范围：

- 仅修改当前仓库里的 `taskbar-widget`
- 仅支持当前主力 Win11、主显示器、主任务栏
- 仅支持 `Shell_TrayWnd` 父窗口、`Start` / `TrayNotifyWnd` 定位锚点、GDI 固定文本
- 仅保留最小诊断输出和人工肉眼验证

非目标：

- 不做经典任务栏支持
- 不做多显示器
- 不做透明背景、D2D、DirectComposition
- 不做插件系统、配置系统、通用宿主抽象
- 不把 `PrintWindow` 图像分析扩成完整自动化验证平台

关联文档：

- [09-win11-diagnosis-replan.md](../plan/mvp-startup/09-win11-diagnosis-replan.md)
- [06-validation-and-debugging.md](../plan/mvp-startup/06-validation-and-debugging.md)
- [07-risks-and-watchlist.md](../plan/mvp-startup/07-risks-and-watchlist.md)
- [08-trafficmonitor-reference-map.md](../plan/mvp-startup/08-trafficmonitor-reference-map.md)
- [taskbar-visibility-diagnosis-loop.md](./taskbar-visibility-diagnosis-loop.md)
- [win11-taskbar-visibility-replan-loop-spec.md](./win11-taskbar-visibility-replan-loop-spec.md)

## Pre-Implementation Checks

- [x] `PRE-01` 阅读并确认 [09-win11-diagnosis-replan.md](../plan/mvp-startup/09-win11-diagnosis-replan.md) 是当前执行基准，而不是旧的“从零启动”计划
- [x] `PRE-02` 确认本次只围绕现有 `taskbar-widget` 落地，不重新新建 PoC 项目
- [x] `PRE-03` 确认当前核心目标文件为 `taskbar-widget/src/main.rs`、`taskbar-widget/src/taskbar.rs`、`taskbar-widget/src/win32.rs`
- [x] `PRE-04` 确认当前诊断脚本入口为 `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
- [x] `PRE-05` 确认当前失败模式为：`RenderPass=True`、`AttachSuccess=True`、`LayoutMoved=True`，但人工上仍未得到稳定可见模块
- [x] `PRE-06` 确认 TrafficMonitor 的 Win11 参考路径是 `Shell_TrayWnd` 作为父窗口，`Start` / `TrayNotifyWnd` 只用于定位
- [x] `PRE-07` 确认最小验证命令至少包含 `cargo check`、`cargo run` 和聚焦后的诊断脚本命令
- [ ] `PRE-08` 记录当前主力机器的任务栏对齐方式、缩放比例和是否开启 Widgets，作为 Win11 定位输入
- [x] `PRE-09` 约定本轮任务反射仍写入 `docs/reflections/`，按 task id 自动生成记录

## Implementation Checklist

### Phase 1: 收敛 Win11 宿主路径

- [x] `P1-01` 在 `taskbar.rs` 中把正常运行路径的父窗口默认值固定为 `Shell_TrayWnd`
- [x] `P1-02` 保留 `Start`、`TrayNotifyWnd` 作为定位锚点输入，但停止把 `task_switch` / `rebar` 作为 Win11 主线父窗口候选
- [x] `P1-03` 将日志和诊断 JSON 字段明确拆分为“宿主父窗口”和“位置锚点”，避免继续混淆
- [x] `P1-04` 收紧运行时默认配置，避免普通 `cargo run` 仍进入矩阵式调试思路
- [x] `P1-05` 运行 `cargo check`，确认默认路径下 `candidate_parent` 指向 `Shell_TrayWnd`

### Phase 2: 修正嵌入窗口样式

- [x] `P2-01` 在 `main.rs` 中把当前顶层 `WS_OVERLAPPED + WS_CAPTION + WS_MINIMIZEBOX` 风格改为更适合嵌入的保守窗口样式
- [x] `P2-02` 增加最小必需的扩展样式，例如 `WS_EX_TOOLWINDOW`，但保持背景非透明
- [x] `P2-03` 在 `attach_to_taskbar()` 中校验样式切换前后是否仍残留不必要的顶层窗口语义
- [x] `P2-04` 确保 `style_before` / `style_after`、`previous_parent` / `current_parent` 继续写入诊断结果
- [ ] `P2-05` 运行 `cargo run`，确认窗口不再表现为普通桌面浮窗

### Phase 3: 收敛 Win11 可见定位

- [ ] `P3-01` 在 `position_in_taskbar()` 中改为基于 `Shell_TrayWnd` 父窗口 + `Start` / `TrayNotifyWnd` 锚点计算位置
- [ ] `P3-02` 为正常路径确定一个默认坐标模式，避免普通运行仍依赖多模式试错
- [ ] `P3-03` 保持模块尺寸、背景色和文本绘制简单醒目，先验证“看得见”
- [ ] `P3-04` 如果启动时单次 `MoveWindow` 不稳定，增加最小重定位循环，但不要引入完整控制器
- [ ] `P3-05` 记录最终 `module_rect`、`parent_rect`、`anchor_rect`，确认模块位于主任务栏可见区域

### Phase 4: 调整诊断与验证闭环

- [ ] `P4-01` 将 `diagnose-taskbar-loop.ps1` 的主验证路径收敛到 `shell` 父窗口优先
- [ ] `P4-02` 降低脚本 `Pass` 对 `PrintWindow` ROI delta 的唯一依赖，保留 `attach`、`layout`、`child render` 等硬证据
- [ ] `P4-03` 在诊断输出中明确区分“已验证事实”和“基于截图的推断”
- [ ] `P4-04` 把“人工可见性结论”写入文档或反射，而不是只看脚本表格
- [ ] `P4-05` 聚焦一条最小命令验证 Win11 主路径，不再默认跑完整父窗口矩阵

### Phase 5: 文档同步与执行入口收敛

- [ ] `P5-01` 更新 `taskbar-widget/README.md` 或等效说明，标明当前只支持 Win11 单路径技术验证
- [ ] `P5-02` 更新 [taskbar-visibility-diagnosis-loop.md](./taskbar-visibility-diagnosis-loop.md)，同步新的验证优先级
- [ ] `P5-03` 在相关文档里明确写出当前接受的限制：单 Win11、单主任务栏、GDI、非透明、人工主验证
- [ ] `P5-04` 将后续扩展项继续标记为 Deferred，不允许悄悄恢复双策略或矩阵式探索
- [ ] `P5-05` 审查文档和代码命名，确认“Win11 主线宿主 = shell”这一结论在仓库里表述一致

## Validation Checklist

- [ ] `VAL-01` 运行 `cargo check`，预期：编译通过，且没有新增与 MVP 无关的模块或依赖
- [ ] `VAL-02` 运行 `cargo run`，预期：日志中能看到非空 `Shell_TrayWnd`，并且默认路径不再使用 `rebar` / `task_switch` 作为主宿主
- [ ] `VAL-03` 人工观察任务栏，预期：能直接看到 `TASKBAR WIDGET` 模块
- [ ] `VAL-04` 人工观察桌面，预期：没有单独普通浮窗残留
- [ ] `VAL-05` 关闭程序后模块消失，再次启动仍能在任务栏内出现
- [ ] `VAL-06` 连续完成至少两轮“启动 -> 观察 -> 关闭 -> 再启动”，预期：可见性结果一致
- [ ] `VAL-07` 运行聚焦后的诊断脚本，预期：`AttachSuccess=True`、`LayoutMoved=True`、`ChildRender=True`；若视觉度量仍异常，必须记录为工具限制
- [ ] `VAL-08` 对失败情况进行归类，至少区分：宿主错误、样式错误、定位错误、观测工具错误

## Documentation Checklist

- [ ] `DOC-01` 在 `taskbar-widget/README.md` 中写明当前 Win11 主路径与运行命令
- [ ] `DOC-02` 记录当前默认父窗口、锚点窗口、坐标模式和模块尺寸
- [ ] `DOC-03` 记录最小人工验证步骤和成功信号
- [ ] `DOC-04` 记录当前诊断脚本哪些字段可信，哪些字段仅作辅助推断
- [ ] `DOC-05` 记录扩展前置条件：只有当前主路径稳定可见后，才允许继续做透明、多屏或经典任务栏

## Cleanup Checklist

- [ ] `CLN-01` 删除或降级不再服务当前 Win11 主路径的默认矩阵式配置
- [ ] `CLN-02` 删除多余噪声日志，但保留最小关键日志与诊断 JSON
- [ ] `CLN-03` 检查是否存在仍暗示 `task_switch` / `rebar` 是 Win11 主线宿主的文案或变量名
- [ ] `CLN-04` 确保没有把一次性实验输出、临时截图或本地绝对路径误写进长期文档
- [ ] `CLN-05` 确保本轮修改没有顺手引入透明、D2D、多屏或通用策略框架

## Completion Criteria

以下条件全部满足，才算本 checklist 对应工作完成：

- 默认 `cargo run` 走一条明确的 Win11 主路径：`Shell_TrayWnd` 作为父窗口，`Start` / `TrayNotifyWnd` 只用于定位
- 用户在当前主力 Win11 机器上肉眼可见任务栏中的固定文本模块
- 模块不是普通桌面浮窗，关闭程序后消失，再次启动稳定复现
- `cargo check` 通过，且最小手动回归至少两轮一致
- 诊断脚本已退回辅助证据角色，不再误充唯一成功标准
- 仓库文档已同步到当前 Win11 重计划，不再默认指向过时的矩阵探索路径

可接受的已知限制：

- 仅支持当前主力 Win11、主显示器、主任务栏
- 模块位置可以不够优雅
- 背景可以不透明
- 不处理 Explorer 重启、多显示器、经典任务栏、动态内容

最终仓库状态要求：

- 代码路径对当前 Win11 问题足够收敛
- checklist 可以直接指导逐项执行
- loop spec 可以直接约束重试、验证和停止条件
- 没有把这轮修复膨胀为产品化平台

## Reflection / Task Summary Generation

每完成一个 checklist item，自动生成一个反射文档：

```text
docs/reflections/task-<task-id>-<timestamp>.md
```

模板：

```md
- Task: <task name>
- Encountered Problem: <problem description>
- Thought Process: <how problem was analyzed>
- Options Considered: <list of solutions considered>
- Chosen Solution: <final decision>
- Rationale: <reason for choosing this solution>
```

生成规则：

- `task-id` 必须对应本 checklist 中的编号，例如 `P2-01`
- 即使任务顺利完成，也要记录为什么当前路径是可接受的 MVP 选择
- 如果某任务涉及放弃 `rebar` / `task_switch` 作为默认宿主，反射里必须说明证据来源
- 如果某任务出现“人工可见但脚本失败”的情况，反射里必须明确标记为“工具限制”还是“实现缺陷”
- 如果某任务触发停止规则，反射里必须写明当前 blocker、已尝试路径和下一轮入口
