import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  AppSnapshot,
  Checklist,
  ConflictResolutionChoice,
  MutationResult,
  ReminderRepeat,
  SnapshotDelta,
  SortMode,
  TodoDraft,
  TodoItem,
  TodoPriority,
  SyncConflictView
  ,StorageInfo
} from '$lib/generated/ipc';

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

const now = Date.now();
let browserSnapshot: AppSnapshot = {
  revision: 0,
  selectedChecklistId: 'main',
  sortMode: 'PRIORITY',
  hideCompleted: false,
  quickDelete: false,
  showDeadlineCountdown: false,
  checklistHistory: [],
  settings: {
    darkTheme: false,
    dock: { plusPlacement: 'CENTER', actions: ['SORT', 'DEADLINE', 'HIDE_DONE', 'DELETE_DONE'] },
    neverShowUpdateDialog: false,
    futureSyncEnabled: false,
    languageMode: 'SYSTEM'
    ,autostartEnabled: true
    ,automaticUpdateCheckEnabled: true
    ,enhancedXhighAlarmEnabled: false
    ,sidebarWidthPx: 320
  },
  auth: { cloudAvailable: true, signedIn: false, userId: null, userEmail: null, insecureHttp: true },
  sync: { state: 'SIGNED_OUT', message: '浏览器预览模式', remoteVersion: null, pendingCount: 0, conflictCount: 0, insecureHttp: true },
  reminder: { state: 'IDLE', activeTodoIds: [], lastFiredAtMillis: null, scheduledCount: 0, scheduleHorizonAtMillis: null, scheduleTruncated: false, message: null },
  update: { state: 'CURRENT', currentVersion: '3.2.2', availableVersion: null, downloadUrl: null, source: 'preview', message: null, downloadedBytes: 0, totalBytes: null, lastCheckedAtMillis: null, nextCheckAtMillis: null },
  checklists: [
    {
      id: 'main',
      name: 'MAIN',
      kind: 'NORMAL',
      createdAtMillis: now,
      updatedAtMillis: now,
      remoteVersion: null,
      items: [
        demoTodo('demo-xhigh', '完成 PixelDone Windows 架构审查', 'XHIGH', now + 45 * 60_000),
        demoTodo('demo-medium', '核对 Android 3.0.3 功能矩阵', 'MEDIUM', now + 3 * 60 * 60_000),
        { ...demoTodo('demo-done', '保留官方 Tauri 模板提交', 'LOW', now - 60_000), completed: true }
      ]
    },
    { id: 'work', name: 'WORK', kind: 'NORMAL', createdAtMillis: now + 1, updatedAtMillis: now + 1, remoteVersion: null, items: [] },
    { id: 'trash', name: 'TRASH', kind: 'TRASH', createdAtMillis: now, updatedAtMillis: now, remoteVersion: null, items: [] },
    { id: 'settings', name: 'SETTINGS', kind: 'SETTINGS', createdAtMillis: now, updatedAtMillis: now, remoteVersion: null, items: [] }
  ]
};

function demoTodo(id: string, title: string, priority: TodoPriority, dueAtMillis: number): TodoItem {
  return {
    id,
    title,
    priority,
    dueAtMillis,
    completed: false,
    createdAtMillis: now,
    updatedAtMillis: now,
    reminderRepeat: 'NONE',
    imageFileName: null,
    trashedFromChecklistId: null,
    trashedFromChecklistName: null,
    trashedAtMillis: null,
    remoteVersion: null
  };
}

function isTauri(): boolean {
  return typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__);
}

function cloneSnapshot(): AppSnapshot {
  return structuredClone(browserSnapshot);
}

