# CC Traffic Light UI Refactor Checklist

## Nothing Signal Console · Rust + Slint UI 重构清单

本文档用于指导 CC Traffic Light 设置窗口的 UI 重构。

目标是将当前 Rust + Slint 设置界面，从普通设置面板改造成 Nothing Signal Console 风格的本地 AI 状态信号控制台。

重点是视觉、布局、组件和信息层级重构。**不要新增业务功能。**

> 验证状态（2026-07-03）
>
> - 已完成代码改造并更新勾选状态。
> - 勾选依据：当前工作区代码审计 + `cargo check` + `cargo test` + `cargo fmt -- --check`。
> - `18.2 失败信号` 保持未勾选，表示这些失败条件未出现。

---

# 0. 项目资料准备

## 0.1 推荐文件位置

```text
docs/ui/current-screenshots/
docs/ui/cc_traffic_light_nothing_demo_strict.html
docs/ui/nothing-signal-console-spec.md
docs/ui/nothing-signal-console-checklist.md
```

## 0.2 文件用途

| 文件                                        | 用途                                 |
| ------------------------------------------- | ------------------------------------ |
| `current-screenshots/`                      | 当前 UI 截图，决定已有页面和功能边界 |
| `cc_traffic_light_nothing_demo_strict.html` | HTML 视觉参考，只决定外观方向        |
| `nothing-signal-console-spec.md`            | 设计规格，补充颜色、字体、组件规则   |
| `nothing-signal-console-checklist.md`       | 执行步骤和验收清单                   |

---

# 1. 核心原则

## 1.1 本次重构目标

将当前 UI 改造成：

> 浅色 Nothing 工业控制台：黑白克制、编号导航、等宽信息、细边框、小圆角、状态灯表达。

重点包括：

- 统一视觉语言
- 统一颜色 token
- 统一字号和间距
- 抽公共 Slint 组件
- 重构左侧导航
- 逐页重构 6 个页面
- 保持当前业务逻辑不变
- 尽量还原 HTML demo 的视觉方向

## 1.2 本次重构不做什么

本次任务不是功能扩展。

禁止新增：

- 新页面
- 新设置项
- 新配置字段
- 新按钮
- 新状态来源
- 新模拟事件
- 新复制功能
- 新日志系统
- 新主题切换逻辑
- 新状态切换器
- 新文件选择器
- 新自动检测逻辑

禁止修改：

- Codex 状态来源
- Claude Code 状态来源
- 状态文件读取逻辑
- hook 负载
- 状态判断逻辑
- 托盘逻辑
- Win32 原生组件逻辑
- 任务栏挂载逻辑
- 配置文件 schema
- Rust 业务模型，除非只是为了适配已有字段显示

---

# 2. HTML Demo 使用规则

## 2.1 HTML Demo 的定位

`docs/ui/cc_traffic_light_nothing_demo_strict.html` 只是视觉参考。

它不是功能需求文档，不是 Slint 实现方案，不是业务逻辑来源，也不是配置字段来源。

## 2.2 可以参考 HTML Demo 的内容

Codex 可以参考：

- 页面布局
- 左侧编号导航
- 色彩风格
- 字号层级
- 间距
- 圆角
- 边框
- 状态点
- 状态标签
- 设置行样式
- 诊断记录样式
- 关于页设备铭牌风格
- 黑白工业控制台气质

## 2.3 不允许参考 HTML Demo 新增的内容

Codex 不允许因为 HTML demo 而新增：

- 新页面
- 新设置项
- 新配置项
- 新业务字段
- 新按钮
- 新状态切换器
- 新模拟事件
- 新复制功能
- 新日志系统
- 新主题切换逻辑
- 新数据结构
- 新 JS 交互对应的 Slint 功能

## 2.4 参考优先级

遇到冲突时，按以下优先级判断：

```text
1. 当前项目源码
   决定真实功能、数据结构、配置字段、回调逻辑。

2. 当前上传截图
   决定当前已有页面和已有功能边界。

3. docs/ui/cc_traffic_light_nothing_demo_strict.html
   只决定视觉方向、布局气质和组件外观。

4. docs/ui/nothing-signal-console-spec.md
   作为视觉规则和组件规范补充。
```

如果 HTML demo 和当前源码 / 截图不一致，以源码和截图为准。

一句话原则：

> HTML demo = 视觉参考  
> 当前源码 + 当前截图 = 功能边界

---

# 3. 总体改造边界 Checklist

开始前必须确认：

- [x] 不新增页面
- [x] 不新增设置项
- [x] 不新增配置字段
- [x] 不新增模拟事件
- [x] 不新增复制按钮
- [x] 不新增状态切换器
- [x] 不新增额外日志系统
- [x] 不新增文件选择器
- [x] 不新增主题切换业务逻辑
- [x] 不改变配置文件结构
- [x] 不改变 Codex / Claude 状态来源
- [x] 不改变状态文件读取逻辑
- [x] 不改变 hook 负载
- [x] 不改变状态判断逻辑
- [x] 不改变 Win32 原生组件逻辑
- [x] 不改变托盘逻辑
- [x] 不改变任务栏挂载逻辑
- [x] 不改变 Rust 端业务模型，除非只是适配已有 UI 字段显示
- [x] 所有原有交互继续可用
- [x] 页面切换继续可用
- [x] 设置开关继续可用
- [x] 监听开关继续可用
- [x] 立即刷新继续可用

