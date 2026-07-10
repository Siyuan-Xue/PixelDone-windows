# 桌面 UI 验收

## 必测视口

- 1180×780：light / dark。
- 1000×680：light / dark，Inspector overlay。
- Maximized。
- Windows DPI：100%、125%、150%、200%。

## 必测状态

- 空清单。
- 普通任务列表。
- 完成、高亮和 2 秒延迟排序。
- Quick Delete。
- Trash 与恢复来源清单。
- Settings。
- Conflict Center。
- Image Preview。
- XHIGH alarm window。
- Offline、同步错误、更新可用。

## 发布判断

视觉验收不替代 `parity:check`。只有布局、设计语言、键鼠操作、DPI 和 parity 100% 同时通过，才可创建 Windows Release。
