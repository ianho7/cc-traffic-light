# V2 UI 元素组件化 + 样式统一管理设计

> 基于已确认的 V2 UI HTML 预览页整理。  
> 当前视觉基准：`docs/design/v2-settings-ui.html`  
> 目标：只做组件拆分与样式系统整理，不新增功能，不改变页面方案，不改变 V1 业务语义。

---

## 0. 背景与约束

这是一个桌面设置窗口 UI，项目大致是 Tauri / React / Rust，当前已有 V1 代码逻辑。现在先不要改业务逻辑，只做 V2 UI 的组件拆分和样式系统整理。

已确认的设计方向：

- UI 主风格：`02 Minimal Dot Matrix`
- 主布局理念：`04 Signal First Focus`
- 总览页：`01 Center Signal Desk`
- 通用页：`03 Toggle Matrix`
- 监听页：`01 Node Cards`
- 外观页：`03 Dot Object`
- 关于页：`01 Version Card`

明确禁止：

- 不要新增功能
- 不要改变页面方案
- 不要改变 V1 的业务语义
- 不要把它改成普通后台 Dashboard
- 不要引入新的 UI 库
- 不要使用玻璃拟态、蓝紫渐变、赛博终端风

---

# 1. 当前 HTML 的 UI 结构总结

## 1.1 App Shell

当前整体结构是：

```text
body
- Dot background: body:before
- .app
  - aside.side
  - main.main
```

`.app` 使用两栏 Grid：左侧固定 `240px`，右侧为主内容区域。`body:before` 是全局低透明点阵背景，并且已经设置了 `pointer-events: none`，这是正确的，后续组件化时必须保留。

当前 App Shell 适合拆成：

```text
SettingsShell
- DotBackground
- SidebarNav
- MainContent
```

## 1.2 导航栏

当前导航结构：

```text
aside.side
- .brand
- nav.nav
  - button[data-p="overview"]
  - button[data-p="general"]
  - button[data-p="monitoring"]
  - button[data-p="appearance"]
  - button[data-p="about"]
```

导航按钮通过 `active` class 控制选中态，并通过脚本同步页面显示。HTML 中的脚本只负责页面切换，不涉及业务逻辑。

适合组件化为：

```text
SidebarNav
- BrandMark
- NavItem[]
```

## 1.3 页面容器

当前 5 个页面都使用相同结构：

```text
section.page
- .header
  - .meta
  - h1
  - .sub
- page-specific content
```

`.header`、`.meta`、`.sub` 已经明显重复，应该优先抽成：

```text
PageFrame
PageHeader
MetaLabel
```

## 1.4 各页面模块

### OverviewPage

当前结构：

```text
OverviewPage
- PageHeader
- .card.signal-desk
  - .traffic
    - .lamp
    - .lamp
    - .lamp.green
  - overall state block
  - .info
- .agents
  - .card.agent
  - .card.agent
```

它是当前 UI 的视觉主锚点，符合 `01 Center Signal Desk`。`SignalDesk`、`SignalLamp`、`AgentStatusCard` 都应该拆出来。

### GeneralPage

当前结构：

```text
GeneralPage
- PageHeader
- .toggle-grid
  - .card.toggle
  - .card.toggle
  - .card.toggle
  - .card.toggle
```

其中前三个是开关，第四个是语言 pill。这里不应该被还原成普通表单，而应该保持 `Toggle Matrix` 卡片式视觉。

### MonitoringPage

当前结构：

```text
MonitoringPage
- PageHeader
- .nodes
  - .card.node
  - .card.node
```

`Node Cards` 使用大号 `ON` 和小圆点状态，适合抽成 `SourceNodeCard`。

### AppearancePage

当前结构：

```text
AppearancePage
- PageHeader
- .objects
  - .card.object
  - .card.object
  - .card.object
- brightness card
  - .slider
```

这里的 `Dot Object` 是外观页的核心隐喻，不能改回普通 color input。当前"未激活亮度"卡片使用了 inline style，组件化时应移入 CSS。

### AboutPage

当前结构：

```text
AboutPage
- PageHeader
- .card.version-card
  - .about-row 产品
  - .about-row 版本
- .about-note
```

关于页已经修正为产品名 + 版本号的结构，不能只保留版本号。

## 1.5 可复用元素

| 元素 | 当前 class | 重复情况 | 建议组件 |
|---|---|---|---:|---|
| 页面标题区 | `.header` | 5 个页面 | `PageHeader` |
| 小型英文标签 | `.meta` | 多处 | `MetaLabel` |
| 辅助说明 | `.sub` | 多处 | `SubText` |
| 卡片容器 | `.card` | 多处 | `BaseCard` |
| 胶囊标签 | `.pill` | 多处 | `ValuePill` / `StatusBadge` |
| 页面显示状态 | `.page.active` | 5 页 | `PageFrame` |
| 状态灯 | `.lamp` | Overview | `SignalLamp` |
| 开关 | `.switch` | General | `TogglePill` |
| 节点小点 | `.dot` | Monitoring | `StatusDot` |
| 色彩圆点 | `.object-dot` | Appearance | `DotObject` |

## 1.6 当前重复较多的部分

最明显的重复：

