# Plan: V2 UI 组件化与样式统一管理

## 一、Objective

**问题：** 当前 `taskbar-settings-tauri` 的 UI 是暗色沉浸主题（D 方案：Supabase × ElevenLabs DNA，带玻璃质感、蓝紫渐变、呼吸发光），与已确认的 V2 设计方向（`02 Minimal Dot Matrix` + `04 Signal First Focus`）不一致。所有 UI 代码集中在单个 920 行的 `App.tsx` 中，样式集中在单个 25KB 的 `styles.css` 中，没有组件化和样式体系管理。

**目标：** 将当前 UI 替换为 V2 HTML 的样式（纸白色背景 + 点阵纹理 + 强边界卡片 + 红黄绿信号灯），实现 100% 视觉还原。同时完成组件拆解和设计 token 统一管理。

**范围内：**
- 替换所有 UI 视觉样式（`styles.css` → 拆分后的 token + 组件 CSS）
- 组件化拆分 `App.tsx` → 目录化组件结构（shell、navigation、layout、primitives、pages）
- 保留所有现有业务逻辑（数据流、IPC 调用、轮询、`saveSettings → notifySettingsApplied` 链路）
- 保留正常功能的页面数量（移除 diagnostics，保留 overview/general/monitoring/appearance/about）

**范围外：**
- 不新增功能
- 不改变页面方案和业务语义
- 不改变 `saveSettings → notifySettingsApplied` 链路
- 不新增 UI 库
- 不改为普通后台 Dashboard
- 不使用玻璃拟态、蓝紫渐变、赛博终端风

---

## 二、Background and Context

### 2.1 现有行为

当前应用是 Tauri 设置窗口，通过 `bootstrapWindow()` 获取初始数据，每 5 秒通过 `getSnapshot() + getSettings()` 轮询更新。6 个页面（overview、general、monitoring、appearance、diagnostics、about）通过 `page` state 切换渲染。

整体 UI 是暗色玻璃质感风格：
- 深色背景 `#0d0d0f` 加渐变底光
- 低透明度玻璃卡片（`rgba(255, 255, 255, 0.035)`）
- 蓝紫渐变文字标题
- 呼吸发光状态圆点
- 蓝色 accent 导航 active 状态

### 2.2 设计源文件

本次 UI 替换的视觉基准和组件化设计依据以下两个稳定文件：

| 文件 | 路径 | 作用 |
|---|---|---|
| V2 视觉 HTML | `docs/design/v2-settings-ui.html` | 100% 视觉基准，所有颜色、尺寸、布局、字体参数的最终来源 |
| 组件化设计文档 | `docs/design/v2-componentization-design.md` | 组件拆分方案、设计系统原则、数据接入指引、落地步骤 |

### 2.3 V2 目标风格

V2 确认的视觉方向是 **Minimal Dot Matrix + Signal First Focus**：
- 温和纸面色背景 `#f5f5f2`，卡片 `#fffefa`
- 全局低透明点阵空气纹理（`opacity: 0.04`）
- 强边界线而非阴影（`#deded8`）
- 大字号状态字（90px font-weight 950）
- 红黄绿信号灯 + Dot Object 展示
- Mono 字体用于系统标签，UI sans 用于主标题

### 2.3 重要文件

| 文件 | 作用 |
|---|---|
| `src/App.tsx` | 全部 UI 组件 + 逻辑（~920行） |
| `src/main.tsx` | 入口，渲染 App + 导入 styles.css |
| `src/styles.css` | 全部样式（~25KB，600+行） |
| `src/lib/tauri.ts` | Tauri IPC 封装（不修改） |
| `src/types.ts` | TypeScript 类型定义（不修改） |
| `src/locales.ts` | 国际化字符串（可能需扩展） |

### 2.4 约束

- 项目使用 **plain global CSS**，没有 CSS Modules / CSS-in-JS / Tailwind
- 不要引入新的 CSS 方案——保持 global CSS，但拆分到目录化文件
- V2 HTML 中的点阵背景必须保留 `pointer-events: none`
- 5 个页面的 header 结构完全相同（`.meta + h1 + .sub`），必须复用

---

## 三、Current State Analysis

