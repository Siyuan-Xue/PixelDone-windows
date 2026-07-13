import { bootstrap, invoke } from '../helpers';

const DOCK_ACTIONS = ['SORT', 'DEADLINE', 'HIDE_DONE', 'DELETE_DONE'] as const;
const PLACEMENTS = ['LEFT_EDGE', 'CENTER', 'RIGHT_EDGE'] as const;

async function selectChecklist(checklistId: string): Promise<void> {
  const snapshot = await bootstrap();
  if (snapshot.selectedChecklistId !== checklistId) {
    await invoke('select_checklist', { expectedRevision: snapshot.revision, checklistId });
  }
  await browser.refresh();
}

async function restoreChecklist(checklistId: string): Promise<void> {
  const snapshot = await bootstrap();
  if (
    snapshot.selectedChecklistId !== checklistId &&
    snapshot.checklists.some((list: any) => list.id === checklistId)
  ) {
    await invoke('select_checklist', { expectedRevision: snapshot.revision, checklistId });
    await browser.refresh();
  }
}

function expectedDockSequence(placement: (typeof PLACEMENTS)[number]): string[] {
  const actions = [...DOCK_ACTIONS];
  if (placement === 'LEFT_EDGE') return ['PLUS', ...actions];
  if (placement === 'RIGHT_EDGE') return [...actions, 'PLUS'];
  const leftCount = Math.ceil(actions.length / 2);
  return [...actions.slice(0, leftCount), 'PLUS', ...actions.slice(leftCount)];
}

async function setCssViewport(width: number, height: number): Promise<{ innerWidth: number; innerHeight: number }> {
  let viewport = await browser.execute(() => ({ innerWidth, innerHeight, scale: devicePixelRatio }));
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const outer = await browser.getWindowSize();
    await browser.setWindowSize(
      Math.max(640, outer.width + Math.round((width - viewport.innerWidth) * viewport.scale)),
      Math.max(480, outer.height + Math.round((height - viewport.innerHeight) * viewport.scale))
    );
    viewport = await browser.execute(() => ({ innerWidth, innerHeight, scale: devicePixelRatio }));
    if (Math.abs(viewport.innerWidth - width) <= 2 && Math.abs(viewport.innerHeight - height) <= 2) break;
  }
  return viewport;
}