1. **Page Header 重复 5 次**  
   每个页面都写了 `.header > .meta + h1 + .sub`。

2. **Card 样式重复很多**  
   `.card` 是所有页面的基本容器，应抽为基础视觉组件。

3. **Mono 标签重复**  
   `.meta`、`.key`、`.about-key`、`.pill` 都是同一类 mono 信息标签，可以归入 typography token。

4. **Grid 布局重复**  
   `.agents`、`.toggle-grid`、`.nodes`、`.objects` 都是页面级 grid，可整理为 layout utility 或页面组件内部样式。

5. **状态色重复且硬编码扩散风险高**  
   当前 root 中已有 `--green`、`--yellow`、`--red`，但阴影颜色、switch 背景、inactive 状态等仍有硬编码，后续应统一 token 化。

---

# 2. 设计系统原则

## 2.1 Minimal Dot Matrix 的视觉规则

这套 UI 不是"炫酷终端"，也不是普通后台 Dashboard。它的核心是：

```text
低噪声底色
+ 少量点阵纹理
+ 强边界卡片
+ 大字号状态
+ 红黄绿信号点
```

规则建议：

1. **背景用温和纸面色，不用纯白**
   - 当前 `--bg: #f5f5f2`
   - 当前 `--paper: #fffefa`

2. **点阵只作为空气纹理，不作为主要装饰**
   - 当前点阵 opacity 是 `.04`，很克制。
   - 后续不要给每张卡片都加点阵背景。

3. **边界比阴影更重要**
   - 当前 `.card` 只有 `border` 和纸面背景，没有厚重阴影。
   - 可以保留极轻 shadow token，但默认不启用。

4. **Mono 字体用于系统标签，不用于所有文字**
   - `.meta`、`.key`、`.pill`、`.version` 使用 mono 语义合理。
   - 中文主标题和卡片标题继续使用 UI sans。

5. **大字号是"状态感"，不是 Dashboard 数据看板**
   - `总览`页面的"空闲"与关于页版本号是视觉锚点，不是 KPI 数字。

## 2.2 Signal First Focus 的页面规则

Signal First Focus 的关键是：**先告诉用户当前状态，再给用户设置入口。**

页面规则：

| 页面 | 当前视觉核心 | 保留原则 |
|---|---|---|
| 总览 | 中央信号台 | 状态最大，辅助信息靠边 |
| 通用 | Toggle Matrix | 设置项卡片化，不写成表格 |
| 监听 | Node Cards | 来源状态节点化，不写成 checkbox list |
| 外观 | Dot Object | 颜色作为对象展示，不写成普通色值表单 |
| 关于 | Version Card | 产品名和版本号作为设备规格展示 |

## 2.3 黑白灰与红黄绿状态色规则

推荐使用方式：

```text
黑色：主文字、导航 active、信号灯壳、slider active
灰色：说明文字、边框、inactive 状态
绿色：正常 / 空闲 / 启用 / 可用
黄色：运行中 / 注意 / 等待
红色：错误 / 异常 / 禁用风险
```

需要克制：

- 不要让整张卡片变成绿色 / 黄色 / 红色。
- 不要大面积状态背景。
- 不要把红黄绿做成渐变光效。
- 不要用蓝紫色表达主状态。

可以强化的地方：

- `SignalLamp`
- `StatusBadge`
- `SourceNodeCard` 的小圆点
- `last_error_summary`
- `pending` 状态下的按钮 / 开关状态
- `fake_mode` 提示

## 2.4 点阵背景 / 点阵元素使用边界

可以使用点阵的地方：

```text
- 全局 body 背景
- 某些页面标题区域的轻微背景，未确认是否需要
- 状态小点
- Dot Object
- 空状态或 pending 状态的轻微纹理，未确认是否需要
```

不要使用点阵的地方：

```text
- 每个 card 内部
- 每个按钮背景
- 大面积彩色点阵
- 表单控件内部
- 文字后方造成阅读干扰的位置
```

## 2.5 克制与强化边界

| 类型 | 应该克制 | 可以强化 |
|---|---|---|
| 背景 | 只保留低透明点阵 | 不建议强化 |
| 卡片 | 边框 + paper 背景 | hover 可以轻微边框变化 |
| 状态色 | 不铺满容器 | 灯、点、pill、状态字 |
| 动效 | 不做炫光 | switch、active nav、lamp glow |
| 页面密度 | 保持留白 | 大状态字和版本号可以巨大 |

---

# 3. 组件拆分方案

## 3.1 App 级组件

| 组件名 | 作用 | 复用位置 | 主要 props | 是否有状态 | 样式 class 建议 |
|---|---|---|---|---|---|
| `SettingsShell` | 设置窗口总壳，包含导航和内容区 | 全局 | `activePage`, `onPageChange`, `children` | 有，建议受控 | `.settings-shell` |
| `DotBackground` | 全局点阵背景 | 全局 | `density?`, `opacity?`，默认使用 token | 无 | `.dot-background` |
| `SidebarNav` | 左侧导航 | 全局 | `items`, `activeId`, `onChange` | 无，受控 | `.sidebar-nav` |
| `BrandMark` | 顶部品牌文字 | Sidebar | `title`, `mark?` | 无 | `.brand-mark` |
| `NavItem` | 单个导航按钮 | Sidebar | `id`, `index`, `label`, `active`, `onClick` | 无 | `.nav-item`, `.nav-item--active` |
| `MainContent` | 主内容容器 | 全局 | `children` | 无 | `.main-content` |
| `PageFrame` | 页面显示容器 | 每个页面 | `pageId`, `active`, `children` | 无，受控 | `.page-frame`, `.page-frame--active` |
| `PageHeader` | 页面标题结构 | 5 个页面 | `meta`, `title`, `subtitle` | 无 | `.page-header` |

