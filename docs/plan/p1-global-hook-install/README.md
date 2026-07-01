# P1: 全局 Hook 安装

## 目标

设计并实现一个安全的用户级 Codex hook 安装路径，让软件在用户完成一次 trust 后，就能监控本机所有本地 Codex 对话。预期结果是：提供可重复执行的 install/restore 流程，能够写入或合并用户级 hooks，而不会破坏用户现有配置。

范围内：用户级 Codex lifecycle hook 安装、备份、恢复、合并策略和 trust 说明。范围外：绕过 Codex trust、企业级 managed hooks、不会加载本地配置的云端会话，以及通用 plugin 架构。

## 背景与上下文

项目级 `.codex/hooks.json` 只适合当前 repo 内测试。最终产品形态需要写入用户级 hooks，例如 `C:\Users\admin\.codex\hooks.json`，或者用户级 `config.toml` 中的 inline hooks。

Codex 对非 managed command hooks 需要 review/trust。trust 与 hook 定义 hash 绑定，所以安装路径必须稳定。如果 command 路径随着每次 build 改变，用户就会被反复要求重新 trust。

## 当前状态分析

当前脚本只覆盖 `notify` 探针和 lifecycle dump 测试，还没有全局 lifecycle hooks 安装器。项目里已有开发用 hook CLI 路径，但最终产品不应该指向 `target\debug`，而应该指向稳定的安装目录。

相关文件：

- `.codex/hooks.json`：项目级测试配置。
- `taskbar-widget/examples.codex-hooks.toml`：inline TOML 示例。
- `taskbar-widget/scripts/codex-notify-probe-config.ps1`：可借鉴其 backup/restore 模式，但它针对的是 `notify`。
- `docs/checklist/codex-lifecycle-hooks-validation.md`：证明 lifecycle hooks 可用。

## 方案建议

新增一个专门的全局 hook 安装脚本，负责：

- 读取现有用户级 `hooks.json`。
- 写入前先备份。
- 以稳定 marker/name 合并本软件的 hook 条目。
- 把 hooks 指向稳定安装路径下的 hook 可执行文件。
- 支持 `dry-run`、`apply`、`restore`。
- 永远不修改 `notify`。

安装器优先选择用户级 `hooks.json`，而不是 inline `config.toml`，因为 JSON 更容易做安全合并，也能避免在同一配置层混用两种 hook 表示方式。

## 备选方案

- 在 `config.toml` 里写 inline hooks：文件更少，但更难与现有配置安全合并。
- 只做项目级 hooks：安全，但不能满足“所有对话覆盖”目标。
- 使用 managed hooks：最适合企业级静默 trust，但不属于当前 MVP 可用路径。

## 实施计划

### Phase 1: 定义安装契约

- 目标：明确稳定 hook 路径和归属标识。
- 文件：本计划文档，以及后续 `taskbar-widget/scripts/install-codex-hooks.ps1`
- 任务：决定安装根目录、可执行文件路径、备份文件名和 merge marker。
- 预期结果：安装器行为完全确定。

### Phase 2: 实现 Dry-Run 安装器

- 目标：在不写文件的前提下检查用户 hooks。
- 文件：`taskbar-widget/scripts/install-codex-hooks.ps1`
- 任务：读取 `C:\Users\admin\.codex\hooks.json`，解析 JSON，输出已有事件和计划修改的脱敏摘要。
- 预期结果：`dry-run` 只打印安全摘要，不写入用户配置。

### Phase 3: 实现 Apply 和 Restore

- 目标：安全安装和卸载用户级 hooks。
- 文件：`taskbar-widget/scripts/install-codex-hooks.ps1`
- 任务：备份现有 hooks，合并本软件 hooks，原子写入，支持从备份恢复。
- 预期结果：用户可安装、回滚，且不会丢失原配置。

### Phase 4: Trust 与覆盖范围验证

- 目标：证明安装后的 hooks 能覆盖本地 Codex 会话。
- 文件：用户级 hooks 文件和状态文件。
- 任务：运行 `/hooks`，trust 一次，打开两个不同的 Codex 对话或项目，触发事件，确认都能写入同一个状态文件。
- 预期结果：trust 后不需要为每个对话单独改配置。

## 验证策略

- 对真实用户配置执行 `dry-run`
- 对临时 fixture 配置执行 `apply/restore`
- 只有在用户明确同意后，才对真实用户配置执行 `apply`
- 人工执行一次 `/hooks` trust
- 双会话 Codex 测试：两个会话都能产生不同的 `codex_<session_id>` task

## 风险与缓解

- Risk: 覆盖用户已有 hooks。Mitigation: 只做 backup + merge，绝不整文件覆盖。
- Risk: 重复 trust 提示。Mitigation: 使用稳定安装路径和稳定 command 定义。
- Risk: 现有 hooks 并发运行产生冲突。Mitigation: 不删除用户现有 hooks，并验证多个 hooks 可共存。
- Risk: 安装后的 hook 二进制路径变化。Mitigation: 由安装器控制固定路径。

## 待确认问题

- 最终产品的 hook 可执行文件应该安装到哪个固定目录？
- 安装器是否要同时支持用户级和项目级两种模式？

## 推荐下一步

在 P0 验证通过后，设计 `install-codex-hooks.ps1`，复用现有 notify helper 的 `dry-run/apply/restore` 模式，但目标改为 lifecycle `hooks.json`。
