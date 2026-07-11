# Android 3.1.0 基线证据

- 正式基线 commit：`63471218345f6a4efcdbbd32c2d4c42acb25491c`。
- 版本：`versionName 3.1.0`、`versionCode 78`、Room v5。
- 2026-07-11 已通过：`testDebugUnitTest`、`lintDebug`、`assembleDebug`、`assembleRelease`。
- `PixelDone-3.1.0-release.apk` 已通过 `apksigner verify --verbose --print-certs`；APK Signature Scheme v2 为 true。
- Supabase HTTP endpoint 的 Auth、REST 和登录态 `pixeldone_pull_changes` RPC 已真实连通，返回 schema `3.1`。
- 阿拉伯语 Settings 的两列语言选择器截图见 `settings-arabic.png`。
- 图片同步策略已由单元测试确认：图片元数据不进入远端 payload，云端合并保留本地图片。
- 云端提醒变化落地后会重新调度 Android 本机提醒，并有 ViewModel 测试覆盖。