## 3.2 通用基础组件

| 组件名 | 作用 | 复用位置 | 主要 props | 是否有状态 | 样式 class 建议 |
|---|---|---|---|---|---|
| `BaseCard` | 统一卡片容器 | 所有页面 | `children`, `className?`, `variant?`, `padding?` | 无 | `.base-card` |
| `MetaLabel` | 英文系统标签 | Header、SignalDesk、About | `children`, `uppercase?` | 无 | `.meta-label` |
| `SubText` | 辅助说明文字 | Header、Overview | `children` | 无 | `.sub-text` |
| `SectionTitle` | 卡片标题 | Agent、Toggle、Node、Object | `children`, `size?` | 无 | `.section-title` |
| `InfoRow` | 信息行 | Overview info、About row | `label`, `value`, `direction?` | 无 | `.info-row` |
| `StatusBadge` | 状态胶囊 | Overview、Monitoring | `tone`, `children`, `dot?` | 无 | `.status-badge`, `.status-badge--green` |
| `ValuePill` | 值标签 | General language、Appearance color | `children`, `tone?` | 无 | `.value-pill` |
| `StatusDot` | 小状态点 | Monitoring、Badge | `tone`, `size?` | 无 | `.status-dot` |
| `ActionButton` | 后续如需保存/重试按钮时统一样式 | 未确认，当前 HTML 没有 | `children`, `variant`, `disabled`, `pending`, `onClick` | 有，受 props 控制 | `.action-button` |
| `TogglePill` | pill 式开关 | General | `checked`, `disabled`, `pending`, `onChange`, `label?` | 无，受控 | `.toggle-pill`, `.toggle-pill--on` |
| `InlineKey` | 设置 key 文本 | General | `children` | 无 | `.inline-key` |
| `SliderTrack` | 简化滑条视觉 | Appearance | `value`, `disabled`, `onChange?` | 无，受控 | `.slider-track` |

## 3.3 业务视觉组件

| 组件名 | 作用 | 复用位置 | 主要 props | 是否有状态 | 样式 class 建议 |
|---|---|---|---|---|---|
| `SignalLamp` | 单个红绿灯圆灯 | Overview、未来 Diagnostics | `tone: "red" \| "yellow" \| "green" \| "idle"`, `active`, `glow?` | 无 | `.signal-lamp`, `.signal-lamp--green` |
| `SignalStack` | 三灯纵向灯体 | Overview | `activeTone`, `order?`, `inactiveTone?` | 无 | `.signal-stack` |
| `SignalDesk` | 总览页主视觉卡片 | Overview | `overallState`, `sources`, `lastRefreshAt`, `errorSummary`, `fakeMode`, `pending` | 无，受数据驱动 | `.signal-desk` |
| `SignalStateText` | 大状态字 | Overview | `label`, `tone?` | 无 | `.signal-state-text` |
| `StatusInfoPanel` | 右侧实时后端 / refresh / error 信息 | Overview | `backendLabel`, `lastRefreshAt`, `errorSummary`, `fakeMode` | 无 | `.status-info-panel` |
| `AgentStatusCard` | Codex / Claude Code 状态卡 | Overview | `name`, `state`, `enabled?`, `tone`, `nodeLabel?` | 无 | `.agent-status-card` |
| `ToggleMatrix` | 通用设置网格 | General | `items`, `pending`, `appliedKeys` | 无 | `.toggle-matrix` |
| `ToggleMatrixCard` | 单个设置卡片 | General | `title`, `settingKey`, `value`, `type`, `pending`, `applied`, `onChange` | 无，受控 | `.toggle-matrix-card` |
| `LanguageModeCard` | 语言项卡片 | General | `value`, `options?`, `pending`, `onChange` | 未确认，当前为静态 pill | `.language-mode-card` |
| `SourceNodeGrid` | 监听来源网格 | Monitoring | `sources`, `pending`, `appliedKeys` | 无 | `.source-node-grid` |
| `SourceNodeCard` | Codex / Claude Code 来源卡 | Monitoring | `sourceId`, `name`, `enabled`, `participates`, `status`, `pending`, `onToggle` | 无，受控 | `.source-node-card` |
| `DotObjectGrid` | 外观色彩对象网格 | Appearance | `palette`, `pending`, `appliedKeys` | 无 | `.dot-object-grid` |
| `DotObject` | 单个大色彩圆对象 | Appearance | `label`, `value`, `tone`, `editable?`, `pending`, `onChange?` | 无，受控 | `.dot-object` |
| `PaletteDotObject` | 绑定 widget palette 的 DotObject | Appearance | `colorKey`, `colorValue`, `defaultValue?`, `onChange` | 无 | `.palette-dot-object` |
| `BrightnessControl` | 未激活亮度控制 | Appearance | `value`, `min`, `max`, `pending`, `onChange` | 无，受控 | `.brightness-control` |
| `VersionCard` | 关于页版本卡 | About | `productName`, `version` | 无 | `.version-card` |
| `AboutSpecRow` | 关于页规格行 | About | `label`, `meta`, `children` | 无 | `.about-spec-row` |
| `AboutNote` | 关于页说明 | About | `children` | 无 | `.about-note` |