---

# 4. 推荐执行顺序

建议按小步推进：

```text
Step 1  审计现有 UI 结构
Step 2  建立全局 Design Tokens
Step 3  抽公共 UI 组件
Step 4  重构左侧导航
Step 5  重构总览页
Step 6  重构通用页
Step 7  重构监听页
Step 8  重构外观页
Step 9  重构诊断页
Step 10 重构关于页
Step 11 最终视觉回归和功能回归
```

不要一次性重构所有页面。

UI 重构要像修仪表盘，先校准刻度，再换面板，最后拧螺丝。不要一榔头把整台设备敲成未来主义煎饼。

---

# 5. Step 1：审计现有 Slint UI 结构

## 5.1 目标

先理解项目，不修改代码。

## 5.2 Checklist

- [x] 找到 Slint UI 入口文件
- [x] 找到主窗口组件
- [x] 找到左侧导航实现
- [x] 找到页面切换逻辑
- [x] 找到 6 个页面的实现位置
- [x] 找到总览页状态字段绑定
- [x] 找到通用页设置开关绑定
- [x] 找到监听页开关绑定
- [x] 找到外观页字段绑定
- [x] 找到诊断页刷新按钮 callback
- [x] 找到关于页字段来源
- [x] 找到当前颜色、字号、间距定义方式
- [x] 判断是否已有公共组件文件
- [x] 判断最适合新增 theme 文件的位置
- [x] 判断最适合新增 components 文件的位置
- [x] 输出改造计划
- [x] 明确哪些文件可以改
- [x] 明确哪些文件不要碰

## 5.3 审计 Prompt

```md
请只审计当前 Rust + Slint UI 结构，不要修改代码。

目标是为后续 Nothing Signal Console 风格重构做准备。

请输出：

1. Slint UI 入口文件在哪里。
2. 主窗口组件在哪里。
3. 左侧导航在哪里。
4. 总览 / 通用 / 监听 / 外观 / 诊断 / 关于 6 个页面分别在哪里。
5. 当前颜色、字号、间距是否有集中定义。
6. 当前设置项开关如何绑定。
7. 当前 Codex / Claude 状态如何传入 UI。
8. “立即刷新”按钮如何绑定。
9. 哪些文件适合新增 theme tokens。
10. 哪些文件适合新增公共 UI 组件。
11. 推荐的分阶段改造顺序。
12. 哪些文件不要碰。

请同时检查：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但注意：这个 HTML demo 不是要迁移成 Slint 的功能代码。

请只从中提取视觉参考：

- 布局结构
- 色彩
- 字体层级
- 间距
- 圆角
- 边框
- 导航样式
- 信息行样式
- 状态组件样式

不要把 HTML demo 里的 JS 交互、DOM 状态、演示逻辑当成项目需求。

重要约束：

这次任务是视觉重构，不是功能扩展。
请不要新增当前项目没有的功能。
请不要修改 Rust 业务逻辑。
在完成审计和计划前，不要修改代码。
```

---

# 6. Step 2：建立全局 Design Tokens

## 6.1 目标

只建立全局样式参数，不重构页面。

建议新增或整理：

```text
theme.slint
```

或使用项目已有公共样式文件。

## 6.2 颜色 Token Checklist

- [x] App 背景：暖白 / 浅米灰
- [x] Panel 背景：近白
- [x] Soft Panel 背景：浅暖白
- [x] 主文字：近黑
- [x] 次文字：中灰
- [x] 弱文字：浅灰
- [x] 默认边框：浅灰
- [x] 强边框：近黑
- [x] 状态 ok：绿色
- [x] 状态 warn：黄色
- [x] 状态 error：红色
- [x] 状态 idle：灰色

推荐语义：

```text
color-bg-app
color-bg-panel
color-bg-panel-soft
color-text-main
color-text-sub
color-text-muted
color-border
color-border-strong
color-status-ok
color-status-warn
color-status-error
color-status-idle
```

## 6.3 字号 Token Checklist

- [x] Display
- [x] Page Title
- [x] Section Title
- [x] Body
- [x] Caption
- [x] Micro

## 6.4 间距 Token Checklist

- [x] 4px
- [x] 8px
- [x] 12px
- [x] 16px
- [x] 20px
- [x] 24px
- [x] 32px
- [x] 40px

## 6.5 圆角 Token Checklist

- [x] radius-xs
- [x] radius-sm
- [x] radius-md
- [x] radius-lg
- [x] radius-pill

## 6.6 实施 Checklist

