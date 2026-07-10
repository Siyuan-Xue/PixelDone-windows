# PixelDone Windows 架构

## 进程与所有权

Tauri Core process 是唯一业务状态权威。Rust 负责领域规则、SQLite 事务、revision、同步协议、图片、凭据、提醒、系统托盘、更新和 Windows API。WebView process 只保存未提交草稿、当前焦点、选中项、菜单、弹窗、拖动与短暂动画。

```text
Svelte intent -> typed invoke(expectedRevision) -> Rust command
             <- MutationResult(revision, changedIds, snapshotDelta)
Rust event   -> snapshot/auth/sync/reminder/update state
```

前端不得发送完整清单或应用状态覆盖 Rust。`STALE_REVISION` 会强制重新 bootstrap，而不是最后写入获胜。

## 目录

- `src/lib/components/`：桌面视图组件，不含领域规则。
- `src/lib/generated/ipc.ts`：Rust 公共类型的 TypeScript 镜像；后续由绑定导出程序生成。
- `src-tauri/src/domain/`：纯 Rust；不得依赖 Tauri、SQLx 或 Windows API。
- `src-tauri/src/application/`：意图命令、revision、事务协调和事件。
- `src-tauri/src/infrastructure/`：SQLite、Supabase、图片、提醒、更新和凭据适配器。
- `src-tauri/src/platform/windows/`：Windows 通知、Credential Manager、生命周期和激活。

## SQLite

Android 与 Windows 各自维护本地 SQLite，不共享 SQLite 文件。两端共享 Supabase Auth、远端业务表、Storage 和同步协议。

当前本地 mutation 先在候选快照执行，通过 SQLite 事务持久化后才替换运行时状态。数据库失败不会把未持久化状态发布给前端。

## TypeScript 6 兼容说明

TypeScript 7.0 已稳定，但尚无供 Svelte embedded-language 工具使用的稳定程序化 API。项目直接固定 `typescript@6.0.2`。Microsoft 推荐的 `@typescript/typescript6` wrapper 在 Bun 1.3.14 下出现嵌套 alias 自引用，因此当前不用 wrapper；当 Bun 修复解析或 Svelte 支持 TypeScript 7 API 时再切换。

## Release 阻断边界

本地清单和任务能力可在开发阶段运行，但 Cloud 写入、图片跨端、提醒 action、Credential Manager、托盘、更新和签名均保持 parity `blocked`，直到真实实现、自动测试与 installed-app 证据齐备。任何 blocked 项都禁止 Release。