---

# 4. 样式体系

## 4.1 CSS Token 变量定义

```css
:root {
  /* 颜色 */
  --bg: #f5f5f2;
  --paper: #fffefa;
  --ink: #151515;
  --muted: #777;
  --line: #deded8;
  --green: #34c759;
  --yellow: #ffcc00;
  --red: #ff3b30;

  /* 字体 */
  --mono: ui-monospace, Consolas, monospace;
  --ui: "SF Pro Display", "Segoe UI", Arial, sans-serif;
}
```

## 4.2 衍生 token（推荐在 tokens.css 统一声明）

```css
:root {
  /* 语义 */
  --bg-sidebar: #fafaf7;
  --bg-note: #fafaf7;

  /* 间距 (4px 基准) */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 12px;
  --radius-pill: 999px;

  /* 字体 scale */
  --text-xs: 11px;
  --text-sm: 12px;
  --text-base: 14px;
  --text-lg: 24px;
  --text-xl: 36px;
  --text-2xl: 58px;
  --text-3xl: 90px;
  --text-4xl: 100px;

  /* 点阵背景 */
  --dot-color: #111;
  --dot-opacity: 0.04;
  --dot-size: 16px;

  /* 导航 */
  --sidebar-width: 240px;
}
```

## 4.3 删除和替换的 token

当前 V1 的 `styles.css` 中有大量 D 方案的 token，这些在 V2 纸质主题中**不应出现**：

```css
/* ❌ 删除 — 暗色玻璃质感相关 */
--bg-app: #0d0d0f;
--accent-gradient: linear-gradient(135deg, #60a5fa, #a78bfa);
--shadow-glass: 0 8px 32px rgba(0, 0, 0, 0.35), ...;
--ok-glow: rgba(34, 197, 94, 0.25);
--warn-glow: rgba(234, 179, 8, 0.25);
--error-glow: rgba(239, 68, 68, 0.25);
/* ... 更多 D 方案 token */
```

---

# 5. 各页面实测结构与视觉拆解

## 5.1 总览页

```text
section#overview.page.active
├── .header
│   ├── .meta    → "STATUS SUMMARY"
│   ├── h1       → "总览"
│   └── .sub     → "01 Center Signal Desk"
├── .card.signal-desk
│   ├── .traffic
│   │   ├── i.lamp          → #333
│   │   ├── i.lamp          → #333
│   │   └── i.lamp.green    → var(--green) + box-shadow
│   ├── div
│   │   ├── .meta           → "OVERALL STATE"
│   │   ├── .state          → "空闲" (90px, 950)
│   │   └── .sub            → "Claude Code 空闲 ｜ Codex 空闲"
│   └── .info
│       ├── div             → "实时后端"
│       ├── div             → "LAST REFRESH / 2026/07/08"
│       └── div             → "ERROR / 无"
└── .agents
    ├── .card.agent
    │   ├── h2              → "CODEX"
    │   ├── strong          → "空闲" (42px)
    │   └── span.pill       → "NODE"
    └── .card.agent
        ├── h2              → "CLAUDE CODE"
        ├── strong          → "空闲"
        └── span.pill       → "NODE"
```

### visual 参数

| 元素 | 视觉参数 |
|---|---|
| `.signal-desk` | `grid-template-columns: 170px 1fr 240px; gap: 24px; align-items: center; padding: 32px` |
| `.traffic` | `background: #111; border-radius: 50px; padding: 18px; display: grid; gap: 14px` |
| `.lamp` | `width: 58px; height: 58px; border-radius: 50%; background: #333` |
| `.lamp.green` | `background: var(--green); box-shadow: 0 0 30px #34c75966` |
| `.state` | `font-size: 90px; font-weight: 950; letter-spacing: -.12em` |
| `.info` | `padding: 18px` |
| `.info div` | `padding: 12px; border-bottom: 1px solid var(--line); font: 700 12px var(--mono)` |
| `.agents` | `grid-template-columns: 1fr 1fr; gap: 18px; margin-top: 20px` |
| `.agent` | `padding: 24px` |
| `.agent h2` | `font-size: 22px` |
| `.agent strong` | `display: block; font-size: 42px; margin: 22px 0` |
| `.pill` | `border: 1px solid var(--line); border-radius: 999px; padding: 8px 12px; font: 700 11px var(--mono)` |

## 5.2 通用页

