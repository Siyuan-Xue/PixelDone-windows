import { bootstrap, invoke, invokeRaw } from '../helpers';

describe('Checklist parity', () => {
  it('requires an in-app confirmation before deleting a checklist', async () => {
    let checklistId: string | null = null;
    try {
      let snapshot = await bootstrap();
      const created = await invoke('create_checklist', {
        expectedRevision: snapshot.revision,
        name: 'E2E DELETE CONFIRM'
      });
      checklistId = created.changedIds[0];
      await browser.refresh();

      const findRow = async () => {
        const rows = await $$('.list-nav .nav-row');
        for (const row of rows) {
          if ((await row.getText()).includes('E2E DELETE CONFIRM')) return row;
        }
        return undefined;
      };
      const firstRow = await findRow();
      if (!firstRow) throw new Error('Delete confirmation checklist row missing');
      await firstRow.$('.row-delete').click();

      await expect($('.destructive-confirmation-modal')).toBeDisplayed();
      await expect($('.destructive-confirmation-target')).toHaveText('E2E DELETE CONFIRM');
      await expect($('.destructive-confirmation-actions .quiet-button')).toBeFocused();
      await browser.keys(['Escape']);
      await expect($('.destructive-confirmation-modal')).not.toBeExisting();
      snapshot = await bootstrap();
      expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(true);

      const secondRow = await findRow();
      if (!secondRow) throw new Error('Checklist disappeared before confirmation');
      await secondRow.$('.row-delete').click();
      await $('.destructive-confirmation-backdrop .modal-dismiss-layer').click();
      await expect(secondRow.$('.row-delete')).toBeFocused();
      snapshot = await bootstrap();
      expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(true);

      const thirdRow = await findRow();
      if (!thirdRow) throw new Error('Checklist disappeared before confirmation');
      await thirdRow.$('.row-delete').click();
      await $('.destructive-confirmation-confirm').click();
      await browser.waitUntil(async () => !(await $('.destructive-confirmation-modal').isExisting()));
      snapshot = await bootstrap();
      expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(false);
      checklistId = null;
    } finally {
      if (await $('.destructive-confirmation-modal').isExisting()) await browser.keys(['Escape']);
      const snapshot = await bootstrap();
      if (checklistId && snapshot.checklists.some((list: any) => list.id === checklistId)) {
        await invoke('delete_checklist', { expectedRevision: snapshot.revision, checklistId });
      }
      await browser.refresh();
    }
  });

  it('creates, renames and deletes a normal checklist with revision protection', async () => {
    let snapshot = await bootstrap();
    const created = await invoke('create_checklist', { expectedRevision: snapshot.revision, name: 'E2E LIST' });
    const checklistId = created.changedIds[0];
    const renamed = await invoke('rename_checklist', {
      expectedRevision: created.revision,
      checklistId,
      name: 'RENAMED E2E'
    });
    snapshot = await bootstrap();
    expect(snapshot.checklists.find((list: any) => list.id === checklistId)?.name).toBe('RENAMED E2E');
    await expect(invokeRaw('rename_checklist', {
      expectedRevision: renamed.revision - 1,
      checklistId,
      name: 'STALE'
    })).rejects.toThrow('当前界面状态已过期');
    await invoke('delete_checklist', { expectedRevision: renamed.revision, checklistId });
    snapshot = await bootstrap();
    expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(false);
    expect(snapshot.checklists.filter((list: any) => list.kind === 'NORMAL')).toHaveLength(1);
  });

  it('silently reloads and retries one stale UI mutation without a red alert', async () => {
    const initial = await bootstrap();
    let checklistId: string | null = null;
    try {
      const created = await invoke('create_checklist', {
        expectedRevision: initial.revision,
        name: 'E2E STALE RETRY'
      });
      checklistId = created.changedIds[0];
      await invoke('select_checklist', {
        expectedRevision: created.revision,
        checklistId: initial.selectedChecklistId
      });
      await browser.refresh();
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_SELECT_CHECKLIST__?: (input: { expectedRevision: number; checklistId: string }) => Promise<unknown>;
          __pixeldoneStaleCalls?: number;
        };
        scope.__pixeldoneStaleCalls = 0;
        scope.__PIXELDONE_E2E_SELECT_CHECKLIST__ = async ({ expectedRevision, checklistId }) => {
          scope.__pixeldoneStaleCalls = (scope.__pixeldoneStaleCalls ?? 0) + 1;
          if (scope.__pixeldoneStaleCalls === 1) {
            throw { code: 'STALE_REVISION', message: 'stale' };
          }
          return {
            revision: expectedRevision + 1,
            changedIds: [checklistId],
            snapshotDelta: {
              upsertedChecklists: [], removedChecklistIds: [], selectedChecklistId: checklistId,
              sortMode: null, hideCompleted: null, quickDelete: null, showDeadlineCountdown: null,
              checklistHistory: null, settings: null, auth: null, sync: null, reminder: null, update: null
            }
          };
        };
      });

      const target = await $$('.list-nav .nav-row .nav-main').find(async (button) => (await button.getText()).includes('E2E STALE RETRY'));
      if (!target) throw new Error('Stale retry checklist button missing');
      await target.click();
      await browser.waitUntil(async () => browser.execute(() => (globalThis as typeof globalThis & { __pixeldoneStaleCalls?: number }).__pixeldoneStaleCalls === 2));
      await expect($('.workspace-status h2')).toHaveText('E2E STALE RETRY');
      expect(await $('.operation-error').isExisting()).toBe(false);
      expect(await browser.execute(() => (globalThis as typeof globalThis & { __pixeldoneStaleCalls?: number }).__pixeldoneStaleCalls)).toBe(2);
    } finally {
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_SELECT_CHECKLIST__?: unknown;
          __pixeldoneStaleCalls?: unknown;
        };
        delete scope.__PIXELDONE_E2E_SELECT_CHECKLIST__;
        delete scope.__pixeldoneStaleCalls;
      });
      let snapshot = await bootstrap();
      if (checklistId && snapshot.checklists.some((list: any) => list.id === checklistId)) {
        if (snapshot.selectedChecklistId === checklistId) {
          await invoke('select_checklist', {
            expectedRevision: snapshot.revision,
            checklistId: initial.selectedChecklistId
          });
          snapshot = await bootstrap();
        }
        await invoke('delete_checklist', { expectedRevision: snapshot.revision, checklistId });
      }
      await browser.refresh();
    }
  });
});