### 3.1 现有 UI 架构

```
src/
  main.tsx          # 入口
  App.tsx           # 全部 UI (~920行)
  styles.css        # 全部样式 (~600行)
  locales.ts       # 国际化字符串
  types.ts          # 类型定义
  lib/
    tauri.ts        # Tauri IPC 封装 (6个函数)
```

### 3.2 关键已确认事实

1. **App.tsx 中的组件**（当前以内联函数形式存在）：
   - `Section` — 带标题的分组容器
   - `SettingRow` — 设置行（可点击交互）
   - `InfoRow` — 信息显示行
   - `ColorSettingRow` — 颜色选择行
   - `RangeSettingRow` — 范围滑条行
   - `TogglePill` — 开关 pill
   - `ValuePill` — 值标签 pill
   - `StatusBadge` — 状态 badge
   - `StatusDot` — 状态圆点

2. **当前样式 token**（在 `styles.css:root` 中）：
   - 暗色体系 `--bg-app: #0d0d0f`
   - 蓝紫 accent `--accent-gradient: linear-gradient(135deg, #60a5fa, #a78bfa)`
   - 语义色 `--ok: #22c55e`, `--warn: #eab308`, `--error: #ef4444`

3. **当前数据模型**（在 `types.ts` 中）：
   - `SettingsPageId` — `"overview" | "general" | "monitoring" | "appearance" | "diagnostics" | "about"`
   - `AppConfig` — 含 `general`, `monitoring`, `appearance`, `widget_visual.palette`, `diagnostics`
   - `WidgetPaletteConfig` — `{ green, yellow, red, inactive_brightness_percent }`
   - `SettingsBootstrapDto` — 含 `fake_mode`, `about`, `snapshot`, `settings`

### 3.3 已知差异（V2 HTML vs 当前代码）

| 维度 | 当前 (V1) | V2 目标 |
|---|---|---|
| 主题 | 暗色玻璃质感 | 纸白色 Minimal Dot Matrix |
| 背景 | `#0d0d0f` + 渐变底光 | `#f5f5f2` + 点阵纹理 |
| 布局 | 220px sidebar + 1fr | 240px sidebar + 1fr |
| Nav 样式 | 分割线导航，蓝紫 active | 纯黑圆角按钮 active |
| 页面数 | 6 页（含 diagnostics）→ **已确认移除** | 5 页 |
| 品牌 | ◆ 小字 + 渐变 | ◆ CC TRAFFIC LIGHT |
| 字体 | Inter + JetBrains Mono | SF Pro Display + ui-monospace |
| 卡片 | 低透明玻璃 + 蓝紫光晕 | 纸白 `#fffefa` + `#deded8` 边框 |
| 开关 | `TogglePill` 圆角 pill | `.switch` 宽 switch 滑块 |
| 信号灯 | 呼吸发光圆点 | 大尺寸物理红绿灯（58px） |
| 状态字 | 较小的 `<p>` | 90px 超大字体 |

---

## 四、Proposed Solution

### 4.1 总体策略

采用 **"先 token + reset → 再壳层 → 再基础组件 → 再页面组件 → 最后接数据"** 的分层策略，确保视觉在每一步都保持可验证。

每个阶段产生可视结果，不出现"改了三天什么都看不到"的情况。

### 4.2 设计决策

1. **CSS 拆分：** 保持 global CSS 方案，将当前单一 `styles.css` 拆分为目录化文件：
   ```
   src/styles/
     reset.css        # box-sizing, body margin, button reset
     tokens.css       # 设计 token (颜色、字号、间距、圆角)
     base.css         # 全局字体、点阵背景、滚动条
     shell.css        # App shell + sidebar + main 布局
     navigation.css   # sidebar nav
     components.css   # 通用可复用组件 (card, pill, badge, lamp, switch)
     pages.css        # 5 个页面的组合布局
   ```