```text
section#general.page
├── .header
│   ├── .meta    → "SYSTEM BEHAVIOR"
│   ├── h1       → "通用"
│   └── .sub     → "03 Toggle Matrix"
└── .toggle-grid (grid-template-columns: 1fr 1fr; gap: 20px)
    ├── .card.toggle (padding: 28px; min-height: 190px)
    │   ├── h2         → "登录时启动" (font-size: 24px)
    │   ├── .key       → "START_ON_LOGIN" (font: 700 11px var(--mono); color: #999)
    │   └── .switch    → (74x38, border-radius: 999px, bg: #eee, border: 1px solid #ccc)
    │       └── :after → (28x28 circle, bg: #999, left: 4px, top: 4px)
    ├── .card.toggle
    │   ├── h2         → "启动最小化到托盘"
    │   ├── .key       → "MINIMIZE_ON_START"
    │   └── .switch.on → (bg: #e8f7ed)
    │       └── :after → (left: 40px, bg: var(--green))
    ├── .card.toggle
    │   ├── h2         → "关闭窗口到托盘"
    │   ├── .key       → "CLOSE_TO_TRAY"
    │   └── .switch.on
    └── .card.toggle
        ├── h2         → "语言"
        ├── .key       → "LANGUAGE_MODE"
        └── span.pill  → "跟随系统"
```

### Switch 视觉机制

```css
.switch {
  margin-top: 30px;
  width: 74px; height: 38px;
  border-radius: 999px;
  background: #eee; border: 1px solid #ccc;
  position: relative;
}
.switch:after {
  content: "";
  width: 28px; height: 28px;
  border-radius: 50%;
  background: #999;
  position: absolute;
  left: 4px; top: 4px;
  transition: left 0.2s; /* 建议添加 */
}
.switch.on { background: #e8f7ed; }
.switch.on:after { left: 40px; background: var(--green); }
```

## 5.3 监听页

```text
section#monitoring.page
├── .header
│   ├── .meta    → "SOURCE MATRIX"
│   ├── h1       → "监听"
│   └── .sub     → "01 Node Cards"
└── .nodes (grid-template-columns: 1fr 1fr; gap: 22px)
    ├── .card.node (padding: 28px; min-height: 260px)
    │   ├── h2         → "Codex" (font-size: 36px)
    │   ├── .big       → "ON" (font-size: 58px; font-weight: 950; margin: 35px 0)
    │   └── span.pill
    │       ├── i.dot  → (width: 12px; height: 12px; border-radius: 50%; bg: var(--green); display: inline-block)
    │       └── text   → "参与判断"
    └── .card.node
        ├── h2         → "Claude Code"
        ├── .big       → "ON"
        └── span.pill
            ├── i.dot
            └── text   → "参与判断"
```

## 5.4 外观页

```text
section#appearance.page
├── .header
│   ├── .meta    → "WIDGET PALETTE"
│   ├── h1       → "外观"
│   └── .sub     → "03 Dot Object"
├── .objects (grid-template-columns: repeat(3, 1fr); gap: 24px)
│   ├── .card.object (padding: 30px; text-align: center)
│   │   ├── h2             → "GREEN"
│   │   ├── .object-dot.g  → (130x130, border-radius: 50%, bg: var(--green), box-shadow: 0 0 40px #34c75955)
│   │   └── span.pill      → "#34C759"
│   ├── .card.object
│   │   ├── h2             → "YELLOW"
│   │   ├── .object-dot.y  → (背景: var(--yellow), 无 glow)
│   │   └── span.pill      → "#FFCC00"
│   └── .card.object
│       ├── h2             → "RED"
│       ├── .object-dot.r  → (背景: var(--red), 无 glow)
│       └── span.pill      → "#FF3B30"
└── .card (padding:20px; margin-top:20px)  ← 当前有 inline style
    ├── b                  → "未激活亮度"
    └── .slider (height:5px; background:#ddd; margin:30px)
        └── :before        → (width: 42%; height:100%; background:#111)
```

### Dot Object 视觉参数

| 元素 | 参数 |
|---|---|
| `.object-dot` | `width: 130px; height: 130px; border-radius: 50%; margin: 25px auto` |
| `.object-dot.g` | `background: var(--green); box-shadow: 0 0 40px #34c75955` |
| `.object-dot.y` | `background: var(--yellow)` |
| `.object-dot.r` | `background: var(--red)` |

### 亮度控制视觉参数

| 元素 | 参数 |
|---|---|
| `.slider` | `height: 5px; background: #ddd; margin: 30px` |
| `.slider:before` | `content: ""; display: block; width: 42%; height: 100%; background: #111` |

## 5.5 关于页

```text
section#about.page
├── .header
│   ├── .meta    → "DEVICE SPEC"
│   ├── h1       → "关于"
│   └── .sub     → "01 Version Card"
├── .card.version-card (padding:0; overflow:hidden)
│   ├── .about-row (grid-template-columns: 220px 1fr; gap: 24px; align-items: center; padding: 34px; border-bottom: 1px solid var(--line))
│   │   ├── div
│   │   │   ├── .about-label → "产品" (font-size: 24px; font-weight: 900; letter-spacing: -.05em)
│   │   │   └── .about-key   → "PRODUCT" (font: 800 11px var(--mono); letter-spacing: .14em; color: #999; margin-top: 8px)
│   │   └── .about-product   → "CC Traffic Light" (font-size: 34px; font-weight: 900; letter-spacing: -.06em)
│   └── .about-row:last-child (border-bottom: 0)
│       ├── div
│       │   ├── .about-label → "版本"
│       │   └── .about-key   → "VERSION"
│       └── .version         → "0.1.0" (font: 950 100px var(--mono); letter-spacing: -.12em)
└── .about-note
    → "版本号来自 bootstrap.about.version，产品名称来自 bootstrap.about.product_name。"
    (border: 1px solid var(--line); background: #fafaf7; color: #777; font: 700 12px var(--mono); letter-spacing: .08em)
```