- [x] 选择最少侵入的位置定义 token
- [x] 优先复用项目已有样式系统
- [x] 不直接重构页面
- [x] 不修改业务逻辑
- [x] 不新增功能
- [x] 编译通过

## 6.7 Step Prompt

```md
现在开始 Step 2：定义全局 UI 样式 token。

请只做全局样式参数，不要重构具体页面布局。

目标：

为 Nothing Signal Console 风格建立统一颜色、字号、圆角、间距。

请根据现有 Slint 项目结构，选择合适位置新增或整理一个公共 theme 文件，例如：

- ui/theme.slint
- src/ui/theme.slint
- 或项目现有的公共 style 文件

如果项目已有类似主题文件，请优先复用，不要重复造多个主题系统。

需要定义的 token 包括：

颜色：

- app background：暖白 / 浅米灰
- panel background：近白
- soft panel background：浅暖白
- main text：近黑
- secondary text：中灰
- muted text：浅灰
- border：浅灰
- strong border：近黑
- status ok：绿色
- status warn：黄色
- status error：红色
- status idle：灰色

形状：

- small radius
- medium radius
- large radius
- pill radius

间距：

- 4 / 8 / 12 / 16 / 20 / 24 / 32 / 40

字号：

- display
- page title
- section title
- body
- caption
- micro

要求：

1. 不改变页面结构。
2. 不改变业务逻辑。
3. 不改变数据模型。
4. 不新增任何设置项。
5. 不新增任何页面。
6. 修改后必须能编译。
7. 如果 Slint 不支持某种全局 token 写法，请用当前项目最自然、最少侵入的方式实现。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

完成后请输出：

- 修改了哪些文件
- 新增了哪些 token
- 如何验证编译通过
```

---

# 7. Step 3：抽公共 UI 组件

## 7.1 目标

抽出可复用 Slint 组件，为后续页面重构做准备。

不要一次性重写所有页面。

## 7.2 推荐组件

- [x] AppShell
- [x] SideNavItem
- [x] Panel
- [x] SectionHeader
- [x] InfoRow
- [x] SettingToggleRow
- [x] StatusDot
- [x] StatusBadge
- [x] AgentStatusCard
- [x] DiagnosticEntry

## 7.3 组件原则

- [x] 组件只负责 UI 展示
- [x] 组件只转发现有交互
- [x] 组件不读取配置文件
- [x] 组件不写配置文件
- [x] 组件不读取状态文件
- [x] 组件不改 Rust 业务逻辑
- [x] 组件通过 property 接收数据
- [x] 组件通过 callback 转发点击
- [x] 保持现有开关逻辑
- [x] 保持现有按钮逻辑
- [x] 不新增功能

## 7.4 Step Prompt

```md
现在开始 Step 3：抽公共 UI 组件。

请基于已经建立的 theme tokens，新增或整理公共 Slint 组件。

目标是为后续页面重构提供复用组件，不要一次性重写所有页面。

需要优先实现这些组件，名称可以根据项目风格调整：

1. AppShell
2. SideNavItem
3. Panel
4. SectionHeader
5. InfoRow
6. SettingToggleRow
7. StatusDot
8. StatusBadge
9. AgentStatusCard
10. DiagnosticEntry

组件要求：

- 只负责 UI 展示和基础交互转发。
- 不读取配置文件。
- 不写配置文件。
- 不读取状态文件。
- 不改变 Rust 业务逻辑。
- 通过 property 接收现有数据。
- 通过 callback 转发现有点击事件。
- 保留现有开关交互能力。
- 保留现有按钮点击能力。

视觉要求：

- 黑白灰为主。
- 状态色只用于状态点 / 状态标签。
- 使用小圆角、细边框。
- 英文标签使用较小字号和等宽风格。
- 不使用大面积阴影。
- 不使用深色主题。
- 不新增图标库。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

不要照搬 HTML demo 的 JS 交互。
不要新增当前项目没有的功能。

完成后请输出：

- 新增 / 修改了哪些组件
- 每个组件的 property / callback 简要说明
- 哪些页面后续会用到它们
- 编译验证结果
```

---

# 8. Step 4：重构左侧导航

## 8.1 目标

重构左侧导航和主窗口外壳，先立住整体气质。

## 8.2 必须保留的页面

- [x] 总览 / OVERVIEW
- [x] 通用 / GENERAL
- [x] 监听 / SOURCES
- [x] 外观 / APPEARANCE
- [x] 诊断 / DIAGNOSTICS
- [x] 关于 / ABOUT

## 8.3 视觉 Checklist

- [x] 左侧顶部有品牌区
- [x] 品牌区显示“信号控制台”或“CC TRAFFIC LIGHT”
- [x] 品牌区辅助说明短，不要长段落
- [x] 导航项使用 01 到 06 编号
- [x] 当前页黑底白字
- [x] 未选中项浅底黑边
- [x] 中文主标题 + 英文小标签
- [x] 圆角减小
- [x] 间距规整
- [x] 页面切换逻辑不变

