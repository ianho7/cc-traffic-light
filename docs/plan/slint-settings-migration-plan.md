# Slint Settings Migration Plan

## Objective

目标是在不破坏当前稳定 Win32 taskbar widget / tray 主路径的前提下，把现有“纯 Win32 自绘 settings 窗口”替换成 `Slint` 实现的现代设置界面。

要解决的问题：

- 当前 `settings_window.rs` 是纯 GDI 自绘 MVP，功能闭环已经基本成立，但界面明显过于简陋。
- 继续用纯 Win32 手工绘制把界面做到“产品级”成本高，而且后续 Diagnostics / Appearance / About 的扩展体验会持续受限。

预期结果：

- `settings` 改成 `Slint` 驱动的现代桌面 UI。
- `widget`、`tray`、taskbar attach / recovery 主路径继续保留 Win32。
- 单进程架构保持不变，状态源仍然是现有 Rust runtime 和统一 `AppStatusSnapshot` / `AppConfig`。
- 所有设置项都必须本地持久化；关闭软件后重新打开，配置应保持上次状态。
- 所有用户可见文案都需要支持中英国际化，而不是只做单语 settings 界面。
- settings 视觉方向明确参考 `Nothing-inspired` 设计语言：单色、强层级、以字体和网格驱动信息结构。

范围内：

- 新增 `Slint` 依赖、UI 文件、settings runtime 桥接层。
- 用 `Slint` 重建 settings 的 Overview / General / Diagnostics / About / 占位页面。
- 把现有 Win32 settings 交互迁移到 `Slint`。
- 补齐 settings 全量持久化策略，覆盖当前已实现项和后续新增项。
- 为 settings 建立中英文本资源与语言切换/跟随策略。
- 为 tray 菜单、tray tooltip、状态标签、来源方法标签和其他用户可见运行时文案建立同一套中英资源体系。
- 为 Slint settings 建立 `Nothing-inspired` 的字体、配色、卡片、按钮和排版 token。

范围外：

- 不重写 `taskbar-widget/src/taskbar.rs`
- 不把 tray 改成 Slint
- 不把 widget 改成 Slint
- 不引入 WebView / Tauri / WinUI
- 不拆成多进程

---

## Background and Context

当前项目已经完成了 GUI / Tray V1 的大部分运行时闭环：

- `main.rs` 管理主消息循环、widget、tray、detector 轮询、settings 打开逻辑。
- `taskbar.rs` 承担 Win11 taskbar attach / positioning 稳定路径。
- `tray_icon.rs` 提供 tray 图标、菜单和刷新入口。
- `settings_window.rs` 当前是纯 Win32 + GDI 自绘壳子。
- `app_config.rs`、`ui_state.rs`、`detector.rs` 已经形成统一的设置与状态契约。

之前的产品约束是 “V1 统一保持纯 Win32”。这已经完成了一个能运行的版本，但用户反馈很明确：功能基本 OK，UI 太丑。

本迁移方案的前提是：

- 接受项目从“纯 Win32”升级为“Win32 主体 + Slint settings 混合栈”。
- 继续保住最脆弱、最值钱的 Win32 部分：taskbar widget attach path。

视觉输入：

- 按用户要求，settings 设计可参考 `$nothing-design`。
- 字体方案先按该 skill 的建议声明：
  - Display: `Doto`
  - Body / UI: `Space Grotesk`
  - Data / Labels: `Space Mono`
- 这些字体的具体加载方式需要在真正实现 Slint UI 前确定；当前计划阶段只先把它们作为明确设计依赖写入。

已验证事实：

- 当前 settings 入口由 Win32 tray / 主窗口控制。
- 当前 settings 的数据来源是内存中的 `AppStatusSnapshot` 和 `AppConfig`。
- 当前 settings 与主 runtime 的通信方式，本质上是“共享状态 + 主窗口命令消息”。

假设：

- `Slint` 在当前 Windows / Rust 构建环境中可用，但仓库尚未实际接入依赖。
- 单进程内同时保留 Win32 消息循环和 Slint settings window 是可行路径；具体集成细节仍需以实际接入为准。

---

## Current State Analysis

### Relevant Files and Roles

- [taskbar-widget/src/main.rs](/D:/project/cc-traffic-light/taskbar-widget/src/main.rs)
  - 程序入口、Win32 消息循环、tray 命令处理、widget 轮询刷新、settings 打开。