function diff(before: AppSnapshot, after: AppSnapshot, changedIds: string[]): MutationResult {
  const upsertedChecklists = after.checklists.filter((list) => {
    const old = before.checklists.find((candidate) => candidate.id === list.id);
    return JSON.stringify(old) !== JSON.stringify(list);
  });
  const snapshotDelta: SnapshotDelta = {
    upsertedChecklists,
    removedChecklistIds: before.checklists
      .filter((list) => !after.checklists.some((candidate) => candidate.id === list.id))
      .map((list) => list.id),
    selectedChecklistId:
      before.selectedChecklistId === after.selectedChecklistId ? null : after.selectedChecklistId,
    sortMode: before.sortMode === after.sortMode ? null : after.sortMode,
    hideCompleted: before.hideCompleted === after.hideCompleted ? null : after.hideCompleted,
    quickDelete: before.quickDelete === after.quickDelete ? null : after.quickDelete,
    showDeadlineCountdown: before.showDeadlineCountdown === after.showDeadlineCountdown ? null : after.showDeadlineCountdown,
    checklistHistory: JSON.stringify(before.checklistHistory) === JSON.stringify(after.checklistHistory) ? null : after.checklistHistory,
    settings: JSON.stringify(before.settings) === JSON.stringify(after.settings) ? null : after.settings,
    auth: JSON.stringify(before.auth) === JSON.stringify(after.auth) ? null : after.auth,
    sync: JSON.stringify(before.sync) === JSON.stringify(after.sync) ? null : after.sync,
    reminder: JSON.stringify(before.reminder) === JSON.stringify(after.reminder) ? null : after.reminder,
    update: JSON.stringify(before.update) === JSON.stringify(after.update) ? null : after.update
  };
  return { revision: after.revision, changedIds, snapshotDelta };
}

function browserMutation(
  expectedRevision: number,
  operation: (snapshot: AppSnapshot) => string[]
): MutationResult {
  if (expectedRevision !== browserSnapshot.revision) {
    throw { code: 'STALE_REVISION', message: '当前界面状态已过期，请重试' };
  }
  const before = cloneSnapshot();
  const next = cloneSnapshot();
  const changedIds = operation(next);
  next.revision += 1;
  browserSnapshot = next;
  return diff(before, next, changedIds);
}

function normalList(snapshot: AppSnapshot, id: string): Checklist {
  const list = snapshot.checklists.find((candidate) => candidate.id === id && candidate.kind === 'NORMAL');
  if (!list) throw { code: 'NOT_FOUND', message: '未找到普通清单' };
  return list;
}

function randomId(): string {
  return crypto.randomUUID();
}

function validatedTitle(draft: TodoDraft): TodoDraft {
  const title = draft.title.trim();
  if (!title) throw { code: 'VALIDATION_ERROR', message: '任务标题不能为空' };
  return { ...draft, title };
}

