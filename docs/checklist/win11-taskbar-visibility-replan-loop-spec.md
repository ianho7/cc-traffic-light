# Win11 Taskbar Visibility Replan Loop Spec

## Loop Level

本轮使用运行时闭环，不引入工作流级多代理编排。

原因：

- 当前只有一个明确目标：让 `taskbar-widget` 在当前 Win11 主任务栏中真实可见
- 主要不确定性集中在宿主选择、窗口样式、定位和观测工具
- 这些问题更适合“小步编辑 -> 本机验证 -> 基于证据重规划”，不需要额外角色拆分

## Goal

- 交付一个可运行的 Win11 单路径 PoC
- 外部可观察结果是：`cargo run` 后任务栏中出现 `TASKBAR WIDGET` 模块，关闭后消失，再次启动稳定复现
- 进展证据不是“又试了几个父窗口”，而是默认主路径是否收敛到 `Shell_TrayWnd + Start/TrayNotify`

## State

源事实：

- 当前实现文件：
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/taskbar.rs`
  - `taskbar-widget/src/win32.rs`
  - `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
- 当前计划基准：
  - [09-win11-diagnosis-replan.md](../plan/mvp-startup/09-win11-diagnosis-replan.md)
- 当前执行清单：
  - [win11-taskbar-visibility-replan-checklist.md](./win11-taskbar-visibility-replan-checklist.md)

必须持久化的信息：

- 当前完成到哪个 checklist task id
- 当前默认父窗口、锚点、坐标模式
- 最近一次 `cargo check` / `cargo run` / 诊断脚本结果
- 最近一次人工观察结论
- 当前失败分类和下一步假设

可总结或丢弃的信息：

- 已被证据推翻的矩阵式宿主猜测
- 与当前 Win11 主路径无关的临时截图比较细节

## Planner

下一步选择规则：

- 永远选“最小且最可能改变验证状态”的未完成任务
- 如果默认主路径还没收敛，不允许先优化诊断脚本细节
- 如果人工上仍不可见，不允许提前扩展到经典任务栏或多显示器
- 如果出现“人工可见但脚本失败”，优先修正文档和工具判读，而不是盲目改产品代码

触发重规划的条件：

- 默认路径仍依赖 `rebar` / `task_switch` 才能运行
- `SetParent` 成功但窗口仍像普通浮窗
- 人工结果与诊断结果长期矛盾
- 为了继续推进，不得不引入透明、多屏、双策略或通用宿主抽象

## Actor

允许动作：

- 编辑当前 `taskbar-widget` 的最小源文件
- 编辑诊断脚本和对应文档
- 运行 `cargo check`、`cargo run`
- 运行聚焦后的诊断脚本命令
- 记录人工观察与反射文档

受限动作：

- 不引入新 GUI 框架
- 不引入 D2D、DirectComposition、透明背景
- 不做经典任务栏或多显示器实现
- 不建立产品级控制器或通用策略层

## Observer

每轮必须记录的原始输出：

- `cargo check` 结果
- `cargo run` 日志中的：
  - `candidate_parent`
  - `position_anchor`
  - `style_before` / `style_after`
  - `module_rect`
- 诊断 JSON 和脚本输出表格
- 人工观察结论：
  - 任务栏可见
  - 普通浮窗
  - 完全不可见
  - 可见但不稳定

证据存放：

- 结构化诊断：`taskbar-widget/target/diagnose-taskbar-loop/`
- 任务反射：`docs/reflections/task-<task-id>-<timestamp>.md`
- 执行清单与 loop spec：`docs/checklist/`

## Verifier

验证顺序：

1. `cargo check`
2. 默认 `cargo run`
3. 人工任务栏可见性检查
4. 人工桌面浮窗排查
5. 重复启动一致性检查
6. 聚焦后的诊断脚本检查

进展判定规则：

- 只有当验证状态发生变化时，才算当前轮真正前进
- “多了一堆日志”不算前进
- “又试了一个候选父窗口”不算前进
- “默认路径更接近最终单宿主实现”才算前进

完成判定：

- 日志、人工观察和重复启动结果同时满足 checklist 的 Completion Criteria

## Failure Semantics

失败分类：

- 宿主失败：
  - 默认父窗口仍不对
  - `current_parent` 不稳定
- 样式失败：
  - `SetParent` 成功但仍像普通浮窗
- 定位失败：
  - 已嵌入但任务栏里看不到
  - `module_rect` 明显跑到错误区域
- 观测失败：
  - 人工可见，但脚本视觉指标失败
- 范围失败：
  - 为通过当前阶段开始引入透明、多屏、双策略

