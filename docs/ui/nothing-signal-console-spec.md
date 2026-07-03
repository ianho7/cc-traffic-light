# CC Traffic Light · Nothing Signal Console Spec

## 1. 目标

将当前 Rust + Slint 设置窗口，从普通设置面板改造成浅色 Nothing 工业控制台风格。

核心目标：

- 只做 UI / 样式 / 布局 / 组件结构重构
- 不新增业务功能
- 不新增配置项
- 不改变 Codex / Claude 状态来源
- 不改变 Win32、托盘、hook、任务栏挂载逻辑
- 参考 `cc_traffic_light_nothing_demo_strict.html` 的视觉方向

一句话：

> 当前源码和截图决定“能做什么”，HTML demo 和本文档决定“长什么样”。

---

## 2. 视觉定位

风格名称：

> Nothing Signal Console

关键词：

- 浅色
- 黑白灰
- 工业控制台
- 编号导航
- 等宽信息
- 细边框
- 小圆角
- 状态灯
- 克制
- 本地工具感

不要做成：

- 后台管理系统
- 赛博霓虹风
- 大面积深色主题
- 营销官网风
- 花哨动效 UI

---

## 3. 色彩规范

### 3.1 基础颜色

建议语义 token：

```text
color-bg-app          #F4F1EA
color-bg-panel        #FAF9F5
color-bg-panel-soft   #F7F5EF
color-text-main       #111111
color-text-sub        #666666
color-text-muted      #8A8A8A
color-border          #D8D2C6
color-border-strong   #111111
color-black           #111111
color-white           #FFFFFF
```

### 3.2 状态颜色

状态色只允许用于状态点、状态标签、少量状态文字。

```text
color-status-ok       #3BA55D
color-status-warn     #D6A439
color-status-error    #D64A3A
color-status-idle     #9A9A9A
```

禁止：

- 把整张卡片染成绿色 / 黄色 / 红色
- 大面积状态色背景
- 霓虹发光
- 多余渐变

---

## 4. 字体规范

### 4.1 中文 / 常规 UI

使用系统字体即可：

```text
Microsoft YaHei UI / Segoe UI / system default
```

### 4.2 英文 / 数字 / ID / 时间

优先使用等宽字体：

```text
Cascadia Mono / Consolas / monospace
```

### 4.3 字号层级

```text
display       44px ~ 52px
page-title    28px ~ 32px
section-title 13px ~ 15px
body          15px ~ 17px
caption       12px ~ 13px
micro         10px ~ 11px
```

---

## 5. 间距规范

使用 4px 基准网格：

```text
space-1   4px
space-2   8px
space-3   12px
space-4   16px
space-5   20px
space-6   24px
space-8   32px
space-10  40px
space-12  48px
```

推荐：

- 页面外边距：28px ~ 32px
- 主面板内边距：28px ~ 36px
- Section 间距：24px ~ 32px
- 信息行高度：56px ~ 68px
- 导航项高度：64px ~ 72px

---

## 6. 圆角规范

当前 UI 圆角偏软，V1 应更硬朗。

```text
radius-xs   4px
radius-sm   8px
radius-md   12px
radius-lg   16px
radius-pill 999px
```

推荐：

- 主面板：16px
- 普通区块：12px
- 导航项：12px
- Badge：6px ~ 8px
- 状态点：999px

避免大量 20px+ 圆角。

---

## 7. 边框与阴影

### 7.1 边框

主要靠边框建立结构。

```text
default border: 1px solid color-border
strong border:  1px / 2px solid color-border-strong
```

### 7.2 阴影

V1 尽量不用阴影。

如果必须使用，只能非常轻。Slint 里可以直接不做阴影。

---

## 8. 布局规范

保持双栏结构：

```text
┌──────────────────────────────────────────────┐
│ Title Bar                                     │
├──────────────┬───────────────────────────────┤
│ Side Nav     │ Main Content                   │
│              │                               │
└──────────────┴───────────────────────────────┘
```

推荐尺寸：

```text
左侧导航宽度：260px ~ 280px
页面外边距：28px ~ 32px
左右间距：28px ~ 32px
主面板内边距：28px ~ 36px
```

暂时不要做：

- 无边框窗口
- 自绘标题栏
- 透明窗口
- 毛玻璃
- 复杂动画

---

## 9. 左侧导航规范

保留 6 个页面：

```text
01 总览 / OVERVIEW
02 通用 / GENERAL
03 监听 / SOURCES
04 外观 / APPEARANCE
05 诊断 / DIAGNOSTICS
06 关于 / ABOUT
```

选中态：

- 黑底
- 白色主文字
- 灰白英文标签

未选中：

- 浅色背景
- 黑色细边框
- 黑色主文字
- 灰色英文标签

顶部品牌区建议：

```text
CC TRAFFIC LIGHT
LOCAL SIGNAL CONSOLE
```

或：

```text
信号控制台
Win32 + Slint
```

---

## 10. 组件规范

优先抽以下公共 Slint 组件：

```text
AppShell
SideNavItem
Panel
SectionHeader
InfoRow
SettingToggleRow
StatusDot
StatusBadge
AgentStatusCard
DiagnosticEntry
```

组件要求：

- 只负责 UI 展示和基础交互转发
- 不读取配置文件
- 不写配置文件
- 不读取状态文件
- 不改变 Rust 业务逻辑
- 通过 property 接收数据
- 通过 callback 转发现有点击事件

---

## 11. 状态组件规范