## 8.4 禁止项

- [x] 不新增页面
- [x] 不新增导航项
- [x] 不新增动画系统
- [x] 不修改状态读取逻辑
- [x] 不修改托盘逻辑
- [x] 不修改 Win32 逻辑

## 8.5 Step Prompt

```md
现在开始 Step 4：重构左侧导航。

只改左侧导航和主窗口外壳，不要重构 6 个页面内容。

目标：

让左侧导航接近 docs/ui/cc_traffic_light_nothing_demo_strict.html 的 Nothing Signal Console 风格。

必须保留现有 6 个页面和页面切换逻辑：

1. 总览 / OVERVIEW
2. 通用 / GENERAL
3. 监听 / SOURCES
4. 外观 / APPEARANCE
5. 诊断 / DIAGNOSTICS
6. 关于 / ABOUT

视觉要求：

- 左侧顶部是品牌区。
- 品牌区显示“信号控制台”或“CC TRAFFIC LIGHT”。
- 可添加很短的辅助说明，例如“LOCAL SIGNAL CONSOLE”或“Win32 + Slint”。
- 导航项使用编号 01 到 06。
- 当前选中项黑底白字。
- 未选中项使用浅色背景、黑色细边框。
- 圆角不要过大。
- 间距更规整。
- 不新增任何导航项。
- 不改变页面切换状态枚举，除非只是重命名 UI 内部显示，不影响 Rust 端逻辑。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增页面。
- 不要新增设置项。
- 不要新增动画系统。
- 不要修改状态读取逻辑。
- 不要修改托盘和 Win32 逻辑。

完成后请运行编译检查，并输出：

- 修改了哪些文件
- 页面切换是否仍然正常
- 是否有任何业务逻辑改动，如果有请撤回
```

---

# 9. Step 5：重构总览页

## 9.1 目标

把总览页从普通卡片改成状态仪表盘。

## 9.2 必须保留的信息

- [x] 整体状态
- [x] Codex 状态
- [x] Claude 状态
- [x] 组件挂载状态
- [x] 最近挂载 / 最近更新时间
- [x] 可信度
- [x] session id，如果当前已有显示

## 9.3 推荐模块

- [x] SIGNAL SUMMARY
- [x] AGENT MATRIX
- [x] MOUNT STATUS

## 9.4 视觉 Checklist

- [x] 大状态文字更清晰
- [x] Codex / Claude 视觉结构统一
- [x] 状态色只用于状态点 / 状态标签
- [x] 长时间戳降低视觉权重
- [x] session id 默认截断
- [x] 卡片不再巨大空旷
- [x] 信息密度提高但不拥挤

## 9.5 禁止项

- [x] 不新增模拟事件按钮
- [x] 不新增状态切换器
- [x] 不新增复制按钮
- [x] 不新增日志
- [x] 不改 Rust 状态判断逻辑
- [x] 不改状态文件读取逻辑

## 9.6 Step Prompt

```md
现在开始 Step 5：重构“总览”页面。

只改总览页 UI，不要改其他页面。

目标：

把总览页从普通卡片布局改成 Nothing Signal Console 风格的状态仪表盘。

必须保留现有信息范围，只展示当前项目已有的数据：

- 整体状态
- Codex 状态
- Claude 状态
- 组件挂载状态
- 最近挂载 / 最近更新时间
- 可信度
- session id，如果当前已有显示，可以截断显示

不要新增任何新的状态、按钮、模拟事件或配置项。

布局建议：

1. SIGNAL SUMMARY
   - 显示整体状态，例如“待处理”
   - 显示 Codex / Claude 简要状态
   - 显示最近更新时间

2. AGENT MATRIX
   - Codex 一行或一张 Agent card
   - Claude 一行或一张 Agent card
   - 显示状态、来源、可信度、更新时间

3. MOUNT STATUS
   - 显示组件是否已挂载
   - 显示最近挂载时间

视觉要求：

- 大状态文字使用黑色或状态小色点，不要整块染色。
- Codex / Claude 卡片样式统一。
- 长时间戳尽量格式化为人类可读时间；如果当前只有毫秒时间戳，至少降低其视觉权重。
- session id 默认截断，例如 codex_019f20fc...aa3c。
- 状态色只用于点、标签或少量文字。
- 不新增交互。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增“模拟事件”按钮。
- 不要新增状态切换器。
- 不要新增复制按钮。
- 不要新增日志。
- 不要改 Rust 状态判断逻辑。
- 不要改状态文件读取逻辑。

完成后请运行编译检查，并输出：

- 总览页使用了哪些公共组件
- 原有信息是否全部保留
- 是否有新增功能，如果有请撤回
```

---

# 10. Step 6：重构通用页

## 10.1 目标

把通用页改成系统行为配置面板。

## 10.2 必须保留的设置项

- [x] 登录时启动
- [x] 启动时最小化到托盘
- [x] 关闭窗口时仅缩到托盘
- [x] 语言

