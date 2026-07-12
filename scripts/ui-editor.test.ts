import { describe, expect, it } from 'bun:test';
import type { TodoItem } from '../src/lib/generated/ipc';
import { reconcileEditorMode } from '../src/lib/components/shell/editor';

const todo: TodoItem = {
  id: 'todo-1',
  title: 'Remote item',
  priority: 'MEDIUM',
  dueAtMillis: 0,
  completed: false,
  createdAtMillis: 0,
  updatedAtMillis: 0,
  reminderRepeat: 'NONE',
  imageFileName: null,
  trashedFromChecklistId: null,
  trashedFromChecklistName: null,
  trashedAtMillis: null,
  remoteVersion: null
};

describe('reconcileEditorMode', () => {
  it('keeps the modal open while the edited task exists', () => {
    expect(reconcileEditorMode({ kind: 'edit', todoId: todo.id }, [todo])).toEqual({
      kind: 'edit',
      todoId: todo.id
    });
  });

  it('closes the modal after a remote snapshot deletes the edited task', () => {
    expect(reconcileEditorMode({ kind: 'edit', todoId: todo.id }, [])).toEqual({ kind: 'closed' });
  });
});

