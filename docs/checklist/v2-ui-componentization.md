# Checklist: V2 UI 组件化与样式统一管理

> 来源: [docs/plan/v2-ui-componentization-plan.md](../plan/v2-ui-componentization-plan.md)
> 视觉基准: [docs/design/v2-settings-ui.html](../design/v2-settings-ui.html)
> 组件化设计: [docs/design/v2-componentization-design.md](../design/v2-componentization-design.md)
> 基于 V2 HTML 视觉基准，将当前暗色玻璃质感 UI 替换为纸白色 Minimal Dot Matrix 风格，并完成组件化和样式体系整理。

---

## Checklist Objective

- **目标产出：** 5 个页面的 Tauri 设置窗口，视觉与 V2 HTML 100% 一致，代码从单文件 `App.tsx` 重构为目录化组件结构
- **范围：** 视觉替换 + 组件拆分 + 样式 token 化管理。不新增功能，不改业务语义，不改 `saveSettings → notifySettingsApplied` 链路
- **非目标：** 不引入 CSS-in-JS / Tailwind / 新 UI 库，不改为 Dashboard 风格

---

## Pre-Implementation Checks

- [ ] **确认目标文件** — 确认 `taskbar-settings-tauri/src/` 下的 `App.tsx`、`styles.css`、`main.tsx`、`types.ts`、`locales.ts`、`lib/tauri.ts` 的当前状态
- [ ] **确认源文件** — 确认 `docs/design/v2-settings-ui.html`（视觉基准）和 `docs/design/v2-componentization-design.md`（组件化设计文档）已保存到项目中
- [ ] **确认构建命令** — `cargo check -p taskbar-settings-tauri --offline` 可通过
- [ ] **确认备份策略** — 在开始前确认 git 工作区干净（`git status`），或知悉可以回退
- [ ] **确认 locales 同步** — 对比 `docs/design/v2-settings-ui.html` 中的每个页面 `.meta`、`h1`、`.sub` 文本与 `locales.ts` 字符串，若缺少则补充（Phase 3 之前完成）
- [ ] **确认 diagnostics 已移除** — 在 Phase 1b 中从 `VISIBLE_PAGE_IDS` 和渲染分支中移除 diagnostics
- [ ] **确认数据流保护规则** — 所有页面组件禁止直接 `import { ... } from "./lib/tauri"`，数据由 App 层通过 props 传入，保存通过回调上抛

---

## Implementation Checklist

### Phase 1a: CSS 基础设施（只建不换）

- [ ] **1a.1 创建 `src/styles/tokens.css`** — 从 `docs/design/v2-settings-ui.html` 的 `:root` 提取视觉 token（参见组件化设计文档第 4 节）：`--bg`, `--paper`, `--ink`, `--muted`, `--line`, `--green`, `--yellow`, `--red`, `--mono`, 以及间距/字号/圆角/z-index 扩展 token
- [ ] **1a.2 创建 `src/styles/reset.css`** — `* { box-sizing }`, `body { margin: 0 }`, `button { font: inherit }`, `:focus-visible` 焦点环
- [ ] **1a.3 创建 `src/styles/base.css`** — body 背景 `var(--bg)`、字体 `var(--ui)`、点阵 `body:before` (`pointer-events: none`)、滚动条、selection
- [ ] **1a.4 更新 `main.tsx`** — 导入新样式文件（次序：tokens.css → reset.css → base.css），旧 `styles.css` 排在最后作为过渡
- [ ] **1a.5 验证 Phase 1a** — `cargo check` 通过，**页面外观与改造前完全一致**（旧 CSS 规则优先级更高，尚无视觉变化）

> ⚠️ **关键约束：** 此阶段只建 CSS 基础设施。旧 `styles.css` 仍完全生效，页面仍是暗色主题。

### Phase 1b: 移除 Diagnostics 页面（纯逻辑变更）

