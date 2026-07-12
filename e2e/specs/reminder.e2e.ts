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

  it('keeps XHIGH non-intrusive by default', async () => {
    let snapshot = await bootstrap();
    expect(snapshot.settings.enhancedXhighAlarmEnabled).toBe(false);
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    await invoke('create_todo', {
      expectedRevision: snapshot.revision,
      checklistId,
      draft: { title: 'XHIGH STANDARD TOAST E2E', priority: 'XHIGH', dueAtMillis: Date.now() + 60_000, reminderRepeat: 'NONE' }
    });
    await browser.pause(1_000);
    expect(await browser.tauri.listWindows()).not.toContain('xhigh-alarm');
  });
});
