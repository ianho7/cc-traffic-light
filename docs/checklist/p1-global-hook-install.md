# P1 全局 Hook 安装 Checklist

日期：2026-07-01

## Checklist Objective

目标是把 [p1-global-hook-install/README.md](/D:/project/cc-traffic-light/docs/plan/p1-global-hook-install/README.md) 转成可执行 checklist，完成用户级 Codex lifecycle hook 安装路径的设计、实现与验证闭环。

目标结果：

- 提供 `taskbar-widget/scripts/install-codex-hooks.ps1`，支持 `dry-run`、`apply`、`restore`。
- 安装器优先操作用户级 `hooks.json`，对现有配置做 backup + merge，而不是整文件覆盖。
- 安装后的 hook command 指向稳定安装路径，避免因为 command hash 漂移反复触发 trust。
- 能证明一次 trust 后，多个本地 Codex 对话都可写入同一个共享状态源。

范围：

- 仅覆盖用户级 Codex lifecycle hooks 安装、备份、恢复、合并、trust 说明和双会话验证。
- 仅覆盖本机本地 Codex 对话，不扩展到云端不加载本地配置的会话。

非目标：

- 不尝试绕过 Codex trust。
- 不实现企业级 managed hooks。
- 不把 `notify` 配置一并迁移或重写。
- 不扩展成通用 plugin / installer 平台。

## Loop Engineering Spec

### Goal

- 交付一个安全、可重复执行的用户级 hooks 安装器，并给出真实用户配置上的 `dry-run` 证据、fixture 上的 `apply/restore` 证据，以及真实双会话覆盖证据。
- 进度证据来自脚本文档、脚本输出摘要、fixture 文件 diff、backup 文件存在性、`/hooks` trust 观察、`taskbar_widget_hook.exe list` 输出和共享状态摘要。
- 完成证据不是“脚本能写文件”，而是“脚本可逆、不会破坏既有 hooks、路径稳定、trust 后覆盖多个本地会话”。

### State

- Source of truth: 本 checklist、[p1-global-hook-install/README.md](/D:/project/cc-traffic-light/docs/plan/p1-global-hook-install/README.md)、现有项目级 hooks 示例和相关脚本。
- Persistent loop state: 当前 phase、当前 task id、目标用户级 hooks 路径、稳定 command 路径约定、最近一次验证结果、失败分类、下一步假设，记录到 `docs/reflections/task-<task-id>-<timestamp>.md`。
- Raw evidence: 用户级 `hooks.json` 脱敏摘要、fixture 输入输出、backup 路径、merge 前后 JSON 摘要、`/hooks` UI 观察、共享状态中的 `codex_<session_id>` 列表。
- Discardable state: 一次性终端全文输出、临时截图失败样本、无需长期保留的非结构化观察。

### Planner

- 默认选择当前 phase 中编号最小、依赖已满足、最能改变验证状态的 task。
- 优先顺序是：先定安装契约，再做 dry-run，再做 fixture apply/restore，最后才碰真实用户配置 apply 和双会话验证。
- 若真实用户配置存在不确定形态，优先增加 fixture 或摘要输出来收敛 merge 行为，不直接在真实配置上试错。
- 不在每轮重写计划；继续以本 checklist 为执行顺序，只有新证据显示当前 merge 契约错误时才局部 replan。

### Actor

- 允许动作：读取仓库文档和脚本、用 `apply_patch` 更新文档与 PowerShell 脚本、运行 `cargo check` / `cargo build`、对 fixture 文件执行 install script 验证、更新 checklist / handoff / reflections。
- 中风险动作：读取真实用户级 `C:\Users\admin\.codex\hooks.json`、对真实用户配置执行 `dry-run`、人工执行 `/hooks` review/trust。
- 高风险动作：对真实用户级配置执行 `apply` / `restore`。只有在前置验证完成且用户明确接受真实配置变更时才执行。
- 非默认动作：修改 `notify`、切到 inline `config.toml`、引入 managed hooks、扩大到通用安装器。

### Observer

- 每次动作后先记录原始观察，再写判断：例如“发现用户已有 `PreToolUse` hooks”与“当前 merge 策略会冲突”必须分开。
- 对真实用户配置只记录脱敏结构摘要，不在文档里保存不必要的本地命令细节或隐私数据。
- 明确区分“脚本输出计划修改”和“配置已实际写入”，避免把 dry-run 当成 apply 成功。

