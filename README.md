# PixelDone Windows

PixelDone for Windows 3.1.1。功能与领域语义以 Android 3.1.1 为权威，桌面端复用 Android 启动图标的矢量几何与色值，并采用适合键鼠操作的 Sidebar、Task Workspace、Inspector 三栏布局。

## 正式版基线

- Android：PixelDone 3.1.0，commit `63471218345f6a4efcdbbd32c2d4c42acb25491c`，Room v5。
- Windows：PixelDone for Windows 3.1.1，安装/更新版本与 Android 业务协议版本独立演进。
- Windows：Rust 1.96.1 / Edition 2024、Tauri 2、Svelte 5.56.4、TypeScript 6.0.2、Bun 1.3.14。
- 支持范围：Windows 11 24H2+，仅发布 x64 NSIS 安装包。
- 本地数据：`%LOCALAPPDATA%\com.milesxue.pixeldone.windows\data\pixeldone.sqlite3`；图片仅保存在本机 `attachments` 目录。
- 云端：Android 与 Windows 共用同一 Supabase Auth、业务表和 3.1 同步协议。
- 提醒：未来 12 个月、最多 4,000 个提醒写入 Windows Scheduled Toast 队列；所有优先级默认使用右下角标准通知。仅当用户主动开启“Windows XHIGH 增强闹钟”后，`XHIGH` 才使用 Windows alarm 场景和循环闹铃音。

## 安装与运行

运行 `PixelDone_3.1.1_x64-setup.exe` 完成当前用户安装。正式默认目录是 `%LOCALAPPDATA%\PixelDone`，主程序名为 `PixelDone.exe`。路径中的 `test` 仅可能来自历史验收时显式指定的临时目录，不是正式安装器配置。

重复运行相同或更高版本的 EXE 会进入 NSIS 维护/升级流程并覆盖同一产品安装，不会创建多个 PixelDone。Beta 使用独立产品标识，允许与正式版并存。

与 3.1.0 一致，正式分发不使用 Authenticode，Windows 可能显示“未知发布者”，用户确认来源和 SHA-256 后可继续安装。应用内更新仍使用 Tauri updater signature 校验下载完整性；它不提供 Windows 发布者身份认证。

## 更新、自启动与提醒

- 应用启动 5 秒后自动检查更新；成功后 24 小时再查，失败后 6 小时重试。自动检查不会自动下载或安装。
- SETTINGS → Updates 提供自动检查开关、立即检查、下载进度和手动安装入口。更新器校验签名后使用 NSIS `/UPDATE` 覆盖安装并重启。
- 开机启动默认开启且以最小化方式运行。用户关闭后不会在下次启动时被重新打开；外部禁用启动项也会被尊重。
- 通知按钮通过 `pixeldone-reminder://` 协议路由 STOP、SNOOZE 和打开应用动作；任务 ID 会在本地数据库中再次校验。
- 安装态应用固定使用 AUMID `com.milesxue.pixeldone.windows`，并在开始菜单快捷方式写入 Stub Toast Activator CLSID。系统通知被关闭或身份注册失败时，Settings 与提醒状态会显示具体原因，不会回退为抢占式窗口。

## 多端同步

- 登录、注册或恢复会话后立即调用现有 `pixeldone_pull_changes` RPC，完整拉取当前用户的清单、代办、设置与 tombstone。
- 常驻期间订阅 `todo_checklists`、`todo_items`、`user_settings`、`sync_tombstones` 四张表的 Supabase Realtime 变更。事件只触发约 500ms 防抖后的事务化同步，不直接写本地数据库。
- Realtime 具备心跳、JWT 更新重连和 1–30 秒指数退避。Windows 不运行固定间隔同步轮询；首次登录、会话恢复、Realtime 建连/重连成功和本地修改均会触发完整游标补拉。
- 本地修改或任一同步完成后立即按最新完整快照重建本机 Scheduled Toast 队列；远程新增/改期会注册或替换提醒，完成、回收站、永久删除及清单删除会撤销提醒。15 分钟校准仍作为系统时间变化兜底。

## 存储与系统侵入边界

PixelDone 不安装 Windows 服务、驱动或每任务的计划任务。系统集成仅包括当前用户安装、开始菜单/卸载项、一个开机启动项、通知协议、Windows 通知调度队列和 WebView2 数据。

| 内容 | 位置 |
| --- | --- |
| 程序 | `%LOCALAPPDATA%\PixelDone\PixelDone.exe` |
| SQLite | `%LOCALAPPDATA%\com.milesxue.pixeldone.windows\data\pixeldone.sqlite3` |
| 图片 | `%LOCALAPPDATA%\com.milesxue.pixeldone.windows\attachments` |
| 缓存/日志 | 同一数据根目录下的 `cache` / `logs` |
| WebView2 | 同一数据根目录下的 `EBWebView` |
| 登录会话 | Windows Credential Manager：`com.milesxue.pixeldone.windows/supabase-session` |

SETTINGS → Storage & privacy 会显示当前实际路径、占用空间并可打开目录。若检测到旧版 `%APPDATA%\com.milesxue.pixeldone.windows\pixeldone.sqlite3`：仅在新 Local 数据库不存在时自动复制迁移；两者同时存在则以 Local 为准，并允许用户确认后删除旧文件。卸载默认保留用户数据，避免升级/重装误删；需要彻底清理时可在卸载后删除上述数据根目录和 Credential Manager 凭据。

## 明文 HTTP 决策

PixelDone 3.1 长期允许连接指定的 HTTP Supabase endpoint，Android 与 Windows 均没有迁移 HTTPS 的计划。Windows Realtime 因而使用对应的 `ws://` 连接。HTTP/WS 不提供 HTTPS/WSS 的链路机密性、完整性和服务器身份保护；同一网络路径上的第三方可能观察或篡改账号、访问令牌及同步流量。此风险是当前小范围部署的明确产品决策。

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
# 与 3.1.0 一致：不使用 Authenticode，只生成 Tauri updater 完整性签名。
bun tauri build --bundles nsis --target x86_64-pc-windows-msvc
```

正式 Release 必须通过 `parity/pixeldone-3.1.0.yaml` 的 100% required gate。Supabase Storage 图片同步和 Android UI 不可达的 batch move 是明确排除项，不计为功能缺失。