重试策略：

- 同一原因最多直接重试 1 次
- 第二次失败后必须换假设、换观测点或缩范围
- 观测失败优先修工具和文档，不优先改产品实现

## Exit Conditions

成功退出：

- [win11-taskbar-visibility-replan-checklist.md](./win11-taskbar-visibility-replan-checklist.md) 的 Completion Criteria 全部满足

阻塞退出：

- 当前 Win11 真实任务栏结构无法靠现有观察确定
- 下一步只能依赖用户提供新的环境信息或人工截图说明

预算退出：

- 连续多轮都只是在微调坐标，但没有新增证据表明更接近可见性
- 继续执行的收益低于同步人工观察结果的收益

风险退出：

- 下一步必须跨出 MVP 边界，例如上透明、上多屏、上双策略

人工接管退出：

- 需要用户明确当前桌面上实际看到了什么
- 需要用户确认任务栏对齐方式、Widgets 状态或其他本地设置

## Policy

- 只允许 Win11 主力机器单路径
- 只允许 `Shell_TrayWnd` 作为默认 Win11 主宿主
- 只允许 `Start` / `TrayNotifyWnd` 作为定位锚点
- 只允许 GDI、非透明、固定文本模块
- 只允许最小代码改动去改变当前验证状态

## Current State

- Relevant files:
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/taskbar.rs`
  - `taskbar-widget/src/win32.rs`
  - `taskbar-widget/scripts/diagnose-taskbar-loop.ps1`
- Current phase/task:
  - 生成执行清单后，下一步从 `P1-01` 开始
- Last verification result:
  - 现有诊断说明子窗口会画、会挂、会移动，但默认 Win11 路径还没证明真实可见
- Current hypothesis:
  - 当前主要问题不是 Rust / Win32 基础能力，而是 Win11 默认宿主路径与样式/定位选择不够收敛
- Remaining scope:
  - 宿主收敛
  - 样式收敛
  - 定位收敛
  - 验证闭环收敛

## Next Action Rule

- 永远从最小未完成 checklist 任务里选择一个能改变验证状态的动作
- 当前优先级：
  1. `P1-*` 收敛默认宿主路径
  2. `P2-*` 修正窗口样式
  3. `P3-*` 修正可见定位
  4. `P4-*` 收敛诊断工具
  5. `P5-*` 同步文档

## Verifier Order

1. `cargo check`
2. `cargo run`
3. 肉眼看任务栏
4. 肉眼看桌面是否残留浮窗
5. 连续两轮重启验证
6. 聚焦版诊断脚本

## Failure Policy

- 瞬时编译或运行异常：直接重试一次
- 同一逻辑失败第二次出现：必须改假设，不允许继续同一招空转
- 缺少人工可见性信息时：优先请求/记录人工观察，而不是继续扩大自动截图逻辑
- 一旦出现范围失控迹象：立即回到 checklist 和 replan 文档，删减动作

## Stop Rule

- Stop complete when:
  - 任务栏模块可见、非浮窗、可重复复现，且文档同步完成
- Stop blocked when:
  - 没有新的本机观察证据就无法继续判断
- Stop budgeted when:
  - 连续多轮微调没有新增验证信号
- Stop risky when:
  - 下一步需要引入超出 MVP 边界的能力

## Risk Register

- 风险 1：Win11 主路径仍然不能只靠 `Shell_TrayWnd` 实现稳定可见
- 风险 2：当前顶层窗口样式残留导致 `SetParent` 后行为异常
- 风险 3：定位需要最小刷新，而不是一次性 `MoveWindow`
- 风险 4：诊断脚本继续把工具观测偏差误判为产品失败

对应缓解：

- 先对齐 TrafficMonitor 的 Win11 路线
- 先收紧窗口样式
- 必要时增加最小重定位循环
- 让人工观察成为主验证信号

## Mapping

- MVP 范围和非目标来自 [09-win11-diagnosis-replan.md](../plan/mvp-startup/09-win11-diagnosis-replan.md)
- 原子任务拆分落在 [win11-taskbar-visibility-replan-checklist.md](./win11-taskbar-visibility-replan-checklist.md)
- 当前诊断背景来自 [taskbar-visibility-diagnosis-loop.md](./taskbar-visibility-diagnosis-loop.md)

## Goal Recommendation

建议在真正开始编码时启用 `/goal`。

原因：

- 这是一轮典型的多轮本机调试闭环任务
- 需要显式保存“当前假设 -> 编辑 -> 验证 -> 重规划”的状态
- 容易因为可见性问题反复空转，没有 loop spec 很容易回到无边界试错