- [ ] **1b.1 从 `VISIBLE_PAGE_IDS` 移除** — `"diagnostics"` 从数组删除
- [ ] **1b.2 移除渲染分支** — 删除 App.tsx 中 `page === "diagnostics"` 的条件分支
- [ ] **1b.3 验证 Phase 1b** — `cargo check` 通过，导航剩 5 项，其余 5 页功能正常

> 完成每个任务时自动生成 `docs/reflections/task-1a.N-<timestamp>.md` 和 `docs/reflections/task-1b.N-<timestamp>.md`

### Phase 2: App Shell 与导航栏

- [ ] **2.1 创建 `src/styles/shell.css`** — `.app` 两栏 grid（240px + 1fr），main content 区域样式
- [ ] **2.2 创建 `src/styles/navigation.css`** — sidebar 240px、nav button 样式、active 态黑底白字圆角 12px
- [ ] **2.3 创建 `BrandMark.tsx`** — "◆ CC TRAFFIC LIGHT" 品牌文字组件，mono 字体，800 weight
- [ ] **2.4 创建 `NavItem.tsx`** — 单个导航按钮，接受 `id`, `index`, `label`, `active`, `onClick` props
- [ ] **2.5 创建 `SidebarNav.tsx`** — 接受 `items` 数组和 `activeId`/`onChange`，映射 NavItem
- [ ] **2.6 创建 `SettingsShell.tsx`** — 包含 sidebar + main content 的两栏布局容器
- [ ] **2.7 修改 App.tsx** — 使用 SettingsShell、SidebarNav、BrandMark 替换内联 shell 结构
- [ ] **2.8 验证 Phase 2** — `cargo check` 通过，导航 5 项可切换，active 状态正确，宽度 240px

### Phase 3: 页面框架与 Header

- [ ] **3.1 创建 `src/styles/components.css`** — 基础组件样式（card、pill 等共用的初始样式）
- [ ] **3.2 创建 `MetaLabel.tsx`** — 小型英文系统标签（`font: 700 11px var(--mono); letter-spacing: .16em; color: #888`）
- [ ] **3.3 创建 `SubText.tsx`** — 辅助说明文字（`color: #777`）
- [ ] **3.4 创建 `PageHeader.tsx`** — 接受 `meta`, `title`, `subtitle` 三个 props，渲染 `.meta + h1 + .sub` 结构
- [ ] **3.5 创建 `PageFrame.tsx`** — 接受 `active` 和 `pageId` props，控制 `.page.active` 类
- [ ] **3.6 修改 App.tsx** — 5 个页面使用 PageFrame + PageHeader 替换重复的内联 header
- [ ] **3.7 验证 Phase 3** — `cargo check` 通过，5 个页面 title/meta/subtitle 与 V2 HTML 一致

### Phase 4: 基础视觉组件

- [ ] **4.1 创建 `BaseCard.tsx`** — `.card` 容器（`border: 1px solid var(--line); background: var(--paper); border-radius: 12px`），支持 `padding` 和 `className` 配置
- [ ] **4.2 创建 `ValuePill.tsx`** — 值标签（`border: 1px solid var(--line); border-radius: 999px; padding: 8px 12px; font: 700 11px var(--mono)`）
- [ ] **4.3 创建 `StatusBadge.tsx`** — 状态胶囊，接受 `tone` prop 映射红黄绿颜色
- [ ] **4.4 创建 `StatusDot.tsx`** — 小状态圆点（12px），用于 monitoring 页
- [ ] **4.5 创建 `InfoRow.tsx`** — 信息行（两栏：label + value）
- [ ] **4.6 创建 `InlineKey.tsx`** — 设置 key 文本（`font: 700 11px var(--mono); color: #999`）
- [ ] **4.7 扩展 `components.css`** — 上述组件的 CSS 类
- [ ] **4.8 修改 App.tsx** — 将现有内联组件（Section、SettingRow 等）替换为新组件引用
- [ ] **4.9 验证 Phase 4** — `cargo check` 通过，card 边框 `#deded8`、pill 圆角 999px、paper 背景 `#fffefa` 与 HTML 一致

