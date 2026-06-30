# Win11 Taskbar Widget Checklist

## Checklist Objective

目标：

- 在当前主力机器的 Win11 环境上，完成一个独立 Rust PoC
- 证明 `Rust + Win32 + SetParent + GDI` 可以把一个固定文本模块稳定嵌入主任务栏
- 输出一套可重复执行、可停在阶段门上的实现清单

范围：

- 单一目标系统：当前主力 Win11
- 单一模块：固定文本，例如 `TASKBAR WIDGET`
- 单一路径：主任务栏，不做多显示器，不做 Win10 分支
- 单一绘制方案：GDI 黑底白字，可选细边框

非目标：

- 不做动态监控数据
- 不做透明背景、D2D、DirectComposition
- 不做插件系统、配置系统、主题系统
- 不做 Explorer 崩溃恢复、自愈、多版本兼容层

关联文档：

- [README.md](../plan/mvp-startup/README.md)
- [01-mvp-plan.md](../plan/mvp-startup/01-mvp-plan.md)
- [04-implementation-phases.md](../plan/mvp-startup/04-implementation-phases.md)
- [06-validation-and-debugging.md](../plan/mvp-startup/06-validation-and-debugging.md)
- [win11-taskbar-widget-preflight.md](./win11-taskbar-widget-preflight.md)
- [win11-taskbar-widget-loop-spec.md](./win11-taskbar-widget-loop-spec.md)
- [win11-taskbar-runtime-map.md](./win11-taskbar-runtime-map.md)

## Pre-Implementation Checks

- [x] `PRE-01` 固定目标环境，记录当前主力机器的 Win11 版本、任务栏对齐方式、缩放比例
- [x] `PRE-02` 确认将使用一个新的独立 Rust 二进制项目，而不是直接改造现有大工程
- [x] `PRE-03` 确认项目最小结构：`Cargo.toml`、`src/main.rs`、`src/taskbar.rs`、`src/win32.rs`
- [x] `PRE-04` 确认唯一核心依赖为 `windows` crate，并列出首批 Win32 API 清单
- [x] `PRE-05` 确认验证命令至少包含 `cargo run` 与 `cargo check`
- [x] `PRE-06` 阅读并摘录 Win11 单路径约束，不允许同时展开 Win10 或多显示器方案
- [x] `PRE-07` 约定反射文档目录为 `docs/reflections/`，任务完成后按 task id 生成记录

## Implementation Checklist

### Phase 1: 项目引导与普通窗口基线

- [x] `P1-01` 新建独立 Rust bin 项目，并补齐最小 `windows` crate feature 集
- [x] `P1-02` 在 `main.rs` 中完成窗口类注册、窗口创建、显示与消息循环
- [x] `P1-03` 在 `WM_PAINT` 中实现固定背景色与固定文本绘制
- [x] `P1-04` 将窗口尺寸固定为保守值，例如 `120 x 24`，确保肉眼容易识别
- [x] `P1-05` 用最小 `println!` 输出程序启动、窗口创建、进入消息循环等关键节点
- [x] `P1-06` 运行 `cargo run`，验证普通窗口稳定可见且可重复启动关闭
- [x] `P1-07` 普通窗口与 `WM_PAINT` 当前稳定，未触发回修停机

### Phase 2: Win11 任务栏窗口探测

- [x] `P2-01` 在 `taskbar.rs` 中实现 `FindWindowW("Shell_TrayWnd")` 并打印句柄
- [x] `P2-02` 在当前 Win11 机器上枚举并记录最小必要子窗口路径，优先围绕 `TrayNotifyWnd`、`Start` 等锚点验证结构
- [x] `P2-03` 明确当前机器的“候选父窗口”和“位置参考窗口”，不要同时保留多套策略
- [x] `P2-04` 为每次句柄探测输出错误码、窗口类名或矩形信息，确保失败可诊断
- [x] `P2-05` 运行探测版程序，确认能稳定打印非空任务栏根句柄与所选参考窗口信息
- [x] `P2-06` 当前机器结构已记录为实际路径，并已按真实结构更新实现

### Phase 3: 子窗口嵌入到任务栏层级

- [x] `P3-01` 在保留普通窗口基线的前提下，为目标窗口引入最小状态结构，至少保存自身句柄、任务栏句柄、父句柄和目标矩形
- [x] `P3-02` 调用 `SetParent` 将自定义窗口挂到选定的 Win11 父窗口下
- [x] `P3-03` 记录 `SetParent` 返回值、`GetLastError`、调用前后父子关系
- [x] `P3-04` 运行程序并确认窗口不再表现为普通独立浮窗
- [x] `P3-05` `SetParent` 当前行为稳定，未触发回修父窗口选择/调用时机

### Phase 4: Win11 最小可见定位

