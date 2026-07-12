# PixelDone Windows 功能复刻报告

> 本文件由 `bun run parity:report` 从 `parity/pixeldone-3.1.0.yaml` 生成，请勿手工维护状态。

基线：PixelDone Android 3.1.0（versionCode 78，commit `63471218345f6a4efcdbbd32c2d4c42acb25491c`，Room v5）。

- Required：41
- Verified：41
- In progress：0
- Blocked：0
- Not started：0
- 完成率：100.00%

| ID | 域 | 功能 | Release required | 状态 | 差异 |
| --- | --- | --- | --- | --- | --- |
| LIST-FIXED-001 | checklist | MAIN、TRASH、SETTINGS 固定页面 | 是 | verified | none |
| LIST-CRUD-001 | checklist | 普通清单创建、重命名与删除 | 是 | verified | none |
| LIST-MINIMUM-001 | checklist | 至少保留一个普通清单 | 是 | verified | none |
| LIST-HISTORY-001 | checklist | 清单返回历史 | 是 | verified | platform-equivalent: Esc / Alt+Left |
| TODO-CRUD-001 | todo | 任务创建、编辑与软删除 | 是 | verified | none |
| TODO-FIELDS-001 | todo | 标题、完成、创建时间、回收站来源和远端版本字段 | 是 | verified | none |
| TODO-PRIORITY-001 | todo | XHIGH、HIGH、MEDIUM、LOW 四级优先级 | 是 | verified | none |
| TODO-DUE-001 | todo | 截止日期和时间 | 是 | verified | none |
| TODO-REPEAT-001 | todo | NONE、DAILY、WEEKLY 重复提醒 | 是 | verified | none |
| TODO-SORT-001 | todo | 优先级与时间排序 | 是 | verified | none |
| TODO-DDL-001 | todo | DDL 截止时间显示开关 | 是 | verified | none |
| TODO-COMPLETE-001 | todo | 完成与重新激活 | 是 | verified | none |
| TODO-DELAY-001 | todo | 完成后两秒延迟排序 | 是 | verified | none |
| TODO-HIGHLIGHT-001 | todo | 任务 reveal 与状态高亮 | 是 | verified | none |
| TODO-COMPACT-001 | todo | 紧凑完成行 | 是 | verified | none |
| DOCK-HIDE-001 | dock | HIDE DONE | 是 | verified | none |
| DOCK-CLEAN-001 | dock | CLEAN DONE | 是 | verified | none |
| DOCK-QUICK-001 | dock | QUICK DELETE | 是 | verified | none |
| TRASH-RESTORE-001 | trash | 恢复任务并重建来源清单 | 是 | verified | none |
| TRASH-PURGE-001 | trash | 永久删除和 tombstone | 是 | verified | none |
| IMAGE-LOCAL-001 | image | 单张本地图片添加、预览、替换、删除、缩放和平移 | 是 | verified | none |
| REMINDER-NORMAL-001 | reminder | 普通 Windows 通知提醒 | 是 | verified | platform-equivalent: installed Windows Scheduled Toast with protocol activation |
| REMINDER-XHIGH-001 | reminder | XHIGH 可选增强提醒 | 是 | verified | platform-equivalent: standard toast by default; local opt-in Windows alarm scenario for XHIGH |
| REMINDER-BATCH-001 | reminder | 同时间批处理、STOP 与 SNOOZE | 是 | verified | platform-equivalent: grouped Scheduled Toast protocol actions without an intrusive application window |
| REMINDER-RECONCILE-001 | reminder | 开机、睡眠恢复和系统时间变化校正 | 是 | verified | platform-equivalent: autostart plus periodic reconciliation |
| SETTINGS-THEME-001 | settings | 浅色与深色主题 | 是 | verified | none |
| SETTINGS-DOCK-001 | settings | Dock 位置与最多四动作配置 | 是 | verified | none |
| SETTINGS-LANGUAGE-001 | settings | 七项语言两列排列、原生名称与 RTL | 是 | verified | none |
| SETTINGS-CLOUD-ICONS-001 | settings | 登录、镜像登出与同步图标几何 | 是 | verified | none |
| AUTH-001 | auth | 注册、登录、退出与密码重置 | 是 | verified | none |
| CREDENTIAL-001 | auth | 凭据安全存储 | 是 | verified | platform-equivalent: Windows Credential Manager |
| SYNC-MANUAL-001 | sync | 手动与自动同步 | 是 | verified | platform-equivalent: event-driven Realtime invalidation plus transactional pull; no fixed-interval polling |
| SYNC-CAS-001 | sync | 原子 mutation、CAS、change feed 与 schema negotiation | 是 | verified | none |
| SYNC-CONFLICT-001 | sync | 冲突记录、字段差异与 LOCAL/CLOUD 审查 | 是 | verified | none |
| SYNC-IMAGE-LOCAL-001 | sync | 两端图片严格本地且不影响其他字段同步 | 是 | verified | intentional shared exclusion: no Supabase Storage |
| SYNC-HTTP-001 | sync | 指定 Supabase endpoint 的长期 HTTP 正式策略 | 是 | verified | accepted insecure HTTP/ws transport risk; no HTTPS migration |
| UPDATE-SOURCE-001 | update | GitHub 优先与 Gitee 回退 | 是 | verified | none |
| UPDATE-CHANNEL-001 | update | stable 与 beta/RC 独立应用标识和清单 | 是 | verified | none |
| WINDOWS-TRAY-001 | platform | 托盘常驻、关闭隐藏、单实例与开机启动 | 是 | verified | platform-equivalent: Windows tray/autostart/single-instance |
| DATABASE-PATH-001 | storage | 官方 LocalAppData 分目录、SQLite 重启恢复与卸载保留 | 是 | verified | platform-equivalent local databases; no shared SQLite file |
| RELEASE-NSIS-001 | release | Windows x64 NSIS、SHA-256 与 Tauri updater signature | 是 | verified | platform-equivalent: unsigned-publisher NSIS plus signed updater; SmartScreen accepted |
| IMAGE-CLOUD-EXCLUDED | excluded | Supabase Storage 图片同步 | 否 | not_started | excluded_by_product_decision |
| SOURCE-BATCH-MOVE-001 | excluded | 源码存在但 Android UI 不可达的批量移动 | 否 | not_started | excluded_source_only |

正式发布门槛固定为 100.00%、0 blocked、0 in_progress、0 not_started、0 missing evidence。非 required 项只允许用于记录双方明确排除的源码能力。