### Phase 5: OverviewPage

- [ ] **5.1 创建 `SignalLamp.tsx`** — 58px 圆灯，三色（红/黄/绿）+ glow（仅绿色启用 `box-shadow`）
- [ ] **5.2 创建 `SignalStack.tsx`** — 纵向排列 3 个 lamp，深色灯壳背景（`background: #111; border-radius: 50px`）
- [ ] **5.3 创建 `SignalStateText.tsx`** — 大字状态展示（`font-size: 90px; font-weight: 950; letter-spacing: -.12em`）
- [ ] **5.4 创建 `StatusInfoPanel.tsx`** — 右侧 info 列（fake_mode 标签、last refresh、error）
- [ ] **5.5 创建 `SignalDesk.tsx`** — 三栏 grid（lamp + state + info），组装 SignalStack + SignalStateText + StatusInfoPanel
- [ ] **5.6 创建 `AgentStatusCard.tsx`** — CODEX/CLAUDE CODE 卡片（h2 标题 + 大字状态 + pill 标签）
- [ ] **5.7 创建 `OverviewPage.tsx`** — 组装 SignalDesk + AgentStatusCard x2，接入 `snapshot` 数据
- [ ] **5.8 创建 `src/styles/pages.css`** — overview 部分的组合布局样式
- [ ] **5.9 修改 App.tsx** — overview 页面路由指向 OverviewPage 组件
- [ ] **5.10 验证 Phase 5** — `cargo check` 通过，Overview 视觉与 V2 HTML 一致（信号台三栏、红绿灯、大字状态、Agent 卡片）

### Phase 6: GeneralPage

- [ ] **6.1 创建 `ToggleSwitch.tsx`** — V2 开关视觉（74px × 38px 圆角矩形，圆形滑块，on 态绿色 `#e8f7ed` + `#34c759`）
- [ ] **6.2 创建 `ToggleMatrixCard.tsx`** — 卡片容器 + h2 标题 + InlineKey + ToggleSwitch，支持 pending/disabled
- [ ] **6.3 创建 `ToggleMatrix.tsx`** — 2 列 grid 容器
- [ ] **6.4 创建 `GeneralPage.tsx`** — 接入 `settings.general.*` 数据，4 张卡片绑定 toggle 事件
- [ ] **6.5 处理语言卡片** — 语言项用 ValuePill 展示，保持 V1 三态切换逻辑（follow_system → zh-CN → en）
- [ ] **6.6 扩展 `pages.css`** — general 部分样式
- [ ] **6.7 修改 App.tsx** — general 页面路由指向 GeneralPage
- [ ] **6.8 验证 Phase 6** — `cargo check` 通过，General 页 2x2 网格 4 卡片，switch 视觉与 HTML 一致

### Phase 7: MonitoringPage

- [ ] **7.1 创建 `SourceNodeCard.tsx`** — 大号 ON/OFF 文字（58px, font-weight: 950）+ StatusDot + 参与判断 pill
- [ ] **7.2 创建 `SourceNodeGrid.tsx`** — 2 列 grid 容器
- [ ] **7.3 创建 `MonitoringPage.tsx`** — 接入 `settings.monitoring.*` 和 `snapshot.sources.*`
- [ ] **7.4 扩展 `pages.css`** — monitoring 部分样式
- [ ] **7.5 修改 App.tsx** — monitoring 页面路由指向 MonitoringPage
- [ ] **7.6 验证 Phase 7** — `cargo check` 通过，2 张 node card，大号 ON/OFF，绿点 pill

### Phase 8: AppearancePage