export const api = {
  async bootstrap(): Promise<AppSnapshot> {
    return isTauri() ? invoke('bootstrap') : cloneSnapshot();
  },
  async selectChecklist(expectedRevision: number, checklistId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('select_checklist', { expectedRevision, checklistId });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.selectedChecklistId = checklistId;
      snapshot.checklistHistory.push(browserSnapshot.selectedChecklistId);
      return [checklistId];
    });
  },
  async createChecklist(expectedRevision: number, name: string): Promise<MutationResult> {
    if (isTauri()) return invoke('create_checklist', { expectedRevision, name });
    return browserMutation(expectedRevision, (snapshot) => {
      const normalized = name.trim();
      if (!normalized) throw { code: 'VALIDATION_ERROR', message: '清单名不能为空' };
      const id = randomId();
      const index = snapshot.checklists.findIndex((list) => list.kind !== 'NORMAL');
      snapshot.checklists.splice(index, 0, {
        id,
        name: normalized,
        kind: 'NORMAL',
        items: [],
        createdAtMillis: Date.now(),
        updatedAtMillis: Date.now(),
        remoteVersion: null
      });
      snapshot.selectedChecklistId = id;
      return [id];
    });
  },
  async renameChecklist(expectedRevision: number, checklistId: string, name: string): Promise<MutationResult> {
    if (isTauri()) return invoke('rename_checklist', { expectedRevision, checklistId, name });
    return browserMutation(expectedRevision, (snapshot) => {
      normalList(snapshot, checklistId).name = name.trim();
      return [checklistId];
    });
  },
  async deleteChecklist(expectedRevision: number, checklistId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('delete_checklist', { expectedRevision, checklistId });
    return browserMutation(expectedRevision, (snapshot) => {
      const normal = snapshot.checklists.filter((list) => list.kind === 'NORMAL');
      if (normal.length <= 1) throw { code: 'VALIDATION_ERROR', message: '至少保留一个普通清单' };
      const source = normalList(snapshot, checklistId);
      const trash = snapshot.checklists.find((list) => list.kind === 'TRASH')!;
      trash.items.push(
        ...source.items.map((item) => ({
          ...item,
          trashedFromChecklistId: source.id,
          trashedFromChecklistName: source.name,
          trashedAtMillis: Date.now()
        }))
      );
      snapshot.checklists = snapshot.checklists.filter((list) => list.id !== checklistId);
      if (snapshot.selectedChecklistId === checklistId) {
        snapshot.selectedChecklistId = snapshot.checklists.find((list) => list.kind === 'NORMAL')!.id;
      }
      return [checklistId, 'trash'];
    });
  },
  async createTodo(expectedRevision: number, checklistId: string, draft: TodoDraft): Promise<MutationResult> {
    if (isTauri()) return invoke('create_todo', { expectedRevision, checklistId, draft });
    return browserMutation(expectedRevision, (snapshot) => {
      const value = validatedTitle(draft);
      const id = randomId();
      normalList(snapshot, checklistId).items.push({
        id,
        ...value,
        completed: false,
        createdAtMillis: Date.now(),
        updatedAtMillis: Date.now(),
        imageFileName: null,
        trashedFromChecklistId: null,
        trashedFromChecklistName: null,
        trashedAtMillis: null,
        remoteVersion: null
      });
      return [checklistId, id];
    });
  },
  async updateTodo(
    expectedRevision: number,
    checklistId: string,
    todoId: string,
    draft: TodoDraft
  ): Promise<MutationResult> {
    if (isTauri()) return invoke('update_todo', { expectedRevision, checklistId, todoId, draft });
    return browserMutation(expectedRevision, (snapshot) => {
      const value = validatedTitle(draft);
      const item = normalList(snapshot, checklistId).items.find((candidate) => candidate.id === todoId)!;
      Object.assign(item, value);
      return [checklistId, todoId];
    });
  },
  async toggleTodo(expectedRevision: number, checklistId: string, todoId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('toggle_todo', { expectedRevision, checklistId, todoId });
    return browserMutation(expectedRevision, (snapshot) => {
      const item = normalList(snapshot, checklistId).items.find((candidate) => candidate.id === todoId)!;
      item.completed = !item.completed;
      return [checklistId, todoId];
    });
  },
  async moveTodoToTrash(expectedRevision: number, checklistId: string, todoId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('move_todo_to_trash', { expectedRevision, checklistId, todoId });
    return browserMutation(expectedRevision, (snapshot) => {
      const source = normalList(snapshot, checklistId);
      const index = source.items.findIndex((item) => item.id === todoId);
      const [item] = source.items.splice(index, 1);
      snapshot.checklists.find((list) => list.kind === 'TRASH')!.items.push({
        ...item,
        trashedFromChecklistId: source.id,
        trashedFromChecklistName: source.name,
        trashedAtMillis: Date.now()
      });
      return [checklistId, 'trash', todoId];
    });
  },
  async restoreTodo(expectedRevision: number, todoId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('restore_todo', { expectedRevision, todoId });
    return browserMutation(expectedRevision, (snapshot) => {
      const trash = snapshot.checklists.find((list) => list.kind === 'TRASH')!;
      const index = trash.items.findIndex((item) => item.id === todoId);
      const [item] = trash.items.splice(index, 1);
      let target = snapshot.checklists.find((list) => list.id === item.trashedFromChecklistId);
      if (!target) {
        target = {
          id: item.trashedFromChecklistId || 'main',
          name: item.trashedFromChecklistName || 'MAIN',
          kind: 'NORMAL',
          items: [],
          createdAtMillis: Date.now()
          ,updatedAtMillis: Date.now(), remoteVersion: null
        };
        snapshot.checklists.splice(snapshot.checklists.findIndex((list) => list.kind !== 'NORMAL'), 0, target);
      }
      target.items.push({
        ...item,
        trashedFromChecklistId: null,
        trashedFromChecklistName: null,
        trashedAtMillis: null
      });
      return ['trash', target.id, todoId];
    });
  },
  async purgeTodo(expectedRevision: number, todoId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('purge_todo', { expectedRevision, todoId });
    return browserMutation(expectedRevision, (snapshot) => {
      const trash = snapshot.checklists.find((list) => list.kind === 'TRASH')!;
      trash.items = trash.items.filter((item) => item.id !== todoId);
      return ['trash', todoId];
    });
  },
  async cleanCompleted(expectedRevision: number, checklistId: string): Promise<MutationResult> {
    if (isTauri()) return invoke('clean_completed', { expectedRevision, checklistId });
    return browserMutation(expectedRevision, (snapshot) => {
      const source = normalList(snapshot, checklistId);
      const moved = source.items.filter((item) => item.completed);
      source.items = source.items.filter((item) => !item.completed);
      snapshot.checklists.find((list) => list.kind === 'TRASH')!.items.push(
        ...moved.map((item) => ({
          ...item,
          trashedFromChecklistId: source.id,
          trashedFromChecklistName: source.name,
          trashedAtMillis: Date.now()
        }))
      );
      return [checklistId, 'trash', ...moved.map((item) => item.id)];
    });
  },
  async setSortMode(expectedRevision: number, sortMode: SortMode): Promise<MutationResult> {
    if (isTauri()) return invoke('set_sort_mode', { expectedRevision, sortMode });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.sortMode = sortMode;
      return ['sort-mode'];
    });
  },
  async setHideCompleted(expectedRevision: number, hideCompleted: boolean): Promise<MutationResult> {
    if (isTauri()) return invoke('set_hide_completed', { expectedRevision, hideCompleted });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.hideCompleted = hideCompleted;
      return ['hide-completed'];
    });
  },
  async setQuickDelete(expectedRevision: number, quickDelete: boolean): Promise<MutationResult> {
    if (isTauri()) return invoke('set_quick_delete', { expectedRevision, quickDelete });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.quickDelete = quickDelete;
      return ['quick-delete'];
    });
  },
  async updateSettings(expectedRevision: number, settings: AppSettings): Promise<MutationResult> {
    if (isTauri()) return invoke('update_settings', { expectedRevision, settings });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.settings = jsonClone(settings);
      return ['settings'];
    });
  },
  async backChecklist(expectedRevision: number): Promise<MutationResult> {
    if (isTauri()) return invoke('back_checklist', { expectedRevision });
    return browserMutation(expectedRevision, (snapshot) => {
      const id = snapshot.checklistHistory.pop();
      if (id) snapshot.selectedChecklistId = id;
      return id ? [id] : [];
    });
  },
  async purgeAllTrash(expectedRevision: number): Promise<MutationResult> {
    if (isTauri()) return invoke('purge_all_trash', { expectedRevision });
    return browserMutation(expectedRevision, (snapshot) => {
      const trash = snapshot.checklists.find((list) => list.kind === 'TRASH')!;
      const ids = trash.items.map((item) => item.id);
      trash.items = [];
      return ['trash', ...ids];
    });
  },
  async setDeadlineCountdown(expectedRevision: number, visible: boolean): Promise<MutationResult> {
    if (isTauri()) return invoke('set_deadline_countdown', { expectedRevision, visible });
    return browserMutation(expectedRevision, (snapshot) => {
      snapshot.showDeadlineCountdown = visible;
      return ['deadline-countdown'];
    });
  },
  async signIn(expectedRevision: number, email: string, password: string): Promise<MutationResult> {
    return invoke('auth_sign_in', { expectedRevision, email, password });
  },
  async signUp(expectedRevision: number, email: string, password: string): Promise<MutationResult> {
    return invoke('auth_sign_up', { expectedRevision, email, password });
  },
  async signOut(expectedRevision: number): Promise<MutationResult> {
    return invoke('auth_sign_out', { expectedRevision });
  },
  async changePassword(expectedRevision: number, currentPassword: string, newPassword: string, confirmation: string): Promise<MutationResult> {
    return invoke('auth_change_password', { expectedRevision, currentPassword, newPassword, confirmation });
  },
  async syncNow(expectedRevision: number): Promise<MutationResult> {
    return invoke('sync_now', { expectedRevision });
  },
  async loadConflicts(): Promise<SyncConflictView[]> {
    return isTauri() ? invoke('load_sync_conflicts') : [];
  },
  async resolveConflict(expectedRevision: number, recordType: string, localId: string, choice: ConflictResolutionChoice): Promise<MutationResult> {
    return invoke('resolve_sync_conflict', { expectedRevision, recordType, localId, choice });
  },
  async attachImage(expectedRevision: number, checklistId: string, todoId: string, sourcePath: string): Promise<MutationResult> {
    return invoke('attach_todo_image', { expectedRevision, checklistId, todoId, sourcePath });
  },
  async deleteImage(expectedRevision: number, checklistId: string, todoId: string): Promise<MutationResult> {
    return invoke('delete_todo_image', { expectedRevision, checklistId, todoId });
  },
  async loadImagePreview(todoId: string): Promise<string> {
    return invoke('load_todo_image_preview', { todoId });
  },
  async checkForUpdate(expectedRevision: number): Promise<MutationResult> {
    return invoke('check_for_update', { expectedRevision });
  },
  async installUpdate(): Promise<void> {
    return invoke('download_and_install_update');
  },
  async getStorageInfo(): Promise<StorageInfo> {
    if (isTauri()) return invoke('get_storage_info');
    return {
      executablePath: 'browser-preview/PixelDone.exe',
      installDirectory: 'browser-preview',
      dataRoot: 'browser-preview/data',
      databasePath: 'browser-preview/data/pixeldone.sqlite3',
      attachmentsPath: 'browser-preview/attachments',
      cachePath: 'browser-preview/cache',
      logsPath: 'browser-preview/logs',
      webviewDataPath: 'browser-preview/EBWebView',
      totalBytes: 0,
      legacyRoamingDatabasePath: null,
      legacyRoamingDatabaseBytes: null,
      credentialManagerTarget: 'com.milesxue.pixeldone.windows/supabase-session'
    };
  },
  async openDataFolder(): Promise<void> {
    if (isTauri()) return invoke('open_data_folder');
  },
  async deleteLegacyRoamingData(confirmed: boolean): Promise<void> {
    if (isTauri()) return invoke('delete_legacy_roaming_data', { confirmed });
  },
  async stopReminder(expectedRevision: number, todoIds: string[]): Promise<MutationResult> {
    return invoke('stop_reminder', { expectedRevision, todoIds });
  },
  async snoozeReminder(expectedRevision: number, todoIds: string[]): Promise<MutationResult> {
    return invoke('snooze_reminder', { expectedRevision, todoIds });
  }
};

