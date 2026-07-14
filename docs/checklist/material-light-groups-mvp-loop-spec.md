# 素材灯组 MVP 闭环执行规范

## Loop Level

使用运行时编码闭环，不建立工作流级多 Agent 编排。

原因：任务有明确依赖顺序和单一代码库；每个阶段都可由一个 Agent 独立编辑、验证和交付。真正需要人类输入的仅是 Windows 桌面可见性与产品体验观察，不应以多 Agent 并行替代。

## Goal

- 交付 `docs/checklist/material-light-groups-mvp.md` 的全部 MVP 行为。
- 完成证据为：自动检查结果、针对 host 的渲染/回退测试、Windows 手工验证和每任务 reflection。

## State

持久来源：

- MVP 约束：[素材灯组 MVP 实施计划](../plan/material-light-groups-mvp.md)
- 原子任务：[素材灯组 MVP 执行 Checklist](material-light-groups-mvp.md)
- 当前阶段、已完成任务、失败分类、原始命令输出、验证的 host exe 路径：任务 reflection 与 handoff 文档。

每轮保留：当前 task ID、改动文件、最近验证结果、当前假设、失败原因、下一动作。

每轮可丢弃：已被证据推翻的实现猜测和重复命令输出的摘要；原始输出路径仍须保留在 reflection。

## Planner

下一动作规则：

1. 选择当前阶段中最小的未完成 task。
2. 若 task 依赖验证，先运行成本最低的聚焦验证。
3. 只有当前 task 的完成证据成立，才勾选并进入下一 task。
4. 上游阶段失败时，不开始下游 UI 或集成工作。

触发重规划：

- P0 数据模型无法在既有 IPC 中表达。
- P1 需要改变状态机、灯位或点击热区才可工作。
- P2 必须引入超出最小权限的文件访问范围。
- P3 仅靠 Canvas 无法实现固定裁剪。
- 两次相同策略失败，且新证据不能支持继续重试。

## Actor Policy

允许：

- 在 checklist 范围内编辑 Rust、TypeScript、CSS、Tauri capability 和文档。
- 运行离线 Rust 检查、现有 pnpm 构建、聚焦测试和本地 Windows 手工验证。
- 创建临时测试素材并在验证后清理。

禁止：

- 扩展到云服务、上传、同步、在线素材、数据库或通用插件体系。
- 重构 detector、taskbar attach、hooks lifecycle、Win32 fallback UI。
- 以自动删除用户素材、修改外部系统或静默修复替代明确回退提示。

## Observer

每轮记录原始证据：

- 修改文件与 `git diff` 摘要。
- 聚焦测试、`cargo check`、`cargo test`、`pnpm build` 输出。
- 运行期日志中的 PNG 加载/回退结果。
- 人工观察：任务栏显示、常亮/闪烁、点击行为、宽/窄设置窗口。
- 实际验证的 `target\debug\taskbar-widget.exe` 或 `target\release\taskbar-widget.exe` 路径。

证据放置：

- 单任务决策与异常：`docs/reflections/task-<task-id>-<timestamp>.md`
- 阶段最终状态与下一建议：`docs/handoff/`

## Verifier Order

1. 格式与聚焦单元测试。
2. 对应 crate 的离线 `cargo check`。
3. 前端 `pnpm build`（仅在 UI 阶段后）。
4. `cargo test --workspace --offline`。
5. 单独构建 settings，再单独构建 host。
6. Windows 人工验证和资源故障回退验证。

进展只由验证状态改变判定；新增代码、日志或自述都不是完成证据。

## Failure Semantics

| 类别 | 判定 | 行动 |
| --- | --- | --- |
| Transient | 一次性构建/运行抖动 | 原命令最多重试一次。 |
| Strategy | 同一实现思路两次验证失败 | 停止重试，更新假设并回到当前阶段最小任务。 |
| Environment | 离线依赖、Tauri capability、Windows 环境缺失 | 记录证据；仅在可替代检查耗尽后请求外部协助。 |
| Policy | 下一步需要超出 MVP 的权限或产品决策 | 停止并交给用户决定。 |
| Unknown | 无法定位失败原因 | 先增加聚焦测试/日志或缩小复现；不得盲改。 |

## Exit Conditions

成功退出：Checklist 的 Completion Criteria 全部满足，证据和 reflection 已落盘。

阻塞退出：连续两次相同外部/环境阻塞，且下一步需要用户选择、权限或真实 Windows 观察。

风险退出：推进需要违反 MVP 非目标，例如新增云同步、通用资源系统、改变状态机或自动破坏用户素材。

人工接管退出：需要确认实际任务栏视觉、窄窗口体验或 Tauri 文件访问授权，且本地代码检查无法替代。

## Current Start State

- Current phase/task: `PRE-01`，之后顺序执行预检和 `P0-01`。
- Current hypothesis: 现有 shared-core 配置、host PNG 解码与前端 Canvas 能以最小增量完成此 MVP。
- Remaining scope: P0 数据契约、P1 host 渲染、P2 原子存储、P3 UI、P4 集成验收。

## Goal Recommendation

建议开始编码时创建一个持久 `/goal`：该功能跨 Rust、Tauri、前端和 Windows 手工验证，且需要多轮记录失败分类与阶段证据。

`/goal` 的完成条件应绑定本 checklist 的 Completion Criteria；它仅保存任务生命周期，不替代 checklist、验证器或重规划规则。