2. **组件目录结构：** 新建 `src/components/` 和 `src/pages/` 目录：
   ```
   src/components/
     shell/           # SettingsShell
     navigation/      # SidebarNav, BrandMark, NavItem
     layout/          # PageFrame, PageHeader
     primitives/      # BaseCard, MetaLabel, ValuePill, StatusBadge, StatusDot, InfoRow, InlineKey, SubText
     signal/          # SignalDesk, SignalStack, SignalLamp, SignalStateText, StatusInfoPanel
     status/          # AgentStatusCard
     toggle/          # ToggleMatrix, ToggleMatrixCard, ToggleSwitch
     source/          # SourceNodeGrid, SourceNodeCard
     appearance/      # DotObjectGrid, DotObject, BrightnessControl
     about/           # VersionCard, AboutSpecRow, AboutNote
     shared/          # ActionButton
   src/pages/
     OverviewPage.tsx
     GeneralPage.tsx
     MonitoringPage.tsx
     AppearancePage.tsx
     AboutPage.tsx
   ```

3. **App.tsx 重构策略：** 不一次性重写，而是逐步提取——先提取组件到新文件（保持接口兼容），最后 App.tsx 变成轻量编排入口。

4. **数据绑定策略（关键约束）：** 页面组件只通过 props 接收数据，**禁止页面组件直接 import `lib/tauri.ts`**（即禁止在页面组件中调用 `getSnapshot()`、`saveSettings()` 等 IPC 函数）。所有数据流保持集中在 App 层级，页面组件通过回调与 App 通信。这样确保 `saveSettings → notifySettingsApplied` 链路不被打乱。

5. **locales 同步规则：** V2 HTML 中的页面标题（"总览"、"通用"、"监听"、"外观"、"关于"）、meta 标签（"STATUS SUMMARY"、"SYSTEM BEHAVIOR"等）、sub 文案（"01 Center Signal Desk"等）必须与 `locales.ts` 中的字符串一致。如果 locales 缺少对应文案（如页面 sub 文案、meta 英文标签），需在 Phase 3 之前补充。对比基准为 `docs/design/v2-settings-ui.html` 中每个页面的 `.meta`、`h1`、`.sub` 文本。

6. **业务视觉要素必须保留：** 以下 V1 业务语义视觉要素**不能因 UI 替换而丢失**：
   - `fake_mode` 在 Overview 页的状态信息区明确展示（影响用户对数据可信度的判断）
   - `last_error_summary` 在 Overview 页的 info 区域展示，empty/null 时显示"无"
   - `pending` 状态在 switch / dot object / action button 上体现（禁用交互）

### 4.3 预期最终行为

- 视觉与 V2 HTML 100% 一致（点阵背景、纸白卡片、信号灯台、Toggle Matrix、Dot Object）
- 所有业务功能不变：bootstrap、5s 轮询、saveSettings、notifySettingsApplied
- 5 个页面正常工作
- 代码结构从单文件 → 目录化组件

---

## 五、Alternatives Considered

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| **A: 直接替换 styles.css 内容** | 最快，改动最小 | 不解决组件化问题，App.tsx 仍臃肿，CSS 仍全在一处 | ❌ 不满足组件化诉求 |
| **B: 完整重写 App.tsx** | 一次性完成 | 风险太大，中间状态不可运行，难以逐阶段验证 | ❌ |
| **C: CSS Modules 或 Tailwind** | 现代化 CSS 方案 | 需引入新依赖，修改构建配置，与项目约束冲突 | ❌ 违反"不引入新 UI 库" |
| **✅ D: 分阶段 global CSS 拆分 + 逐步组件提取** | 每步可验证，持续可用，不新增依赖 | 周期略长，但可控 | ✅ **推荐** |

---

## 六、Implementation Plan

### Phase 1a: CSS 基础设施 — 只建不换

- **Goal:** 创建 token/reset/base CSS 文件并导入，但不改变运行时视觉
- **Files:**
  - `src/styles/tokens.css`（新建）
  - `src/styles/reset.css`（新建）
  - `src/styles/base.css`（新建）
  - `src/main.tsx`（追加导入新文件，旧 `styles.css` 排在最后以保持最高优先级）
