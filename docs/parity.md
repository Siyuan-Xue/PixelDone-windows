# PixelDone Windows 功能复刻报告

> 本文件由 `bun run parity:report` 从 `parity/pixeldone-3.0.3.yaml` 生成，请勿手工维护状态。

基线：PixelDone Android 3.0.3（versionCode 76，commit `89763b6ac3b8f5bea8760c4e48d2cbf3d02591f7`，Room v4）。

- Required：33
- Verified：0
- In progress：18
- Blocked：14
- Not started：1
- 完成率：0.00%

| ID | 域 | 功能 | Release required | 状态 | 差异 |
| --- | --- | --- | --- | --- | --- |
| LIST-FIXED-001 | checklist | MAIN/TRASH/SETTINGS 固定页面 | 是 | in_progress | none |
| LIST-CRUD-001 | checklist | 普通清单创建、重命名、删除 | 是 | in_progress | none |
| LIST-MINIMUM-001 | checklist | 至少保留一个普通清单 | 是 | in_progress | none |
| LIST-HISTORY-001 | checklist | 清单返回历史 | 是 | not_started | platform-equivalent: Alt+Left / Back |
| TODO-CRUD-001 | todo | 任务创建、编辑、软删除 | 是 | in_progress | none |
| TODO-PRIORITY-001 | todo | XHIGH/HIGH/MEDIUM/LOW 四级优先级 | 是 | in_progress | none |
| TODO-DUE-001 | todo | 截止日期和时间 | 是 | in_progress | none |
| TODO-REPEAT-001 | todo | NONE/DAILY/WEEKLY 重复提醒 | 是 | in_progress | none |
| TODO-SORT-001 | todo | 优先级与时间排序 | 是 | in_progress | none |
| TODO-COMPLETE-001 | todo | 完成与重新激活 | 是 | in_progress | none |
| TODO-DELAY-001 | todo | 完成后两秒延迟排序与高亮 | 是 | in_progress | none |
| TODO-COMPACT-001 | todo | 紧凑完成行 | 是 | in_progress | none |
| DOCK-HIDE-001 | dock | HIDE DONE | 是 | in_progress | none |
| DOCK-CLEAN-001 | dock | CLEAN DONE | 是 | in_progress | none |
| DOCK-QUICK-001 | dock | QUICK DELETE | 是 | in_progress | none |
| TRASH-RESTORE-001 | trash | 恢复任务并重建原清单 | 是 | in_progress | none |
| TRASH-PURGE-001 | trash | 永久删除与 tombstone | 是 | in_progress | none |
| IMAGE-SINGLE-001 | image | 单张图片添加、预览、替换与删除 | 是 | blocked | none |
| IMAGE-CLOUD-001 | image | Supabase Storage 跨端图片同步 | 是 | blocked | none |
| REMINDER-NORMAL-001 | reminder | 普通短通知提醒 | 是 | blocked | platform-equivalent: Windows notification |
| REMINDER-XHIGH-001 | reminder | XHIGH 独立强提醒窗口 | 是 | blocked | platform-equivalent: XHIGH alarm window |
| REMINDER-BATCH-001 | reminder | 同时间提醒批处理、STOP 与 SNOOZE | 是 | blocked | none |
| REMINDER-RECONCILE-001 | reminder | 开机、恢复、时间变化后的提醒校正 | 是 | blocked | platform-equivalent: autostart + reconciliation |
| SETTINGS-THEME-001 | settings | 浅色与深色主题 | 是 | in_progress | none |
| SETTINGS-DOCK-001 | settings | Dock 位置与最多四动作配置 | 是 | in_progress | none |
| AUTH-001 | auth | 注册、登录、退出与密码重置 | 是 | blocked | none |
| CREDENTIAL-001 | auth | 凭据安全存储 | 是 | blocked | platform-equivalent: Credential Manager |
| SYNC-MANUAL-001 | sync | 手动与自动同步 | 是 | blocked | none |
| SYNC-CAS-001 | sync | 原子 mutation、CAS、change feed 与 schema negotiation | 是 | blocked | none |
| SYNC-CONFLICT-001 | sync | 冲突记录与 LOCAL/CLOUD 审查 | 是 | blocked | none |
| UPDATE-SOURCE-001 | update | GitHub 优先与 Gitee 回退 | 是 | blocked | none |
| UPDATE-CHANNEL-001 | update | stable 与 beta/RC 独立渠道 | 是 | blocked | none |
| RELEASE-NSIS-001 | release | Windows x64 NSIS 安装与双签名 | 是 | blocked | platform-equivalent: signed NSIS/Tauri updater |
| SOURCE-BATCH-MOVE-001 | excluded | 源码存在但 Android UI 不可达的批量移动 | 否 | not_started | excluded_source_only |

正式发布门槛固定为 100.00%、0 blocked、0 in_progress、0 not_started、0 missing evidence。
