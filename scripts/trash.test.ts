import { describe, expect, test } from 'bun:test';
import type { TodoItem, TodoPriority } from '$lib/generated/ipc';
import {
  filterTrashItems,
  trashSourceName,
  trashSourceOptions,
  type TrashFilters
} from '$lib/components/shell/trash';

function todo(
  id: string,
  title: string,
  priority: TodoPriority,
  checklistName: string | null
): TodoItem {
  return {
    id,
    title,
    priority,
    dueAtMillis: 0,
    completed: false,
    createdAtMillis: 1,
    updatedAtMillis: 1,
    reminderRepeat: 'NONE',
    imageFileName: null,
    trashedFromChecklistId: checklistName ? `list-${id}` : null,
    trashedFromChecklistName: checklistName,
    trashedAtMillis: 1,
    remoteVersion: null
  };
}

const items = [
  todo('one', 'Quarterly Report', 'XHIGH', 'WORK'),
  todo('two', 'quarterly notes', 'MEDIUM', 'PERSONAL'),
  todo('three', 'Buy milk', 'LOW', null)
];

function filters(overrides: Partial<TrashFilters> = {}): TrashFilters {
  return { query: '', priority: '', checklistName: '', ...overrides };
}

describe('Trash filters', () => {
  test('returns every item when all filters are empty', () => {
    expect(filterTrashItems(items, filters(), 'en')).toEqual(items);
  });

  test('matches trimmed todo-title text without case sensitivity', () => {
    expect(filterTrashItems(items, filters({ query: '  QUARTERLY  ' }), 'en').map((item) => item.id))
      .toEqual(['one', 'two']);
  });

  test('filters by exact priority and source checklist', () => {
    expect(filterTrashItems(items, filters({ priority: 'MEDIUM' }), 'en').map((item) => item.id))
      .toEqual(['two']);
    expect(filterTrashItems(items, filters({ checklistName: 'WORK' }), 'en').map((item) => item.id))
      .toEqual(['one']);
  });

  test('combines search, priority and source filters with AND semantics', () => {
    expect(filterTrashItems(items, filters({
      query: 'quarterly',
      priority: 'MEDIUM',
      checklistName: 'PERSONAL'
    }), 'en').map((item) => item.id)).toEqual(['two']);
    expect(filterTrashItems(items, filters({
      query: 'quarterly',
      priority: 'LOW',
      checklistName: 'PERSONAL'
    }), 'en')).toEqual([]);
  });

  test('uses MAIN for missing source names and exposes unique sorted options', () => {
    expect(trashSourceName(items[2])).toBe('MAIN');
    expect(trashSourceOptions([...items, todo('four', 'Duplicate source', 'HIGH', 'WORK')], 'en'))
      .toEqual(['MAIN', 'PERSONAL', 'WORK']);
    expect(filterTrashItems(items, filters({ checklistName: 'MAIN' }), 'en').map((item) => item.id))
      .toEqual(['three']);
  });
});