## 10.3 推荐英文 Key

- [x] START_ON_LOGIN
- [x] MINIMIZE_ON_START
- [x] CLOSE_TO_TRAY
- [x] LANGUAGE_MODE

## 10.4 视觉 Checklist

- [x] 使用 SettingToggleRow / InfoRow
- [x] 行高统一
- [x] 右侧显示 ON / OFF 或当前值
- [x] 左侧中文标题
- [x] 小号等宽英文 key
- [x] 开关绑定保持不变
- [x] 语言保持原行为

## 10.5 禁止项

- [x] 不新增设置项
- [x] 不新增配置字段
- [x] 不新增语言列表
- [x] 不改变设置保存逻辑
- [x] 不改变开关 callback 语义

## 10.6 Step Prompt

```md
现在开始 Step 6：重构“通用”页面。

只改通用页 UI，不要改其他页面。

必须保留现有设置项，不新增、不删除：

- 登录时启动
- 启动时最小化到托盘
- 关闭窗口时仅缩到托盘
- 语言

目标：

把通用页改成 Nothing Signal Console 风格的系统行为配置面板。

布局建议：

Section: SYSTEM BEHAVIOR

每个设置项使用 SettingToggleRow 或 InfoRow：

- 中文名称
- 英文 key
- 当前值
- 现有开关或显示值

英文 key 建议：

- START_ON_LOGIN
- MINIMIZE_ON_START
- CLOSE_TO_TRAY
- LANGUAGE_MODE

视觉要求：

- 行高统一。
- 底部分隔线。
- 左侧中文标题。
- 下方或旁边显示小号等宽英文 key。
- 右侧显示 ON / OFF 或当前值。
- 开关可以使用现有开关逻辑，但视觉可以改成 ON ● / OFF ○ 风格。
- 语言只显示当前已有值，不要新增语言选择功能，除非原项目已经有。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增设置项。
- 不要新增配置字段。
- 不要新增语言列表。
- 不要改变设置保存逻辑。
- 不要改变开关 callback 的语义。

完成后请运行编译检查，并输出：

- 每个设置项是否仍然绑定原来的状态和 callback
- 是否改变了配置 schema，如果改变了请撤回
```

---

# 11. Step 7：重构监听页

## 11.1 目标

把监听页改成 SOURCE MATRIX 风格。

## 11.2 必须保留的设置项

- [x] 监听 Codex
- [x] 监听 Claude Code

## 11.3 视觉 Checklist

- [x] 使用 SettingToggleRow / InfoRow
- [x] Codex 和 Claude Code 结构统一
- [x] 当前打开显示 ON ●
- [x] 当前关闭显示 OFF ○
- [x] 说明文字缩短
- [x] 细边框和分隔线
- [x] 原开关绑定保持不变

## 11.4 禁止项

- [x] 不新增监听来源
- [x] 不新增文件路径选择
- [x] 不新增状态测试按钮
- [x] 不新增自动检测功能
- [x] 不改变监听开关保存逻辑

## 11.5 Step Prompt

```md
现在开始 Step 7：重构“监听”页面。

只改监听页 UI，不要改其他页面。

必须保留现有设置项，不新增、不删除：

- 监听 Codex
- 监听 Claude Code

目标：

把监听页改成 Nothing Signal Console 风格的 SOURCE MATRIX。

布局建议：

Section: SOURCE MATRIX

每个来源一行或一个小面板：

- 来源名称：Codex / Claude Code
- 是否监听：ON / OFF
- 当前项目已有的说明文字可以保留，但要简短
- 不要新增来源类型、路径、可信度等新字段，除非当前页面或数据模型已经有

视觉要求：

- 使用统一 InfoRow / SettingToggleRow。
- 保留原来的开关绑定。
- 当前打开显示 ON ●。
- 当前关闭显示 OFF ○。
- 使用细边框和分隔线。
- 不要做成普通表格。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增监听来源。
- 不要新增文件路径选择。
- 不要新增状态测试按钮。
- 不要新增自动检测功能。
- 不要改变监听开关保存逻辑。

完成后请运行编译检查，并输出：

- Codex / Claude Code 两个开关是否仍然可用
- 是否有任何业务逻辑改动，如果有请撤回
```

---

# 12. Step 8：重构外观页

## 12.1 目标

把外观页改成 DISPLAY SURFACE 配置面板。

## 12.2 必须保留的项目

- [x] 界面主题
- [x] 指示器样式
- [x] 组件尺寸
- [x] 显示标签
- [x] 减少动效

## 12.3 推荐英文 Key

- [x] THEME_MODE
- [x] INDICATOR_STYLE
- [x] COMPONENT_SIZE
- [x] SHOW_LABELS
- [x] REDUCE_MOTION

## 12.4 视觉 Checklist

