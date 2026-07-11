<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import Icon from '$lib/components/common/Icon.svelte';
  import { localeFor, message, type MessageKey } from '$lib/generated/i18n';
  import type {
    AppLanguage,
    AppError,
    AppSettings,
    AppSnapshot,
    Checklist,
    ConflictResolutionChoice,
    DockAction,
    DockPlusPlacement,
    MutationResult,
    SortMode,
    SyncConflictView,
    TodoDraft,
    TodoItem,
    TodoPriority
  } from '$lib/generated/ipc';
  import {
    api,
    applyMutation,
    dateTimeLocalValue,
    emptyDraft,
    millisFromDateTimeLocal,
    repeatLabel
  } from '$lib/ipc/client';

  let snapshot = $state<AppSnapshot>(null!);
  let loading = $state(true);
  let errorMessage = $state('');
  let selectedTodoId = $state<string | null>(null);
  let draft = $state<TodoDraft>(emptyDraft());
  let isNewTodo = $state(false);
  let creatingList = $state(false);
  let newListName = $state('');
  let editingListId = $state<string | null>(null);
  let listNameDraft = $state('');
  let inspectorOpen = $state(false);
  let completionHold = $state<Record<string, boolean>>({});
  let highlightedTodoId = $state<string | null>(null);
  let authEmail = $state('');
  let authPassword = $state('');
  let authMode = $state<'sign-in' | 'sign-up'>('sign-in');
  let authBusy = $state(false);
  let conflicts = $state<SyncConflictView[]>([]);
  let conflictOpen = $state(false);
  let previewData = $state<string | null>(null);
  let previewZoom = $state(1);
  let previewX = $state(0);
  let previewY = $state(0);
  let draggingPreview = $state(false);
  let dragStart = $state({ x: 0, y: 0, offsetX: 0, offsetY: 0 });

  let selectedList = $derived(
    snapshot?.checklists.find((list) => list.id === snapshot?.selectedChecklistId) ?? null
  );
  let selectedTodo = $derived(
    selectedList?.items.find((item) => item.id === selectedTodoId) ?? null
  );
  let normalLists = $derived(snapshot?.checklists.filter((list) => list.kind === 'NORMAL') ?? []);
  let specialLists = $derived(snapshot?.checklists.filter((list) => list.kind !== 'NORMAL') ?? []);
  let locale = $derived(localeFor(snapshot?.settings.languageMode ?? 'SYSTEM'));
  let rtl = $derived(locale === 'ar');
  let displayItems = $derived.by(() => {
    if (!selectedList || !snapshot) return [];
    const items = selectedList.items.filter((item) => !snapshot.hideCompleted || !item.completed);
    return [...items].sort((left, right) => compareTodo(left, right, snapshot.sortMode));
  });

  const languageOptions: Array<{ value: AppLanguage; label: string | null }> = [
    { value: 'SYSTEM', label: null },
    { value: 'ENGLISH', label: 'English' },
    { value: 'SIMPLIFIED_CHINESE', label: '简体中文' },
    { value: 'ARABIC', label: 'العربية' },
    { value: 'FRENCH', label: 'Français' },
    { value: 'RUSSIAN', label: 'Русский' },
    { value: 'SPANISH', label: 'Español' }
  ];
  const allDockActions: DockAction[] = ['SORT', 'DEADLINE', 'HIDE_DONE', 'DELETE_DONE', 'BATCH_DELETE'];

  onMount(() => {
    const cleanups: Array<() => void> = [];
    void (async () => {
      try {
        snapshot = await api.bootstrap();
        applyPresentationSettings();
        cleanups.push(await listen<MutationResult>('snapshot://delta', ({ payload }) => {
          snapshot = applyMutation(snapshot, payload);
          applyPresentationSettings();
        }));
      } catch (error) {
        errorMessage = errorText(error);
      } finally {
        loading = false;
      }
    })();
    return () => cleanups.forEach((cleanup) => cleanup());
  });

  function t(key: MessageKey): string {
    return message(locale, key);
  }

  function applyPresentationSettings(): void {
    if (!snapshot) return;
    document.documentElement.dataset.theme = snapshot.settings.darkTheme ? 'dark' : 'light';
    document.documentElement.lang = localeFor(snapshot.settings.languageMode);
    document.documentElement.dir = localeFor(snapshot.settings.languageMode) === 'ar' ? 'rtl' : 'ltr';
  }

  function compareTodo(left: TodoItem, right: TodoItem, mode: SortMode): number {
    const leftCompleted = left.completed && !completionHold[left.id] ? 1 : 0;
    const rightCompleted = right.completed && !completionHold[right.id] ? 1 : 0;
    if (leftCompleted !== rightCompleted) return leftCompleted - rightCompleted;
    if (mode === 'TIME' && left.dueAtMillis !== right.dueAtMillis) return left.dueAtMillis - right.dueAtMillis;
    const priority = priorityRank(left.priority) - priorityRank(right.priority);
    if (priority !== 0) return priority;
    if (mode === 'PRIORITY' && left.dueAtMillis !== right.dueAtMillis) return left.dueAtMillis - right.dueAtMillis;
    return left.createdAtMillis - right.createdAtMillis;
  }

  function priorityRank(priority: TodoPriority): number {
    return { XHIGH: 0, HIGH: 1, MEDIUM: 2, LOW: 3 }[priority];
  }

  async function commit(promise: Promise<MutationResult>): Promise<MutationResult | null> {
    if (!snapshot) return null;
    errorMessage = '';
    try {
      const result = await promise;
      snapshot = applyMutation(snapshot, result);
      applyPresentationSettings();
      return result;
    } catch (error) {
      const appError = error as Partial<AppError>;
      if (appError.code === 'STALE_REVISION') snapshot = await api.bootstrap();
      errorMessage = errorText(error);
      return null;
    }
  }

  async function chooseChecklist(id: string): Promise<void> {
    if (!snapshot || id === snapshot.selectedChecklistId) return;
    selectedTodoId = null;
    isNewTodo = false;
    inspectorOpen = false;
    await commit(api.selectChecklist(snapshot.revision, id));
  }

  function chooseTodo(item: TodoItem): void {
    selectedTodoId = item.id;
    isNewTodo = false;
    draft = {
      title: item.title,
      priority: item.priority,
      dueAtMillis: item.dueAtMillis,
      reminderRepeat: item.reminderRepeat
    };
    inspectorOpen = true;
  }

  function beginNewTodo(): void {
    if (selectedList?.kind !== 'NORMAL') return;
    selectedTodoId = null;
    isNewTodo = true;
    draft = emptyDraft();
    inspectorOpen = true;
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>('#todo-title')?.focus());
  }

  async function saveTodo(): Promise<void> {
    if (!snapshot || !selectedList) return;
    const result = isNewTodo
      ? await commit(api.createTodo(snapshot.revision, selectedList.id, { ...draft }))
      : selectedTodoId
        ? await commit(api.updateTodo(snapshot.revision, selectedList.id, selectedTodoId, { ...draft }))
        : null;
    if (result) {
      const todoId = result.changedIds.find((id) => id !== selectedList.id) ?? selectedTodoId;
      highlight(todoId ?? null);
      isNewTodo = false;
      selectedTodoId = todoId ?? null;
    }
  }

  async function toggleTodo(item: TodoItem): Promise<void> {
    if (!snapshot || !selectedList) return;
    if (!item.completed) {
      completionHold = { ...completionHold, [item.id]: true };
      window.setTimeout(() => {
        const next = { ...completionHold };
        delete next[item.id];
        completionHold = next;
      }, 2000);
    } else {
      highlight(item.id);
    }
    await commit(api.toggleTodo(snapshot.revision, selectedList.id, item.id));
  }

  function highlight(todoId: string | null): void {
    highlightedTodoId = todoId;
    if (todoId) window.setTimeout(() => highlightedTodoId === todoId && (highlightedTodoId = null), 1600);
  }

  async function moveSelectedToTrash(): Promise<void> {
    if (!snapshot || !selectedList || !selectedTodoId) return;
    if (await commit(api.moveTodoToTrash(snapshot.revision, selectedList.id, selectedTodoId))) {
      selectedTodoId = null;
      inspectorOpen = false;
    }
  }

  async function submitNewList(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!snapshot || !newListName.trim()) return;
    if (await commit(api.createChecklist(snapshot.revision, newListName))) {
      newListName = '';
      creatingList = false;
    }
  }

  function beginRename(list: Checklist): void {
    editingListId = list.id;
    listNameDraft = list.name;
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>('#rename-list')?.select());
  }

  async function saveListName(): Promise<void> {
    if (!snapshot || !editingListId) return;
    if (await commit(api.renameChecklist(snapshot.revision, editingListId, listNameDraft))) editingListId = null;
  }

  async function deleteList(list: Checklist): Promise<void> {
    if (!snapshot || !confirm(`${t('delete_list_title')}: ${list.name}`)) return;
    await commit(api.deleteChecklist(snapshot.revision, list.id));
  }

  async function toggleSort(): Promise<void> {
    if (snapshot) await commit(api.setSortMode(snapshot.revision, snapshot.sortMode === 'PRIORITY' ? 'TIME' : 'PRIORITY'));
  }
  async function toggleHideDone(): Promise<void> {
    if (snapshot) await commit(api.setHideCompleted(snapshot.revision, !snapshot.hideCompleted));
  }
  async function toggleQuickDelete(): Promise<void> {
    if (snapshot) await commit(api.setQuickDelete(snapshot.revision, !snapshot.quickDelete));
  }
  async function toggleDeadline(): Promise<void> {
    if (snapshot) await commit(api.setDeadlineCountdown(snapshot.revision, !snapshot.showDeadlineCountdown));
  }
  async function cleanCompleted(): Promise<void> {
    if (snapshot && selectedList?.kind === 'NORMAL') await commit(api.cleanCompleted(snapshot.revision, selectedList.id));
  }

  async function updateSettings(settings: AppSettings): Promise<void> {
    if (snapshot) await commit(api.updateSettings(snapshot.revision, settings));
  }

  async function setLanguage(languageMode: AppLanguage): Promise<void> {
    await updateSettings({ ...snapshot.settings, languageMode });
  }

  async function setDockPlacement(plusPlacement: DockPlusPlacement): Promise<void> {
    await updateSettings({ ...snapshot.settings, dock: { ...snapshot.settings.dock, plusPlacement } });
  }

  async function toggleDockAction(action: DockAction): Promise<void> {
    const actions = snapshot.settings.dock.actions.includes(action)
      ? snapshot.settings.dock.actions.filter((value) => value !== action)
      : snapshot.settings.dock.actions.length < 4
        ? [...snapshot.settings.dock.actions, action]
        : [...snapshot.settings.dock.actions.slice(1), action];
    await updateSettings({ ...snapshot.settings, dock: { ...snapshot.settings.dock, actions } });
  }

  async function trashAction(item: TodoItem, action: 'restore' | 'purge'): Promise<void> {
    if (!snapshot) return;
    await commit(action === 'restore' ? api.restoreTodo(snapshot.revision, item.id) : api.purgeTodo(snapshot.revision, item.id));
    selectedTodoId = null;
  }

  async function authAction(): Promise<void> {
    if (!snapshot) return;
    authBusy = true;
    const result = authMode === 'sign-in'
      ? await commit(api.signIn(snapshot.revision, authEmail, authPassword))
      : await commit(api.signUp(snapshot.revision, authEmail, authPassword));
    if (result) authPassword = '';
    authBusy = false;
  }

  async function syncNow(): Promise<void> {
    if (!snapshot) return;
    if (await commit(api.syncNow(snapshot.revision)) && snapshot.sync.conflictCount > 0) await openConflicts();
  }

  async function openConflicts(): Promise<void> {
    conflicts = await api.loadConflicts();
    conflictOpen = true;
  }

  async function resolveConflict(conflict: SyncConflictView, choice: ConflictResolutionChoice): Promise<void> {
    if (!snapshot) return;
    if (await commit(api.resolveConflict(snapshot.revision, conflict.recordType, conflict.localId, choice))) {
      conflicts = await api.loadConflicts();
      conflictOpen = conflicts.length > 0;
    }
  }

  async function chooseImage(): Promise<void> {
    if (!snapshot || !selectedList || !selectedTodoId) return;
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'webp'] }]
    });
    if (typeof selected !== 'string') return;
    if (await commit(api.attachImage(snapshot.revision, selectedList.id, selectedTodoId, selected))) {
      highlight(selectedTodoId);
      await showImagePreview(selectedTodoId);
    }
  }

  async function showImagePreview(todoId: string): Promise<void> {
    previewData = await api.loadImagePreview(todoId);
    previewZoom = 1;
    previewX = 0;
    previewY = 0;
  }

  function dockAction(action: DockAction): void {
    if (action === 'SORT') void toggleSort();
    if (action === 'DEADLINE') void toggleDeadline();
    if (action === 'HIDE_DONE') void toggleHideDone();
    if (action === 'DELETE_DONE') void cleanCompleted();
    if (action === 'BATCH_DELETE') void toggleQuickDelete();
  }

  function dockActive(action: DockAction): boolean {
    return action === 'DEADLINE' ? snapshot.showDeadlineCountdown
      : action === 'HIDE_DONE' ? snapshot.hideCompleted
        : action === 'BATCH_DELETE' ? snapshot.quickDelete
          : action === 'SORT' ? snapshot.sortMode === 'TIME'
            : false;
  }

  function formatDue(item: TodoItem): string {
    if (item.dueAtMillis <= 0) return t('deadline_none');
    if (snapshot.showDeadlineCountdown) {
      const delta = item.dueAtMillis - Date.now();
      const absolute = Math.abs(delta);
      const days = Math.floor(absolute / 86_400_000);
      const hours = Math.floor((absolute % 86_400_000) / 3_600_000);
      const minutes = Math.floor((absolute % 3_600_000) / 60_000);
      return `${delta < 0 ? '-' : ''}${days}D ${hours}H ${minutes}M`;
    }
    return new Intl.DateTimeFormat(locale, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(item.dueAtMillis);
  }

  function errorText(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error && typeof error === 'object' && 'message' in error) return String(error.message);
    return t('error');
  }

  function handleKeys(event: KeyboardEvent): void {
    if (event.ctrlKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      beginNewTodo();
    } else if (event.altKey && event.key === 'ArrowLeft' && snapshot?.checklistHistory.length) {
      event.preventDefault();
      void commit(api.backChecklist(snapshot.revision));
    } else if (event.key === 'Escape') {
      inspectorOpen = false;
      previewData = null;
      conflictOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeys} />
<svelte:head><title>PixelDone</title></svelte:head>

{#if loading}
  <main class="launch-state"><span class="launch-mark">PD</span><p>{t('ready')}…</p></main>
{:else if snapshot && selectedList}
  <main class:dark={snapshot.settings.darkTheme} class:rtl class="app-shell">
    <aside class="sidebar" aria-label={t('app')}>
      <div class="brand-row">
        <div><span class="eyebrow">PIXEL UTILITY</span><h1>PixelDone</h1></div>
        <button class="icon-button" title={t('new_list')} onclick={() => (creatingList = !creatingList)}><Icon name="plus" /></button>
      </div>

      <nav class="list-nav" aria-label={t('app')}>
        <span class="section-label">CHECKLISTS</span>
        {#each normalLists as list (list.id)}
          <div class:active={list.id === snapshot.selectedChecklistId} class="nav-row">
            <button class="nav-main" oncontextmenu={(event) => { event.preventDefault(); beginRename(list); }} onclick={() => void chooseChecklist(list.id)}>
              <span class="nav-icon"><Icon name="list" /></span>
              {#if editingListId === list.id}
                <input id="rename-list" aria-label={t('list_name')} bind:value={listNameDraft} onblur={() => void saveListName()} onkeydown={(event) => event.key === 'Enter' && void saveListName()} onclick={(event) => event.stopPropagation()} />
              {:else}
                <span class="nav-name">{list.name}</span><span class="nav-count">{list.items.filter((item) => !item.completed).length}</span>
              {/if}
            </button>
            {#if editingListId !== list.id}<button class="row-more" title={t('edit_list')} onclick={() => beginRename(list)}><Icon name="more" /></button>{/if}
          </div>
        {/each}
        {#if creatingList}
          <form class="inline-create" onsubmit={submitNewList}><input aria-label={t('new_list')} placeholder="NEW LIST" bind:value={newListName} /><button type="submit">{t('add')}</button></form>
        {/if}
      </nav>

      <nav class="special-nav" aria-label={t('app_options')}>
        {#each specialLists as list (list.id)}
          <button class:active={list.id === snapshot.selectedChecklistId} class="special-row" onclick={() => void chooseChecklist(list.id)}>
            <span class="nav-icon"><Icon name={list.kind === 'TRASH' ? 'trash' : 'settings'} /></span><span>{list.name}</span>
            {#if list.kind === 'TRASH' && list.items.length}<span class="nav-count">{list.items.length}</span>{/if}
          </button>
        {/each}
      </nav>

      <div class="cloud-state"><Icon name="cloud" /><span><strong>{snapshot.sync.state}</strong>{snapshot.auth.userEmail ?? t('signed_out')}</span></div>
    </aside>

    <section class="workspace">
      <header class="workspace-header">
        <div class="title-stack">
          <span class="eyebrow">{selectedList.kind === 'NORMAL' ? 'CHECKLIST' : 'SYSTEM'}</span>
          <div class="title-line">
            {#if snapshot.checklistHistory.length}<button class="icon-button" title="Alt+Left" onclick={() => void commit(api.backChecklist(snapshot.revision))}><Icon name="back" /></button>{/if}
            <h2>{selectedList.name}</h2><span>{selectedList.items.filter((item) => !item.completed).length} {t('active_done_count').split('%')[0]}</span>
          </div>
        </div>
        <div class="header-actions">
          {#if selectedList.kind === 'NORMAL'}
            <button class="quiet-button" onclick={() => void toggleSort()}>{snapshot.sortMode} SORT</button>
            {#if normalLists.length > 1}<button class="danger-button" onclick={() => void deleteList(selectedList)}>{t('delete_list')}</button>{/if}
          {/if}
        </div>
      </header>

      {#if selectedList.kind === 'NORMAL'}
        <div class="task-list" aria-label={selectedList.name}>
          {#if displayItems.length === 0}
            <div class="empty-state"><span class="empty-glyph">□</span><h3>{t('ready')}</h3><p>{t('add_task_to_begin')}</p><button class="primary-button" onclick={beginNewTodo}>{t('new_task')}</button></div>
          {:else}
            {#each displayItems as item (item.id)}
              <div role="button" class:completed={item.completed} class:selected={item.id === selectedTodoId} class:held={completionHold[item.id]} class:highlighted={highlightedTodoId === item.id} class="task-row priority-{item.priority.toLowerCase()}" onclick={() => chooseTodo(item)} onkeydown={(event) => event.key === 'Enter' && chooseTodo(item)} tabindex="0">
                <button class:checked={item.completed} class="completion-control" aria-label={item.completed ? t('show') : t('hide')} onclick={(event) => { event.stopPropagation(); void toggleTodo(item); }}>{#if item.completed}<Icon name="check" size={12} />{/if}</button>
                <div class="task-copy"><strong>{item.title}</strong>{#if !item.completed}<span class:overdue={item.dueAtMillis <= Date.now()}>{formatDue(item)} · {item.priority} · {repeatLabel(item.reminderRepeat)}</span>{/if}</div>
                {#if item.imageFileName}<button class="attachment-badge" onclick={(event) => { event.stopPropagation(); void showImagePreview(item.id); }}><Icon name="image" /></button>{/if}
                {#if snapshot.quickDelete}<button class="delete-slot" onclick={(event) => { event.stopPropagation(); selectedTodoId = item.id; void moveSelectedToTrash(); }}>{t('delete')}</button>{/if}
              </div>
            {/each}
          {/if}
        </div>

        <div class="dock" data-placement={snapshot.settings.dock.plusPlacement}>
          {#each snapshot.settings.dock.actions as action}
            <button class:active={dockActive(action)} onclick={() => dockAction(action)}>{action === 'DEADLINE' ? 'DDL' : action === 'DELETE_DONE' ? 'CLEAN DONE' : action === 'BATCH_DELETE' ? 'QUICK DELETE' : action.replace('_', ' ')}</button>
          {/each}
          <button class="dock-add" title={t('new_task')} onclick={beginNewTodo}><Icon name="plus" size={18} /></button>
        </div>
      {:else if selectedList.kind === 'TRASH'}
        <div class="task-list trash-list">
          {#if selectedList.items.length === 0}
            <div class="empty-state"><span class="empty-glyph">×</span><h3>{t('trash_empty')}</h3><p>{t('trash_retention')}</p></div>
          {:else}
            <div class="trash-toolbar"><button class="danger-button" onclick={() => void commit(api.purgeAllTrash(snapshot.revision))}>{t('delete_all')}</button></div>
            {#each selectedList.items as item (item.id)}
              <article class="task-row trash-row"><div class="task-copy"><strong>{item.title}</strong><span>{t('from_list')} {item.trashedFromChecklistName ?? 'MAIN'}</span></div><button class="quiet-button" onclick={() => void trashAction(item, 'restore')}>{t('restore_task')}</button><button class="danger-button" onclick={() => void trashAction(item, 'purge')}>{t('delete')}</button></article>
            {/each}
          {/if}
        </div>
      {:else}
        <div class="settings-page">
          <section>
            <span class="section-label">{t('settings_cloud')}</span>
            <div class="setting-row cloud-account">
              <div><strong>{t('account')}</strong><p>{snapshot.auth.userEmail ?? t('signed_out')}</p></div>
              <div class="cloud-actions">
                {#if snapshot.auth.signedIn}
                  <button class="cloud-icon-button" title={t('sign_out')} onclick={() => void commit(api.signOut(snapshot.revision))}><Icon name="logout" size={22} /></button>
                  <button class="cloud-icon-button" title={t('sync_now')} onclick={() => void syncNow()}><Icon name="sync" size={22} /></button>
                {:else}
                  <button class="cloud-icon-button" title={t('sign_in')} onclick={() => document.querySelector<HTMLInputElement>('#auth-email')?.focus()}><Icon name="login" size={22} /></button>
                {/if}
              </div>
            </div>
            {#if !snapshot.auth.signedIn}
              <form class="auth-form" onsubmit={(event) => { event.preventDefault(); void authAction(); }}>
                <div class="auth-modes"><button type="button" class:active={authMode === 'sign-in'} onclick={() => (authMode = 'sign-in')}>{t('sign_in')}</button><button type="button" class:active={authMode === 'sign-up'} onclick={() => (authMode = 'sign-up')}>{t('sign_up')}</button></div>
                <input id="auth-email" type="email" placeholder={t('email')} bind:value={authEmail} />
                <input type="password" placeholder={t('password')} bind:value={authPassword} />
                <div class="auth-buttons"><button class="primary-button" disabled={authBusy}>{authMode === 'sign-in' ? t('sign_in') : t('sign_up')}</button><button type="button" class="quiet-button" onclick={() => void commit(api.resetPassword(snapshot.revision, authEmail))}>{t('reset')}</button></div>
              </form>
            {/if}
            <div class="setting-row"><div><strong>{t('sync')}</strong><p>{snapshot.sync.message ?? snapshot.sync.state}</p></div><div class="setting-actions"><span class="setting-value">{snapshot.sync.pendingCount} {t('pending')}</span>{#if snapshot.sync.conflictCount}<button class="primary-button" onclick={() => void openConflicts()}>{t('review')} {snapshot.sync.conflictCount}</button>{/if}</div></div>
          </section>

          <section>
            <span class="section-label">{t('settings_display')}</span>
            <div class="setting-row"><div><strong>{t('settings_language')}</strong><p>{languageOptions.find((item) => item.value === snapshot.settings.languageMode)?.label ?? t('language_system')}</p></div></div>
            <div class="language-grid">
              {#each languageOptions as language}
                <button dir="auto" class:selected={snapshot.settings.languageMode === language.value} onclick={() => void setLanguage(language.value)}><span class="pixel-choice"></span>{language.label ?? t('language_system')}</button>
              {/each}
              {#if languageOptions.length % 2}<span aria-hidden="true"></span>{/if}
            </div>
            <div class="setting-row"><div><strong>{t('settings_theme')}</strong><p>{snapshot.settings.darkTheme ? t('theme_dark') : t('theme_light')}</p></div><button aria-label={t('settings_theme')} class:active={snapshot.settings.darkTheme} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, darkTheme: !snapshot.settings.darkTheme })}><span></span></button></div>
          </section>

          <section>
            <span class="section-label">{t('settings_dock')}</span>
            <div class="segmented-row">{#each ['LEFT_EDGE', 'CENTER', 'RIGHT_EDGE'] as placement}<button class:active={snapshot.settings.dock.plusPlacement === placement} onclick={() => void setDockPlacement(placement as DockPlusPlacement)}>{placement}</button>{/each}</div>
            <div class="dock-choice-grid">{#each allDockActions as action}<button class:selected={snapshot.settings.dock.actions.includes(action)} onclick={() => void toggleDockAction(action)}><span class="pixel-choice"></span>{action}</button>{/each}</div>
          </section>

          <section>
            <span class="section-label">{t('settings_updates')}</span>
            <div class="setting-row"><div><strong>{t('current_version')}</strong><p>{snapshot.update.currentVersion} · FORMAL · x64 NSIS</p></div><button class="quiet-button" onclick={() => void commit(api.checkForUpdate(snapshot.revision))}>{t('check_update')}</button></div>
            {#if snapshot.update.state === 'AVAILABLE'}<button class="primary-button" onclick={() => void api.installUpdate()}>{t('install_updates')} {snapshot.update.availableVersion}</button>{/if}
          </section>
        </div>
      {/if}
    </section>

    <aside class:open={inspectorOpen || selectedList.kind !== 'NORMAL'} class="inspector" aria-label="INSPECTOR">
      <div class="inspector-head"><div><span class="eyebrow">INSPECTOR</span><h2>{isNewTodo ? t('new_task') : selectedTodo?.title ?? selectedList.name}</h2></div><button class="icon-button inspector-close" title={t('close')} onclick={() => (inspectorOpen = false)}><Icon name="close" /></button></div>
      {#if selectedList.kind === 'NORMAL' && (selectedTodo || isNewTodo)}
        <form class="inspector-form" onsubmit={(event) => { event.preventDefault(); void saveTodo(); }}>
          <label>{t('name')}<input id="todo-title" bind:value={draft.title} placeholder={t('new_task')} /></label>
          <fieldset><legend>{t('field_priority')}</legend><div class="priority-segments">{#each ['XHIGH', 'HIGH', 'MEDIUM', 'LOW'] as priority}<button type="button" class:active={draft.priority === priority} class="priority-{priority.toLowerCase()}" onclick={() => (draft.priority = priority as TodoPriority)}>{priority}</button>{/each}</div></fieldset>
          <label>{t('time')}<input type="datetime-local" value={dateTimeLocalValue(draft.dueAtMillis)} onchange={(event) => (draft.dueAtMillis = millisFromDateTimeLocal(event.currentTarget.value))} /></label>
          <label>{t('field_repeat')}<select bind:value={draft.reminderRepeat}><option value="NONE">{t('repeat_none')}</option><option value="DAILY">{t('repeat_daily')}</option><option value="WEEKLY">{t('repeat_weekly')}</option></select></label>
          {#if selectedTodo}
            <div class="attachment-panel"><span class="section-label">{t('task_image')}</span><div class="image-actions"><button type="button" class="quiet-button" onclick={() => void chooseImage()}>{selectedTodo.imageFileName ? t('change') : t('add')}</button>{#if selectedTodo.imageFileName}<button type="button" class="quiet-button" onclick={() => void showImagePreview(selectedTodo.id)}>{t('preview')}</button><button type="button" class="danger-button" onclick={() => void commit(api.deleteImage(snapshot.revision, selectedList.id, selectedTodo.id))}>{t('remove')}</button>{/if}</div></div>
          {/if}
          <div class="form-actions"><button type="submit" class="primary-button">{t('save')}</button>{#if selectedTodo}<button type="button" class="danger-button" onclick={() => void moveSelectedToTrash()}>{t('move_task_to_trash')}</button>{/if}</div>
        </form>
      {:else}
        <div class="inspector-summary"><span class="summary-number">{selectedList.items.length}</span><p>{t('items_count').replace('%1$d', '')}</p><dl><div><dt>ACTIVE</dt><dd>{selectedList.items.filter((item) => !item.completed).length}</dd></div><div><dt>COMPLETED</dt><dd>{selectedList.items.filter((item) => item.completed).length}</dd></div></dl>{#if selectedList.kind === 'NORMAL'}<button class="primary-button" onclick={beginNewTodo}>{t('new_task')}</button>{/if}<div class="shortcut-card"><span class="section-label">SHORTCUTS</span><p><kbd>Ctrl</kbd> + <kbd>N</kbd></p><p><kbd>Alt</kbd> + <kbd>←</kbd></p><p><kbd>Esc</kbd></p></div></div>
      {/if}
    </aside>

    <footer class="status-strip"><span class="status-ready">● {snapshot.sync.state}</span><span>REV {snapshot.revision}</span><span>{snapshot.reminder.state}</span><span>{snapshot.update.state}</span>{#if errorMessage}<strong class="status-error">{errorMessage}</strong>{/if}</footer>
  </main>

  {#if conflictOpen}
    <div class="modal-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && (conflictOpen = false)}><section class="conflict-modal" role="dialog" aria-modal="true" tabindex="-1"><header><h2>{t('sync_conflicts')}</h2><button class="icon-button" onclick={() => (conflictOpen = false)}><Icon name="close" /></button></header>{#if conflicts.length === 0}<p>{t('no_conflicts')}</p>{/if}{#each conflicts as conflict}<article class="conflict-card"><h3>{conflict.title}</h3><p>{conflict.fields.join(', ')}</p><div class="conflict-columns"><pre>{JSON.stringify(conflict.localPayload, null, 2)}</pre><pre>{JSON.stringify(conflict.cloudPayload, null, 2)}</pre></div><div class="form-actions"><button class="quiet-button" onclick={() => void resolveConflict(conflict, 'KEEP_LOCAL')}>{t('keep_local')}</button><button class="primary-button" onclick={() => void resolveConflict(conflict, 'KEEP_CLOUD')}>{t('keep_cloud')}</button></div></article>{/each}</section></div>
  {/if}

  {#if previewData}
    <div class="modal-backdrop image-backdrop" role="presentation" onclick={() => (previewData = null)} onwheel={(event) => { event.preventDefault(); previewZoom = Math.min(5, Math.max(.25, previewZoom + (event.deltaY < 0 ? .15 : -.15))); }} onpointermove={(event) => { if (draggingPreview) { previewX = dragStart.offsetX + event.clientX - dragStart.x; previewY = dragStart.offsetY + event.clientY - dragStart.y; } }} onpointerup={() => (draggingPreview = false)}>
      <button class="icon-button preview-close" onclick={() => (previewData = null)}><Icon name="close" /></button>
      <img src={previewData} alt={t('task_image_preview')} style:transform={`translate(${previewX}px, ${previewY}px) scale(${previewZoom})`} onpointerdown={(event) => { event.stopPropagation(); draggingPreview = true; dragStart = { x: event.clientX, y: event.clientY, offsetX: previewX, offsetY: previewY }; }} ondblclick={(event) => { event.stopPropagation(); previewZoom = 1; previewX = 0; previewY = 0; }} />
    </div>
  {/if}
{:else}
  <main class="launch-state error"><span class="launch-mark">!</span><p>{errorMessage || 'PixelDone'}</p></main>
{/if}