---

# 6. 页面切换与状态同步

## 6.1 当前 HTML 切换机制（不再使用，但需理解）

```javascript
// HTML 内联脚本 — React 化后不再需要
buttons.forEach(btn => {
  btn.addEventListener("click", () => {
    buttons.forEach(x => x.classList.remove("active"));
    pages.forEach(x => x.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById(btn.dataset.p).classList.add("active");
  });
});
```

## 6.2 React 化后的替换机制

React 中不需要 DOM 操作脚本，用 state 驱动：

```tsx
// App 层：const [page, setPage] = useState<SettingsPageId>("overview");
// SidebarNav：接受 activeId + onChange，setPage 调用在 App 层
// PageFrame：接受 active prop，active === true 时添加 .page--active 类
```

---

# 7. V1 数据接入说明

## 7.1 总体数据流

```text
bootstrapWindow() → bootstrap { snapshot, settings, about, fake_mode, default_widget_palette }
                    ↓
              App state: bootstrap, page, pending
                    ↓
              props 分发到各页面组件
                    ↓
              用户交互 → updateConfig() → saveSettings() → notifySettingsApplied()
```

## 7.2 各组件的数据要求

### App 级组件

| 组件 | 需要的数据 | 回调 | 注意事项 |
|---|---|---|---|
| `SidebarNav` | `activePage` | `onPageChange(pageId)` | 导航 active 必须与页面 active 同步 |
| `OverviewPage` | `snapshot.overall_state`, `snapshot.sources`, `pending`, `fake_mode`, `last_detection_refresh_at`, `last_error_summary` | 无 | 不要在 Overview 内发起新轮询 |
| `SignalDesk` | `snapshot.overall_state`, `snapshot.sources`, `fake_mode`, `last_detection_refresh_at`, `last_error_summary` | 无 | 只展示状态，不改变检测逻辑 |
| `SignalStack` | 从 `snapshot.overall_state` 派生 `activeTone` | 无 | 状态到颜色映射集中管理，不写死在 JSX |
| `SignalLamp` | `activeTone`, `settings.widget_visual.*` 中的颜色配置 | 无 | 不要硬编码 `default_widget_palette` |
| `SignalStateText` | `snapshot.overall_state` | 无 | 文案映射保持 V1 语义，例如 idle 显示"空闲" |
| `StatusInfoPanel` | `fake_mode`, `last_detection_refresh_at`, `last_error_summary` | 无 | `fake_mode` 应明确展示为模拟/真实来源，不要混淆 |
| `AgentStatusCard` | `snapshot.sources.codex`, `snapshot.sources.claude_code` | 无 | 不要把 Codex / Claude Code 写成静态"空闲" |
| `GeneralPage` | `settings.general.*`, `pending`, `appliedKeys` | `onSettingChange(key, value)` | 页面不直接保存，沿用 V1 设置流 |
| `ToggleMatrix` | `settings.general.*`, `pending`, `appliedKeys` | 透传事件 | 不要把多个设置合并成一个表单提交 |
| `ToggleMatrixCard` | 单个 `settings.general.<key>` | `onChange(key, nextValue)` | 每项独立 pending / appliedKeys，不能全局粗暴禁用 |
| `LanguageModeCard` | `settings.general.language` | `onChange(key, nextValue)` | 保持 V1 三态切换，不新增交互 |
| `SourceNodeCard` | `settings.monitoring.<sourceKey>`, `snapshot.sources.<sourceKey>` | `onToggle(sourceKey, enabled)` | 参与判断状态和实时状态要区分 |
| `AppearancePage` | `settings.widget_visual.*`, `pending`, `appliedKeys` | `onWidgetVisualChange(key, value)` | 不要在组件内生成默认 palette |
| `PaletteDotObject` | `settings.widget_visual.palette.green/yellow/red` | `onChange(colorKey, colorValue)` | 色值来自 V1 / bootstrap，不从 HTML 常量写死 |
| `BrightnessControl` | `settings.widget_visual.inactive_brightness_percent` | `onChange(value)` | 保留 V1 保存顺序 |
| `VersionCard` | `bootstrap.about.product_name`, `bootstrap.about.version` | 无 | 关于页不能漏产品名 |
| `AboutNote` | `bootstrap.about.*` | 无 | 文案可保留"来自 bootstrap"的说明 |

## 状态映射建议

可以新增一个纯函数文件，但不要改业务逻辑：

```ts
type SignalTone = "green" | "yellow" | "red" | "idle";

function mapOverallStateToSignalTone(overallState: string): SignalTone {
  switch (overallState) {
    case "idle":
      return "green";
    case "running":
    case "waiting":
      return "yellow";
    case "error":
      return "red";
    default:
      return "idle";
  }
}
```

注意：上面只是结构建议，具体枚举值以 V1 实际 `snapshot.overall_state` 为准。

---

# 8. 需要避免的问题

组件化时最容易踩的坑：

