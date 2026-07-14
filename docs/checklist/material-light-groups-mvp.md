# 素材灯组 MVP 执行 Checklist

## Checklist Objective

完成本地素材灯组 MVP：用户以绿、黄、红三张裁剪后的图片创建可复用灯组，分别应用给 Codex 或 Claude Code；任务栏保留 Agent 标识，沿用现有状态映射、亮灭、闪烁与整组点击行为。

范围以 [素材灯组 MVP 实施计划](../plan/material-light-groups-mvp.md) 为准。不得实现同步、在线素材库、动图、布局定制、状态映射定制或无关重构。

## 执行规则

- 阶段严格按 `P0 → P1 → P2 → P3 → P4` 串行；下游阶段不得在上游验收前开始。
- 每次只执行一个未完成任务，先记录原始验证输出，再决定下一步。
- 每个完成任务都要自动生成 `docs/reflections/task-<task-id>-<timestamp>.md`，记录问题、思路、备选方案、最终方案和理由。
- 任务只在其验收证据存在时勾选；Agent 声明完成不是证据。
- 当前阶段出现两次相同逻辑失败时，停止重复尝试，更新假设或回到上游阶段排查。

## Pre-Implementation Checks

- [x] `PRE-01` 阅读 `docs/plan/material-light-groups-mvp.md`，确认固定绿/黄/红语义、64×64 PNG 输出和非目标未变。（完成后生成 reflection）
- [x] `PRE-02` 审查 `crates/shared-core/src/app_config.rs` 的 schema、默认值、normalize 与配置测试，记录最小扩展点。（完成后生成 reflection）
- [x] `PRE-03` 审查 `taskbar-widget/src/widget_render.rs`、`widget_image.rs` 的三灯布局、alpha 与 RGBA 混合路径，记录不得改变的接口。（完成后生成 reflection）
- [x] `PRE-04` 审查 Tauri 的 command、capability 与前端导航/本地化模式；确认文件选择和本地写入所需的最小能力。（完成后生成 reflection）
- [x] `PRE-05` 记录验证命令：`cargo test --workspace --offline`、`cargo check -p taskbar-widget --offline`、`pnpm build`，以及单独构建 settings 后构建 host 的顺序。（完成后生成 reflection）

## Implementation Checklist

### P0：数据契约与兼容回退

完成门槛：旧 `config.json` 可读、无素材配置时行为与当前版本一致、未知引用不会导致 host 使用无效资源。

- [x] `P0-01` 在 shared-core 定义最小 `MaterialGroup`：`id`、`name`、`green_path`、`yellow_path`、`red_path`；不创建通用资源框架。（完成后生成 reflection）
- [x] `P0-02` 在 `WidgetVisualConfig` 增加灯组列表及 Codex/Claude 的可选灯组 ID 绑定，默认均为内建灯组。（完成后生成 reflection）
- [x] `P0-03` 提升配置 schema，并让旧 schema 自动得到空素材库和默认绑定。（完成后生成 reflection）
- [x] `P0-04` 在配置 normalize 中处理空 ID、缺槽位、重复 ID 和未知绑定：保留安全配置语义并让渲染端可回退。（完成后生成 reflection）
- [x] `P0-05` 同步 TypeScript `AppConfig`/DTO 镜像，确保设置端与 host 的序列化字段一致。（完成后生成 reflection）
- [x] `P0-06` 添加或更新配置测试：旧配置兼容、round-trip、两个 Agent 独立绑定、未知 ID 回退。（完成后生成 reflection）
- [x] `P0-07` 运行 shared-core 相关测试与 `cargo fmt --check`；记录原始输出。（完成后生成 reflection）

### P1：Host PNG 缓存与灯位替换

前置条件：P0 全部通过。

完成门槛：有效素材替换内建圆灯；所有状态仍复用原 alpha；任意资源错误回退默认灯组；点击热区不变。

- [x] `P1-01` 在 `widget_image.rs` 增加本地 PNG 的可释放缓存模型，按灯组和文件变化重载，不在每帧读取/解码。（完成后生成 reflection）
- [x] `P1-02` 增加 RGBA 素材绘制函数，复用现有 `PixelBuffer::blend_pixel`；保持 16×16 既有灯位和间距。（完成后生成 reflection）
- [x] `P1-03` 在 `widget_render.rs` 选择每个 Agent 的默认/自定义灯组，三图常驻，非激活项按暗态 alpha，激活项按原常亮/闪烁 alpha。（完成后生成 reflection）
- [x] `P1-04` 为图片缺失、无权限或 PNG 解码失败实现“该 Agent 回退内建圆灯”的路径；不写回配置或删除文件。（完成后生成 reflection）
- [x] `P1-05` 核对 `hot_zones` 与点击处理未改变：点击任意素材仍打开设置。（完成后生成 reflection）
- [x] `P1-06` 添加 host 测试：默认回归、有效替换、暗态、激活态、闪烁态、单图失败回退、同组/不同组双 Agent。（完成后生成 reflection）
- [x] `P1-07` 运行 `cargo check -p taskbar-widget --offline` 和聚焦测试；记录结果。（完成后生成 reflection）

### P2：素材写入与原子一致性

前置条件：P1 全部通过。

