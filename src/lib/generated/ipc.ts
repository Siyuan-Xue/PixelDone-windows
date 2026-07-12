export type TodoPriority = 'XHIGH' | 'HIGH' | 'MEDIUM' | 'LOW';
export type ReminderRepeat = 'NONE' | 'DAILY' | 'WEEKLY';
export type SortMode = 'PRIORITY' | 'TIME';
export type ChecklistKind = 'NORMAL' | 'TRASH' | 'SETTINGS';
export type DockPlusPlacement = 'CENTER' | 'LEFT_EDGE' | 'RIGHT_EDGE';
export type DockAction = 'SORT' | 'DEADLINE' | 'HIDE_DONE' | 'DELETE_DONE' | 'BATCH_DELETE';
export type AppLanguage = 'SYSTEM' | 'ENGLISH' | 'SIMPLIFIED_CHINESE' | 'ARABIC' | 'FRENCH' | 'RUSSIAN' | 'SPANISH';
export type SyncState = 'LOCAL_ONLY' | 'SIGNED_OUT' | 'IDLE' | 'SYNCING' | 'SYNCED' | 'CONFLICT' | 'ERROR' | 'SERVER_UPDATE_REQUIRED';
export type ConflictResolutionChoice = 'KEEP_LOCAL' | 'KEEP_CLOUD';

export interface TodoItem {
  id: string;
  title: string;
  priority: TodoPriority;
  dueAtMillis: number;
  completed: boolean;
  createdAtMillis: number;
  updatedAtMillis: number;
  reminderRepeat: ReminderRepeat;
  imageFileName: string | null;
  trashedFromChecklistId: string | null;
  trashedFromChecklistName: string | null;
  trashedAtMillis: number | null;
  remoteVersion: number | null;
}

export interface TodoDraft {
  title: string;
  priority: TodoPriority;
  dueAtMillis: number;
  reminderRepeat: ReminderRepeat;
}

export interface Checklist {
  id: string;
  name: string;
  kind: ChecklistKind;
  items: TodoItem[];
  createdAtMillis: number;
  updatedAtMillis: number;
  remoteVersion: number | null;
}

export interface DockConfig {
  plusPlacement: DockPlusPlacement;
  actions: DockAction[];
}

export interface AppSettings {
  darkTheme: boolean;
  dock: DockConfig;
  neverShowUpdateDialog: boolean;
  futureSyncEnabled: boolean;
  languageMode: AppLanguage;
  autostartEnabled: boolean;
  automaticUpdateCheckEnabled: boolean;
  enhancedXhighAlarmEnabled: boolean;
}

export interface StorageInfo {
  executablePath: string;
  installDirectory: string;
  dataRoot: string;
  databasePath: string;
  attachmentsPath: string;
  cachePath: string;
  logsPath: string;
  webviewDataPath: string;
  totalBytes: number;
  legacyRoamingDatabasePath: string | null;
  legacyRoamingDatabaseBytes: number | null;
  credentialManagerTarget: string;
}

export interface AuthView {
  cloudAvailable: boolean;
  signedIn: boolean;
  userId: string | null;
  userEmail: string | null;
  insecureHttp: boolean;
}

export interface SyncRunView {
  state: SyncState;
  message: string | null;
  remoteVersion: number | null;
  pendingCount: number;
  conflictCount: number;
  insecureHttp: boolean;
}

export interface SyncConflictView {
  recordType: string;
  localId: string;
  title: string;
  fields: string[];
  localPayload: Record<string, unknown> | null;
  cloudPayload: Record<string, unknown> | null;
  message: string;
}

export interface ReminderRunView {
  state: string;
  activeTodoIds: string[];
  lastFiredAtMillis: number | null;
  scheduledCount: number;
  scheduleHorizonAtMillis: number | null;
  scheduleTruncated: boolean;
  message: string | null;
}

export interface UpdateView {
  state: string;
  currentVersion: string;
  availableVersion: string | null;
  downloadUrl: string | null;
  source: string | null;
  message: string | null;
  downloadedBytes: number;
  totalBytes: number | null;
  lastCheckedAtMillis: number | null;
  nextCheckAtMillis: number | null;
}

export interface AppSnapshot {
  revision: number;
  checklists: Checklist[];
  selectedChecklistId: string;
  sortMode: SortMode;
  hideCompleted: boolean;
  quickDelete: boolean;
  showDeadlineCountdown: boolean;
  checklistHistory: string[];
  settings: AppSettings;
  auth: AuthView;
  sync: SyncRunView;
  reminder: ReminderRunView;
  update: UpdateView;
}

export interface SnapshotDelta {
  upsertedChecklists: Checklist[];
  removedChecklistIds: string[];
  selectedChecklistId: string | null;
  sortMode: SortMode | null;
  hideCompleted: boolean | null;
  quickDelete: boolean | null;
  showDeadlineCountdown: boolean | null;
  checklistHistory: string[] | null;
  settings: AppSettings | null;
  auth: AuthView | null;
  sync: SyncRunView | null;
  reminder: ReminderRunView | null;
  update: UpdateView | null;
}

export interface MutationResult {
  revision: number;
  changedIds: string[];
  snapshotDelta: SnapshotDelta;
}

export interface AppError {
  code: string;
  message: string;
}