1. **背景伪元素挡住点击**  
   `DotBackground` 必须保留 `pointer-events: none`。

2. **点阵样式过度使用**  
   全局已有低透明点阵，不要给每个 card 再加点阵。

3. **状态色面积太大**  
   红黄绿只用于灯、点、pill、Dot Object，不要整卡铺色。

4. **把 Toggle Matrix 写回普通表单**  
   通用页应该保持卡片矩阵，不要变成 label + input 列表。

5. **把 Dot Object 写回普通 color input**  
   外观页视觉重点是颜色对象，不是表格化配置项。

6. **关于页漏掉产品名称**  
   当前 HTML 明确有产品名和版本号，不能只显示版本号。

7. **导航 active 和页面 active 不同步**  
   当前脚本同时更新 nav button 和 page active，React 化后也必须保持同步。

8. **全局 pending 被误拆成局部 pending**  
   可以传给组件显示，但不能改变 V1 的 pending 语义。

9. **每个设置项的 `appliedKeys` 被合并**  
   这会破坏"单项设置已应用"的反馈。

10. **硬编码 `default_widget_palette`**  
   外观页颜色必须来自 V1 数据或 bootstrap，不要从 HTML 复制死值。

11. **`saveSettings → notifySettingsApplied` 顺序被打乱**  
   UI 拆分不能改变保存链路。

12. **移除 5 秒轮询**  
   组件化只改展示层，不碰轮询机制。

13. **把 `fake_mode` 当成普通文本丢掉**  
   它影响用户对数据可信度的判断，应在状态信息区体现。

14. **把 `last_error_summary` 隐藏掉**  
   总览页已有 ERROR 区域，接入时要保留。

15. **inline style 继续扩散**  
   当前"未激活亮度"卡片有 inline style，第一阶段应该消除这类样式债。

---

# 9. 第一阶段落地计划

以下路径为推荐路径。原则是：**先 tokens，再壳，再基础组件，再页面组件，最后接 V1 数据。**

## Step 1

目标：抽出全局 tokens，保持视觉完全不变。

修改文件：

```text
src/styles/tokens.css
src/styles/reset.css
src/styles/base.css
src/main.tsx
```

验证方式：

```text
- 页面背景色一致
- 点阵背景一致
- 字体观感一致
- 红黄绿颜色一致
- 没有业务逻辑 diff
```

## Step 2

目标：拆出 App Shell，不改页面内容。

修改文件：

```text
src/components/shell/SettingsShell.tsx
src/components/shell/settings-shell.css
src/components/navigation/SidebarNav.tsx
src/components/navigation/sidebar-nav.css
```

验证方式：

```text
- 左侧导航宽度仍为 240px
- 导航 active 状态正常
- 5 个页面切换正常
- body 点阵不挡点击
```

## Step 3

目标：抽出 `PageFrame` 和 `PageHeader`，替换 5 个重复 header。

修改文件：

```text
src/components/layout/PageFrame.tsx
src/components/layout/PageHeader.tsx
src/components/layout/page-layout.css
```

验证方式：

```text
- 5 个页面标题、meta、subtitle 与 HTML 一致
- 页面间距一致
- 不改任何设置项逻辑
```

## Step 4

目标：抽出基础视觉组件。

修改文件：

```text
src/components/primitives/BaseCard.tsx
src/components/primitives/MetaLabel.tsx
src/components/primitives/ValuePill.tsx
src/components/primitives/StatusBadge.tsx
src/components/primitives/StatusDot.tsx
src/components/primitives/primitives.css
```

验证方式：

```text
- card 边框和 paper 背景一致
- pill 尺寸、字体、圆角一致
- meta 字号、字距、颜色一致
```

## Step 5

目标：组件化 OverviewPage，但只做展示重组。

修改文件：

```text
src/pages/OverviewPage.tsx
src/components/signal/SignalDesk.tsx
src/components/signal/SignalStack.tsx
src/components/signal/SignalLamp.tsx
src/components/status/AgentStatusCard.tsx
src/components/signal/signal.css
```

验证方式：

```text
- Signal Desk 三栏结构一致
- 灯体尺寸和 glow 一致
- CODEX / CLAUDE CODE 卡片一致
- snapshot 数据显示不变
```

## Step 6

目标：组件化 GeneralPage 的 Toggle Matrix。

修改文件：

```text
src/pages/GeneralPage.tsx
src/components/toggle/ToggleMatrix.tsx
src/components/toggle/ToggleMatrixCard.tsx
src/components/toggle/TogglePill.tsx
src/components/toggle/toggle.css
```

验证方式：

```text
- 4 个卡片仍是 2x2 grid
- 开关视觉一致
- settings.general.* 绑定不变
- saveSettings → notifySettingsApplied 顺序不变
- appliedKeys 仍按单项工作
```

## Step 7

目标：组件化 MonitoringPage 的 Node Cards。

修改文件：

```text
src/pages/MonitoringPage.tsx
src/components/source/SourceNodeGrid.tsx
src/components/source/SourceNodeCard.tsx
src/components/source/source-node.css
```

验证方式：

```text
- Codex / Claude Code 两张卡一致
- ON 大字一致
- 小绿点 + "参与判断" pill 一致
- settings.monitoring.* 绑定不变
```