### Verifier

- Verifier order:
- 1. focused doc/script review：确认现有 hooks 示例、notify backup/restore 模式和最终命令 shape。
- 2. `cargo check`。
- 3. `cargo build`，确认 hook 可执行文件仍可生成。
- 4. install script 的 fixture `dry-run`。
- 5. fixture `apply` 与结果检查。
- 6. fixture `restore` 与回滚检查。
- 7. 真实用户配置 `dry-run`。
- 8. 在用户批准后执行真实 `apply` / `/hooks` trust / 双会话验证。
- Actor 不能自证完成；必须有文件 diff、命令结果、backup 证据或真实状态写入证据。

### Failure Semantics

- Transient failure: 文件被占用、PowerShell 临时解析异常、一次性构建失败，可直接重试 1 次。
- Script failure: JSON 解析、merge 规则、路径拼接或原子写入失败，回到最小相关 task 修复脚本。
- Strategy failure: 同一 merge 问题连续 2 次没有新证据，停止在真实配置上试错，回到 fixture 扩样本。
- Environment failure: 用户级配置不存在、`/hooks` 未刷新、真实多会话无法在当前轮同时触发，记录 blocked evidence。
- Policy failure: 下一步需要修改真实用户配置但用户未批准，停止在 `dry-run` 证据并等待外部决定。

### Exit Conditions

- Success exit: Completion Criteria 满足，且 handoff / reflection 已记录真实安装与验证结论。
- Blocked exit: 继续推进必须依赖用户批准真实配置写入、人工 trust 或外部会话配合，且当前轮已无更多本地验证动作。
- Budget exit: 同一 phase 连续 3 次没有新证据，停止并产出 handoff。
- Risk exit: 为继续推进需要改写 `notify`、迁移到 `config.toml` 或扩大到企业级管理方案。
- Human takeover exit: 需要用户决定最终稳定安装目录、真实 apply 时机或是否接受当前 trust 交互。

### Policy

- 不整文件覆盖真实用户 hooks 配置。
- 不删除或改写用户现有非本软件 hooks。
- 不保存完整用户 hooks payload 或无必要的本地隐私路径到仓库文档。
- 不自动完成 trust；只记录为人工 gate。
- 不把真实用户配置 apply 当作默认步骤；默认先停在 `dry-run + fixture pass`。

## Runtime Loop Protocol

每轮执行遵循：

1. Inspect：读取当前 phase/task、计划文档、相关脚本和最近 reflection。
2. Choose：按 planner 规则选一个最小可验证 task。
3. Act：做最小脚本或文档修改，或执行最小验证命令。
4. Observe：记录脚本输出摘要、文件变化或人工 `/hooks` 观察。
5. Verify：运行该 task 的最小 verifier。
6. Reflect：完成、失败、跳过、blocked 都生成对应 reflection。
7. Decide：继续下一 task、重试、replan、blocked、risk exit 或 complete。

继续条件：

- 当前 task 有明确下一步，且上一轮带来了新的结构化证据。
- 失败已被分类，且还在对应 retry / replan 预算内。

停止条件：

- Completion Criteria 已满足。
- 下一步需要真实用户配置写入批准、人工 trust 或外部多会话协作，而当前轮无法继续。
- 同一 phase 反复失败且没有新证据。
- 下一步会把范围扩大到本轮非目标。

## Pre-Implementation Checks

- [ ] GHI-PRE-01 阅读 [p1-global-hook-install/README.md](/D:/project/cc-traffic-light/docs/plan/p1-global-hook-install/README.md)，确认本轮范围只做用户级 lifecycle hooks 安装闭环。
- [ ] GHI-PRE-02 阅读 `.codex/hooks.json`，确认当前项目级测试配置与用户级安装目标的差异。
- [ ] GHI-PRE-03 阅读 [taskbar-widget/examples.codex-hooks.toml](/D:/project/cc-traffic-light/taskbar-widget/examples.codex-hooks.toml)，确认 inline hooks 只是对照方案而不是默认落地方案。
- [ ] GHI-PRE-04 阅读 [taskbar-widget/scripts/codex-notify-probe-config.ps1](/D:/project/cc-traffic-light/taskbar-widget/scripts/codex-notify-probe-config.ps1)，抽取可复用的 backup / restore / dry-run 模式。
- [ ] GHI-PRE-05 确认 hook 可执行文件的稳定安装目录候选、最终 command shape 和 marker/name 约定。
- [ ] GHI-PRE-06 确认验证命令至少包含 `cargo check`、`cargo build`、install script dry-run / apply / restore，以及 `taskbar_widget_hook.exe list`。
- [ ] GHI-PRE-07 确认真实用户配置 apply 不是默认路径，必须晚于 fixture 验证并经过用户批准。

