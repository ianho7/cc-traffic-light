# 04. 实施阶段

## Phase 1: 普通窗口与 GDI 文本

- Goal: 建立最小 Win32 Rust 程序
- Files:
  - `Cargo.toml`
  - `src/main.rs`
  - 可选 `src/win32.rs`
- Tasks:
  - 创建窗口类
  - 创建普通窗口
  - 实现消息循环
  - 在 `WM_PAINT` 中绘制固定文本
- Expected Result:
  - 程序启动后出现一个普通小窗口，显示 `TASKBAR WIDGET`
- MVP Check:
  - Why this phase is necessary:
    - 它验证 Rust Win32 基础链路是通的
  - What is intentionally not included:
    - 不包括任务栏查找
    - 不包括 `SetParent`
    - 不包括定位逻辑

## Phase 2: 任务栏句柄探测

- Goal: 拿到目标任务栏宿主句柄
- Files:
  - `src/taskbar.rs`
  - `src/main.rs`
- Tasks:
  - 实现 `FindWindowW("Shell_TrayWnd")`
  - 如有需要，实现最小 `FindWindowExW`
  - 输出句柄和错误码
- Expected Result:
  - 程序能打印出任务栏宿主句柄
- MVP Check:
  - Why this phase is necessary:
    - 没有宿主句柄无法嵌入
  - What is intentionally not included:
    - 不包括多显示器
    - 不包括完整窗口树遍历

## Phase 3: 子窗口嵌入

- Goal: 把自定义窗口挂到任务栏父窗口下
- Files:
  - `src/taskbar.rs`
  - `src/main.rs`
- Tasks:
  - 调 `SetParent`
  - 记录结果和错误码
  - 确认窗口不再作为独立浮窗存在
- Expected Result:
  - 自定义窗口层级属于任务栏父窗口
- MVP Check:
  - Why this phase is necessary:
    - 这是 MVP 的核心验证点
  - What is intentionally not included:
    - 不追求最终可见位置
    - 不追求视觉效果

## Phase 4: 最小位置计算

- Goal: 让模块在任务栏内可见
- Files:
  - `src/taskbar.rs`
- Tasks:
  - 读取任务栏矩形
  - 读取辅助窗口矩形或使用保守固定偏移
  - 调 `MoveWindow`
  - 在日志里输出最终矩形
- Expected Result:
  - 模块能稳定出现在任务栏中
- MVP Check:
  - Why this phase is necessary:
    - “嵌入但不可见”不算 MVP 成功
  - What is intentionally not included:
    - 不做复杂避让策略
    - 不做 Widgets 检测
    - 不做任务栏左右对齐全兼容

## Phase 5: 文档化与收尾

- Goal: 固化复现方式和已知限制
- Files:
  - `README.md`
  - 可选 `notes.md`
- Tasks:
  - 记录目标系统
  - 记录运行命令
  - 记录已知限制
  - 记录下一步扩展方向
- Expected Result:
  - 这个 MVP 可以被你单独复制、运行、继续扩展
- MVP Check:
  - Why this phase is necessary:
    - 否则这个 PoC 很快会变成不可维护的一次性实验
  - What is intentionally not included:
    - 不做正式对外文档
    - 不做大而全设计说明

## 阶段间停止规则

如果某一阶段没有稳定完成，不允许继续往后堆功能。

特别是：

- Phase 3 没稳定，不要做 Phase 4 的复杂布局
- Phase 4 没稳定，不要做动态数据

## 执行顺序纪律

正确顺序：

1. 先看到普通窗口
2. 再看到任务栏句柄
3. 再完成 `SetParent`
4. 再完成可见定位

错误顺序：

- 一开始就写完整架构
- 一开始就做透明
- 一开始就做 Win11 兼容层
