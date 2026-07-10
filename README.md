# PixelDone Windows

PixelDone Android 的 Rust-first Windows 客户端。桌面布局借鉴 macOS Reminders 的信息架构，但功能、视觉语言和行为语义以 PixelDone Android 为唯一产品权威。

## 当前基线

- Android：PixelDone 3.0.3，commit `89763b6`，Room v4
- Rust：1.96.1 stable，Edition 2024
- Desktop：Tauri 2，Svelte 5.56.4，TypeScript 6.0.2，Bun 1.3.14
- 平台：Windows 11 24H2+，仅发布 x64 NSIS
- Release：`parity/pixeldone-3.0.3.yaml` 中所有 required 功能达到 100% verified 后才允许创建

## 开发

```powershell
bun install
bun run check
bun run test
bun run build
bun tauri dev
```

Rust 后端独立验证：

```powershell
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Parity 报告：

```powershell
bun run parity:report
bun run parity:check
```

`parity:check` 在功能未完全验证时故意返回非零状态；这是 Release 阻断器，不是开发期应绕过的检查。

## 目录边界

- 根目录由 Bun 管理 Svelte/Vite 前端与开发脚本。
- `src-tauri/` 是 Cargo 管理的标准 Rust/Tauri crate。
- Rust 独占领域规则、SQLite、同步、提醒、凭据和 Windows 系统能力。
- Svelte 只管理草稿、焦点、选中项、菜单和动画等临时 UI 状态。
- 不存在独立 Node 后端、前端 SQLite 或第二个 Git 仓库。

TypeScript 直接固定为 6.0.2。Microsoft 的 `@typescript/typescript6` compatibility alias 在 Bun 1.3.14 中会将其嵌套 `@typescript/old` 错误解析为自身，导致 `svelte-check` 获得空 API；直接固定得到的正是 wrapper 要 re-export 的同一套稳定 API。
