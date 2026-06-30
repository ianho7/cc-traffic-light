# 05. 推荐文件布局

## 1. 最小布局

```text
taskbar-widget/
  Cargo.toml
  src/
    main.rs
    taskbar.rs
    win32.rs
```

## 2. 每个文件应该负责什么

### `src/main.rs`

职责：

- 程序入口
- 初始化窗口类
- 创建窗口
- 启动消息循环
- 串联 taskbar 逻辑

不要放：

- 大量 Win32 辅助函数
- 复杂定位代码

### `src/taskbar.rs`

职责：

- 查找任务栏窗口
- 调 `SetParent`
- 读取宿主矩形
- 计算目标位置
- 移动窗口

不要放：

- 绘制逻辑
- 通用 UI 抽象

### `src/win32.rs`

职责：

- 放一些重复的 Win32 小工具
- 例如 UTF-16 辅助、错误码获取、小型包装函数

不要放：

- 大型对象模型
- 自己发明一层完整 Win32 封装

## 3. 如果后续扩展，允许增加哪些文件

MVP 成功后，可以再考虑：

```text
src/
  app.rs
  taskbar.rs
  paint.rs
  layout.rs
  debug.rs
```

但在 MVP 阶段不建议先拆这么多。

## 4. 建议保留的最小状态结构

你大概率只需要一个状态结构：

```rust
struct AppState {
    hwnd: HWND,
    taskbar_hwnd: HWND,
    parent_hwnd: HWND,
    module_rect: RECT,
}
```

足够了。

不要在 MVP 阶段建立：

- `WidgetManager`
- `TaskbarHostStrategy`
- `ThemeService`
- `RendererFactory`

这些都太早。

## 5. 回调与状态管理建议

Win32 窗口过程通常需要全局或静态状态。

MVP 建议：

- 接受一个最小、明确、可控的状态持有方式
- 不要为“纯函数式优雅”把问题复杂化

在 Rust 里你很可能要使用：

- `static mut` 的替代方案
- 通过窗口用户数据保存指针
- 或受控的 `Box<AppState>`

原则不是“最优雅”，而是“先稳定可调试”。

## 6. 文件组织的禁止项

以下组织方式在 MVP 阶段不要做：

- `core/infra/platform/presentation` 四层架构
- `traits/` 下放十几个 trait
- `widgets/` 下放通用可扩展模块系统
- `plugins/` 目录
- `config/` 目录

你现在只需要一个短路径 PoC。
