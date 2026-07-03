# GUI / Tray V1 Requirements

## 目标

把当前以任务栏文本 widget 为中心的 Win11 PoC，演进成一个可常驻运行的本地桌面应用：保留任务栏 widget 主路径，新增托盘图标、设置窗口、诊断页和用户级启动项，并在尽量零配置的前提下感知 `Codex` 与 `Claude Code` 的本地状态。

预期结果：

- 单进程常驻后台，统一管理 widget、tray、settings。
- 正常情况下任务栏 widget 始终显示；挂载失败时自动降级到仅托盘并后台重试。
- 设置窗口提供状态总览、监听开关、样式设置、诊断信息和 About 页。
- 仅通过本地被动观测识别 `Codex` / `Claude Code` 状态，不做注入、抓屏或 shell I/O hook。

## 范围

范围内：

- `taskbar-widget/` 现有 Win32 / Rust 程序内新增 tray、settings、autostart、detector runtime、settings persistence。
- 基于固定来源优先级实现 `Codex` / `Claude Code` 的零配置观测和状态聚合。
- 将现有单状态 widget 扩展为双来源并列的三灯面板。
- 新增只读诊断页和对应文档。

范围外：

- 不引入 WebView、GUI 跨平台框架、D2D、DirectComposition。
- 不做系统级服务、管理员提权、多用户安装、多显示器支持。
- 不自动修改用户外部 Codex / Claude 配置。
- 不在 V1 做完整 Nothing design 复刻，只吸收其信息层级和黑白灰视觉原则。

## 当前状态

当前仓库已经具备：

- Rust + Win32 单进程任务栏 widget。
- Win11 任务栏 attach / layered / positioning 稳定路径。
- `Codex` / `Claude Code` hook 状态 schema 与 `state.json` 读写。
- 任务栏 UI 已有基础状态映射与 GDI 绘制能力。

当前缺失：

- 托盘图标与菜单。
- 设置窗口与用户级配置持久化。
- 自动发现 `Codex` / `Claude Code` 本地活动的 detector runtime。
- 来源可信度、冲突降级和诊断页面。
- 用户级自启动。

## 核心产品决策

### 形态

- 单进程常驻后台应用。
- 托盘图标始终保留。
- 设置窗口作为控制台式入口，不承载主实时交互。
- 任务栏 widget 是主展示面，不提供“永久关闭 widget”的产品入口。

### 运行与降级

- 正常情况下始终尝试显示 widget。
- widget 挂载失败时自动降级为仅托盘运行。
- 后台周期性重试 widget 恢复。
- 当前用户级自启动；启动后静默常驻并自动尝试恢复 widget。

### 观测边界

仅允许本地被动观测：

- 本地日志文件
- 本地状态 / 缓存文件
- 会话 / 历史文件
- 进程存在性与命令行参数
- 已存在的 hook 写入结果复用

明确不做：

- 注入
- 抓屏
- 键鼠监控
- 终端画面解析
- shell I/O hook

### 状态粒度

单来源状态固定为：

- `未发现`
- `空闲`
- `工作中`
- `需关注`
- `需立即处理`
- `状态不可信`

总聚合优先级：

- `需立即处理 > 需关注 > 工作中 > 空闲 > 未发现`
- `状态不可信` 单独显示在来源卡片与诊断页，不直接覆盖明确的红色告警

### 来源优先级

`Codex`：

1. 本地会话 / 日志文件
2. 本地状态 / 缓存文件
3. 进程存在性与命令行参数
4. 已有 hook 写入结果

`Claude Code`：

1. 本地日志 / 状态文件
2. 进程存在性与命令行参数
3. 已有 hook 写入结果

规则：

- 优先级固定写入实现，不运行时漂移。
- 进程检测仅做兜底，不能单独高置信度声明“工作中”。
- 同级来源冲突时，优先最近更新时间；若同样新鲜且互相矛盾，则降级为 `状态不可信`。

## V1 UI Requirements

### 任务栏 Widget

- 固定双来源并列：`Codex`、`Claude`
- 每组固定三灯：绿、黄、红
- 状态通过“高亮哪一灯 + 动效”表达，不通过增删灯位表达

推荐灯语：

- `空闲`：绿灯常亮
- `工作中`：绿灯呼吸
- `需关注`：黄灯慢闪
- `需立即处理`：红灯闪烁
- `未发现`：三灯全灭或极暗
- `状态不可信`：三灯灰态慢闪或灰罩覆盖

要求：

- `未发现` 与 `状态不可信` 必须视觉区分
- widget 继续使用 Win32 + GDI
- 视觉风格为 Nothing-inspired：黑白灰主色、高对比、强层级、少装饰