export function applyMutation(snapshot: AppSnapshot, result: MutationResult): AppSnapshot {
  const next = jsonClone(snapshot);
  const { snapshotDelta: delta } = result;
  next.revision = result.revision;
  next.checklists = next.checklists.filter((list) => !delta.removedChecklistIds.includes(list.id));
  for (const upserted of delta.upsertedChecklists) {
    const index = next.checklists.findIndex((list) => list.id === upserted.id);
    if (index >= 0) next.checklists[index] = upserted;
    else {
      const specialIndex = next.checklists.findIndex((list) => list.kind !== 'NORMAL');
      next.checklists.splice(specialIndex < 0 ? next.checklists.length : specialIndex, 0, upserted);
    }
  }
  if (delta.selectedChecklistId !== null) next.selectedChecklistId = delta.selectedChecklistId;
  if (delta.sortMode !== null) next.sortMode = delta.sortMode;
  if (delta.hideCompleted !== null) next.hideCompleted = delta.hideCompleted;
  if (delta.quickDelete !== null) next.quickDelete = delta.quickDelete;
  if (delta.showDeadlineCountdown !== null) next.showDeadlineCountdown = delta.showDeadlineCountdown;
  if (delta.checklistHistory !== null) next.checklistHistory = delta.checklistHistory;
  if (delta.settings !== null) next.settings = delta.settings;
  if (delta.auth !== null) next.auth = delta.auth;
  if (delta.sync !== null) next.sync = delta.sync;
  if (delta.reminder !== null) next.reminder = delta.reminder;
  if (delta.update !== null) next.update = delta.update;
  return next;
}

function jsonClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export function emptyDraft(): TodoDraft {
  return { title: '', priority: 'MEDIUM', dueAtMillis: Date.now() + 60 * 60_000, reminderRepeat: 'NONE' };
}

export function dateTimeLocalValue(millis: number): string {
  const date = new Date(millis);
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

export function millisFromDateTimeLocal(value: string): number {
  return value ? new Date(value).getTime() : 0;
}

export function repeatLabel(repeat: ReminderRepeat): string {
  return repeat === 'NONE' ? '不重复' : repeat === 'DAILY' ? '每天' : '每周';
}