- [taskbar-widget/src/settings_window.rs](/D:/project/cc-traffic-light/taskbar-widget/src/settings_window.rs)
  - 当前 settings 的全部 Win32 绘制与点击逻辑。
- [taskbar-widget/src/tray_icon.rs](/D:/project/cc-traffic-light/taskbar-widget/src/tray_icon.rs)
  - tray 菜单与 settings 打开入口。
- [taskbar-widget/src/app_config.rs](/D:/project/cc-traffic-light/taskbar-widget/src/app_config.rs)
  - settings 持久化契约。
- [taskbar-widget/src/ui_state.rs](/D:/project/cc-traffic-light/taskbar-widget/src/ui_state.rs)
  - 统一状态快照结构。
- [taskbar-widget/src/detector.rs](/D:/project/cc-traffic-light/taskbar-widget/src/detector.rs)
  - detector 聚合输出。

### Existing Implementation Details

- 当前 settings 页面是自绘矩形、文本和按钮，没有现代控件体系。
- `General` 页中真实可写设置已经接上：
  - `autostart_enabled`
  - `start_minimized_to_tray`
  - `close_to_tray`
- `Diagnostics` 页已经能展示真实 detector 数据，并支持手动刷新。
- settings 当前通过 Win32 `show_window/hide_window` 控制显示，不是独立进程。
- 当前 `AppConfig` 已经覆盖一部分持久化字段，但还没有证明“未来所有 settings 项”都纳入统一 schema。
- 当前没有国际化层；页面文案直接写死在 Rust Win32 绘制代码里。
- 当前 tray 菜单、tooltip、状态名、来源方法名和 diagnostics 说明文案也都是英文硬编码或英文 key 直出。
- 当前没有统一的视觉 token 层；字号、间距、层级和按钮/卡片形态都只是 Win32 自绘 MVP 表现，不具备 Nothing-inspired 的结构化设计语言。

### Known Limitations

- 当前 settings UI 是 MVP 壳子，布局、层次、状态反馈、可扩展性都弱。
- 页面的交互逻辑和绘制逻辑强耦合在一个文件里。
- 后续如果继续加 Appearance / Monitoring / About 的真实交互，Win32 自绘复杂度会快速膨胀。
- 未来若直接在 Slint 组件里散落状态字段而不统一回写 `AppConfig`，会再次出现“部分可持久化、部分不可持久化”的漂移。
- 若不提前设计 i18n key / 文本资源结构，后续中英双语会变成大面积字符串返工。
- 若只把 Slint settings 做双语，而忽略 tray / tooltip / 状态标签，用户看到的最终产品仍然会是半套国际化。

### Dependencies / External Systems

- Rust `windows` crate
- Win32 消息循环
- Windows taskbar / tray runtime
- `%APPDATA%\CcTrafficLight\config.json`
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

---

## Proposed Solution

### High-Level Approach

采用“Win32 主体不动，Slint 只接管 settings window”的混合方案：

- `widget`：继续 Win32 + GDI
- `tray`：继续 Win32
- `settings`：改成 Slint UI
- `runtime/detector/config`：继续 Rust 原有模块

### Main Design Decisions

1. 不动 `taskbar.rs`。
2. 不动 `tray_icon.rs` 的宿主职责。
3. 用一个新的 `settings_runtime` / `settings_bridge` 层，把现有 `AppStatusSnapshot`、`AppConfig` 和 UI 事件桥接给 Slint。
4. 保留单进程，不把 settings 拆到子进程。
5. 先迁移 Settings UI，不顺手重构 detector / widget / tray。
6. 把“设置项是否可持久化”作为 schema gate：没有进入 `AppConfig` 的设置，不允许进入 Slint 正式界面。
7. 把 i18n 作为架构项前置，而不是等 UI 做完后再补翻译。
8. 把 i18n 范围定义为“所有用户可见字符串”，而不是仅 `settings.slint` 内部文本。
9. 把 `Nothing-inspired` 视觉系统限定在 settings 视觉层：单色基调、Typography-first、无阴影、结构即装饰；不强迫 widget/tray 同步做高风险视觉重写。

### Expected Behavior After Implementation

