export type TodoPriority = 'XHIGH' | 'HIGH' | 'MEDIUM' | 'LOW';
export type ReminderRepeat = 'NONE' | 'DAILY' | 'WEEKLY';
export type SortMode = 'PRIORITY' | 'TIME';
export type ChecklistKind = 'NORMAL' | 'TRASH' | 'SETTINGS';
export type DockPlusPlacement = 'CENTER' | 'LEFT_EDGE' | 'RIGHT_EDGE';
export type DockAction = 'SORT' | 'DEADLINE' | 'HIDE_DONE' | 'DELETE_DONE' | 'BATCH_DELETE';

export interface TodoItem {
  id: string;
  title: string;
  priority: TodoPriority;
  dueAtMillis: number;
  completed: boolean;
  createdAtMillis: number;
  reminderRepeat: ReminderRepeat;
  imageFileName: string | null;
  trashedFromChecklistId: string | null;
  trashedFromChecklistName: string | null;
  trashedAtMillis: number | null;
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
}

export interface AppSnapshot {
  revision: number;
  checklists: Checklist[];
  selectedChecklistId: string;
  sortMode: SortMode;
  hideCompleted: boolean;
  quickDelete: boolean;
  settings: AppSettings;
}

export interface SnapshotDelta {
  upsertedChecklists: Checklist[];
  removedChecklistIds: string[];
  selectedChecklistId: string | null;
  sortMode: SortMode | null;
  hideCompleted: boolean | null;
  quickDelete: boolean | null;
  settings: AppSettings | null;
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
