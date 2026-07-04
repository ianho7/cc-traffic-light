# Tauri Settings Visual Fidelity Pass 1

日期：2026-07-03

## Objective

记录 `taskbar-settings-tauri/` 第一轮静态 UI fidelity 的对照结果，用于完成：

- `TSM-E-04` 明确哪些地方要求近似还原，哪些地方允许为了工程稳定做实现层调整
- `TSM-E-05` 产出一轮视觉对照记录，逐项标记与 HTML demo 的已对齐项和未对齐项

当前证据来源：

- `taskbar-settings-tauri/src/App.tsx`
- `taskbar-settings-tauri/src/styles.css`
- `docs/ui/cc_traffic_light_nothing_demo_strict.html`
- `docs/ui/nothing-signal-console-spec.md`
- `docs/ui/nothing-signal-console-checklist.md`

## 1. 近似还原边界

以下部分要求近似还原 HTML demo / Nothing Signal Console 的视觉结构：

- 双栏结构：左侧编号导航 + 右侧主内容面板
- 暖白背景、近白面板、细边框、小圆角
- 6 页结构与编号导航顺序
- 页面 section 的标题、note、row 节奏
- 状态卡、状态点、状态 badge、胶囊式 toggle / value 容器
- Diagnostics 的只读 trace 风格
- About 的设备铭牌式参数行风格

以下部分允许为了工程稳定或现阶段边界做实现层调整：

- 不实现 demo 的自绘 titlebar、窗口按钮和纯静态窗口外壳
- 不追求像素级字号、留白、卡片高度和 hover motion 一致
- 不把 demo 中的演示脚本、假切换逻辑、假刷新逻辑迁入正式实现
- 不为了匹配 demo 而新增状态字段、配置项、复制按钮、模拟动作
- 不为第一轮静态 fidelity 引入完整主题系统、图标系统或动画系统
- About / protocol / transport 等只读元数据允许通过 Tauri bootstrap 合成，而不是要求宿主额外开新 IPC 命令

## 2. 当前实现选择

当前 Tauri 首轮静态壳层采用以下实现策略：

- 视觉 token 使用 `styles.css` 顶部 CSS variables 管理，不引入单独 CSS-in-JS 或主题 provider
- 页面结构在 `App.tsx` 中直接按 `overview/general/monitoring/appearance/diagnostics/about` 组织
- 交互控件优先使用原生 HTML：`button` 作为 nav、setting row 和 refresh action
- 开关与 value 展示通过轻量样式组件完成，不引入自定义复杂 widget
- `About` 页只补只读 shell metadata，不扩张宿主 IPC 命令面

## 3. 对照记录

| 对照项 | 目标基线 | 当前状态 | 备注 |
| --- | --- | --- | --- |
| 双栏布局 | 左侧导航 + 右侧主面板 | 已对齐 | `app-frame` 固定为 275px sidebar + main panel |
| 编号导航 | 01-06 编号、黑底选中态、英文副标签 | 已对齐 | 当前为 React 按钮导航，保留页面切换和 `last_opened_page` 写回 |
| 品牌区 | `CC TRAFFIC LIGHT` / 控制台式短说明 | 已对齐 | 文案改为 Tauri settings shell 边界说明 |
| 页面层级 | page title + section + rows | 已对齐 | 所有 6 页均落到统一 section/row 结构 |
| Overview 状态卡 | 总体状态、挂载状态、来源矩阵 | 已对齐 | 用真实 snapshot 填充，不复刻 demo 的假时间戳 |
| Row 样式 | 中文标题 + 英文 key + 右侧 value/control | 已对齐 | `SettingRow` / `InfoRow` 已建立统一模式 |
| Toggle 形态 | 黑白胶囊 + 状态点 | 已对齐 | 使用原生按钮包装，不自造复杂 switch |
| Diagnostics trace | 只读记录风格 | 已对齐 | 保留手动刷新按钮，trace 使用等宽文本 |
| About 设备铭牌 | product/version/runtime/config/language | 已对齐 | 已补充 bootstrap about metadata |
| 颜色 token | 暖白 + 黑白灰 + 少量状态色 | 已对齐 | 通过 CSS variables 统一管理 |
| 字体角色 | UI 字体 + mono 辅助信息 | 已对齐 | mono 用于 key、badge、trace、protocol |
| 响应式适配 | 桌面与窄屏都能读 | 已对齐 | `1080px` / `720px` 断点已覆盖 |
| 自绘 titlebar | demo 风格窗口 chrome | 未对齐 | 当前明确不做，避免把窗口系统行为混入本阶段 |
| 像素级 card 高度/留白 | 与 demo 完全一致 | 未对齐 | 当前只保层级、气质和结构，不追像素复刻 |
| Demo 演示脚本 | 假 toggle / 假刷新 / 假页面脚本 | 故意未对齐 | 正式实现只保真实数据和现有命令路径 |
| 完整视觉截图归档 | 实际渲染截图对比 | 未完成 | 当前轮只有代码与 build 证据，后续在运行验证阶段补人工截图 |

## 4. 结论

当前第一轮静态 fidelity 已满足：

- 结构、节奏、信息层级和 Nothing Signal Console 气质已落到 Tauri 前端
- 没有为了追 demo 而扩张业务边界
- 未对齐项主要集中在窗口 chrome、像素级细节和人工截图证据，这些都属于当前阶段允许保留的差异

当前未完成但不阻塞 `TSM-E-04` / `TSM-E-05` 的项：

- 真机窗口截图对照
- 运行态轮询后的动态视觉核验
- 默认主入口切换后的用户路径验收
