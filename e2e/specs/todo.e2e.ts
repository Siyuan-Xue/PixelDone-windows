import { bootstrap, invoke } from '../helpers';

async function invokeWithLatestRevision(command: string, args: Record<string, unknown>): Promise<any> {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const snapshot = await bootstrap();
    try {
      return await invoke(command, { ...args, expectedRevision: snapshot.revision });
    } catch (error) {
      if (attempt === 2) throw error;
    }
  }
}

describe('Todo and Dock parity', () => {
  it('confirms recoverable deletes while keeping quick delete one-click', async () => {
    const initial = await bootstrap();
    const originalSettings = initial.settings;
    const originalSelection = initial.selectedChecklistId;
    const originalHideCompleted = initial.hideCompleted;
    const originalQuickDelete = initial.quickDelete;
    let checklistId: string | null = null;
    const todoIds: string[] = [];

    async function createTodo(title: string, completed = false): Promise<string> {
      let snapshot = await bootstrap();
      const created = await invoke('create_todo', {
        expectedRevision: snapshot.revision,
        checklistId,
        draft: { title, priority: 'MEDIUM', dueAtMillis: 0, reminderRepeat: 'NONE' }
      });
      const todoId = created.changedIds.find((id: string) => id !== checklistId);
      if (!todoId) throw new Error(`Created todo id missing for ${title}`);
      todoIds.push(todoId);
      if (completed) {
        await invoke('toggle_todo', { expectedRevision: created.revision, checklistId, todoId });
      }
      return todoId;
    }

    try {
      let snapshot = await bootstrap();
      await invoke('create_checklist', {
        expectedRevision: snapshot.revision,
        name: 'E2E DESTRUCTIVE CONFIRM'
      });
      snapshot = await bootstrap();
      checklistId = snapshot.selectedChecklistId;
      const editorTodo = await createTodo('EDITOR DELETE CONFIRM');
      const completedTodo = await createTodo('COMPLETED DELETE CONFIRM', true);
      const keepTodo = await createTodo('KEEP ACTIVE');

      snapshot = await bootstrap();
      const settingsUpdated = await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: {
          ...snapshot.settings,
          languageMode: 'ENGLISH',
          dock: { ...snapshot.settings.dock, actions: ['SORT', 'DELETE_DONE', 'BATCH_DELETE'] }
        }
      });
      const shown = await invoke('set_hide_completed', {
        expectedRevision: settingsUpdated.revision,
        hideCompleted: false
      });
      await invoke('set_quick_delete', { expectedRevision: shown.revision, quickDelete: false });
      await browser.refresh();

      const editorRow = $(`//*[contains(@class,'task-row')][.//strong[normalize-space(.)='EDITOR DELETE CONFIRM']]`);
      await editorRow.$('.task-open').click();
      await expect($('.todo-editor-modal')).toBeDisplayed();
      const editorDelete = $('.editor-delete');
      await editorDelete.click();
      await expect($('.destructive-confirmation-modal')).toBeDisplayed();
      await expect($('.destructive-confirmation-target')).toHaveText('EDITOR DELETE CONFIRM');
      const confirmationLayer = await browser.execute(() => {
        const confirmation = document.querySelector<HTMLElement>('.destructive-confirmation-backdrop')!;
        const editor = document.querySelector<HTMLElement>('.editor-backdrop')!;
        const confirmButton = document.querySelector<HTMLElement>('.destructive-confirmation-confirm')!;
        const rect = confirmButton.getBoundingClientRect();
        const topElement = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
        return {
          aboveEditor: Number.parseInt(getComputedStyle(confirmation).zIndex, 10)
            > Number.parseInt(getComputedStyle(editor).zIndex, 10),
          confirmButtonReceivesPointer: topElement === confirmButton || confirmButton.contains(topElement)
        };
      });
      expect(confirmationLayer).toEqual({ aboveEditor: true, confirmButtonReceivesPointer: true });
      expect((await bootstrap()).checklists.find((list: any) => list.id === checklistId).items
        .some((item: any) => item.id === editorTodo)).toBe(true);
      await $('.destructive-confirmation-actions .quiet-button').click();
      await expect(editorDelete).toBeFocused();
      await editorDelete.click();
      await $('.destructive-confirmation-confirm').click();
      await browser.waitUntil(async () => !(await $('.todo-editor-modal').isExisting()));
      snapshot = await bootstrap();
      expect(snapshot.checklists.find((list: any) => list.id === checklistId).items
        .some((item: any) => item.id === editorTodo)).toBe(false);
      expect(snapshot.checklists.find((list: any) => list.kind === 'TRASH').items
        .some((item: any) => item.id === editorTodo)).toBe(true);

      const cleanCompleted = $('[data-action="DELETE_DONE"]');
      await cleanCompleted.click();
      await expect($('.destructive-confirmation-target')).toHaveText('1 task');
      expect((await bootstrap()).checklists.find((list: any) => list.id === checklistId).items
        .some((item: any) => item.id === completedTodo)).toBe(true);
      await browser.keys(['Escape']);
      await expect(cleanCompleted).toBeFocused();
      await cleanCompleted.click();
      await $('.destructive-confirmation-confirm').click();
      await browser.waitUntil(async () => !(await $('.destructive-confirmation-modal').isExisting()));
      snapshot = await bootstrap();
      const remaining = snapshot.checklists.find((list: any) => list.id === checklistId).items;
      expect(remaining.some((item: any) => item.id === completedTodo)).toBe(false);
      expect(remaining.some((item: any) => item.id === keepTodo)).toBe(true);

      const quickTodo = await createTodo('QUICK DELETE DIRECT');
      await browser.refresh();
      const batchDelete = $('[data-action="BATCH_DELETE"]');
      await batchDelete.click();
      await browser.waitUntil(async () => (await batchDelete.getAttribute('class')).includes('active'));
      const quickRow = $(`//*[contains(@class,'task-row')][.//strong[normalize-space(.)='QUICK DELETE DIRECT']]`);
      await quickRow.$('.delete-slot').click();
      await browser.waitUntil(async () => {
        const latest = await bootstrap();
        return latest.checklists.find((list: any) => list.kind === 'TRASH').items
          .some((item: any) => item.id === quickTodo);
      });
      expect(await $('.destructive-confirmation-modal').isExisting()).toBe(false);
    } finally {
      if (await $('.destructive-confirmation-modal').isExisting()) await browser.keys(['Escape']);
      if (await $('.todo-editor-modal').isExisting()) await browser.keys(['Escape']);
      let snapshot = await bootstrap();
      if (snapshot.selectedChecklistId !== originalSelection
        && snapshot.checklists.some((list: any) => list.id === originalSelection)) {
        for (let attempt = 0; attempt < 2; attempt += 1) {
          snapshot = await bootstrap();
          try {
            await invoke('select_checklist', {
              expectedRevision: snapshot.revision,
              checklistId: originalSelection
            });
            break;
          } catch (error) {
            if (attempt === 1) throw error;
          }
        }
      }
      for (const todoId of todoIds) {
        for (let attempt = 0; attempt < 2; attempt += 1) {
          snapshot = await bootstrap();
          if (!snapshot.checklists.some((list: any) => list.kind === 'TRASH'
            && list.items.some((item: any) => item.id === todoId))) break;
          try {
            await invoke('purge_todo', { expectedRevision: snapshot.revision, todoId });
            break;
          } catch (error) {
            if (attempt === 1) throw error;
          }
        }
      }
      snapshot = await bootstrap();
      if (checklistId && snapshot.checklists.filter((list: any) => list.kind === 'NORMAL').length > 1
        && snapshot.checklists.some((list: any) => list.id === checklistId)) {
        for (let attempt = 0; attempt < 2; attempt += 1) {
          snapshot = await bootstrap();
          try {
            await invoke('delete_checklist', { expectedRevision: snapshot.revision, checklistId });
            break;
          } catch (error) {
            if (attempt === 1) throw error;
          }
        }
      }
      snapshot = await bootstrap();
      if (snapshot.hideCompleted !== originalHideCompleted) {
        for (let attempt = 0; attempt < 3; attempt += 1) {
          snapshot = await bootstrap();
          try {
            await invoke('set_hide_completed', {
              expectedRevision: snapshot.revision,
              hideCompleted: originalHideCompleted
            });
            break;
          } catch (error) {
            if (attempt === 2) throw error;
          }
        }
      }
      snapshot = await bootstrap();
      if (snapshot.quickDelete !== originalQuickDelete) {
        for (let attempt = 0; attempt < 3; attempt += 1) {
          snapshot = await bootstrap();
          try {
            await invoke('set_quick_delete', {
              expectedRevision: snapshot.revision,
              quickDelete: originalQuickDelete
            });
            break;
          } catch (error) {
            if (attempt === 2) throw error;
          }
        }
      }
      for (let attempt = 0; attempt < 3; attempt += 1) {
        snapshot = await bootstrap();
        try {
          await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
          break;
        } catch (error) {
          if (attempt === 2) throw error;
        }
      }
      await browser.refresh();
    }
  });

  it('persists fields, completion and list view controls', async () => {
    let snapshot = await bootstrap();
    const originalSettings = snapshot.settings;
    const checklistId = snapshot.checklists.find((list: any) => list.kind === 'NORMAL').id;
    const dueAtMillis = Date.now() + 86_400_000;
    const created = await invokeWithLatestRevision('create_todo', {
      checklistId,
      draft: { title: 'E2E TODO', priority: 'XHIGH', dueAtMillis, reminderRepeat: 'DAILY' }
    });
    const todoId = created.changedIds.find((id: string) => id !== checklistId);
    await invokeWithLatestRevision('toggle_todo', { checklistId, todoId });
    await invokeWithLatestRevision('set_sort_mode', { sortMode: 'TIME' });
    await invokeWithLatestRevision('set_hide_completed', { hideCompleted: true });
    await invokeWithLatestRevision('set_deadline_countdown', { visible: true });
    snapshot = await bootstrap();
    const todo = snapshot.checklists.find((list: any) => list.id === checklistId).items.find((item: any) => item.id === todoId);
    expect(todo).toMatchObject({ title: 'E2E TODO', priority: 'XHIGH', reminderRepeat: 'DAILY', completed: true });
    expect(todo.dueAtMillis).toBe(dueAtMillis);
    expect(snapshot).toMatchObject({ sortMode: 'TIME', hideCompleted: true, showDeadlineCountdown: true });

    await invokeWithLatestRevision('set_hide_completed', { hideCompleted: false });
    await invokeWithLatestRevision('toggle_todo', { checklistId, todoId });
    await invokeWithLatestRevision('select_checklist', { checklistId });
    snapshot = await bootstrap();
    await invokeWithLatestRevision('update_settings', {
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

    await invokeWithLatestRevision('update_settings', { settings: originalSettings });
  });
});
