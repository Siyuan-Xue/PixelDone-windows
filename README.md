# PixelDone Windows

PixelDone Android 3.1.0 的 Rust-first Windows 客户端。功能与领域语义以 Android 为权威，桌面端保留 PixelDone 视觉语言，并采用适合键鼠操作的 Sidebar、Task Workspace、Inspector 三栏布局。

## 正式版基线

- Android：PixelDone 3.1.0，commit `63471218345f6a4efcdbbd32c2d4c42acb25491c`，Room v5。
- Windows：Rust 1.96.1 / Edition 2024、Tauri 2、Svelte 5.56.4、TypeScript 6.0.2、Bun 1.3.14。
- 支持范围：Windows 11 24H2+，仅发布 x64 NSIS 安装包。
- 本地数据：`%LOCALAPPDATA%\com.milesxue.pixeldone.windows\data\pixeldone.sqlite3`；图片仅保存在本机 `attachments` 目录。
- 云端：Android 与 Windows 共用同一 Supabase Auth、业务表和 3.1 同步协议。

## 安装与运行

运行 `PixelDone_3.1.0_x64-setup.exe` 完成当前用户安装，然后可直接点击桌面/开始菜单中的 PixelDone，或直接点击安装目录内的 `pixeldone-windows.exe`。

本项目的小范围正式分发不使用 Authenticode。Windows SmartScreen 可能在首次启动时显示“未知发布者”，用户确认来源和 SHA-256 后可选择“仍要运行”。安装包仍使用 Tauri updater signature 校验应用内更新。

## 明文 HTTP 决策

PixelDone 3.1 长期允许连接指定的 HTTP Supabase endpoint，Android 与 Windows 均没有迁移 HTTPS 的计划。HTTP 不提供 HTTPS 的链路机密性、完整性和服务器身份保护；同一网络路径上的第三方可能观察或篡改账号及同步流量。此风险是当前小范围部署的明确产品决策。

## 开发与验证

```powershell
bun install
bun run check
bun run test
bun run build
bun run e2e
bun run parity:check

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`build.rs` 优先读取构建环境变量；本地开发可安全复用相邻 Android 仓库 `PixelDone/local.properties` 中的 `pixeldone.supabaseUrl` 和 `pixeldone.supabasePublishableKey`。脚本不会打印密钥，`local.properties` 和 updater 私钥均不会进入 Git。

正式构建：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = 'src-tauri/signing/pixeldone-updater.key'
bun tauri build --bundles nsis --target x86_64-pc-windows-msvc
```

正式 Release 必须通过 `parity/pixeldone-3.1.0.yaml` 的 100% required gate。Supabase Storage 图片同步和 Android UI 不可达的 batch move 是明确排除项，不计为功能缺失。