## Implementation Checklist

### Phase 1: 定义安装契约

当前进展注记（2026-07-01 18:45）：

- 安装脚本已落地为 [install-codex-hooks.ps1](/D:/project/cc-traffic-light/taskbar-widget/scripts/install-codex-hooks.ps1)。
- 当前实现采用的稳定契约是：
  - 用户级目标文件：`%USERPROFILE%\.codex\hooks.json`
  - 默认稳定 hook 路径：`%LOCALAPPDATA%\CcTrafficLight\bin\taskbar_widget_hook.exe`
  - 受管 marker：`CcTrafficLight Codex <HookName>`（落在 `statusMessage`）
  - backup：`hooks.json.cc-traffic-light-global-hooks.bak`
  - backup meta：`hooks.json.cc-traffic-light-global-hooks.bak.meta.json`
- 当前脚本会拒绝 `target\debug` / `target\release` 一类 cargo 构建目录，防止 command hash 随构建漂移。
- Phase 1 的契约已经体现在脚本实现中，但本轮先把重点放在脚本可运行和 fixture 验证，Phase 1 的显式勾选留到下一轮与真实 apply gate 一起收口。

进展补充（2026-07-01 18:55）：

- 稳定 exe 已复制到 `C:\Users\admin\AppData\Local\CcTrafficLight\bin\taskbar_widget_hook.exe`。
- 真实用户级 `C:\Users\admin\.codex\hooks.json` 已成功执行一次 `apply`。
- apply 后再次 dry-run 的结果是 `changes_required = false`，7 个受管 lifecycle hooks 都是 `unchanged`。

- [ ] GHI-A-01 明确用户级目标文件路径、备份文件命名、临时写入文件命名和原子替换策略。
- [ ] GHI-A-02 明确本软件 hook 的稳定 marker/name、覆盖事件集合和 command 参数顺序。
- [ ] GHI-A-03 明确稳定 hook 可执行文件安装路径要求，禁止指向 `target\debug` 一类易漂移路径。
- [ ] GHI-A-04 明确 merge 规则：只更新本软件 marker 对应条目，不触碰其他已有 hooks。
- [ ] GHI-A-05 明确 install script 的参数契约、退出码和摘要输出格式。

### Phase 2: 实现 Dry-Run 安装器

- [x] GHI-B-01 新增或初始化 [taskbar-widget/scripts/install-codex-hooks.ps1](/D:/project/cc-traffic-light/taskbar-widget/scripts/install-codex-hooks.ps1) 脚本骨架。
- [x] GHI-B-02 实现读取用户级 `hooks.json` 与缺失文件兜底初始化逻辑。
- [x] GHI-B-03 实现 JSON 解析和结构校验，遇到未知结构时输出可诊断错误。
- [x] GHI-B-04 实现 merge 规划逻辑，在内存中生成“原配置摘要 + 计划修改摘要”，但不写盘。
- [x] GHI-B-05 确保 dry-run 输出脱敏、稳定、适合 review，不泄露不必要的用户本地信息。

### Phase 3: 实现 Apply 和 Restore

- [x] GHI-C-01 实现写入前 backup，保证 backup 路径可预测且不会覆盖旧备份。
- [x] GHI-C-02 实现 apply 写入流程，保证只落地 merge 后结果而不是整文件盲覆盖。
- [x] GHI-C-03 实现原子写入或最小风险写入流程，避免半写入状态破坏用户配置。
- [x] GHI-C-04 实现 restore 流程，可从最近一次有效 backup 恢复到 apply 前状态。
- [x] GHI-C-05 确保脚本永远不修改 `notify` 配置，也不迁移到 inline `config.toml`。