- [x] 使用 InfoRow / SettingToggleRow
- [x] 当前值显示在右侧
- [x] 行间细分隔线
- [x] 说明文字缩短
- [x] 原本可点的继续可点
- [x] 原本只读的继续只读
- [x] 不把显示项擅自变成交互项

## 12.5 禁止项

- [x] 不新增外观配置项
- [x] 不新增主题业务功能
- [x] 不新增配置字段
- [x] 不改变配置文件结构
- [x] 不新增深色主题切换逻辑
- [x] 不新增点阵密度滑杆
- [x] 不新增预览面板

## 12.6 Step Prompt

```md
现在开始 Step 8：重构“外观”页面。

只改外观页 UI，不要改其他页面。

必须保留现有项目，不新增、不删除：

- 界面主题
- 指示器样式
- 组件尺寸
- 显示标签
- 减少动效

目标：

把外观页改成 Nothing Signal Console 风格的 DISPLAY SURFACE 配置面板。

注意：

截图里外观页的这些项目大多是显示当前值，不一定都有真实交互。请保持现有行为。原来能点的继续能点，原来只是显示的不要擅自改成可编辑控件。

建议英文 key：

- THEME_MODE
- INDICATOR_STYLE
- COMPONENT_SIZE
- SHOW_LABELS
- REDUCE_MOTION

视觉要求：

- 使用 InfoRow / SettingToggleRow。
- 当前值在右侧。
- 行之间用细分隔线。
- 保留“这些设置只影响本地显示层，不会修改外部工具或 hook 负载”这类说明，但缩短。
- 不要新增深色主题切换逻辑。
- 不要新增点阵密度滑杆。
- 不要新增预览面板。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增外观配置项。
- 不要新增主题系统的业务功能。
- 不要新增配置字段。
- 不要改变配置文件结构。
- 不要把显示项改成真实可交互项，除非原项目已经支持。

完成后请运行编译检查，并输出：

- 哪些项是只读显示
- 哪些项是可交互开关
- 是否保持了原行为
```

---

# 13. Step 9：重构诊断页

## 13.1 目标

把诊断页改成只读检测记录面板。

## 13.2 必须保留的信息

- [x] 最近刷新
- [x] 最近错误
- [x] Codex 检测依据
- [x] Codex 可信度
- [x] Codex 更新时间
- [x] Codex session id，如果当前已有
- [x] Claude 检测依据
- [x] Claude 可信度
- [x] Claude 更新时间
- [x] 立即刷新按钮

## 13.3 推荐模块

- [x] LATEST CHECK
- [x] SIGNAL TRACE
- [x] REFRESH ACTION

## 13.4 视觉 Checklist

- [x] 不再大段自然语言堆叠
- [x] 使用类似日志行的结构
- [x] 时间 / 来源 / 依据 / 可信度 / 状态
- [x] 长 session id 截断
- [x] 时间戳降低视觉权重
- [x] 立即刷新按钮保留原 callback
- [x] 诊断页仍然只读

## 13.5 禁止项

- [x] 不新增日志系统
- [x] 不保存诊断历史
- [x] 不新增清空日志按钮
- [x] 不新增复制按钮
- [x] 不改变刷新逻辑
- [x] 不改变状态读取逻辑

## 13.6 Step Prompt

```md
现在开始 Step 9：重构“诊断”页面。

只改诊断页 UI，不要改其他页面。

必须保留现有信息范围：

- 最近刷新
- 最近错误
- Codex 检测依据 / 可信度 / 更新时间 / session id
- Claude 检测依据 / 可信度 / 更新时间
- 立即刷新按钮

目标：

把诊断页改成 Nothing Signal Console 风格的只读检测记录面板。

布局建议：

1. LATEST CHECK
   - 最近刷新
   - 最近错误

2. SIGNAL TRACE
   - Codex 一行
   - Claude 一行

3. 底部保留“立即刷新”按钮

视觉要求：

- 不要大段自然语言堆叠。
- 使用类似日志行的结构：
  时间 / 来源 / 依据 / 可信度 / 状态
- 长 session id 默认截断。
- 时间戳降低视觉权重。
- 立即刷新按钮保留现有 callback。
- 按钮可以改成黑白胶囊边框风格。
- 诊断页仍然是只读显示。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增日志系统。
- 不要保存诊断历史。
- 不要新增清空日志按钮。
- 不要新增复制按钮。
- 不要改变刷新逻辑。
- 不要改变状态读取逻辑。

完成后请运行编译检查，并输出：

- 立即刷新按钮是否仍然触发原 callback
- 诊断信息是否都来自原数据
- 是否新增了任何诊断功能，如果有请撤回
```

---

# 14. Step 10：重构关于页

## 14.1 目标

把关于页改成 DEVICE SPEC 设备铭牌页。

## 14.2 必须保留字段

- [x] 版本
- [x] 运行时
- [x] 配置路径
- [x] 语言模式
- [x] 页面说明文字

## 14.3 推荐字段显示