- **Tasks:**
  1. 从 `docs/design/v2-settings-ui.html` 的 `:root` 提取 token：`--bg`, `--paper`, `--ink`, `--muted`, `--line`, `--green`, `--yellow`, `--red`, `--mono`
  2. 新增 token 扩展：间距 scale、字号 scale、圆角、z-index（参照组件化设计文档第 4 节）
  3. 创建 `reset.css`：`* { box-sizing }`, `body { margin: 0 }`, `button { font: inherit }`, `:focus-visible`
  4. 创建 `base.css`：body 背景 `var(--bg)`、字体 `var(--ui)`、点阵 `body:before` (`pointer-events: none`)、滚动条、selection
  5. 更新 `main.tsx`：import 顺序为 tokens.css → reset.css → base.css → styles.css（旧文件最后）
- **⚠️ 约束：** 新 CSS 导入后**不应产生视觉变化**。旧 `styles.css` 中 `body { background: var(--bg-app) }` 等规则优先级高于新 `base.css`。这是纯基础设施搭建阶段。
- **Expected Result:** `cargo check` 通过，页面外观与改造前完全一致（仍是暗色主题），所有新 token 变量在 CSS 中可用但尚未激活

### Phase 1b: 移除 Diagnostics 页面

- **Goal:** 从导航和路由中移除 diagnostics 页面（纯逻辑变更，无视觉影响）
- **Files:**
  - `src/App.tsx`（移除 diagnostics 页面引用）
  - `src/types.ts`（可选：从 `SettingsPageId` 移除 `diagnostics`）
- **Tasks:**
  1. 从 `VISIBLE_PAGE_IDS` 移除 `"diagnostics"`
  2. 移除 App.tsx 中 diagnostics 页面的渲染分支
  3. 确认导航只显示 5 项
- **Expected Result:** diagnostics 页面和导航项消失，其余 5 个页面工作正常

### 关于旧 styles.css 的过渡策略：

旧 `styles.css` 在整个 Phase 1-4 期间保留但逐步压缩：
- Phase 1-2：完整保留（确保视觉不跳变）
- Phase 3-4：逐渐删除被新 CSS 覆盖的规则
- Phase 5-9：只保留旧 CSS 中未被页面组件替换的零散样式
- Phase 10：完全删除

### Phase 2: App Shell 与导航栏 — "搭房子"

- **Goal:** 重构 `app-frame` 和 sidebar，匹配 V2 布局
- **Files:**
  - `src/styles/shell.css`（新建）
  - `src/styles/navigation.css`（新建）
  - `src/components/shell/SettingsShell.tsx`（新建）
  - `src/components/navigation/SidebarNav.tsx`（新建）
  - `src/components/navigation/NavItem.tsx`（新建）
  - `src/components/navigation/BrandMark.tsx`（新建）
  - `src/App.tsx`（修改 — 替换内联 shell 部分）
- **Tasks:**
  1. 创建 `SettingsShell` 组件（两栏 grid：sidebar + main）
  2. 创建 `BrandMark` 组件（"◆ CC TRAFFIC LIGHT"）
  3. 创建 `NavItem` 组件（index + label，active 态黑底白字）
  4. 创建 `SidebarNav` 组件（items 数组映射）
  5. 编写 shell 和 nav CSS（`var(--line)` 边框，240px 固定宽度）
  6. 修改 App.tsx 使用新组件替换内联 shell 结构
- **Expected Result:** 左侧导航 240px，品牌文字正常，5 个 nav item 可切换，active 状态为新风格

### Phase 3: 页面框架与 Header — "标准化页面"

- **Goal:** 抽出 `PageFrame` 和 `PageHeader` 消除 5 个页面的重复 header
- **Files:**
  - `src/styles/components.css`（新建 — 开头部分）
  - `src/components/layout/PageFrame.tsx`（新建）
  - `src/components/layout/PageHeader.tsx`（新建）
  - `src/components/primitives/MetaLabel.tsx`（新建）
  - `src/components/primitives/SubText.tsx`（新建）
  - `src/App.tsx`（修改）
- **Tasks:**
  1. 创建 `MetaLabel` 组件（`font: 700 11px var(--mono); letter-spacing: .16em`）
  2. 创建 `SubText` 组件（`color: #777`）
  3. 创建 `PageHeader` 组件（`<header>` > `.meta + h1 + .sub` 结构）
  4. 创建 `PageFrame` 组件（显示/隐藏控制、`page.active` 类）
  5. 修改 App.tsx，5 个页面用 `PageFrame` + `PageHeader` 替换内联 header
