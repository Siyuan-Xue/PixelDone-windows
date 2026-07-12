import type { TodoItem } from '$lib/generated/ipc';

export type TodoEditorMode =
  | { kind: 'closed' }
  | { kind: 'new' }
  | { kind: 'edit'; todoId: string };

/** Closes an edit surface when Realtime/snapshot replacement removes the edited task. */
export function reconcileEditorMode(mode: TodoEditorMode, items: TodoItem[]): TodoEditorMode {
  if (mode.kind !== 'edit') return mode;
  return items.some((item) => item.id === mode.todoId) ? mode : { kind: 'closed' };
}