### Phase 4: Fixture 与真实配置验证

- [x] GHI-D-01 构造最小 fixture：空配置、已有无关 hooks 配置、已有本软件旧 marker 配置。
- [x] GHI-D-02 对每个 fixture 运行 `dry-run`，确认摘要与预期 merge 行为一致。
- [x] GHI-D-03 对每个 fixture 运行 `apply`，检查 backup、输出文件和 marker 更新结果。
- [x] GHI-D-04 对每个 fixture 运行 `restore`，确认恢复后内容与 apply 前一致。
- [x] GHI-D-05 对真实用户级 `hooks.json` 执行 `dry-run`，记录脱敏结构摘要和计划修改摘要。
- [x] GHI-D-06 只有在用户明确同意后，对真实用户配置执行 `apply`，并确认 backup 已生成。

### Phase 5: Trust 与覆盖范围验证

当前验证补充（2026-07-01 19:05）：

- 用户按侧边新对话提示完成了两个本地 Codex 对话测试：
  - `019f1d1d-0adb-71f3-9f22-2312ee73a03d`
  - `019f1d1d-d7e2-7930-8704-9931d7efa378`
- 随后读取共享状态文件和 `taskbar_widget_hook.exe list`，都观察到：
  - `codex_019f1d1d-0adb-71f3-9f22-2312ee73a03d`
  - `codex_019f1d1d-d7e2-7930-8704-9931d7efa378`
- 这两条记录都满足：
  - `session_id_source = payload`
  - `state = done`
  - `hook_name = Stop`
- 这说明“不同本地对话 -> 同一共享状态源”已经有直接证据。
- 但本轮没有直接截图或记录 `/hooks` UI，因此 `E-02` 仍保持未勾选。

- [ ] GHI-E-01 在真实 apply 后重新打开或刷新本机 Codex 会话，确保用户级 hooks 被重新加载。
- [ ] GHI-E-02 在 Codex 中运行 `/hooks`，确认目标 lifecycle hooks 已出现且 source 正确。
- [ ] GHI-E-03 完成一次 hooks review/trust，确认 stable command 定义已被允许执行。
- [x] GHI-E-04 打开两个不同的本地 Codex 对话或项目，分别触发最小 lifecycle 事件。
- [x] GHI-E-05 运行 `taskbar_widget_hook.exe list` 或等价状态查看命令，确认两个不同 `codex_<session_id>` 都写入同一共享状态源。
- [ ] GHI-E-06 确认 trust 后不需要因为 command 路径变化再次 review；若出现重复 trust，记录为安装契约失败。

## Validation Checklist

- [x] GHI-VAL-01 运行 `cargo check`，期望无编译错误。
- [x] GHI-VAL-02 运行 `cargo build`，期望 hook 可执行文件成功生成。
- [x] GHI-VAL-03 对 fixture 运行 install script `dry-run`，期望只输出摘要、不写文件。
- [x] GHI-VAL-04 对 fixture 运行 `apply`，期望生成 backup 且只合并本软件 hooks。
- [x] GHI-VAL-05 对 fixture 运行 `restore`，期望内容完整回滚到 apply 前状态。
- [x] GHI-VAL-06 对真实用户配置运行 `dry-run`，期望成功输出脱敏结构摘要和计划修改摘要。
- [ ] GHI-VAL-07 若用户批准真实 apply，期望 `/hooks` 中能看到目标 lifecycle hooks 且 trust 流程可完成。
- [x] GHI-VAL-08 若用户批准真实 apply，期望两个不同本地会话都能写入同一个共享状态文件。
- [ ] GHI-VAL-09 若出现重复 trust，优先排查稳定 command 路径和参数漂移，而不是扩大到 managed hooks。
- [ ] GHI-VAL-10 若 merge 结果影响了非本软件 hooks，记录为失败并回到 merge 规则修正。

当前验证补充（2026-07-01 18:55）：

- 真实 apply 摘要：
  - `config_existed = false`
  - `written = true`
  - 7 个事件都执行了 `add`
- apply 后复核 dry-run 摘要：
  - `config_existed = true`
  - `changes_required = false`
  - 7 个事件都为 `unchanged`