- [ ] **8.1 创建 `DotObject.tsx`** — 130px 大圆 + 对应颜色 + glow（仅绿色）+ 色值 pill。**必须保留交互式 color picker**——点击色圆或色值 pill 时弹出原生 `<input type="color">`，选色后调用 `onChange`。不能做成纯静态展示
- [ ] **8.2 创建 `DotObjectGrid.tsx`** — 3 列 grid 容器
- [ ] **8.3 创建 `BrightnessControl.tsx`** — V2 视觉样式的**可拖动 slider**。使用 `<input type="range">` 驱动，用 CSS 隐藏原生外观并模拟 V2 HTML 的 slider 条样式。不能做成纯 CSS 伪元素静态展示
- [ ] **8.4 创建 `ActionButton.tsx`** — V2 风格的 action button（边框 + paper 背景）
- [ ] **8.5 创建 `AppearancePage.tsx`** — 接入 `settings.widget_visual.palette.*`，DotObject x3 + BrightnessControl + 重置按钮
- [ ] **8.6 扩展 `pages.css`** — appearance 部分样式
- [ ] **8.7 扩展 `components.css`** — slider 样式
- [ ] **8.8 修改 App.tsx** — appearance 页面路由指向 AppearancePage
- [ ] **8.9 验证 Phase 8** — `cargo check` 通过，3 个大色彩圆 + 滑条 + 重置按钮，色值来自 V1 数据不硬编码

### Phase 9: AboutPage

- [ ] **9.1 创建 `AboutSpecRow.tsx`** — 两栏 grid 规格行（label + meta / value），底部边框分割
- [ ] **9.2 创建 `VersionCard.tsx`** — 产品名行 + 版本号行，边框圆角卡片
- [ ] **9.3 创建 `AboutNote.tsx`** — 底部说明 note（`border: 1px solid var(--line); background: #fafaf7; color: #777`）
- [ ] **9.4 创建 `AboutPage.tsx`** — 接入 `bootstrap.about.product_name` 和 `bootstrap.about.version`
- [ ] **9.5 扩展 `pages.css`** — about 部分样式
- [ ] **9.6 修改 App.tsx** — about 页面路由指向 AboutPage
- [ ] **9.7 验证 Phase 9** — `cargo check` 通过，关于页显示产品名 + 大版本号（100px, font-weight 950）

### Phase 10: 样式收尾 & 旧文件清理

- [ ] **10.1 搜索并消除残留 inline style** — 检查 App.tsx 和所有新组件中是否有 `style={{}}` 残留
- [ ] **10.2 搜索硬编码颜色值** — 确认所有颜色值都已通过 token 引用 (`var(--xxx)`)
- [ ] **10.3 删除旧的 `src/styles.css`** — 确认所有样式已迁移到 `src/styles/` 目录化文件
- [ ] **10.4 更新 `main.tsx`** — 移除 `import "./styles.css"`，确认只导入新的拆分的样式文件
- [ ] **10.5 最终编译验证** — `cargo check -p taskbar-settings-tauri --offline` 和 `cargo build -p taskbar-settings-tauri --offline` 均通过
- [ ] **10.6 最终 App.tsx 瘦身确认** — App.tsx 作为轻量编排入口，所有 UI 组件已提取到独立文件

---

## Validation Checklist

