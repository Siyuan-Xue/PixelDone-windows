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
    const restored = snapshot.checklists.find((list: any) => list.name === 'RESTORE SOURCE');
    expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(false);
    expect(restored?.id).not.toBe(checklistId);
    expect(restored?.name).toBe('RESTORE SOURCE');
    expect(restored?.items.some((item: any) => item.id === todoId)).toBe(true);
  });

  it('searches and filters deleted tasks while keeping accessible borderless actions', async () => {
    const checklistIds: string[] = [];
    const todoIds: string[] = [];
    let originalLanguage = 'SYSTEM';
    let trashCountBeforeDeleteAll = 0;

    async function createChecklist(name: string): Promise<string> {
      const before = await bootstrap();
      await invoke('create_checklist', { expectedRevision: before.revision, name });
      const after = await bootstrap();
      checklistIds.push(after.selectedChecklistId);
      return after.selectedChecklistId;
    }

    async function createAndTrash(
      checklistId: string,
      title: string,
      priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'XHIGH'
    ): Promise<string> {
      const before = await bootstrap();
      const created = await invoke('create_todo', {
        expectedRevision: before.revision,
        checklistId,
        draft: { title, priority, dueAtMillis: 0, reminderRepeat: 'NONE' }
      });
      const todoId = created.changedIds.find((id: string) => id !== checklistId);
      if (!todoId) throw new Error(`Created todo id missing for ${title}`);
      await invoke('move_todo_to_trash', {
        expectedRevision: created.revision,
        checklistId,
        todoId
      });
      todoIds.push(todoId);
      return todoId;
    }

    async function visibleRowCount(): Promise<number> {
      return (await $$('[data-testid="trash-row"]')).length;
    }

    async function visibleCreatedRowCount(): Promise<number> {
      let count = 0;
      for (const todoId of todoIds) {
        if (await $(`[data-todo-id="${todoId}"]`).isExisting()) count += 1;
      }
      return count;
    }

    async function selectFilter(testId: string, value: string): Promise<void> {
      await browser.execute((selector, nextValue) => {
        const select = document.querySelector<HTMLSelectElement>(selector)!;
        select.value = nextValue;
        select.dispatchEvent(new Event('input', { bubbles: true }));
        select.dispatchEvent(new Event('change', { bubbles: true }));
      }, `[data-testid="${testId}"]`, value);
    }

    try {
      let snapshot = await bootstrap();
      originalLanguage = snapshot.settings.languageMode;
      if (originalLanguage !== 'ENGLISH') {
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
        });
      }

      const alphaChecklist = await createChecklist('TRASH FILTER ALPHA');
      const reportTodo = await createAndTrash(alphaChecklist, 'Quarterly Report', 'XHIGH');
      await createAndTrash(alphaChecklist, 'Buy milk', 'LOW');
      const betaChecklist = await createChecklist('TRASH FILTER BETA');
      await createAndTrash(betaChecklist, 'quarterly notes', 'MEDIUM');
      const permanentTodo = await createAndTrash(betaChecklist, 'PERMANENT DELETE ME', 'HIGH');

      snapshot = await bootstrap();
      const trash = snapshot.checklists.find((list: any) => list.kind === 'TRASH');
      await invoke('select_checklist', {
        expectedRevision: snapshot.revision,
        checklistId: trash.id
      });
      await browser.refresh();
      await $('[data-testid="trash-search"]').waitForDisplayed();
      expect(await visibleCreatedRowCount()).toBe(4);
      const trashCountBeforePermanentDelete = (await bootstrap()).checklists
        .find((list: any) => list.kind === 'TRASH').items.length;
      expect(await visibleRowCount()).toBe(trashCountBeforePermanentDelete);

      const permanentDelete = $(`[data-todo-id="${permanentTodo}"] .trash-delete`);
      await permanentDelete.click();
      await expect($('.destructive-confirmation-modal')).toBeDisplayed();
      await expect($('.destructive-confirmation-target')).toHaveText('PERMANENT DELETE ME');
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforePermanentDelete);
      await $('.destructive-confirmation-actions .quiet-button').click();
      await expect(permanentDelete).toBeFocused();
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforePermanentDelete);

      await permanentDelete.click();
      await browser.execute(() => {
        const confirm = document.querySelector<HTMLButtonElement>('.destructive-confirmation-confirm')!;
        confirm.click();
        confirm.click();
      });
      await browser.waitUntil(async () => !(await $('.destructive-confirmation-modal').isExisting()));
      await browser.waitUntil(async () => {
        const latest = await bootstrap();
        return !latest.checklists.find((list: any) => list.kind === 'TRASH').items
          .some((item: any) => item.id === permanentTodo);
      });
      expect(await visibleCreatedRowCount()).toBe(3);
      trashCountBeforeDeleteAll = trashCountBeforePermanentDelete - 1;
      expect(await visibleRowCount()).toBe(trashCountBeforeDeleteAll);

      await expect($(`[data-todo-id="${reportTodo}"] .trash-source`)).toHaveText('TRASH FILTER ALPHA');
      const rowText = await $(`[data-todo-id="${reportTodo}"]`).getText();
      expect(rowText).not.toContain('%1$s');
      expect(rowText).not.toContain('From');

      const actionMetrics = await browser.execute((todoId) => {
        const row = document.querySelector<HTMLElement>(`[data-todo-id="${todoId}"]`)!;
        return Array.from(row.querySelectorAll<HTMLButtonElement>('.trash-action')).map((button) => ({
          text: button.textContent?.trim() ?? '',
          label: button.getAttribute('aria-label') ?? '',
          border: getComputedStyle(button).borderWidth
        }));
      }, reportTodo);
      expect(actionMetrics).toHaveLength(2);
      expect(actionMetrics.map((action) => action.text)).toEqual(['', '']);
      expect(actionMetrics.every((action) => action.label.includes('Quarterly Report'))).toBe(true);
      expect(actionMetrics.every((action) => action.border === '0px')).toBe(true);

      const search = $('[data-testid="trash-search"]');
      await search.setValue('quarterly');
      await browser.waitUntil(async () => await visibleCreatedRowCount() === 2);
      await $('.list-nav .nav-row .nav-main').click();
      await browser.waitUntil(async () => !(await $('[data-testid="trash-search"]').isExisting()));
      const specialRows = await $$('.special-nav .special-row');
      await specialRows[0].click();
      await $('[data-testid="trash-search"]').waitForDisplayed();
      await expect($('[data-testid="trash-search"]')).toHaveValue('');
      expect(await visibleCreatedRowCount()).toBe(3);
      expect(await visibleRowCount()).toBe(trashCountBeforeDeleteAll);

      snapshot = await bootstrap();
      await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: { ...snapshot.settings, languageMode: 'ARABIC' }
      });
      await browser.setWindowSize(1000, 680);
      await browser.refresh();
      await expect($('.app-shell.rtl')).toBeDisplayed();
      const narrowRtlLayout = await browser.execute(() => {
        const list = document.querySelector<HTMLElement>('.trash-list')!;
        const controls = Array.from(document.querySelectorAll<HTMLElement>(
          '.trash-filters input, .trash-filters select, .trash-delete-all'
        ));
        const listRect = list.getBoundingClientRect();
        return {
          direction: getComputedStyle(document.querySelector<HTMLElement>('.trash-toolbar')!).direction,
          noHorizontalOverflow: list.scrollWidth <= list.clientWidth + 1,
          controlsOnOneRow: Math.max(...controls.map((control) => {
            const rect = control.getBoundingClientRect();
            return rect.top + rect.height / 2;
          })) - Math.min(...controls.map((control) => {
            const rect = control.getBoundingClientRect();
            return rect.top + rect.height / 2;
          })) <= 1,
          controlsInsideList: controls.every((control) => {
            const rect = control.getBoundingClientRect();
            return rect.left >= listRect.left - 1 && rect.right <= listRect.right + 1;
          })
        };
      });
      expect(narrowRtlLayout).toEqual({
        direction: 'rtl',
        noHorizontalOverflow: true,
        controlsOnOneRow: true,
        controlsInsideList: true
      });

      await $('[data-testid="trash-delete-all"]').click();
      await expect($('.destructive-confirmation-modal')).toBeDisplayed();
      await expect($('.destructive-confirmation-actions .quiet-button')).toBeFocused();
      const narrowRtlDialog = await browser.execute(() => {
        const dialog = document.querySelector<HTMLElement>('.destructive-confirmation-modal')!;
        const rect = dialog.getBoundingClientRect();
        const labelledBy = dialog.getAttribute('aria-labelledby');
        const describedBy = dialog.getAttribute('aria-describedby');
        return {
          role: dialog.getAttribute('role'),
          direction: getComputedStyle(dialog).direction,
          labelled: Boolean(labelledBy && document.getElementById(labelledBy)),
          described: Boolean(describedBy && document.getElementById(describedBy)),
          insideViewport: rect.left >= 0 && rect.right <= innerWidth && rect.top >= 0 && rect.bottom <= innerHeight,
          noHorizontalOverflow: dialog.scrollWidth <= dialog.clientWidth + 1
        };
      });
      expect(narrowRtlDialog).toEqual({
        role: 'alertdialog',
        direction: 'rtl',
        labelled: true,
        described: true,
        insideViewport: true,
        noHorizontalOverflow: true
      });
      await browser.keys(['Escape']);
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforeDeleteAll);

      snapshot = await bootstrap();
      await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
      });
      await browser.setWindowSize(1180, 780);
      await browser.refresh();
      await $('[data-testid="trash-search"]').setValue('  QUARTERLY  ');
      await browser.waitUntil(async () => await visibleCreatedRowCount() === 2);
      await selectFilter('trash-priority-filter', 'MEDIUM');
      await browser.waitUntil(async () => await visibleCreatedRowCount() === 1);
      await selectFilter('trash-checklist-filter', 'TRASH FILTER BETA');
      expect(await visibleCreatedRowCount()).toBe(1);
      await selectFilter('trash-checklist-filter', 'TRASH FILTER ALPHA');
      await $('[data-testid="trash-no-results"]').waitForDisplayed();
      expect(await visibleRowCount()).toBe(0);

      await $('[data-testid="trash-delete-all"]').click();
      await expect($('.destructive-confirmation-modal')).toBeDisplayed();
      const trashCountLabel = trashCountBeforeDeleteAll === 1
        ? '1 task'
        : `${trashCountBeforeDeleteAll} tasks`;
      await expect($('.destructive-confirmation-target')).toHaveText(trashCountLabel);
      expect(await $('#destructive-confirmation-detail').getText()).toContain(String(trashCountBeforeDeleteAll));
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforeDeleteAll);
      await browser.keys(['Escape']);
      await expect($('[data-testid="trash-delete-all"]')).toBeFocused();
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforeDeleteAll);

      await $('[data-testid="trash-delete-all"]').click();
      await $('.destructive-confirmation-backdrop .modal-dismiss-layer').click();
      expect((await bootstrap()).checklists.find((list: any) => list.kind === 'TRASH').items)
        .toHaveLength(trashCountBeforeDeleteAll);

      await $('[data-testid="trash-delete-all"]').click();
      await $('.destructive-confirmation-confirm').click();
      await browser.waitUntil(async () => {
        const latest = await bootstrap();
        return latest.checklists.find((list: any) => list.kind === 'TRASH').items.length === 0;
      });
      await expect($('.trash-list .empty-state')).toBeDisplayed();
    } finally {
      await browser.setWindowSize(1180, 780);
      let snapshot = await bootstrap();
      if (snapshot.settings.languageMode !== originalLanguage) {
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: { ...snapshot.settings, languageMode: originalLanguage }
        });
      }
      for (const checklistId of checklistIds) {
        snapshot = await bootstrap();
        const normalLists = snapshot.checklists.filter((list: any) => list.kind === 'NORMAL');
        if (normalLists.length > 1 && normalLists.some((list: any) => list.id === checklistId)) {
          await invoke('delete_checklist', {
            expectedRevision: snapshot.revision,
            checklistId
          });
        }
      }
      for (const todoId of todoIds) {
        snapshot = await bootstrap();
        if (snapshot.checklists.some((list: any) =>
          list.kind === 'TRASH' && list.items.some((item: any) => item.id === todoId))) {
          await invoke('purge_todo', { expectedRevision: snapshot.revision, todoId });
        }
      }
      await browser.refresh();
      await $('.sidebar-header').waitForDisplayed();
    }
  });
});