- 双会话共享状态摘要：
  - `codex_019f1d1d-0adb-71f3-9f22-2312ee73a03d -> done`
  - `codex_019f1d1d-d7e2-7930-8704-9931d7efa378 -> done`
- 两条记录都来自真实 payload session id，而不是 fallback `unknown`
- 这说明真实用户级 `hooks.json` 已处于脚本期望的稳定已安装状态；当前未完成部分已经收敛为 Codex UI reload / trust / 双会话行为验证。

## Documentation Checklist

- [ ] GHI-DOC-01 新增本 checklist 文档并与 P1 计划一一对应。
- [ ] GHI-DOC-02 在脚本完成后补充脚本使用说明、参数示例和风险说明。
- [ ] GHI-DOC-03 若最终稳定安装目录和 marker 契约确定，更新相关 plan / handoff 文档。
- [x] GHI-DOC-04 如真实 apply 或 trust 结论改变后续路线，更新 `docs/handoff/` 或新增 handoff。
- [ ] GHI-DOC-05 每个完成、跳过或阻塞的 task 都生成 reflection。

## Cleanup Checklist

- [ ] GHI-CLN-01 确认没有修改或破坏现有 `notify` 相关脚本与配置。
- [ ] GHI-CLN-02 确认没有把真实用户 hooks 内容、私有路径或敏感命令细节提交进仓库。
- [ ] GHI-CLN-03 确认没有把 fixture 之外的临时用户配置副本留在仓库中。
- [ ] GHI-CLN-04 确认脚本错误信息、marker 命名和文档术语保持一致。
- [ ] GHI-CLN-05 确认没有顺手扩展到 inline `config.toml`、managed hooks 或通用安装框架。
- [ ] GHI-CLN-06 确认 backup / restore / dry-run 行为都可被后续调试轮复用。

## Completion Criteria

以下条件满足时，本轮可判定完成：

- `taskbar-widget/scripts/install-codex-hooks.ps1` 已支持 `dry-run`、`apply`、`restore`。
- 安装器对用户级 `hooks.json` 执行 backup + merge，而不是整文件覆盖。
- merge 规则只更新本软件 marker 对应条目，不破坏用户现有其他 hooks。
- hook command 已指向稳定安装路径，而不是构建产物临时路径。
- fixture 上的 `dry-run`、`apply`、`restore` 全部通过。
- 对真实用户配置至少完成 `dry-run` 并得到安全摘要。
- 若用户批准真实 apply，则 `/hooks` trust 与双会话共享状态覆盖验证通过。
- 已知限制、人工 gate 和后续建议已写入 handoff 或相关文档。

可接受的已知限制：

- 真实用户配置 apply 可以延后到用户明确批准时执行。
- 本轮只覆盖用户级 `hooks.json`，不同时支持 inline `config.toml`。
- trust 仍需要一次人工 review；本轮只保证其稳定性，不尝试消除该交互。

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

- task id 必须对应本 checklist 中的条目，例如 `GHI-B-04`。
- 完成、阻塞、跳过都要生成 reflection，并写明原因。
- 涉及真实用户配置的任务必须说明是 `dry-run` 还是 `apply`。
- 涉及 trust 的任务必须记录是否完成人工步骤。
- 涉及 merge 行为的任务必须区分“计划修改摘要”和“实际写盘结果”。

## Goal Usage Recommendation

这项工作适合用 `/goal` 或等价长期目标执行，因为它具备多阶段脚本实现、fixture 验证、真实配置人工 gate 和明确的 blocked / complete 语义。

建议 objective：

```text
Implement and validate a safe user-scoped Codex lifecycle hook installer that can dry-run, apply, and restore hooks.json with backup+merge semantics, then prove trusted hooks cover multiple local Codex conversations through one shared state source.
```

Continue condition：

- 还有未完成的 phase task，且上一轮带来了新的结构化证据。

Completion condition：

- Completion Criteria 全部满足，且最新 handoff / reflection 已记录真实验证结果。

Blocked condition：

- 连续多轮都卡在真实用户配置批准、人工 trust 或外部多会话配合，且本地已无更多可验证动作。

Budget boundary：

- 同一 phase 连续 3 次没有新证据，或继续执行只会重复真实配置试错时，停止并转入 handoff / reflection。