- [ ] **业务语义防护规则** — 确认所有页面组件未直接 `import { ... } from "./lib/tauri"`。数据由 App 层通过 props 传入，保存通过回调上抛
- [ ] **Phase 1a 验证** — `cargo check` 通过，页面视觉与改造前完全一致
- [ ] **Phase 1b 验证** — 导航 5 项，其余 5 页功能正常（纯逻辑变更）
- [ ] **Phase 2 验证** — 导航 5 项，active 黑底白字，宽度 240px，切换正常
- [ ] **Phase 3 验证** — 5 页面 header 区 meta/title/subtitle 与 HTML 一致；locales 字符串已同步
- [ ] **Phase 4 验证** — card 边框 `#deded8`，pill 圆角 999px，paper 背景 `#fffefa`
- [ ] **Phase 5 验证** — Overview: 信号台三栏、3 盏灯、90px 大字、Agent 卡片；**确认 fake_mode 和 last_error_summary 可见**
- [ ] **Phase 6 验证** — General: 2x2 网格、switch 滑块、语言 pill
- [ ] **Phase 7 验证** — Monitoring: 2 张 node card、大号 ON/OFF、绿点 pill
- [ ] **Phase 8 验证** — Appearance: 3 个大色彩圆、色值 pill（可交互编辑）、滑条（可拖动）、重置按钮
- [ ] **Phase 9 验证** — About: 产品名 + 大版本号（100px）
- [ ] **Phase 10 验证** — 无 inline style 残留、无硬编码颜色、旧 styles.css 已删除
- [ ] **业务逻辑回归** — bootstrap 加载正常、5s 轮询正常、toggle 开关正常、颜色选择正常、语言切换正常、重置 palette 正常
- [ ] **最终构建** — `cargo build -p taskbar-settings-tauri --offline` 通过

---

## Documentation Checklist

- [ ] **更新 `docs/plan/v2-ui-componentization-plan.md`** — 如有实施过程中的偏离或新增决策
- [ ] **更新 `docs/handoff/`** — 如本次实施跨越多个会话，记录当前完成阶段
- [ ] **组件目录 README** — 可选：在 `src/components/` 添加简短 README 说明组件设计原则

---

## Cleanup Checklist

- [ ] **删除 `src/styles.css`** — 所有样式已迁移到目录化文件
- [ ] **删除 diagnostics 相关代码** — `VISIBLE_PAGE_IDS`、渲染分支、导航项
- [ ] **确认无未使用的 CSS 类** — 旧 styles.css 中的类不再被引用
- [ ] **确认无死 import** — 所有组件的 import 路径正确
- [ ] **确认 `pointer-events: none`** — 点阵背景伪元素保留此属性
- [ ] **确认 naming consistency** — 组件命名与计划文档一致

---

## Completion Criteria

- [ ] 5 个页面视觉与 V2 HTML 100% 一致
- [ ] 所有业务逻辑正常运行（bootstrap、轮询、设置保存）
- [ ] `cargo build -p taskbar-settings-tauri --offline` 编译通过
- [ ] 代码从单文件 `App.tsx` 重构为 `src/components/` + `src/pages/` 目录结构
- [ ] 样式从单一 `styles.css` 拆分为 `src/styles/` 下 6 个主题化文件
- [ ] 所有颜色值通过 CSS token 引用，无硬编码色值
- [ ] `saveSettings → notifySettingsApplied` 链路不变
- [ ] `appliedKeys` 独立设置项机制不变
- [ ] diagnostics 页面已移除，导航剩 5 项
- [ ] **数据流保护：** 无页面组件直接 import `lib/tauri.ts`，所有 IPC 调用集中在 App 层
- [ ] **locale 同步：** 页面标题、meta 标签、sub 文案与 V2 HTML 一致
- [ ] **fake_mode 保留：** Overview 状态信息区展示 fake_mode 标识
- [ ] **last_error_summary 保留：** Overview info 区域展示错误摘要（空时显示 "无"）

---

## Reflection Generation Note

每个 checklist 任务完成后，在 `docs/reflections/task-<phase>.<task>-<timestamp>.md` 自动生成反思文档：

```markdown
# Task: <task name>

- **Phase:** <phase number>
- **Encountered Problem:** <问题描述>
- **Thought Process:** <分析过程>
- **Options Considered:** <考虑的方案>
- **Chosen Solution:** <最终决定>
- **Rationale:** <选择理由>
- **Files Changed:** <修改的文件>
```

> 反思在任务完成后立即生成，不等待用户提示。时间戳格式：`YYYYMMDD-HHmmss`。