### 11.1 StatusDot

状态映射：

```text
ok       绿色实心点
pending  黄色实心点
error    红色实心点
idle     灰色点或空心点
offline  灰色空心点
unknown  灰色点
```

示例：

```text
● VERIFIED
● WAITING
○ IDLE
● DEGRADED
```

### 11.2 StatusBadge

用于显示：

```text
待处理
空闲
已挂载
可信度降级
已确认
```

要求：

- 高度 24px ~ 28px
- 圆角 6px ~ 8px
- 小字号
- 近白背景
- 状态色只用于边框或小点

---

## 12. 信息行规范

### 12.1 InfoRow

结构：

```text
左侧：中文标题 + 英文 key
右侧：当前值 / 控件
```

示例：

```text
登录时启动
START_ON_LOGIN                         OFF
```

### 12.2 SettingToggleRow

显示：

```text
启动时最小化到托盘
MINIMIZE_ON_START                      ON  ●
```

或：

```text
登录时启动
START_ON_LOGIN                         OFF ○
```

保持原有开关逻辑，不新增配置字段。

---

## 13. 页面规范

### 13.1 总览页

推荐模块：

```text
SIGNAL SUMMARY
AGENT MATRIX
MOUNT STATUS
```

必须保留：

- 整体状态
- Codex 状态
- Claude 状态
- 组件挂载状态
- 最近更新时间
- 可信度
- session id，如果当前已有

禁止新增：

- 模拟事件
- 状态切换器
- 复制按钮
- 额外日志系统

---

### 13.2 通用页

推荐模块：

```text
SYSTEM BEHAVIOR
LANGUAGE
```

必须保留：

```text
登录时启动
启动时最小化到托盘
关闭窗口时仅缩到托盘
语言
```

英文 key：

```text
START_ON_LOGIN
MINIMIZE_ON_START
CLOSE_TO_TRAY
LANGUAGE_MODE
```

---

### 13.3 监听页

推荐模块：

```text
SOURCE MATRIX
```

必须保留：

```text
监听 Codex
监听 Claude Code
```

不要新增：

- 新来源
- 文件路径选择
- 测试按钮
- 自动检测功能

---

### 13.4 外观页

推荐模块：

```text
DISPLAY SURFACE
INDICATOR
MOTION
```

必须保留：

```text
界面主题
指示器样式
组件尺寸
显示标签
减少动效
```

英文 key：

```text
THEME_MODE
INDICATOR_STYLE
COMPONENT_SIZE
SHOW_LABELS
REDUCE_MOTION
```

注意：

原本只读的继续只读，原本可交互的继续可交互。

---

### 13.5 诊断页

推荐模块：

```text
LATEST CHECK
SIGNAL TRACE
REFRESH ACTION
```

必须保留：

- 最近刷新
- 最近错误
- Codex 检测依据 / 可信度 / 更新时间 / session id
- Claude 检测依据 / 可信度 / 更新时间
- 立即刷新按钮

诊断页应像只读信号记录，不要做成大段说明文字。

---

### 13.6 关于页

推荐模块：

```text
DEVICE SPEC
RUNTIME
PATHS
```

字段显示：

```text
PRODUCT       CC TRAFFIC LIGHT
VERSION       0.1.0
RUNTIME       Win32 组件 + 托盘，Slint 设置窗口
CONFIG        当前配置路径
LANGUAGE      跟随系统
```

禁止新增：

- 官网按钮
- GitHub 按钮
- License 按钮
- 复制路径按钮
- 检查更新
- 版本检测功能

---

## 14. 文案规范

### 14.1 状态文案统一

```text
pending   待处理   WAITING
idle      空闲     IDLE
mounted   已挂载   ATTACHED
verified  已确认   VERIFIED
degraded  降级     DEGRADED
error     错误     ERROR
offline   未接入   OFFLINE
unknown   未知     UNKNOWN
```

### 14.2 时间戳

不要高权重裸露毫秒时间戳。

优先显示：

```text
11:54:32
2026-07-03 11:54:32
```

### 14.3 Session ID

默认截断：

```text
codex_019f20fc...aa3c
```

不要让完整长 ID 撑破 UI。

---

## 15. HTML Demo 使用边界

`docs/ui/cc_traffic_light_nothing_demo_strict.html` 只用于视觉参考。

可以参考：

- 颜色
- 布局
- 导航
- 设置行
- 状态点
- 状态标签
- 诊断行
- 设备铭牌风格

不可以参考来新增：

- JS 交互
- 模拟事件
- 新状态切换
- 复制按钮
- 新配置
- 新数据结构
- 新业务逻辑

冲突时：

```text
源码 > 当前截图 > HTML demo > spec
```

---

## 16. 验收标准

成功标准：

- 页面不再像普通后台设置页
- 左侧导航有编号控制台感
- 页面统一使用黑白灰 + 少量状态色
- 设置行统一
- 状态表达统一
- 诊断页像只读信号记录
- 关于页像设备铭牌
- 没有新增业务功能
- 没有改变状态来源
- 没有改变配置 schema
- 编译通过
- 原有交互正常

失败信号：

- 新增了 HTML demo 里的演示交互
- 新增了配置字段
- 改了状态读取逻辑
- 改了托盘或 Win32 挂载逻辑
- 页面切换失效
- 设置保存失效
- 样式散落在各页面，未组件化
- 状态色被用于大面积背景
- UI 变成赛博霓虹风或后台管理系统风
