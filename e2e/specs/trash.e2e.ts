import { bootstrap, invoke } from '../helpers';

describe('Trash parity', () => {
  it('restores a todo and recreates its deleted source checklist', async () => {
    let snapshot = await bootstrap();
    const listResult = await invoke('create_checklist', { expectedRevision: snapshot.revision, name: 'RESTORE SOURCE' });
    snapshot = await bootstrap();
    const checklistId = snapshot.selectedChecklistId;
    const todoResult = await invoke('create_todo', {
      expectedRevision: listResult.revision,
      checklistId,
      draft: { title: 'RESTORE ME', priority: 'LOW', dueAtMillis: Date.now() + 60_000, reminderRepeat: 'NONE' }
    });
    const todoId = todoResult.changedIds.find((id: string) => id !== checklistId);
    const trashed = await invoke('move_todo_to_trash', { expectedRevision: todoResult.revision, checklistId, todoId });
    const deleted = await invoke('delete_checklist', { expectedRevision: trashed.revision, checklistId });
    await invoke('restore_todo', { expectedRevision: deleted.revision, todoId });
    snapshot = await bootstrap();
    const restored = snapshot.checklists.find((list: any) => list.id === checklistId);
    expect(restored?.name).toBe('RESTORE SOURCE');
    expect(restored?.items.some((item: any) => item.id === todoId)).toBe(true);
  });
});
