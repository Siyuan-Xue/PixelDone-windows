# Windows 3.1.0 验证证据

- 2026-07-11 已通过：`bun run check`、`bun test`、`bun run build`、`bunx tsc --noEmit`。
- Rust 已通过：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`；7 个单元测试和 SQLite migration 集成测试全部通过。
- WebdriverIO Tauri service 已驱动真实 debug Tauri/WebView2 二进制，验证三栏启动、清单 CRUD/revision、任务字段与 Dock、两秒完成延迟、Trash 来源清单重建、本地图片、STOP/SNOOZE、独立 XHIGH 窗口、两列语言与 RTL、更新版本。
- 临时账号的真实 HTTP 云端闭环已通过：注册、Credential Manager 会话、创建本地清单/任务、3.1 mutation RPC、change pull、remoteVersion、pending 清零和退出。
- UI 截图：`main-1180x780.png`、`settings-arabic.png`。
- 测试构建使用独立临时 SQLite 和独立 Credential Manager target；WebDriver 插件仅在 Rust debug build 注册，正式 release binary 不包含自动化服务。
- 正式 x64 NSIS 已构建并完成 Tauri updater signature 反向验证；SHA-256 为 `2407B13D2B1947AAE2378F04264ADD73CCB9948D17B492B6C860702DD95B2B5E`。
- clean install 已验证：静默安装退出码 0，安装目录只包含 `pixeldone-windows.exe` 与 `uninstall.exe`；直接启动 exe 后进程持续运行并创建正式 SQLite，静默卸载退出码 0。
- 普通 Windows toast XML 包含 STOP 与 SNOOZE 10 MIN action，并由 WinRT Activated 回调进入与 XHIGH 窗口相同的 Rust reminder command；用户任务文本经过 XML 转义测试。