- **Expected Result:** 5 个页面的标题区视觉与 V2 HTML 一致，meta 标签与大标题对应

### Phase 4: 基础视觉组件 — "通用零件"

- **Goal:** 抽出所有跨页面复用的基础组件
- **Files:**
  - `src/components/primitives/BaseCard.tsx`（新建）
  - `src/components/primitives/ValuePill.tsx`（新建）
  - `src/components/primitives/StatusBadge.tsx`（新建）
  - `src/components/primitives/StatusDot.tsx`（新建）
  - `src/components/primitives/InfoRow.tsx`（新建）
  - `src/components/primitives/InlineKey.tsx`（新建）
  - `src/styles/components.css`（扩展）
  - `src/App.tsx`（修改 — 引用新组件）
- **Tasks:**
  1. `BaseCard`：`border: 1px solid var(--line); background: var(--paper)`，可配置 padding
  2. `ValuePill`：`border: 1px solid var(--line); border-radius: 999px; padding: 8px 12px; font: 700 11px var(--mono)`
  3. `StatusBadge`：tone 参数映射红黄绿色
  4. `StatusDot`：小圆点（用于 Monitoring）
  5. 修改 App.tsx 将现有内联组件替换为新组件
- **Expected Result:** 所有卡片的 border + paper 背景统一，pill 样式匹配 V2

### Phase 5: OverviewPage — "总览信号台"

- **Goal:** 重构 Overview 页为 SignalDesk + SignalLamp + AgentStatusCard
- **Files:**
  - `src/pages/OverviewPage.tsx`（新建）
  - `src/components/signal/SignalDesk.tsx`（新建）
  - `src/components/signal/SignalStack.tsx`（新建）
  - `src/components/signal/SignalLamp.tsx`（新建）
  - `src/components/signal/SignalStateText.tsx`（新建）
  - `src/components/signal/StatusInfoPanel.tsx`（新建）
  - `src/components/status/AgentStatusCard.tsx`（新建）
  - `src/styles/pages.css`（新建 — overview 部分）
  - `src/styles/components.css`（扩展）
  - `src/App.tsx`（修改 — 页面路由走 OverviewPage）
- **Tasks:**
  1. `SignalLamp`：58px 圆灯，三色 + glow（仅绿色启用 glow）
  2. `SignalStack`：纵向排列 3 个 lamp
  3. `SignalDesk`：三栏 grid（lamp + state + info），与 V2 HTML 一致
  4. `SignalStateText`：90px 大字
  5. `StatusInfoPanel`：右侧 info 列（实时后端、refresh、error）
  6. `AgentStatusCard`：CODEX/CLAUDE CODE 卡片
  7. `OverviewPage` 组装上述组件，接入 `snapshot` 数据
- **Expected Result:** Overview 页视觉与 V2 HTML 完全一致：信号台三栏、红绿灯、大字状态、Agent 卡片

### Phase 6: GeneralPage — "Toggle Matrix"

- **Goal:** 重构通用页为 2x2 Toggle Matrix 卡片
- **Files:**
  - `src/pages/GeneralPage.tsx`（新建）
  - `src/components/toggle/ToggleMatrix.tsx`（新建）
  - `src/components/toggle/ToggleMatrixCard.tsx`（新建）
  - `src/components/toggle/ToggleSwitch.tsx`（新建）
  - `src/styles/pages.css`（扩展 — general 部分）
  - `src/App.tsx`（修改 — 页面路由走 GeneralPage）
- **Tasks:**
  1. `ToggleSwitch`：V2 开关视觉（74px × 38px 圆角矩形，圆形滑块）
  2. `ToggleMatrixCard`：卡片容器 + h2 标题 + key + switch
  3. `ToggleMatrix`：2 列 grid
  4. `GeneralPage`：接入 `settings.general.*` 数据，绑定 toggle 事件
  5. 语言项作为特殊 card（pill 展示 `跟随系统`）
