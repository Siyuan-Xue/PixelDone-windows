import { bootstrap, invoke } from '../helpers';

describe('Reminder parity', () => {
  it('snoozes and stops a reminder through Rust commands', async () => {
    let snapshot = await bootstrap();
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    const created = await invoke('create_todo', {
      expectedRevision: snapshot.revision,
      checklistId,
      draft: { title: 'REMINDER E2E', priority: 'XHIGH', dueAtMillis: Date.now() - 1_000, reminderRepeat: 'NONE' }
    });
    const todoId = created.changedIds.find((id: string) => id !== checklistId);
    const snoozed = await invoke('snooze_reminder', { expectedRevision: created.revision, todoIds: [todoId] });
    snapshot = await bootstrap();
    expect(snapshot.reminder.state).toBe('SNOOZED');
    expect(snapshot.reminder.activeTodoIds).not.toContain(todoId);
    await invoke('stop_reminder', { expectedRevision: snoozed.revision, todoIds: [todoId] });
    snapshot = await bootstrap();
    expect(snapshot.reminder.activeTodoIds).not.toContain(todoId);
  });

  it('opens the dedicated XHIGH alarm window for a due task', async () => {
    let snapshot = await bootstrap();
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    const created = await invoke('create_todo', {
      expectedRevision: snapshot.revision,
      checklistId,
      draft: { title: 'XHIGH WINDOW E2E', priority: 'XHIGH', dueAtMillis: Date.now() - 1_000, reminderRepeat: 'NONE' }
    });
    const todoId = created.changedIds.find((id: string) => id !== checklistId);
    await browser.pause(16_000);
    snapshot = await bootstrap();
    expect(snapshot.reminder.state).toBe('XHIGH');
    expect(snapshot.reminder.activeTodoIds).toContain(todoId);
    expect(await browser.tauri.listWindows()).toContain('xhigh-alarm');
    await invoke('stop_reminder', { expectedRevision: snapshot.revision, todoIds: [todoId] });
  });
});