- [x] PRODUCT：CC TRAFFIC LIGHT
- [x] VERSION：0.1.0
- [x] RUNTIME：Win32 组件 + 托盘，Slint 设置窗口
- [x] CONFIG：当前配置路径
- [x] LANGUAGE：跟随系统

## 14.4 视觉 Checklist

- [x] 像设备参数铭牌
- [x] 使用 InfoRow
- [x] 字段 key 使用等宽英文
- [x] 中文值保留
- [x] 配置路径控制字号和换行
- [x] 页面说明文字缩短
- [x] 不加入营销文案

## 14.5 禁止项

- [x] 不新增官网按钮
- [x] 不新增 GitHub 按钮
- [x] 不新增 License 按钮
- [x] 不新增复制路径按钮
- [x] 不新增检查更新
- [x] 不新增版本检测功能

## 14.6 Step Prompt

```md
现在开始 Step 10：重构“关于”页面。

只改关于页 UI，不要改其他页面。

必须保留现有字段：

- 版本
- 运行时
- 配置路径
- 语言模式
- 页面说明文字

目标：

把关于页改成 Nothing Signal Console 风格的 DEVICE SPEC 设备铭牌页。

布局建议：

Section: DEVICE SPEC

字段显示为参数行：

- PRODUCT       CC TRAFFIC LIGHT
- VERSION       0.1.0
- RUNTIME       Win32 组件 + 托盘，Slint 设置窗口
- CONFIG        当前配置路径
- LANGUAGE      跟随系统

视觉要求：

- 像设备参数铭牌。
- 使用 InfoRow。
- 字段 key 使用等宽英文。
- 中文值保留。
- 配置路径可以显示完整路径，但要控制字号和换行，不要撑破布局。
- 页面说明文字保留但缩短。

请同时参考：

docs/ui/cc_traffic_light_nothing_demo_strict.html

但它只作为视觉参考，不作为功能需求。

禁止：

- 不要新增官网、GitHub、License 按钮。
- 不要新增复制配置路径按钮。
- 不要新增检查更新。
- 不要新增版本检测功能。

完成后请运行编译检查，并输出：

- 原字段是否全部保留
- 是否有新增功能，如果有请撤回
```

---

# 15. Step 11：最终视觉回归和功能回归

## 15.1 页面存在性检查

- [x] 总览还在
- [x] 通用还在
- [x] 监听还在
- [x] 外观还在
- [x] 诊断还在
- [x] 关于还在

## 15.2 原有交互检查

- [x] 左侧页面切换正常
- [x] 通用页开关正常
- [x] 监听页开关正常
- [x] 外观页原本可交互项正常
- [x] 诊断页立即刷新正常
- [x] 窗口关闭 / 托盘行为正常
- [x] Win32 原生组件行为正常
- [x] 状态读取正常

## 15.3 禁止新增功能检查

确认没有新增：

- [x] 模拟事件
- [x] 状态切换器
- [x] 复制按钮
- [x] 点阵密度调节
- [x] 深色主题实际切换
- [x] 诊断历史日志系统
- [x] 新配置项
- [x] 新页面
- [x] 新状态来源
- [x] 新文件选择器
- [x] 新测试按钮

## 15.4 业务逻辑回归检查

确认没有误改：

- [x] Codex 状态来源
- [x] Claude Code 状态来源
- [x] 状态文件读取
- [x] hook 逻辑
- [x] 托盘逻辑
- [x] Win32 挂载逻辑
- [x] 配置保存 schema
- [x] Rust 状态模型
- [x] 任务栏组件逻辑

## 15.5 HTML Demo 视觉还原检查

- [x] 左侧导航接近 HTML demo 的编号导航风格
- [x] 主背景接近暖白 / 浅米灰
- [x] 主面板接近近白底 + 浅灰边框
- [x] 当前选中导航是黑底白字
- [x] 设置行接近 demo 的 InfoRow / ToggleRow
- [x] 状态点和状态标签统一
- [x] 诊断页接近 demo 的只读信号记录样式
- [x] 关于页接近 demo 的设备铭牌样式
- [x] 没有照搬 HTML demo 的 JS 交互
- [x] 没有新增 HTML demo 中演示用但当前项目没有的功能
- [x] 没有为了还原 demo 而改 Rust 业务逻辑

## 15.6 视觉统一检查

- [x] 使用统一颜色 token
- [x] 使用统一字号
- [x] 使用统一边框
- [x] 使用统一圆角
- [x] 使用统一间距
- [x] 使用统一 InfoRow
- [x] 使用统一 SettingToggleRow
- [x] 使用统一 StatusDot
- [x] 使用统一 StatusBadge
- [x] 状态色只用于状态点或状态标签
- [x] 长 ID 已截断显示
- [x] 时间戳不再高权重裸奔
- [x] 大面积卡片不再空旷
- [x] 不使用大面积阴影
- [x] 不使用赛博霓虹风

## 15.7 检查命令

根据项目情况运行：

```bash
cargo check
cargo test
cargo fmt
```

如果项目有额外命令，也运行：