- **Expected Result:** 通用页 2x2 网格，4 张卡片视觉与 V2 HTML 一致，switch 滑块正常交互

### Phase 7: MonitoringPage — "Node Cards"

- **Goal:** 重构监听页为 Source Node Cards
- **Files:**
  - `src/pages/MonitoringPage.tsx`（新建）
  - `src/components/source/SourceNodeGrid.tsx`（新建）
  - `src/components/source/SourceNodeCard.tsx`（新建）
  - `src/styles/pages.css`（扩展 — monitoring 部分）
  - `src/App.tsx`（修改 — 页面路由走 MonitoringPage）
- **Tasks:**
  1. `SourceNodeCard`：大号 ON/OFF 文字（58px, font-weight 950）+ 小绿点 pill "参与判断"
  2. `SourceNodeGrid`：2 列 grid
  3. `MonitoringPage`：接入 `settings.monitoring.*` 和 `snapshot.sources.*`
- **Expected Result:** 监听页 2 张 node card，大号 ON/OFF，绿点 pill，参与判断标签

### Phase 8: AppearancePage — "Dot Object"

- **Goal:** 重构外观页为 Dot Object 展示
- **Files:**
  - `src/pages/AppearancePage.tsx`（新建）
  - `src/components/appearance/DotObjectGrid.tsx`（新建）
  - `src/components/appearance/DotObject.tsx`（新建）
  - `src/components/appearance/BrightnessControl.tsx`（新建）
  - `src/components/shared/ActionButton.tsx`（新建 — 用于"重置为默认 palette"按钮）
  - `src/styles/pages.css`（扩展 — appearance 部分）
  - `src/styles/components.css`（扩展 — slider 样式）
  - `src/App.tsx`（修改 — 页面路由走 AppearancePage）
- **Tasks:**
  1. `DotObject`：130px 大圆 + 颜色对应的 glow（仅绿色启用）+ 色值 pill。**保留交互式 color picker**——点击色圆或色值 pill 时弹出原生 `<input type="color">`，选择后立即调用 `onChange` 更新设置。不能做成纯静态展示
  2. `DotObjectGrid`：3 列 grid
  3. `BrightnessControl`：V2 视觉样式的可拖动 slider。使用真正的 `<input type="range">`，用 CSS 隐藏原生外观并模拟 V2 HTML 的 `.slider:before` 条样式（`height: 5px; background: #ddd` + 填充段 `background: #111`）。不能做成纯 CSS 伪元素静态展示
  4. `ActionButton`：V2 风格的 action button（边框 + paper 背景）
  5. 接入 `settings.widget_visual.palette.*` 数据
  6. 保留"重置为默认 palette"按钮逻辑，接入 `bootstrap.default_widget_palette`
- **Expected Result:** 外观页 3 个大色彩圆 + 亮度滑条 + 重置按钮，视觉与 V2 HTML 一致（按钮除外）

### Phase 9: AboutPage — "Version Card"

- **Goal:** 重构关于页为 Version Card 展示
- **Files:**
  - `src/pages/AboutPage.tsx`（新建）
  - `src/components/about/VersionCard.tsx`（新建）
  - `src/components/about/AboutSpecRow.tsx`（新建）
  - `src/components/about/AboutNote.tsx`（新建）
  - `src/styles/pages.css`（扩展 — about 部分）
  - `src/App.tsx`（修改 — 页面路由走 AboutPage）
- **Tasks:**
  1. `AboutSpecRow`：两栏 grid（label + meta / value）
  2. `VersionCard`：产品名行 + 版本号行
  3. `AboutNote`：底部的说明 note
  4. 接入 `bootstrap.about.product_name` 和 `bootstrap.about.version`
- **Expected Result:** 关于页显示产品名 + 大版本号，视觉与 V2 HTML 一致

### Phase 10: 样式收尾 & 旧文件清理

- **Goal:** 最终样式去重、清理旧文件、构建验证
- **Files:**
  - `src/styles.css`（删除 — 所有样式已迁移到新文件）
  - `src/App.tsx`（最终精简 — 确认只剩编排逻辑）
