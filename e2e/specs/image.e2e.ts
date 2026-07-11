import { resolve } from 'node:path';
import { bootstrap, invoke } from '../helpers';

describe('Local image parity', () => {
  it('copies, previews and deletes one local task image without cloud metadata', async () => {
    let snapshot = await bootstrap();
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    const created = await invoke('create_todo', {
      expectedRevision: snapshot.revision,
      checklistId,
      draft: { title: 'IMAGE E2E', priority: 'MEDIUM', dueAtMillis: Date.now() + 60_000, reminderRepeat: 'NONE' }
    });
    const todoId = created.changedIds.find((id: string) => id !== checklistId);
    const attached = await invoke('attach_todo_image', {
      expectedRevision: created.revision,
      checklistId,
      todoId,
      sourcePath: resolve('src-tauri/icons/32x32.png')
    });
    const preview = await invoke<string>('load_todo_image_preview', { todoId });
    expect(preview.startsWith('data:image/png;base64,')).toBe(true);
    snapshot = await bootstrap();
    const todo = snapshot.checklists.find((list: any) => list.id === checklistId).items.find((item: any) => item.id === todoId);
    expect(todo.imageFileName).toMatch(/\.png$/);
    expect(todo).not.toHaveProperty('imageRemotePath');
    await invoke('delete_todo_image', { expectedRevision: attached.revision, checklistId, todoId });
    snapshot = await bootstrap();
    expect(snapshot.checklists.find((list: any) => list.id === checklistId).items.find((item: any) => item.id === todoId).imageFileName).toBeNull();
  });
});