describe('PixelDone 3.1.3 desktop layout', () => {
  it('uses one 64 px header rhythm and keeps product identity in the window title', async () => {
    const metrics = await browser.execute(() => {
      const sidebar = document.querySelector<HTMLElement>('.sidebar-header');
      const workspace = document.querySelector<HTMLElement>('.workspace-status');
      const addList = document.querySelector<HTMLElement>('.new-list-button');
      return {
        title: document.title,
        sidebarHeight: sidebar?.getBoundingClientRect().height ?? 0,
        workspaceHeight: workspace?.getBoundingClientRect().height ?? 0,
        eyebrowCount: document.querySelectorAll('.workspace-status .eyebrow, .sidebar .eyebrow').length,
        makerCount: document.querySelectorAll('.maker-line').length,
        syncChipCount: document.querySelectorAll('.workspace-status .sync-chip').length,
        addListBorder: addList ? getComputedStyle(addList).borderStyle : '',
        addListColor: addList ? getComputedStyle(addList).color : ''
      };
    });

    expect(metrics.title).toBe('PixelDone — CODEX & XUE');
    expect(metrics.sidebarHeight).toBe(64);
    expect(metrics.workspaceHeight).toBe(64);
    expect(metrics.eyebrowCount).toBe(0);
    expect(metrics.makerCount).toBe(0);
    expect(metrics.syncChipCount).toBe(0);
    expect(metrics.addListBorder).toBe('none');
    expect(metrics.addListColor).toBe('rgb(217, 119, 87)');
  });

  it('persists and clamps the keyboard-accessible sidebar width in LTR and RTL', async () => {
    const initial = await bootstrap();
    const originalSettings = initial.settings;

    try {
      await invoke('update_settings', {
        expectedRevision: initial.revision,
        settings: { ...initial.settings, languageMode: 'ENGLISH', sidebarWidthPx: 320 }
      });
      await browser.refresh();
      await $('.sidebar-resizer').click();
      await browser.keys(['ArrowRight']);
      await browser.waitUntil(async () => (await bootstrap()).settings.sidebarWidthPx === 328);
      await browser.refresh();
      expect(Math.round(await $('.sidebar').getSize('width'))).toBe(328);

      let snapshot = await bootstrap();
      await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: { ...snapshot.settings, sidebarWidthPx: 999 }
      });
      await browser.refresh();
      expect((await bootstrap()).settings.sidebarWidthPx).toBe(560);
      expect(Math.round(await $('.sidebar').getSize('width'))).toBe(560);

      snapshot = await bootstrap();
      await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: { ...snapshot.settings, languageMode: 'ARABIC', sidebarWidthPx: 320 }
      });
      await browser.refresh();
      await $('.sidebar-resizer').click();
      await browser.keys(['ArrowRight']);
      await browser.waitUntil(async () => (await bootstrap()).settings.sidebarWidthPx === 312);
    } finally {
      const snapshot = await bootstrap();
      await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
      await browser.refresh();
    }
  });

  it('keeps checklist creation in the sidebar and task creation in the Dock', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    const normalList = initial.checklists.find((list: any) => list.kind === 'NORMAL');
    const normalListCount = initial.checklists.filter((list: any) => list.kind === 'NORMAL').length;

    try {
      await selectChecklist(normalList.id);

      await $('.new-list-button').click();
      await expect($('.inline-create')).toBeDisplayed();
      expect((await bootstrap()).checklists.filter((list: any) => list.kind === 'NORMAL')).toHaveLength(normalListCount);

      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.inline-create').isExisting()));

      await $('.dock-add').click();
      await expect($('.todo-editor-modal')).toBeDisplayed();
      expect(await $('.inline-create').isExisting()).toBe(false);
      expect((await bootstrap()).checklists.filter((list: any) => list.kind === 'NORMAL')).toHaveLength(normalListCount);

      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.todo-editor-modal').isExisting()));
    } finally {
      await restoreChecklist(originalChecklistId);
    }
  });

  it('renders separate square Dock buttons and inserts plus at every configured position', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    const originalSettings = initial.settings;
    const normalList = initial.checklists.find((list: any) => list.kind === 'NORMAL');

    try {
      await selectChecklist(normalList.id);

      for (const placement of PLACEMENTS) {
        const snapshot = await bootstrap();
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: {
            ...snapshot.settings,
            languageMode: 'ENGLISH',
            dock: { plusPlacement: placement, actions: [...DOCK_ACTIONS] }
          }
        });
        await browser.refresh();
        await expect($('.dock')).toBeDisplayed();

        const metrics = await browser.execute(() => {
          const dock = document.querySelector<HTMLElement>('.dock');
          if (!dock) return null;
          const buttons = Array.from(dock.querySelectorAll<HTMLElement>(':scope > button'));
          const style = getComputedStyle(dock);
          const gap = Number.parseFloat(style.columnGap || style.gap) || 0;
          return {
            directChildCount: dock.children.length,
            display: style.display,
            gap,
            sequence: buttons.map((button) =>
              button.classList.contains('dock-add') ? 'PLUS' : button.dataset.action ?? ''
            ),
            sizes: buttons.map((button) => {
              const rect = button.getBoundingClientRect();
              return {
                width: rect.width,
                height: rect.height,
                bottom: rect.bottom,
                plus: button.classList.contains('dock-add')
              };
            })
          };
        });

        expect(metrics).not.toBeNull();
        if (!metrics) throw new Error('Dock metrics unavailable');
        expect(['flex', 'inline-flex', 'grid', 'inline-grid']).toContain(metrics.display);
        expect(metrics.directChildCount).toBe(DOCK_ACTIONS.length + 1);
        expect(metrics.gap).toBe(18);
        expect(metrics.sequence).toEqual(expectedDockSequence(placement));
        expect(metrics.sizes.every(({ width, height, plus }) => width === (plus ? 56 : 44) && height === (plus ? 56 : 44))).toBe(true);
        expect(new Set(metrics.sizes.map(({ bottom }) => Math.round(bottom))).size).toBe(1);
      }
    } finally {
      let snapshot = await bootstrap();
      await invoke('update_settings', {
        expectedRevision: snapshot.revision,
        settings: originalSettings
      });
      snapshot = await bootstrap();
      if (
        snapshot.selectedChecklistId !== originalChecklistId &&
        snapshot.checklists.some((list: any) => list.id === originalChecklistId)
      ) {
        await invoke('select_checklist', {
          expectedRevision: snapshot.revision,
          checklistId: originalChecklistId
        });
      }
      await browser.refresh();
    }
  });

  it('keeps the two-pane shell overflow-free and preserves physical Dock edges in RTL', async () => {
    const initial = await bootstrap();
    const originalSettings = initial.settings;

    try {
      for (const [width, height] of [[1000, 680], [1180, 780], [1440, 900]]) {
        const viewport = await setCssViewport(width, height);
        const metrics = await browser.execute(() => ({
          innerWidth,
          scrollWidth: document.documentElement.scrollWidth,
          regions: Array.from(document.querySelector('.app-shell')?.children ?? []).map((element) => element.className)
        }));
        expect(Math.abs(viewport.innerWidth - width)).toBeLessThanOrEqual(2);
        expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.innerWidth);
        expect(metrics.regions).toHaveLength(2);
      }

      for (const placement of ['LEFT_EDGE', 'RIGHT_EDGE'] as const) {
        const snapshot = await bootstrap();
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: {
            ...snapshot.settings,
            languageMode: 'ARABIC',
            dock: { plusPlacement: placement, actions: [...DOCK_ACTIONS] }
          }
        });
        await browser.refresh();
        await expect($('.app-shell')).toBeDisplayed();
        await browser.waitUntil(async () => browser.execute(() => document.querySelector('.app-shell')?.classList.contains('rtl') ?? false));
        const positions = await browser.execute(() => {
          const buttons = Array.from(document.querySelectorAll<HTMLElement>('.dock > button'));
          const plus = document.querySelector<HTMLElement>('.dock-add');
          return {
            rtl: document.querySelector('.app-shell')?.classList.contains('rtl') ?? false,
            plusX: plus?.getBoundingClientRect().x ?? -1,
            xs: buttons.map((button) => button.getBoundingClientRect().x)
          };
        });
        expect(positions.rtl).toBe(true);
        expect(positions.plusX).toBe(placement === 'LEFT_EDGE' ? Math.min(...positions.xs) : Math.max(...positions.xs));
      }
    } finally {
      const snapshot = await bootstrap();
      await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
      await setCssViewport(1180, 780);
      await browser.refresh();
    }
  });

  it('does not let global navigation shortcuts escape an open task editor', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    const normalList = initial.checklists.find((list: any) => list.kind === 'NORMAL');

    try {
      await selectChecklist(normalList.id);
      await $('.dock-add').click();
      await expect($('.todo-editor-modal')).toBeDisplayed();
      await $('#todo-title').setValue('UNSAVED SHORTCUT GUARD');
      const selectedBefore = (await bootstrap()).selectedChecklistId;

      await browser.keys(['Control', 'n']);
      expect(await $('#todo-title').getValue()).toBe('UNSAVED SHORTCUT GUARD');
      await browser.keys(['Control', 'Shift', 'n']);
      expect(await $('.inline-create').isExisting()).toBe(false);
      await browser.keys(['Alt', 'ArrowLeft']);
      expect((await bootstrap()).selectedChecklistId).toBe(selectedBefore);

      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.todo-editor-modal').isExisting()));
    } finally {
      await restoreChecklist(originalChecklistId);
    }
  });

  it('keeps empty lists and very long task titles within the minimum desktop viewport', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    let checklistId: string | null = null;
    let todoId: string | null = null;

    try {
      const createdList = await invoke('create_checklist', {
        expectedRevision: initial.revision,
        name: 'E2E CONTENT EDGE'
      });
      checklistId = createdList.changedIds[0];
      if (!checklistId) throw new Error('Created checklist id missing');
      await setCssViewport(1000, 680);
      await browser.refresh();
      await expect($('.empty-state')).toBeDisplayed();
      expect(await browser.execute(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);

      const longTitle = `E2E ${'PIXELDONE DESKTOP LONG TITLE '.repeat(10)}`.trim();
      const createdTodo = await invoke('create_todo', {
        expectedRevision: (await bootstrap()).revision,
        checklistId,
        draft: { title: longTitle, priority: 'MEDIUM', dueAtMillis: 0, reminderRepeat: 'NONE' }
      });
      todoId = createdTodo.changedIds.find((id: string) => id !== checklistId) ?? null;
      await browser.refresh();
      const title = await $('.task-row .task-copy strong');
      await expect(title).toBeDisplayed();
      expect(await title.getText()).toBe(longTitle);
      expect(await browser.execute(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    } finally {
      let snapshot = await bootstrap();
      if (checklistId && snapshot.selectedChecklistId === checklistId) {
        await invoke('select_checklist', {
          expectedRevision: snapshot.revision,
          checklistId: originalChecklistId
        });
        snapshot = await bootstrap();
      }
      if (checklistId && snapshot.checklists.some((list: any) => list.id === checklistId)) {
        await invoke('delete_checklist', { expectedRevision: snapshot.revision, checklistId });
        snapshot = await bootstrap();
      }
      if (todoId && snapshot.checklists.some((list: any) => list.kind === 'TRASH' && list.items.some((item: any) => item.id === todoId))) {
        await invoke('purge_todo', { expectedRevision: snapshot.revision, todoId });
      }
      await setCssViewport(1180, 780);
      await restoreChecklist(originalChecklistId);
    }
  });
});