- **Tasks:**
  1. 搜索并消除残留的 inline style
  2. 搜索并消除硬编码的颜色值（应全部走 token）
  3. 搜索是否还有重复 hardcoded color
  4. 检查是否有新 UI 库引入
  5. 删除旧的 `src/styles.css`
  6. 最终构建验收
- **Expected Result:** 所有样式在目录化文件中，无遗留 inline style，无重复 token

---

## 七、Validation Strategy

### 7.1 每个阶段的即时验证

| Phase | 验证方式 |
|---|---|
| Phase 1a | `cargo check -p taskbar-settings-tauri --offline` 通过，页面视觉与改造前完全一致（新旧 CSS 并存） |
| Phase 1b | 导航剩 5 项，其余 5 页功能正常 |
| Phase 2 | 手动点测 5 个导航切换，导航宽度 240px，active 态黑底白字 |
| Phase 3 | 5 个页面的 header 区：meta 标签、h1 大标题、sub 说明文字与 HTML 一致；确认 locales 字符串已同步 |
| Phase 4 | card 边框 `#deded8`、pill 圆角 999px、paper 背景 `#fffefa` 与 HTML 一致 |
| Phase 5-9 | 每个页面截屏对比 V2 HTML，逐元素验证布局、颜色、字号、间距；确认 fake_mode 和 last_error_summary 可见 |
| Phase 10 | `cargo check` + `cargo build -p taskbar-settings-tauri --offline` |

### 7.2 最终验收

```powershell
# 1. 编译检查
cargo check -p taskbar-settings-tauri --offline

# 2. 构建
cargo build -p taskbar-settings-tauri --offline

# 3. 手动视觉回归：5 个页面逐一与 V2 HTML 对比
# 4. 业务逻辑回归：开关切换、颜色选择、所有设置项正常
```

---

## 八、Risks and Mitigations

| 风险 | 影响 | 缓解措施 |
|---|---|---|
| **点阵背景挡住点击** | 按钮无法点击 | `body:before` 保留 `pointer-events: none` |
| **拆组件时破坏保存链路** | 设置无法应用 | `saveSettings → notifySettingsApplied` 链路集中管理，组件只 emit 事件 |
| **状态色铺太满** | 样式偏离 V2，变成 Dashboard | 遵循"灯/点/pill/Dot Object 着色"原则，不整卡铺色 |
| **导航 active 与页面不同步** | 点击导航后页面不切换 | `PageFrame` 受控于 `active` prop，与 nav 共享同一 state |
| **默认 palette 硬编码** | 色值与 V1 不匹配 | 色值始终从 `bootstrap.default_widget_palette` 或 `settings.widget_visual.palette` 读取 |
| **pending 状态丢失** | 用户重复点击设置 | pending 作为 prop 透传给 switch/dot object，禁用交互 |
| **LanguageMode 交互不足** | 语言切换只有 pill 展示 | 保持 V1 的三态切换逻辑（follow_system → zh-CN → en），但视觉用 pill 展示 |

---

## 九、Open Questions

以下问题已在计划制定过程中通过用户确认解决：

| 问题 | 决策 |
|---|---|
| **Diagnostics 页面** | ❌ 移除该页和导航项 |
| **重置 palette 按钮** | ✅ 保留，以 V2 风格 action button 放置 |
| **手动刷新按钮** | ❌ 移除 |

---

## 十、Recommended Next Step

立即开始 **Phase 1a: CSS 基础设施** — 具体操作：

1. 创建 `src/styles/` 目录
2. 从 `docs/design/v2-settings-ui.html` 提取 `:root` token 写入 `tokens.css`
3. 创建 `reset.css`（box-sizing、body margin、button reset、focus ring）
4. 创建 `base.css`（body 背景、字体、点阵 `body:before`、滚动条、selection）
5. 更新 `main.tsx`：import 顺序 tokens.css → reset.css → base.css → styles.css（旧文件最后）

**完成后：** 页面外观与改造前完全一致（新旧 CSS 并存，旧规则优先级更高）。新 token 变量已可用但未激活。`cargo check` 通过。

然后继续 **Phase 1b: 移除 Diagnostics 页面** — 从 `VISIBLE_PAGE_IDS` 移除 `"diagnostics"`，移除渲染分支。
