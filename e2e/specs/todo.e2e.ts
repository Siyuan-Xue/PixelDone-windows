import { bootstrap, invoke } from '../helpers';

describe('Todo and Dock parity', () => {
  it('persists fields, completion and list view controls', async () => {
    let snapshot = await bootstrap();
    const originalSettings = snapshot.settings;
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    const dueAtMillis = Date.now() + 86_400_000;
    const created = await invoke('create_todo', {
      expectedRevision: snapshot.revision,
      checklistId,
      draft: { title: 'E2E TODO', priority: 'XHIGH', dueAtMillis, reminderRepeat: 'DAILY' }
    });
    const todoId = created.changedIds.find((id: string) => id !== checklistId);
    const toggled = await invoke('toggle_todo', { expectedRevision: created.revision, checklistId, todoId });
    const sorted = await invoke('set_sort_mode', { expectedRevision: toggled.revision, sortMode: 'TIME' });
    const hidden = await invoke('set_hide_completed', { expectedRevision: sorted.revision, hideCompleted: true });
    await invoke('set_deadline_countdown', { expectedRevision: hidden.revision, visible: true });
    snapshot = await bootstrap();
    const todo = snapshot.checklists.find((list: any) => list.id === checklistId).items.find((item: any) => item.id === todoId);
    expect(todo).toMatchObject({ title: 'E2E TODO', priority: 'XHIGH', reminderRepeat: 'DAILY', completed: true });
    expect(todo.dueAtMillis).toBe(dueAtMillis);
    expect(snapshot).toMatchObject({ sortMode: 'TIME', hideCompleted: true, showDeadlineCountdown: true });

    const visible = await invoke('set_hide_completed', { expectedRevision: snapshot.revision, hideCompleted: false });
    const reactivated = await invoke('toggle_todo', { expectedRevision: visible.revision, checklistId, todoId });
    await invoke('select_checklist', { expectedRevision: reactivated.revision, checklistId });
    snapshot = await bootstrap();
    await invoke('update_settings', {
      expectedRevision: snapshot.revision,
      settings: { ...snapshot.settings, languageMode: 'SIMPLIFIED_CHINESE' }
    });
    await browser.refresh();
    const row = await $(`//*[contains(@class,'task-row')][.//strong[normalize-space(.)='E2E TODO']]`);
    await expect(row).toBeDisplayed();
    const metadata = await row.$('.task-copy > span').getText();
    expect(metadata).toContain('超高');
    expect(metadata).not.toContain('XHIGH');
    const completion = await row.$('button.completion-control');
    expect(await completion.getSize('width')).toBe(28);
    expect(await completion.getSize('height')).toBe(28);
    await completion.click();
    expect(await row.getAttribute('class')).toContain('held');
    await browser.waitUntil(async () => {
      const completedRow = await $(`//*[contains(@class,'task-row')][.//strong[normalize-space(.)='E2E TODO']]`);
      const classes = await completedRow.getAttribute('class');
      return Boolean(classes?.includes('completed') && !classes.includes('held'));
    });
    const completedRow = await $(`//*[contains(@class,'task-row')][.//strong[normalize-space(.)='E2E TODO']]`);
    expect(await completedRow.getAttribute('class')).not.toContain('held');
    expect(await completedRow.getAttribute('class')).toContain('completed');

    snapshot = await bootstrap();
    await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
  });
});