- [x] `P4-01` 读取任务栏根窗口与所选锚点窗口的矩形
- [x] `P4-02` 基于当前 Win11 路径选择一种保守定位规则，例如相对 `TrayNotifyWnd` 左侧留出固定模块区域
- [x] `P4-03` 使用固定宽高与固定偏移调用 `MoveWindow`，先追求“稳定可见”，不追求优雅布局
- [x] `P4-04` 为最终定位输出完整坐标、宽高和参考矩形
- [x] `P4-05` 已通过任务栏窗口捕获图确认任务栏内可见固定文本模块
- [x] `P4-06` 当前嵌入后已可见，未触发保守坐标回修

### Phase 5: 稳定性与重复验证

- [x] `P5-01` 连续执行至少两轮“启动 -> 观察模块 -> 关闭 -> 再启动”回归
- [x] `P5-02` 验证每轮都能重新找到句柄、完成嵌入、完成定位，没有偶发丢失
- [x] `P5-03` 检查退出时模块随进程消失，不残留悬挂窗口
- [x] `P5-04` 记录当前已接受限制：仅当前 Win11、仅主任务栏、仅固定文本、仅 GDI
- [x] `P5-05` 当前重复运行结果一致，未触发稳定性回修

### Phase 6: 文档化与收尾

- [x] `P6-01` 记录当前主力 Win11 机器的最终宿主路径、锚点选择和定位规则
- [x] `P6-02` 记录最小运行步骤：构建、运行、观察、退出、再运行
- [x] `P6-03` 记录已知限制、失败症状与最小排查顺序
- [x] `P6-04` 记录下一阶段候选扩展项，但明确全部标记为 Deferred
- [x] `P6-05` 审查代码与文档，确认没有提前引入插件系统、双策略框架或通用 Widget 抽象

## Validation Checklist

- [x] `VAL-01` `cargo check` 通过，且没有因为过度抽象引入不必要模块
- [x] `VAL-02` `cargo run` 后普通窗口基线可工作，作为 Phase 1 独立验收信号
- [x] `VAL-03` 探测日志中可见非空 `Shell_TrayWnd` 与当前选定 Win11 参考窗口信息
- [x] `VAL-04` `SetParent` 调用结果明确，错误时能看到 `GetLastError`
- [x] `VAL-05` 模块在任务栏内可见，显示固定文本，而非桌面浮窗
- [x] `VAL-06` 关闭程序后模块消失，再次启动能重复出现
- [x] `VAL-07` 至少两轮回归结果一致；若不一致，视为未通过 MVP 验证
- [x] `VAL-08` 验证时不引入透明、动态刷新、多显示器、Win10 分支等额外变量

## Documentation Checklist

- [x] `DOC-01` 在项目 README 或等效文档中写明“这是 Win11 单路径技术验证型 MVP”
- [x] `DOC-02` 记录依赖的 Win32 API 与 `windows` crate feature
- [x] `DOC-03` 记录当前机器的任务栏宿主结构假设与实际验证结果
- [x] `DOC-04` 记录最小调试输出字段：任务栏句柄、父句柄、错误码、矩形、最终坐标
- [x] `DOC-05` 记录下一步扩展前置条件：只有当前路径稳定后才允许扩范围

## Cleanup Checklist

- [x] `CLN-01` 删除临时探测代码中不再需要的噪声输出，但保留最小关键日志
- [x] `CLN-02` 删除实验性窗口分支、废弃定位算法和未使用 helper
- [x] `CLN-03` 确认文件布局仍保持最小，不新增无实际价值的架构目录
- [x] `CLN-04` 确认错误信息可读，失败时能定位到句柄探测、嵌入或定位哪一层
- [x] `CLN-05` 确认仓库中没有无意义的调试垃圾文件，并把新增文档链接收敛为仓库内相对路径

## Completion Criteria

以下条件全部满足，才算这个 MVP checklist 对应工作完成：

- 在当前主力 Win11 机器上，程序启动后任务栏中出现固定文本模块
- 模块属于任务栏层级，而不是独立桌面浮窗
- 程序关闭后模块消失，再次启动能稳定复现
- `cargo check` 与最小手动回归通过
- 已记录当前机器的宿主路径、锚点、定位规则和已知限制
- 代码结构仍然保持 PoC 级别最小复杂度

可接受的已知限制：

- 仅支持当前主力 Win11
- 模块位置可以不够优雅
- 背景可以不透明
- 不处理 Explorer 重启、多显示器、动态内容

最终仓库状态要求：

- checklist 可直接指导实际编码
- loop spec 可直接约束阶段门、验证顺序和停止条件
- 没有把验证型 MVP 膨胀为产品化架构

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

- `task-id` 必须对应本 checklist 中的编号，例如 `P3-02`
- 如果某任务执行顺利，也要简短记录为什么顺利，不只在失败时写
- 如果某任务卡在 Win11 窗口结构差异，必须写明观察到的真实窗口路径
- 如果某任务触发阶段停止规则，反射里必须记录停止原因与下一步入口
