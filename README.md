# PixelDone Windows

PixelDone 的 Rust-first Windows 客户端。首个提交保留官方 `create-tauri-app` 的 SvelteKit + TypeScript 示例，用于记录脚手架来源；业务壳层从后续提交开始演进。

- Rust 1.96.1 stable，Edition 2024
- Tauri 2 + Svelte 5 + TypeScript 6 compatibility package
- Bun 管理前端，Cargo 管理 `src-tauri/`
- 仅支持 Windows 11 24H2+ x64 NSIS
- 发布必须通过 Android 3.0.3 功能复刻门禁

## 推荐开发环境

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)。