- 用户点击 tray 的 `Open Settings` 后，打开的是 Slint settings 窗口。
- Overview 显示总状态、widget mount、两张来源卡片。
- General 中的开关直接读写现有 `AppConfig` 与 autostart backend。
- Diagnostics 显示现有 detector 信息，并能触发手动刷新。
- About / Monitoring / Appearance 至少有结构化页面，不再是 Win32 文本占位感。
- 所有设置项在关闭软件后重新打开仍能恢复上次值。
- settings、tray、tooltip、状态标签和其他用户可见文案都可以按语言资源切换中英显示。

### How the Solution Fits the Existing Project

- 当前最稳定的 runtime 是 Win32 主循环；Slint 只吃状态、发命令。
- 现有统一状态结构已经足够为 Slint 页面提供 view model。
- 当前 settings 逻辑已经集中在单文件，便于整体替换，而不是全局分散。
- `Nothing-inspired` 的设计语言更适合信息密集的 settings / diagnostics，而不是立刻外溢到 taskbar attach 主路径。

### Why This Approach Is Preferred

- 解决了用户真正不满的点：settings 太丑。
- 不冒险重写 taskbar widget attach 路径。
- 保持 Rust 技术栈，不引入 Web 技术栈。
- 比继续纯 Win32 自绘更能持续支撑后续页面扩展。
- `Slint + Nothing-inspired` 能把视觉投入集中在最需要的 settings，而不是把整个项目拖进全量 UI 重写。

---

## Alternatives Considered

### Alternative 1: 继续纯 Win32，自绘重构 settings

- Advantages:
  - 不引入新依赖
  - 保持纯 Win32 边界
- Disadvantages:
  - 要自己继续维护控件、状态、布局、视觉系统
  - 做到“产品感”成本高
- Risks:
  - 很容易继续停留在“能用但丑”
- Reason Not Selected:
  - 用户反馈已经明确指出视觉不可接受，继续纯 Win32 的投入产出比差。

### Alternative 1b: 接入 Slint，但不定义明确视觉系统

- Advantages:
  - 技术接入更快
  - 先把 Win32 settings 换掉
- Disadvantages:
  - 最终容易退化成“换了框架但 UI 还是普通”
- Risks:
  - 解决不了用户对“太丑”的核心不满
- Reason Not Selected:
  - 既然已经决定迁移 settings，应该同时把视觉方向固定下来，而不是只做技术替换。

### Alternative 2: 全量改成 Slint

- Advantages:
  - UI 栈统一
  - settings 之外的界面也可一起现代化
- Disadvantages:
  - taskbar widget attach 路径需要重估
  - tray / native interop 风险明显上升
- Risks:
  - 破坏当前最稳定的 Win32 主路径
- Reason Not Selected:
  - 对当前项目来说，这属于过度迁移。

### Alternative 3: 用 Tauri / WebView 重写 settings

- Advantages:
  - UI 开发速度快
  - 样式最容易做漂亮
- Disadvantages:
  - 引入 Web 技术栈
  - 与当前纯 Rust/Win32 主体不一致
- Risks:
  - 依赖、打包、运行时体积复杂度上升
- Reason Not Selected:
  - 用户当前倾向 Rust 原生 UI，Slint 更贴合。

### Alternative 4: 用 WinUI 3 重写 settings

- Advantages:
  - Windows 原生观感最强
  - 现代控件成熟
- Disadvantages:
  - 工程集成复杂
  - 与 Rust / Win32 混合心智负担更高
- Risks:
  - 集成成本高于 Slint
- Reason Not Selected:
  - 在“现代感 / Rust 体验 / 集成复杂度”之间，Slint 更平衡。

---

## Implementation Plan

### Phase 1: Define the Hybrid Boundary

- Goal:
  - 明确 Win32 与 Slint 的职责边界，并同步固定持久化与 i18n 的基础约束。