完成门槛：完整灯组可原子保存到 `%APPDATA%\CcTrafficLight\assets\<group-id>\`，配置不会指向半完成资源。

- [x] `P2-01` 确认 Tauri capability 中实现文件选择和应用数据目录访问所需的最小权限；不得添加无关文件系统范围。（完成后生成 reflection）
- [x] `P2-02` 实现保存完整灯组的命令：只接受已裁剪的 64×64 PNG，写入固定 `green.png`、`yellow.png`、`red.png` 路径。（完成后生成 reflection）
- [x] `P2-03` 使用临时文件和替换顺序保证三图均成功后才更新配置；失败保留先前完整版本。（完成后生成 reflection）
- [x] `P2-04` 实现删除未被引用灯组的命令；若 Codex 或 Claude 正在使用，返回可显示的拒绝原因。（完成后生成 reflection）
- [x] `P2-05` 将保存、删除、应用与恢复默认接到既有 settings IPC/通知路径，使 host 失效缓存并重绘。（完成后生成 reflection）
- [x] `P2-06` 添加针对路径、无效 PNG、写入失败、引用删除拒绝和成功替换的测试。（完成后生成 reflection）
- [x] `P2-07` 运行 Tauri/shared-core 相关测试；记录资产目录、配置变更和原始输出。（完成后生成 reflection）

### P3：素材库主流程与响应式 UI

前置条件：P2 全部通过。

完成门槛：用户在设置页可独立完成“选择图 → 裁剪 → 保存 → 应用 → 任务栏生效”。

- [x] `P3-01` 在现有设置导航添加“素材灯组”页及中英文文案，沿用当前导航、页面容器和 pending 模式。（完成后生成 reflection）
- [x] `P3-02` 实现灯组列表：显示默认灯组、保存组、每个 Agent 当前绑定及“应用到 Codex/Claude”“恢复默认”动作。（完成后生成 reflection）
- [x] `P3-03` 实现创建/编辑表单：名称和绿/黄/红三个固定素材槽位；名称为空或任一槽位未确认时禁用保存。（完成后生成 reflection）
- [x] `P3-04` 用浏览器 Canvas 实现单图裁剪：输入 PNG/JPEG/WebP、固定正方形、拖动平移、缩放滑杆、导出 64×64 PNG；不引入裁剪依赖。（完成后生成 reflection）
- [x] `P3-05` 为覆盖图片、删除灯组提供确认提示；对在使用的灯组展示不可删除原因。（完成后生成 reflection）
- [x] `P3-06` 展示素材不可用状态，说明 host 已回退内建灯组；不做自动修复或后台扫描。（完成后生成 reflection）
- [x] `P3-07` 编写/更新前端测试（若项目现有测试基础允许）或可重复的手动 UI 验证记录：宽窗口三列、窄窗口单列、三图保存限制、应用与恢复默认。（完成后生成 reflection）
- [x] `P3-08` 运行 `pnpm build`；修复仅限该功能引入的类型、构建或本地化错误。（完成后生成 reflection）

### P4：集成验收与交付证据

前置条件：P3 全部通过。

完成门槛：所有自动检查通过，并有真实 Windows 桌面行为证据；用户未提供的安装或录屏证据不得标记为通过。

- [x] `P4-01` 运行 `cargo test --workspace --offline`，记录失败属于代码、环境或既有问题的证据。（完成后生成 reflection）
- [x] `P4-02` 运行 `cargo check -p taskbar-widget --offline`。（完成后生成 reflection）
- [x] `P4-03` 运行 `pnpm build`。（完成后生成 reflection）
- [x] `P4-04` 分别构建 `taskbar-settings-tauri` 后再构建 `taskbar-widget`；记录实际验证的根 `target\debug` 或 `target\release` host exe 路径。（完成后生成 reflection）
- [x] `P4-05` 手工验证：三图缺失不可保存、同组可复用给两个 Agent、分别应用不同组、恢复默认、删除保护。（完成后生成 reflection）
- [x] `P4-06` 手工验证：默认/暗态/常亮/闪烁、点击任意素材打开设置、任务栏无布局漂移。（完成后生成 reflection）
- [x] `P4-07` 手工删除一张已应用素材并重载，验证对应 Agent 回退内建灯组且设置页提示素材不可用。（完成后生成 reflection）
- [x] `P4-08` 验证设置页桌面与窄窗口布局；记录截图或人工观察结果。（完成后生成 reflection）
- [x] `P4-09` 在 `docs/handoff/` 或 `docs/reflections/` 记录 schema、资产路径、验证命令、实际 host exe 路径、人工观察和未通过项。（完成后生成 reflection）

## Cleanup Checklist

- [x] `CLN-01` 删除仅为开发创建的测试素材、临时 PNG 和调试日志；不得删除用户既有资源。（完成后生成 reflection）
- [x] `CLN-02` 检查无 Base64 图片被写入 `config.json`，无本机绝对用户路径进入受版本控制的配置或源文件。（完成后生成 reflection）
- [x] `CLN-03` 检查错误信息明确区分“素材不可用”“保存失败”“灯组正在使用”，并保持现有中英文文案模式。（完成后生成 reflection）
- [x] `CLN-04` 审核 diff，确认没有无关 detector、taskbar attach、hooks lifecycle 或 fallback UI 重构。（完成后生成 reflection）

## Completion Criteria

- P0–P4 与清理项全部具备完成证据和对应 reflection。
- 三张固定语义素材可保存、复用、应用、恢复默认；不完整组绝不可保存。
- Host 在不改变布局、状态语义、动画和点击行为的条件下渲染素材。
- 任何素材文件故障不会使 widget 失效，且只影响对应 Agent 并回退默认灯组。
- `cargo test --workspace --offline`、`cargo check -p taskbar-widget --offline`、`pnpm build` 的结果已记录；若有失败，已明确说明是否为本变更引起。
- 已完成 Windows 桌面与窄窗口人工验证，并记录实际被验证的 host 可执行文件路径。

## 可接受的 MVP 限制

- 仅本地文件；不支持同步、分享或导入导出。
- 仅静态 PNG 输出；不支持 GIF、SVG、滤镜、旋转或自定义布局。
- 自定义图片使用既有 16×16 灯位；不提供大小、间距或状态映射编辑。