### 托盘

- 极简聚合状态图标
- 不承载双来源细节
- tooltip 提供简短摘要
- 菜单至少包含：
  - 打开设置
  - 立即重新探测
  - 退出

### 设置窗口

设置窗口采用纯 Win32，实现“偏控制台”的状态总览页，而不是文档页。

首页包含：

- 总聚合状态
- widget 挂载状态
- 最近一次成功观测时间
- `Codex` 来源卡片
- `Claude Code` 来源卡片
- 高频开关入口

来源卡片固定展示：

- 状态
- 识别方式
- 最近更新时间
- 可信度

一级页面：

- `通用`
  - 开机启动
  - 启动后最小化到托盘
  - 关闭窗口时隐藏到托盘
- `监听`
  - 启用 `Codex`
  - 启用 `Claude Code`
  - 当前状态文件路径
  - 配置说明入口
- `外观`
  - 灯样式
  - 尺寸
  - 文字标签显示 / 隐藏
  - 动效强度
- `诊断`
  - 只读诊断页
  - 手动刷新
- `关于`
  - 软件名
  - 版本
  - 开发者
  - 仓库地址
  - 反馈入口
  - 配置 / 状态目录入口
  - 许可证信息

## 技术决策

### GUI 技术栈

- V1 统一使用纯 Win32
- 不新增 WebView、egui、tauri 等第二套 GUI 运行时
- `Nothing design` 仅作为视觉方向，不作为需要完整复刻的 design system

### 运行时拆分

建议在单进程内明确拆成以下模块：

- `widget host`
- `tray host`
- `settings host`
- `detector runtime`
- `settings store`
- `status aggregator`

### 配置持久化

新增独立用户级配置文件，保存：

- autostart
- minimize_to_tray
- close_to_tray
- detector enable flags
- widget style settings
- diagnostics metadata

### 诊断能力

V1 只做只读诊断：

- widget 当前挂载状态
- 最近挂载成功时间
- `Codex` / `Claude` 命中的来源类型
- 最近更新时间
- 当前可信度
- 最近一次错误摘要
- `立即重新探测` 按钮

## 实施阶段建议

### Phase 1: Runtime 与配置骨架

- 定义配置 schema、状态聚合 schema 和 detector 输出契约
- 拆出 tray / settings / detector / widget 的消息边界
- 确定单消息循环与后台探测循环的协调方式

### Phase 2: Tray 与 Settings Shell

- 实现托盘图标、菜单与设置窗口框架
- 实现设置持久化和用户级 autostart
- 实现首页总览和基础导航

### Phase 3: Detector 与状态聚合

- 落地 `Codex` / `Claude Code` 多来源 detector
- 实现来源优先级、冲突降级、可信度规则
- 将 detector 输出接入现有状态模型

### Phase 4: Widget 扩展

- 把现有单状态 widget 扩展为双来源并列三灯面板
- 保持现有 attach / layered / positioning 稳定路径不变
- 将总聚合状态同步到托盘图标

### Phase 5: 诊断与验证收口

- 实现只读诊断页
- 补齐运行期验证、失败恢复与文档
- 决定是否进入后续 runtime hardening

## 验证策略

- `cargo fmt -- --check`
- `cargo check`
- 任务栏 widget 人工可见性验证
- tray 菜单与设置窗口基本交互验证
- autostart 启停验证
- `Codex` / `Claude Code` 来源探测矩阵验证
- 冲突与 stale / untrusted 场景验证
- widget 挂载失败后的托盘降级与自动恢复验证

## 风险与缓解

- Risk: 纯 Win32 设置窗口实现成本高于预期。
  Mitigation: V1 控件与页面严格收口，不追求高保真视觉复刻。

- Risk: 零配置 detector 在 `Codex` / `Claude Code` 侧数据源不稳定。
  Mitigation: 固定来源优先级、显式可信度、只读诊断页、允许复用已有 hook。

- Risk: widget、tray、settings 三者共享状态后消息复杂度上升。
  Mitigation: 先定义统一聚合状态与消息边界，再分阶段接入 UI。

- Risk: widget 强制显示与 Win11 任务栏脆弱路径冲突。
  Mitigation: 允许自动降级到托盘并后台重试，不把挂载失败升级成进程失败。

## 推荐下一步

- 基于本需求文档执行 [gui-tray-v1.md](/D:/project/cc-traffic-light/docs/checklist/gui-tray-v1.md)。
- Phase 1 先交付 runtime / config / loop 契约，再进入具体 UI 改造。