## Step 8

目标：组件化 AppearancePage 的 Dot Object 与 BrightnessControl。

修改文件：

```text
src/pages/AppearancePage.tsx
src/components/appearance/DotObjectGrid.tsx
src/components/appearance/PaletteDotObject.tsx
src/components/appearance/BrightnessControl.tsx
src/components/appearance/appearance.css
```

验证方式：

```text
- 三个颜色对象尺寸一致
- GREEN / YELLOW / RED 排列一致
- 色值来自 settings.widget_visual.*，不硬编码 default_widget_palette
- 未激活亮度视觉一致
- 移除 inline style
```

## Step 9

目标：组件化 AboutPage。

修改文件：

```text
src/pages/AboutPage.tsx
src/components/about/VersionCard.tsx
src/components/about/AboutSpecRow.tsx
src/components/about/AboutNote.tsx
src/components/about/about.css
```

验证方式：

```text
- 产品名仍来自 bootstrap.about.product_name
- 版本号仍来自 bootstrap.about.version
- 关于页不是纯版本号
```

## Step 10

目标：做一次样式去重和视觉回归。

修改文件：

```text
src/styles/components.css 或各组件 css
src/styles/pages.css
```

验证方式：

```text
- 搜索是否还有重复 hardcoded color
- 搜索是否还有 inline style
- 搜索是否有新 UI 库引入
- 检查 5 个页面截图是否与 HTML 视觉一致
- 检查业务逻辑文件 diff 是否最小
```

---

# 10. 最终简洁结论

## 10.1 推荐的组件目录结构

```text
src/
  components/
    shell/
      SettingsShell.tsx
    navigation/
      SidebarNav.tsx
      NavItem.tsx
      BrandMark.tsx
    layout/
      PageFrame.tsx
      PageHeader.tsx
    primitives/
      BaseCard.tsx
      MetaLabel.tsx
      SubText.tsx
      SectionTitle.tsx
      InfoRow.tsx
      ValuePill.tsx
      StatusBadge.tsx
      StatusDot.tsx
    signal/
      SignalDesk.tsx
      SignalStack.tsx
      SignalLamp.tsx
      SignalStateText.tsx
      StatusInfoPanel.tsx
    status/
      AgentStatusCard.tsx
    toggle/
      ToggleMatrix.tsx
      ToggleMatrixCard.tsx
      TogglePill.tsx
      LanguageModeCard.tsx
    source/
      SourceNodeGrid.tsx
      SourceNodeCard.tsx
    appearance/
      DotObjectGrid.tsx
      DotObject.tsx
      PaletteDotObject.tsx
      BrightnessControl.tsx
    about/
      VersionCard.tsx
      AboutSpecRow.tsx
      AboutNote.tsx

  pages/
    OverviewPage.tsx
    GeneralPage.tsx
    MonitoringPage.tsx
    AppearancePage.tsx
    AboutPage.tsx

  styles/
    reset.css
    tokens.css
    base.css
    shell.css
    navigation.css
    components.css
    pages.css
    utilities.css
```

## 10.2 推荐的 CSS 文件结构

```text
styles/
  reset.css        # box-sizing、body margin、button 基础 reset
  tokens.css       # 颜色、字号、间距、圆角、z-index、动画变量
  base.css         # body、全局字体、点阵背景
  shell.css        # App shell、main content
  navigation.css   # sidebar、brand、nav item
  components.css   # card、pill、badge、lamp、switch、dot 等
  pages.css        # 5 个页面的组合布局
  utilities.css    # 少量工具类，谨慎使用
```

## 10.3 最先应该抽出的 5 个组件

```text
1. PageHeader
2. BaseCard
3. ValuePill / StatusBadge
4. SignalLamp
5. ToggleMatrixCard
```

这 5 个一抽，重复度会立刻下降，视觉也不容易散架。

## 10.4 最容易出错的 5 个点

```text
1. 把 Toggle Matrix 写回普通表单
2. 把 Dot Object 写回普通 color input
3. 状态色铺太满，变成 Dashboard
4. 拆组件时破坏 saveSettings → notifySettingsApplied 顺序
5. 硬编码 default_widget_palette 或静态状态文案
```

## 10.5 下一步可以直接交给 Codex 实现的任务边界

```text
只做 V2 UI 组件化与样式整理：

- 从现有 V2 HTML 视觉基准抽 tokens.css
- 拆 SettingsShell / SidebarNav / PageFrame / PageHeader
- 拆 BaseCard / ValuePill / StatusBadge / SignalLamp / TogglePill
- 按现有 5 页结构拆 OverviewPage、GeneralPage、MonitoringPage、AppearancePage、AboutPage
- 保持当前页面方案、布局、文案、状态语义不变
- 接入 V1 现有数据字段，不引入 mock 状态
- 不改 saveSettings → notifySettingsApplied
- 不移除 5 秒轮询
- 不改变 appliedKeys 独立设置项机制
- 不新增 UI 库
- 不新增功能
```

这次的组件化方向可以理解成：**把已经定稿的 V2 HTML 从"一张漂亮海报"切成可维护的零件柜**。视觉保持原样，零件开始编号，螺丝别多拧，状态灯继续站在舞台中央。