```bash
cargo clippy
```

或项目 README / CI 中定义的检查命令。

## 15.8 最终回归 Prompt

```md
现在开始最终视觉回归和清理。

请整体检查 UI 重构结果。

目标：

确保当前 Rust + Slint 设置窗口尽量接近 docs/ui/cc_traffic_light_nothing_demo_strict.html 的 Nothing Signal Console 风格，同时没有新增原项目不存在的功能。

请检查：

1. 6 个页面是否都还在：
   - 总览
   - 通用
   - 监听
   - 外观
   - 诊断
   - 关于

2. 原有交互是否还在：
   - 页面切换
   - 通用页开关
   - 监听页开关
   - 外观页原本已有交互
   - 诊断页立即刷新

3. 是否误新增了功能：
   - 模拟事件
   - 状态切换器
   - 复制按钮
   - 点阵密度调节
   - 深色主题实际切换
   - 日志历史系统
   - 新配置项
   - 新状态来源
   - 新文件选择器
   - 新测试按钮

4. 是否误改了业务逻辑：
   - Codex 状态来源
   - Claude Code 状态来源
   - 状态文件读取
   - hook 逻辑
   - 托盘逻辑
   - Win32 挂载逻辑
   - 配置保存 schema

5. HTML demo 是否只被用于视觉参考：
   - 没有照搬 JS 交互
   - 没有新增 demo 中的演示功能
   - 没有为了还原 demo 而改变业务字段

6. 视觉是否统一：
   - 使用统一颜色 token
   - 使用统一字号
   - 使用统一边框
   - 使用统一圆角
   - 使用统一 InfoRow / SettingToggleRow / StatusBadge / StatusDot
   - 状态色只用于状态点或状态标签
   - 长 ID 已截断显示
   - 时间戳不再高权重裸奔

请运行所有可用检查：

- cargo check
- cargo test，如果项目有测试
- cargo fmt，如果适合
- 其他项目已有检查命令

完成后输出：

1. 最终修改文件列表。
2. 是否通过编译。
3. 是否通过测试。
4. 是否有任何业务逻辑改动。
5. 是否有任何新增功能。
6. 如果有偏离 strict HTML demo 的地方，说明原因。
```

---

# 16. 每轮 Prompt 末尾固定追加

建议每次发给 Codex 时，都在最后追加这段：

```md
重要约束：

这次任务是视觉重构，不是功能扩展。

请不要新增原截图和当前项目里不存在的功能。
请不要新增配置项。
请不要新增业务状态。
请不要改变状态来源。
请不要改变 Rust 业务逻辑。
请不要为了还原 HTML demo 而创造新的交互。

HTML demo 只是视觉参考，当前项目源码和截图才是功能边界。
```

---

# 17. 最推荐的执行节奏

最稳妥：

```text
第 1 轮：审计现有 UI 结构
第 2 轮：建立 theme token
第 3 轮：抽公共组件
第 4 轮：重构左侧导航
第 5 轮：重构总览页
第 6 轮：重构通用页
第 7 轮：重构监听页
第 8 轮：重构外观页
第 9 轮：重构诊断页
第 10 轮：重构关于页
第 11 轮：最终回归
```

省额度但风险略高：

```text
第 1 轮：审计 + 计划
第 2 轮：theme token + 公共组件
第 3 轮：左侧导航 + 总览页
第 4 轮：通用 + 监听 + 外观
第 5 轮：诊断 + 关于 + 回归
```

推荐第一种。

UI 重构最怕“一口气全改”。  
尤其 Slint + Rust + Win32 这种组合，应该小步推进，每一步都编译检查。

---

# 18. 最终验收标准

## 18.1 成功标准

重构完成后应满足：

- [x] 整体不再像普通后台设置页
- [x] 左侧导航具有编号控制台风格
- [x] 页面统一使用黑白灰 + 少量状态色
- [x] 状态表达统一
- [x] 设置行统一
- [x] 诊断页更像只读信号记录
- [x] 关于页更像设备铭牌
- [x] 页面信息密度提高
- [x] 没有新增业务功能
- [x] 没有改变状态来源
- [x] 没有改变配置 schema
- [x] 编译通过
- [x] 原有交互正常

## 18.2 失败信号

出现以下情况，应立即暂停并回滚相关改动：

- [ ] 新增了 HTML demo 里的演示交互
- [ ] 新增了配置字段
- [ ] 改了状态读取逻辑
- [ ] 改了托盘或 Win32 挂载逻辑
- [ ] 页面切换失效
- [ ] 设置保存失效
- [ ] 编译失败但改动范围很大
- [ ] 样式散落在各页面，未组件化
- [ ] 状态色被用于大面积背景
- [ ] UI 变成赛博霓虹风或后台管理系统风

---

# 19. 一句话总原则

> 当前源码和截图决定“能做什么”。  
> HTML demo 和设计规格决定“长什么样”。  
> 本次重构只换外壳和仪表盘，不改发动机。
