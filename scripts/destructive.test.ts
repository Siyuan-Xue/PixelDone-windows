import { describe, expect, it } from 'bun:test';
import {
  checklistConfirmation,
  completedConfirmation,
  destructivePolicy,
  legacyDataConfirmation,
  todoConfirmation,
  trashAllConfirmation,
  trashItemConfirmation
} from '../src/lib/components/shell/destructive';

describe('destructive confirmation policy', () => {
  it('keeps stable targets for every destructive action', () => {
    expect(todoConfirmation('list-1', 'todo-1', 'Write report')).toMatchObject({
      kind: 'todo', id: 'todo:list-1:todo-1', checklistId: 'list-1', todoId: 'todo-1'
    });
    expect(completedConfirmation('list-1', 3)).toMatchObject({
      kind: 'completed', id: 'completed:list-1', checklistId: 'list-1', count: 3
    });
    expect(checklistConfirmation('list-1', 'Work')).toMatchObject({
      kind: 'checklist', id: 'checklist:list-1', checklistId: 'list-1'
    });
    expect(trashItemConfirmation('todo-2', 'Archived')).toMatchObject({
      kind: 'trash-item', id: 'trash-item:todo-2', todoId: 'todo-2'
    });
    expect(trashAllConfirmation(7)).toMatchObject({ kind: 'trash-all', id: 'trash-all', count: 7 });
    expect(legacyDataConfirmation('C:\\old\\pixeldone.sqlite3')).toMatchObject({
      kind: 'legacy-data', id: 'legacy-data:C:\\old\\pixeldone.sqlite3'
    });
  });

  it('marks reversible moves separately from permanent deletion', () => {
    const reversible = [
      todoConfirmation('list-1', 'todo-1', 'Write report'),
      completedConfirmation('list-1', 3),
      checklistConfirmation('list-1', 'Work')
    ];
    const permanent = [
      trashItemConfirmation('todo-2', 'Archived'),
      trashAllConfirmation(7),
      legacyDataConfirmation('C:\\old\\pixeldone.sqlite3')
    ];
    expect(reversible.map((action) => destructivePolicy(action).permanent)).toEqual([false, false, false]);
    expect(permanent.map((action) => destructivePolicy(action).permanent)).toEqual([true, true, true]);
  });

  it('provides the correct localized title, detail and count metadata', () => {
    expect(destructivePolicy(todoConfirmation('list-1', 'todo-1', 'Write report'))).toMatchObject({
      title: { source: 'shared', key: 'delete_task_title' },
      detailKey: 'deleteTodoDetail',
      target: { kind: 'label', value: 'Write report' }
    });
    expect(destructivePolicy(completedConfirmation('list-1', 3))).toMatchObject({
      title: { source: 'shared', key: 'delete_done_title' },
      detailKey: 'deleteCompletedDetail',
      target: { kind: 'count', count: 3 }
    });
    expect(destructivePolicy(checklistConfirmation('list-1', 'Work'))).toMatchObject({
      title: { source: 'shared', key: 'delete_list_title' },
      detailKey: 'deleteChecklistDetail'
    });
    expect(destructivePolicy(trashItemConfirmation('todo-2', 'Archived'))).toMatchObject({
      title: { source: 'windows', key: 'permanentlyDeleteTaskTitle' },
      detailKey: 'permanentlyDeleteTodoDetail'
    });
    expect(destructivePolicy(trashAllConfirmation(7))).toMatchObject({
      title: { source: 'shared', key: 'delete_trash_title' },
      detailKey: 'permanentlyDeleteTrashDetail',
      target: { kind: 'count', count: 7 },
      confirmKey: 'delete_all'
    });
    expect(destructivePolicy(legacyDataConfirmation('C:\\old\\pixeldone.sqlite3'))).toMatchObject({
      title: { source: 'windows', key: 'deleteLegacyTitle' },
      detailKey: 'deleteLegacyDetail',
      context: 'storage'
    });
  });
});