- Files:
  - `docs/plan/slint-settings-migration-plan.md`
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/settings_window.rs`
  - `taskbar-widget/src/app_config.rs`
- Tasks:
  - 记录 `settings_window.rs` 当前承担的职责：显示、绘制、点击、配置写回、手动刷新。
  - 把这些职责拆成：
    - `settings presentation`
    - `settings state adapter`
    - `settings commands`
    - `settings persistence contract`
    - `settings localization contract`
  - 定义哪些能力继续由 Win32 提供：
    - tray 菜单
    - main loop
    - widget host
  - 定义哪些能力交给 Slint：
    - settings 页面布局
    - 视觉组件
    - 控件交互
- Expected Result:
  - 迁移边界固定，不会误把 taskbar/widget 主路径纳入重构范围。

### Phase 2: Introduce Slint Infrastructure

- Goal:
  - 在不替换现有 settings 的前提下，把 Slint 工程基础、i18n 资源骨架和 Nothing-inspired 视觉 token 骨架接进来。
- Files:
  - `taskbar-widget/Cargo.toml`
  - `taskbar-widget/build.rs`
  - `taskbar-widget/ui/settings.slint`
  - `taskbar-widget/ui/i18n/en.json`
  - `taskbar-widget/ui/i18n/zh-CN.json`
  - `taskbar-widget/src/settings_slint.rs`
- Tasks:
  - 添加 `slint` 依赖和构建脚本。
  - 明确字体依赖及其加载方式：
    - `Doto`
    - `Space Grotesk`
    - `Space Mono`
  - 新建 `settings.slint`，先实现静态壳子：
    - 左侧导航
    - Overview 卡片区
    - General 列表区
    - Diagnostics 面板区
  - 确定 Slint 侧文本不直接写死业务字符串，统一走语言资源 key。
  - 建立第一版视觉 token：
    - monochrome palette
    - display/body/label typography roles
    - spacing scale
    - card/button radius and border rules
  - 新建 `settings_slint.rs` 负责窗口创建与最小生命周期管理。
- Expected Result:
  - 能单独创建一个 Slint settings 窗口，并看到带双语资源骨架和基础 Nothing-inspired token 的静态结构。

### Phase 3: Bridge Runtime State into Slint

- Goal:
  - 让 Slint 窗口消费现有 `AppStatusSnapshot` 与 `AppConfig`，并把所有 settings 项纳入统一持久化模型。
- Files:
- `taskbar-widget/src/settings_slint.rs`
- `taskbar-widget/src/ui_state.rs`
- `taskbar-widget/src/app_config.rs`
- `taskbar-widget/src/tray_icon.rs`
- `taskbar-widget/ui/settings.slint`
- Tasks:
  - 定义 Slint 侧需要的 view model：
    - overall state
    - widget mount state
    - codex/claude source cards
    - general toggles
    - diagnostics rows
    - language selection / effective locale
    - localized tray labels / tooltip labels
    - localized state labels / method labels
  - 实现 Rust -> Slint 的状态推送。
  - 审计现有 settings 页所有字段，确认没有“只存在于 UI、没有进入 `AppConfig`”的配置项。
  - 如有缺项，先扩 `AppConfig` schema，再接 UI。
  - 定义统一的本地化访问层，避免 settings 和 tray 各自维护两套字符串来源。
  - 保持 `main.rs` 中现有 snapshot 更新路径，只把目标从 Win32 自绘窗口改成 Slint settings host。
- Expected Result:
  - Slint settings 能真实显示当前运行状态，且每个设置项都有明确的持久化落点；tray 与状态标签后续也可复用同一套 i18n 数据。

### Phase 4: Bridge UI Commands Back to Runtime

- Goal:
  - 让 Slint 控件真正驱动现有设置与命令，并验证“改完即持久化”。
- Files:
- `taskbar-widget/src/settings_slint.rs`
- `taskbar-widget/src/autostart.rs`
- `taskbar-widget/src/main.rs`
- `taskbar-widget/src/tray_icon.rs`
- `taskbar-widget/ui/settings.slint`
- Tasks:
  - 把 `General` 页的开关事件回调接到现有 config/autostart 逻辑。
  - 把 `Diagnostics -> Refresh Now` 接到现有主窗口刷新命令。
  - 如果引入语言切换入口，把它也持久化到 `AppConfig`，保证重启后仍保持上次语言。
  - 把 tray 菜单和 tooltip 改为从统一 i18n 层取文案，而不是继续写死英文。
  - 把用户可见状态名、来源方法名、诊断说明文本改为 label 映射，而不是直接显示内部 key。
  - 如需要，抽出一个 `settings_commands.rs`，避免把 Slint 回调直接散落在 `main.rs`。
- Expected Result:
  - Slint settings 不只是展示层，而是完整替代当前 Win32 settings 交互；tray / tooltip / 状态标签也接入统一 i18n，且设置修改后重启仍能恢复。

### Phase 5: Swap the Settings Host

- Goal:
  - 用 Slint settings 替换当前 `settings_window.rs` 的主入口。
- Files:
  - `taskbar-widget/src/main.rs`
  - `taskbar-widget/src/settings_window.rs`
  - `taskbar-widget/src/settings_slint.rs`
- Tasks:
  - 修改 `Open Settings` 流程，默认打开 Slint settings。
  - 保留旧 `settings_window.rs` 作为短期 fallback，直到 Slint 联调稳定。
  - 确定最终是否删除 Win32 自绘 settings，还是只保留最小 fallback 壳子。
- Expected Result:
  - 运行时默认使用 Slint settings，用户不再看到旧 GDI 自绘页面。

### Phase 6: Visual Polish and Documentation

- Goal:
  - 让新的 Slint settings 真正达到“比现在明显更像产品”的视觉水平，并完成文档、持久化和 i18n 收口。
- Files:
- `taskbar-widget/ui/settings.slint`
- `taskbar-widget/src/tray_icon.rs`
- `taskbar-widget/src/settings_window.rs`
- `docs/plan/gui-tray-v1-requirements.md`
- `docs/checklist/gui-tray-v1.md`
- `docs/handoff/*`
- Tasks:
  - 统一 Slint 主题 token：
    - spacing
    - type scale
    - border radius
    - neutral palette
    - semantic state colors
  - 明确 `Nothing-inspired` 落地规则：
    - 主屏只保留三层视觉层级
    - Primary / Secondary / Tertiary 文本角色固定
    - 单色为基底，状态色只用于数据事件
    - 无阴影，仅靠边框 / 间距 / 对比建立层次
    - 允许一个“hero moment”，例如 Overview 的大状态或关键数据
  - 明确国际化策略：
    - 默认语言是否跟随系统
    - 是否允许用户手动切换
    - 文案 key 命名规范
    - tray / tooltip / 状态标签是否与 settings 共用同一语言配置
  - 把需求文档从“纯 Win32 settings”更新为“Win32 主体 + Slint settings”。
  - 新建或改写 checklist，专门覆盖 Slint 迁移验证项。
- Expected Result:
  - 方案、实现、文档三者对齐，不再存在“代码已经混合栈，但文档仍写纯 Win32”的漂移，同时中英和持久化行为都可解释。

---

## Validation Strategy

### Build / Static Validation

- `cargo check`
- `cargo fmt -- --check`
- 如引入 `build.rs` / `slint-build` 后，确认生成步骤稳定通过

### Manual Integration Checks

1. 从 tray 打开 settings
   - 期望：打开 Slint 窗口，而不是旧 Win32 自绘窗口
2. Overview 状态展示
   - 期望：总状态、widget mount、Codex/Claude 卡片与实际运行一致
3. General 开关
   - 期望：可切换、可保存、重开后状态一致
4. Diagnostics
   - 期望：显示真实 detector 数据，手动刷新有效
5. 关闭软件后重新打开
   - 期望：所有设置项保持上次状态，不丢失、不回默认
6. 中英国际化
   - 期望：settings、tray 菜单、tooltip、状态标签、来源方法标签和关键诊断文案都支持中英显示，切换或重启后行为符合设计
7. Tray / Widget 不回归
   - 期望：不因 settings 切到 Slint 影响 tray、widget、attach/retry 路径
8. Nothing-inspired 视觉一致性
   - 期望：Settings 的排版层级、卡片/按钮规则、单色基底和状态色使用方式符合既定视觉方向，而不是普通默认工具 UI

### Failure Cases to Test

- Slint settings 未创建成功时，tray `Open Settings` 是否有 fallback
- settings 打开/关闭多次后，是否出现窗口句柄泄漏或状态不同步
- 手动刷新期间，Slint UI 是否会卡死或丢状态
- 切换语言后，tray 菜单、tooltip 和 settings 是否同步更新
- 重启后，语言是否保持为上次选择或正确回退到系统语言策略
- 字体不可用或加载失败时，fallback 是否仍保持层级和布局可用

### Recommended Test Additions

- 为 `AppStatusSnapshot -> Slint view model` 的转换加单元测试
- 为 `AppConfig -> General page state` 的转换加单元测试
- 为语言资源 key 完整性加校验
- 为“配置 round-trip”加测试：load -> mutate -> save -> reload
- 为状态枚举 / 来源方法到本地化 label 的映射加测试

---

## Risks and Mitigations

### Risk: Slint 与 Win32 主消息循环集成复杂

- Impact:
  - settings 可能打不开，或窗口生命周期混乱
- Likelihood:
  - 中
- Mitigation:
  - 先最小接入 Slint 静态窗口，再做状态桥接
- Fallback Plan:
  - 暂时保留 `settings_window.rs` 作为 fallback 入口

### Risk: 迁移时误伤 widget/tray 主路径

- Impact:
  - 破坏当前已验证通过的核心运行时能力
- Likelihood:
  - 中
- Mitigation:
  - 明确规定只替换 settings host，不改 `taskbar.rs`、`tray_icon.rs` 职责
- Fallback Plan:
  - 若联调发现回归，回退到旧 settings host，再单独修桥接层

### Risk: 文档与实现漂移

- Impact:
  - checklist / requirements 失真，后续执行混乱
- Likelihood:
  - 高
- Mitigation:
  - 把 Slint 迁移单独写成新的计划文档，并在实施时同步改 checklist / requirements
- Fallback Plan:
  - 若未立即迁移，则保持现有纯 Win32 文档不动

### Risk: 部分设置项没有进入统一持久化模型

- Impact:
  - 用户修改后重启丢配置，直接违背需求
- Likelihood:
  - 中高
- Mitigation:
  - 把 `AppConfig` 作为 settings schema gate，没有 schema 就不上 UI
- Fallback Plan:
  - 若某页暂时没有完成 schema 设计，则该页只做只读或占位，不暴露可写设置

### Risk: 国际化补得太晚导致 UI 文案返工

- Impact:
  - Slint 组件中散落硬编码文案，后续切双语成本高
- Likelihood:
  - 高
- Mitigation:
  - 在 Phase 2 就建立语言资源骨架和 key 命名规则
- Fallback Plan:
  - 若短期只先交付中文，也必须先走 key 化，而不是把中文硬编码到最终组件中

### Risk: 国际化只覆盖 settings，导致产品呈现半中半英

- Impact:
  - 用户实际感知仍然割裂，需求不算真正满足
- Likelihood:
  - 高
- Mitigation:
  - 在计划阶段就把 tray、tooltip、状态标签、方法标签纳入统一 i18n 范围
- Fallback Plan:
  - 若某些区域暂时无法切换，必须在 checklist 中显式列为未完成项，而不是默认忽略

### Risk: Slint 最终观感提升有限

- Impact:
  - 引入新栈但仍没解决“太丑”的核心抱怨
- Likelihood:
  - 中
- Mitigation:
  - 在方案中明确把视觉 token、卡片布局、页面结构纳入交付目标，而不是只做技术替换
- Fallback Plan:
  - 若最终 Slint 方案达不到预期，再评估 WinUI 3 或更强设计投入

### Risk: 只借用了 Nothing 的表面元素，没有形成真正的层级和结构感

- Impact:
  - UI 可能变成“黑白配色 + 一些圆角”，但仍然普通
- Likelihood:
  - 中高
- Mitigation:
  - 在计划中明确 Typography-first、三层视觉层级、无阴影、单色基底和状态色事件化这些硬规则
- Fallback Plan:
  - 若首版 Slint UI 仍然平庸，需要先做视觉审计，再继续功能扩张

---

## Open Questions

- 是否接受项目文档从“纯 Win32 V1”正式改成“Win32 主体 + Slint settings”？
- 是否希望保留旧 `settings_window.rs` 作为开发期开关 / fallback，还是在 Slint 稳定后直接删除？
- `Monitoring` / `Appearance` 页面是只要先做视觉完整占位，还是要趁迁移一起补齐真实交互？
- 国际化默认策略要不要“跟随系统语言 + 允许手动覆盖”？
- 是否接受把国际化范围明确扩到：
  - settings 全部文案
  - tray 菜单
  - tray tooltip
  - 状态标签
  - 来源方法标签
  - diagnostics / about 等运行时说明文本
- 是否先按 Nothing-inspired 的 dark mode 作为第一交付目标，再补 light mode，还是两者一起做？

这些问题不会阻止先写方案，但会影响具体迁移范围和文档措辞。

---

## Recommended Next Step

最佳下一步是：**先做 Phase 1，把 settings 职责从现有 `settings_window.rs` 中抽象成“状态输入 + 命令输出 + 展示层”边界，然后再接入 Slint 静态壳子。**

这样做能先锁定迁移面，避免一上来就在 Win32 / Slint 互操作细节里失控。
