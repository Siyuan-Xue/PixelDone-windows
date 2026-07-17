import type { MessageKey } from '$lib/generated/i18n';
import type { WindowsMessageKey } from '$lib/i18n/windows';

export type DestructiveConfirmation =
  | { kind: 'todo'; id: string; checklistId: string; todoId: string; title: string }
  | { kind: 'completed'; id: string; checklistId: string; count: number }
  | { kind: 'checklist'; id: string; checklistId: string; name: string }
  | { kind: 'trash-item'; id: string; todoId: string; title: string }
  | { kind: 'trash-all'; id: string; count: number }
  | { kind: 'legacy-data'; id: string; path: string };

export type DestructiveContext = 'checklists' | 'trash' | 'storage';

export interface DestructivePolicy {
  context: DestructiveContext;
  title: { source: 'shared'; key: MessageKey } | { source: 'windows'; key: WindowsMessageKey };
  detailKey: WindowsMessageKey;
  target: { kind: 'label'; value: string } | { kind: 'count'; count: number };
  permanent: boolean;
  confirmKey: MessageKey;
}

export function todoConfirmation(checklistId: string, todoId: string, title: string): DestructiveConfirmation {
  return { kind: 'todo', id: `todo:${checklistId}:${todoId}`, checklistId, todoId, title };
}

export function completedConfirmation(checklistId: string, count: number): DestructiveConfirmation {
  return { kind: 'completed', id: `completed:${checklistId}`, checklistId, count };
}

export function checklistConfirmation(checklistId: string, name: string): DestructiveConfirmation {
  return { kind: 'checklist', id: `checklist:${checklistId}`, checklistId, name };
}

export function trashItemConfirmation(todoId: string, title: string): DestructiveConfirmation {
  return { kind: 'trash-item', id: `trash-item:${todoId}`, todoId, title };
}

export function trashAllConfirmation(count: number): DestructiveConfirmation {
  return { kind: 'trash-all', id: 'trash-all', count };
}

export function legacyDataConfirmation(path: string): DestructiveConfirmation {
  return { kind: 'legacy-data', id: `legacy-data:${path}`, path };
}

export function destructivePolicy(action: DestructiveConfirmation): DestructivePolicy {
  switch (action.kind) {
    case 'todo':
      return {
        context: 'checklists',
        title: { source: 'shared', key: 'delete_task_title' },
        detailKey: 'deleteTodoDetail',
        target: { kind: 'label', value: action.title },
        permanent: false,
        confirmKey: 'delete'
      };
    case 'completed':
      return {
        context: 'checklists',
        title: { source: 'shared', key: 'delete_done_title' },
        detailKey: 'deleteCompletedDetail',
        target: { kind: 'count', count: action.count },
        permanent: false,
        confirmKey: 'delete'
      };
    case 'checklist':
      return {
        context: 'checklists',
        title: { source: 'shared', key: 'delete_list_title' },
        detailKey: 'deleteChecklistDetail',
        target: { kind: 'label', value: action.name },
        permanent: false,
        confirmKey: 'delete'
      };
    case 'trash-item':
      return {
        context: 'trash',
        title: { source: 'windows', key: 'permanentlyDeleteTaskTitle' },
        detailKey: 'permanentlyDeleteTodoDetail',
        target: { kind: 'label', value: action.title },
        permanent: true,
        confirmKey: 'delete'
      };
    case 'trash-all':
      return {
        context: 'trash',
        title: { source: 'shared', key: 'delete_trash_title' },
        detailKey: 'permanentlyDeleteTrashDetail',
        target: { kind: 'count', count: action.count },
        permanent: true,
        confirmKey: 'delete_all'
      };
    case 'legacy-data':
      return {
        context: 'storage',
        title: { source: 'windows', key: 'deleteLegacyTitle' },
        detailKey: 'deleteLegacyDetail',
        target: { kind: 'label', value: action.path },
        permanent: true,
        confirmKey: 'delete'
      };
  }
}
