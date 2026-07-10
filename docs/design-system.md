# PixelDone Windows 设计系统

## 权威与参考

PixelDone Android 是视觉语言和功能语义权威。macOS Reminders 只提供桌面信息架构参考：左侧清单、中部工作区、右侧 Inspector。不得复制 Apple 字体、SF Symbols、毛玻璃、卡片视觉、文案或 Android 中不存在的 smart list 功能。

## Token

- 4px 网格；2px 主边框；1px 分隔线。
- 圆角仅 0 或 4px。
- UI 字体：Cascadia Mono、Consolas、系统等宽回退。
- 背景 `#FAF9F5`，表面 `#F0EEE6`，边框 `#E8E6DC`。
- 主文字 `#141413`，次文字 `#5E5D59`，错误 `#DF6666`。
- LOW `#34A853`、MEDIUM `#4285F4`、HIGH `#FBBC05`、XHIGH `#EA4335`。

深色模式使用同一 semantic token，不在组件散布颜色值。

## 布局

- 默认 1180×780，最小 1000×680。
- ≥1120px：228px Sidebar + 弹性 Workspace + 360px Inspector。
- 1000–1119px：Inspector 变为右侧 overlay。
- 使用 Windows 原生标题栏，不绘制伪 macOS 窗口按钮。

## 交互

- hover、selected、keyboard focus、completed highlight 使用不同表达，不能只靠颜色。
- 完成后 2000ms 保持原位置并高亮，再进入排序位置。
- Dock 保留 Android 语义，最多四动作；快捷键永远不是唯一入口。
- 编辑使用 Inspector，不使用阻断扫描流程的中心 modal。
- 动画以 120–180ms 状态过渡为主，并尊重 reduced motion。
