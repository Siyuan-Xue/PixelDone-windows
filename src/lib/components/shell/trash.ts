import type { TodoItem, TodoPriority } from '$lib/generated/ipc';

export type TrashPriorityFilter = TodoPriority | '';

export interface TrashFilters {
  query: string;
  priority: TrashPriorityFilter;
  checklistName: string;
}

export function trashSourceName(item: TodoItem): string {
  return item.trashedFromChecklistName?.trim() || 'MAIN';
}

export function trashSourceOptions(items: TodoItem[], locale: string): string[] {
  const names = [...new Set(items.map(trashSourceName))];
  return names.sort(new Intl.Collator(locale, { sensitivity: 'base' }).compare);
}

export function filterTrashItems(
  items: TodoItem[],
  filters: TrashFilters,
  locale: string
): TodoItem[] {
  const query = filters.query.trim().toLocaleLowerCase(locale);
  return items.filter((item) => {
    if (query && !item.title.toLocaleLowerCase(locale).includes(query)) return false;
    if (filters.priority && item.priority !== filters.priority) return false;
    if (filters.checklistName && trashSourceName(item) !== filters.checklistName) return false;
    return true;
  });
}
