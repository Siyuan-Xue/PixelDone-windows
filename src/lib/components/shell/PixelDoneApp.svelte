<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import Icon from '$lib/components/common/Icon.svelte';
  import TodoDock from '$lib/components/shell/TodoDock.svelte';
  import TodoEditorModal from '$lib/components/shell/TodoEditorModal.svelte';
  import { reconcileEditorMode, type TodoEditorMode } from '$lib/components/shell/editor';
  import { localeFor, message, type MessageKey } from '$lib/generated/i18n';
  import { windowsMessage, type WindowsMessageKey } from '$lib/i18n/windows';
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
    StorageInfo,
    SyncConflictView,
    TodoDraft,
    TodoItem,
    TodoPriority
  } from '$lib/generated/ipc';
  import {
    api,
    applyMutation,
    emptyDraft
  } from '$lib/ipc/client';

  let snapshot = $state<AppSnapshot>(null!);
  let loading = $state(true);
  let errorMessage = $state('');
  let selectedTodoId = $state<string | null>(null);
  let draft = $state<TodoDraft>(emptyDraft());
  let editorMode = $state<TodoEditorMode>({ kind: 'closed' });
  let editorTrigger = $state<HTMLElement | null>(null);
  let creatingList = $state(false);
  let newListName = $state('');
  let editingListId = $state<string | null>(null);
  let listNameDraft = $state('');
  let completionHold = $state<Record<string, boolean>>({});
  let highlightedTodoId = $state<string | null>(null);
  let authEmail = $state('');
  let authPassword = $state('');
  let authMode = $state<'sign-in' | 'sign-up'>('sign-in');
  let authBusy = $state(false);
  let passwordEditorOpen = $state(false);
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let passwordBusy = $state(false);
  let conflicts = $state<SyncConflictView[]>([]);
  let conflictOpen = $state(false);
  let previewData = $state<string | null>(null);
  let previewZoom = $state(1);
  let previewX = $state(0);
  let previewY = $state(0);
  let draggingPreview = $state(false);
  let dragStart = $state({ x: 0, y: 0, offsetX: 0, offsetY: 0 });
  let storageInfo = $state<StorageInfo | null>(null);
  let updateProgress = $state<{ downloadedBytes: number; totalBytes: number | null } | null>(null);
  let sidebarWidth = $state(256);
  let resizingSidebar = $state(false);

  let selectedList = $derived(
    snapshot?.checklists.find((list) => list.id === snapshot?.selectedChecklistId) ?? null
  );
  let selectedTodo = $derived.by(() => {
    const mode = editorMode;
    if (mode.kind !== 'edit') return null;
    return selectedList?.items.find((item) => item.id === mode.todoId) ?? null;
  });
  let normalLists = $derived(snapshot?.checklists.filter((list) => list.kind === 'NORMAL') ?? []);
  let specialLists = $derived(snapshot?.checklists.filter((list) => list.kind !== 'NORMAL') ?? []);
  let locale = $derived(localeFor(snapshot?.settings.languageMode ?? 'SYSTEM'));
  let rtl = $derived(locale === 'ar');
  let displayItems = $derived.by(() => {
    if (!selectedList || !snapshot) return [];
    const items = selectedList.items.filter(
      (item) => !snapshot.hideCompleted || !item.completed || completionHold[item.id]
    );
    return [...items].sort((left, right) => compareTodo(left, right, snapshot.sortMode));
  });
  let activeItems = $derived(displayItems.filter((item) => !item.completed || completionHold[item.id]));
  let completedItems = $derived(displayItems.filter((item) => item.completed && !completionHold[item.id]));

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
        storageInfo = await api.getStorageInfo();
        applyPresentationSettings();
        cleanups.push(await listen<MutationResult>('snapshot://delta', ({ payload }) => {
          snapshot = applyMutation(snapshot, payload);
          applyPresentationSettings();
        }));
        cleanups.push(await listen<{ downloadedBytes: number; totalBytes: number | null }>('update://progress', ({ payload }) => {
          updateProgress = payload;
        }));
      } catch (error) {
        errorMessage = errorText(error);
      } finally {
        loading = false;
      }
    })();
    return () => cleanups.forEach((cleanup) => cleanup());
  });

  $effect(() => {
    if (!selectedList || editorMode.kind !== 'edit') return;
    const reconciled = reconcileEditorMode(editorMode, selectedList.items);
    if (reconciled.kind === 'closed') {
      editorMode = reconciled;
      selectedTodoId = null;
      requestAnimationFrame(() => editorTrigger?.focus());
    }
  });

  function t(key: MessageKey): string {
    return message(locale, key);
  }

  function wt(key: WindowsMessageKey): string {
    return windowsMessage(locale, key);
  }

  function syncStateLabel(): string {
    const keys: Record<string, MessageKey> = {
      LOCAL_ONLY: 'local_only',
      SIGNED_OUT: 'signed_out',
      IDLE: 'ready',
      SYNCING: 'syncing',
      SYNCED: 'synced',
      CONFLICT: 'conflict',
      ERROR: 'error',
      SERVER_UPDATE_REQUIRED: 'server_update_required'
    };
    const key = keys[snapshot.sync.state];
    return key ? t(key) : snapshot.sync.state;
  }

  function syncDetailMessage(): string {
    if (snapshot.sync.state === 'SIGNED_OUT') return wt('signInToSyncAndroid');
    return snapshot.sync.message ?? syncStateLabel();
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  async function deleteLegacyData(): Promise<void> {
    if (!storageInfo?.legacyRoamingDatabasePath) return;
    if (!confirm(wt('legacyDeleteConfirm'))) return;
    await api.deleteLegacyRoamingData(true);
    storageInfo = await api.getStorageInfo();
  }

  function applyPresentationSettings(): void {
    if (!snapshot) return;
    if (!resizingSidebar) sidebarWidth = clampSidebarWidth(snapshot.settings.sidebarWidthPx);
    document.documentElement.dataset.theme = snapshot.settings.darkTheme ? 'dark' : 'light';
    document.documentElement.lang = localeFor(snapshot.settings.languageMode);
    document.documentElement.dir = localeFor(snapshot.settings.languageMode) === 'ar' ? 'rtl' : 'ltr';
  }

  function clampSidebarWidth(value: number): number {
    return Math.min(420, Math.max(220, Math.round(value)));
  }

  function sidebarWidthFromPointer(clientX: number): number {
    return clampSidebarWidth(rtl ? window.innerWidth - clientX : clientX);
  }

  function beginSidebarResize(event: PointerEvent): void {
    resizingSidebar = true;
    sidebarWidth = sidebarWidthFromPointer(event.clientX);
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function moveSidebarResize(event: PointerEvent): void {
    if (resizingSidebar) sidebarWidth = sidebarWidthFromPointer(event.clientX);
  }

  async function finishSidebarResize(event: PointerEvent): Promise<void> {
    if (!resizingSidebar) return;
    resizingSidebar = false;
    const target = event.currentTarget as HTMLElement;
    if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId);
    await persistSidebarWidth(sidebarWidth);
  }

  async function persistSidebarWidth(width: number): Promise<void> {
    const next = clampSidebarWidth(width);
    sidebarWidth = next;
    if (snapshot.settings.sidebarWidthPx !== next) {
      await updateSettings({ ...snapshot.settings, sidebarWidthPx: next });
    }
  }

  async function resizeSidebarWithKeyboard(event: KeyboardEvent): Promise<void> {
    let next = sidebarWidth;
    if (event.key === 'Home') next = 256;
    else if (event.key === 'ArrowRight') next += rtl ? -8 : 8;
    else if (event.key === 'ArrowLeft') next += rtl ? 8 : -8;
    else return;
    event.preventDefault();
    await persistSidebarWidth(next);
  }

  function sidebarResize(node: HTMLElement): { destroy: () => void } {
    node.tabIndex = 0;
    const pointerDown = (event: PointerEvent) => beginSidebarResize(event);
    const pointerMove = (event: PointerEvent) => moveSidebarResize(event);
    const pointerUp = (event: PointerEvent) => void finishSidebarResize(event);
    const keyDown = (event: KeyboardEvent) => void resizeSidebarWithKeyboard(event);
    const doubleClick = () => void persistSidebarWidth(256);
    node.addEventListener('pointerdown', pointerDown);
    node.addEventListener('pointermove', pointerMove);
    node.addEventListener('pointerup', pointerUp);
    node.addEventListener('pointercancel', pointerUp);
    node.addEventListener('keydown', keyDown);
    node.addEventListener('dblclick', doubleClick);
    return {
      destroy: () => {
        node.removeEventListener('pointerdown', pointerDown);
        node.removeEventListener('pointermove', pointerMove);
        node.removeEventListener('pointerup', pointerUp);
        node.removeEventListener('pointercancel', pointerUp);
        node.removeEventListener('keydown', keyDown);
        node.removeEventListener('dblclick', doubleClick);
      }
    };
  }

  function reminderNeedsAttention(): boolean {
    return snapshot.reminder.scheduleTruncated
      || ['ERROR', 'IDENTITY_ERROR', 'DISABLED_BY_SYSTEM'].includes(snapshot.reminder.state);
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
    closeEditor(false);
    await commit(api.selectChecklist(snapshot.revision, id));
  }

  function chooseTodo(item: TodoItem, trigger?: HTMLElement): void {
    selectedTodoId = item.id;
    editorMode = { kind: 'edit', todoId: item.id };
    editorTrigger = trigger ?? (document.activeElement as HTMLElement | null);
    draft = {
      title: item.title,
      priority: item.priority,
      dueAtMillis: item.dueAtMillis,
      reminderRepeat: item.reminderRepeat
    };
  }

  function beginNewTodo(trigger?: HTMLElement): void {
    if (selectedList?.kind !== 'NORMAL') return;
    selectedTodoId = null;
    editorMode = { kind: 'new' };
    editorTrigger = trigger ?? (document.activeElement as HTMLElement | null);
    draft = emptyDraft();
  }

  function closeEditor(restoreFocus = true): void {
    editorMode = { kind: 'closed' };
    selectedTodoId = null;
    if (restoreFocus) requestAnimationFrame(() => editorTrigger?.focus());
  }

  async function saveTodo(): Promise<void> {
    if (!snapshot || !selectedList) return;
    const result = editorMode.kind === 'new'
      ? await commit(api.createTodo(snapshot.revision, selectedList.id, { ...draft }))
      : editorMode.kind === 'edit'
        ? await commit(api.updateTodo(snapshot.revision, selectedList.id, editorMode.todoId, { ...draft }))
        : null;
    if (result) {
      const todoId = result.changedIds.find((id) => id !== selectedList.id) ?? selectedTodoId;
      highlight(todoId ?? null);
      closeEditor();
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
      closeEditor();
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

  async function changePassword(): Promise<void> {
    if (!snapshot || passwordBusy) return;
    passwordBusy = true;
    const result = await commit(api.changePassword(
      snapshot.revision,
      currentPassword,
      newPassword,
      confirmPassword
    ));
    passwordBusy = false;
    if (result) {
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      passwordEditorOpen = false;
    }
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
    try {
      previewData = await api.loadImagePreview(todoId);
      previewZoom = 1;
      previewX = 0;
      previewY = 0;
    } catch (error) {
      errorMessage = errorText(error);
    }
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

  function dockEnabled(action: DockAction): boolean {
    if (action === 'DELETE_DONE') return Boolean(selectedList?.items.some((item) => item.completed));
    if (action === 'BATCH_DELETE') return Boolean(selectedList?.items.length);
    return true;
  }

  function dockLabel(action: DockAction): string {
    return t(action === 'SORT' ? 'toggle_sort'
      : action === 'DEADLINE' ? 'toggle_deadline'
        : action === 'HIDE_DONE' ? 'toggle_done_visibility'
          : action === 'DELETE_DONE' ? 'clean_completed_tasks'
            : 'toggle_quick_delete');
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

  function repeatText(item: TodoItem): string {
    return t(item.reminderRepeat === 'NONE' ? 'repeat_none' : item.reminderRepeat === 'DAILY' ? 'repeat_daily' : 'repeat_weekly');
  }

  function errorText(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error && typeof error === 'object' && 'message' in error) return String(error.message);
    return t('error');
  }

  function handleKeys(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      if (previewData) {
        previewData = null;
      } else if (conflictOpen) {
        conflictOpen = false;
      } else if (editorMode.kind !== 'closed') {
        closeEditor();
      } else {
        creatingList = false;
        editingListId = null;
      }
      return;
    }

    const target = event.target instanceof HTMLElement ? event.target : null;
    const isEditingText = Boolean(target?.closest('input, textarea, select, [contenteditable="true"]'));
    if (editorMode.kind !== 'closed' || conflictOpen || previewData || isEditingText) return;

    if (event.ctrlKey && event.shiftKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      creatingList = true;
      requestAnimationFrame(() => document.querySelector<HTMLInputElement>('#new-list-name')?.focus());
    } else if (event.ctrlKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      beginNewTodo();
    } else if (event.altKey && event.key === 'ArrowLeft' && snapshot?.checklistHistory.length) {
      event.preventDefault();
      void commit(api.backChecklist(snapshot.revision));
    }
  }
</script>

<svelte:window onkeydown={handleKeys} />
<svelte:head><title>PixelDone — CODEX &amp; XUE</title></svelte:head>

{#if loading}
  <main class="launch-state"><span class="launch-mark">PD</span><p>{t('ready')}…</p></main>
{:else if snapshot && selectedList}
  <main class:dark={snapshot.settings.darkTheme} class:rtl class:resizing={resizingSidebar} class="app-shell" style={`--sidebar-width: ${sidebarWidth}px`}>
    <aside class="sidebar" aria-label={t('app')}>
      <div class="sidebar-header">
        <span class="section-label">{wt('checklists')}</span>
        <button class="new-list-button" title={`${t('new_list')} · Ctrl+Shift+N`} aria-label={t('new_list')} aria-expanded={creatingList} onclick={() => { creatingList = !creatingList; requestAnimationFrame(() => document.querySelector<HTMLInputElement>('#new-list-name')?.focus()); }}><Icon name="plus" /></button>
      </div>

      <nav class="list-nav" aria-label={t('app')}>
        {#if creatingList}
          <form class="inline-create" onsubmit={submitNewList}><input id="new-list-name" aria-label={t('new_list')} placeholder={t('list_name')} bind:value={newListName} /><button type="submit">{t('add')}</button></form>
        {/if}
        {#each normalLists as list (list.id)}
          <div class:active={list.id === snapshot.selectedChecklistId} class="nav-row">
            {#if editingListId === list.id}
              <div class="nav-main nav-main-edit">
                <span class="nav-icon"><Icon name="list" /></span>
                <input id="rename-list" aria-label={t('list_name')} bind:value={listNameDraft} onblur={() => void saveListName()} onkeydown={(event) => event.key === 'Enter' && void saveListName()} />
              </div>
            {:else}
              <button class="nav-main" onclick={() => void chooseChecklist(list.id)}>
                <span class="nav-icon"><Icon name="list" /></span>
                <span class="nav-name">{list.name}</span><span class="nav-count">{list.items.filter((item) => !item.completed).length}</span>
              </button>
            {/if}
            {#if editingListId !== list.id}<button class="row-more" title={t('edit_list')} aria-label={`${t('edit_list')}: ${list.name}`} onclick={() => beginRename(list)}><Icon name="edit" /></button>{/if}
            {#if editingListId !== list.id && normalLists.length > 1}<button class="row-delete" title={t('delete_list')} aria-label={`${t('delete_list')}: ${list.name}`} onclick={() => void deleteList(list)}><Icon name="trash" /></button>{/if}
          </div>
        {/each}
      </nav>

      <nav class="special-nav" aria-label={t('app_options')}>
        {#each specialLists as list (list.id)}
          <button class:active={list.id === snapshot.selectedChecklistId} class="special-row" onclick={() => void chooseChecklist(list.id)}>
            <span class="nav-icon"><Icon name={list.kind === 'TRASH' ? 'trash' : 'settings'} /></span><span>{list.name}</span>
            {#if list.kind === 'TRASH' && list.items.length}<span class="nav-count">{list.items.length}</span>{/if}
          </button>
        {/each}
      </nav>

      <footer class="sidebar-footer">
        <div class="cloud-state">
          <button class="sidebar-account-button" aria-label={t('account')} onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}><Icon name="cloud" /><span><strong>{syncStateLabel()}</strong>{snapshot.auth.userEmail ?? t('signed_out')}</span></button>
          {#if snapshot.auth.signedIn}<button class="sidebar-signout" title={t('sign_out')} aria-label={t('sign_out')} onclick={() => void commit(api.signOut(snapshot.revision))}><Icon name="logout" size={20} /></button>{/if}
        </div>
      </footer>
      <div
        use:sidebarResize
        class="sidebar-resizer"
        role="separator"
        aria-label={wt('checklists')}
        aria-orientation="vertical"
        aria-valuemin="220"
        aria-valuemax="420"
        aria-valuenow={sidebarWidth}
      ></div>
    </aside>

    <section class="workspace">
      <header class="workspace-header workspace-status">
        <div class="title-stack status-title">
          <div class="title-line">
            {#if snapshot.checklistHistory.length}<button class="icon-button" title="Alt+Left" onclick={() => void commit(api.backChecklist(snapshot.revision))}><Icon name="back" /></button>{/if}
            <h2 dir="auto">{selectedList.name}</h2>
            {#if selectedList.kind === 'NORMAL'}<span class="status-counts"><span class="status-count">{wt('active')} {selectedList.items.filter((item) => !item.completed).length}</span><span class="status-count">{wt('done')} {selectedList.items.filter((item) => item.completed).length}</span></span>{/if}
          </div>
        </div>
        <div class="header-actions status-actions status-signals">
          {#if snapshot.sync.conflictCount}<button class="status-chip status-signal conflict" onclick={() => void openConflicts()}>{t('conflicts')} {snapshot.sync.conflictCount}</button>{/if}
          {#if snapshot.update.state === 'AVAILABLE'}<button class="status-chip status-signal update-chip" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{wt('updateReady')}</button>{/if}
          {#if reminderNeedsAttention()}<button class="status-chip status-signal error" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{wt('notificationIssue')}</button>{/if}
        </div>
      </header>

      {#if snapshot.settings.enhancedXhighAlarmEnabled && snapshot.reminder.activeTodoIds.length}
        <div class="workspace-alert"><strong>{wt('alarmRinging')}</strong><button class="quiet-button" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{wt('openSettings')}</button></div>
      {/if}

      {#if errorMessage}
        <div class="workspace-alert operation-error" role="alert" aria-live="assertive">
          <span>{errorMessage}</span>
          <button class="icon-button" type="button" aria-label={t('close')} onclick={() => (errorMessage = '')}><Icon name="close" /></button>
        </div>
      {/if}

      {#if selectedList.kind === 'NORMAL'}
        <div class="task-list" aria-label={selectedList.name}>
          <div class="task-list-inner">
          {#if selectedList.items.length === 0}
            <div class="empty-state"><span class="empty-glyph">□</span><h3>{t('ready')}</h3><p>{t('add_task_to_begin')}</p><button class="primary-button" onclick={(event) => beginNewTodo(event.currentTarget)}>{t('new_task')}</button></div>
          {:else}
            {#each activeItems as item (item.id)}
              <article class:completed={item.completed} class:selected={item.id === selectedTodoId} class:held={completionHold[item.id]} class:highlighted={highlightedTodoId === item.id} class="task-row priority-{item.priority.toLowerCase()}">
                <button class:checked={item.completed} class="completion-control" aria-label={item.completed ? t('show') : t('hide')} onclick={(event) => { event.stopPropagation(); void toggleTodo(item); }}>{#if item.completed}<Icon name="check" size={12} />{/if}</button>
                <button class="task-open task-copy" onclick={(event) => chooseTodo(item, event.currentTarget)}><strong dir="auto">{item.title}</strong>{#if !item.completed}<span class:overdue={item.dueAtMillis > 0 && item.dueAtMillis <= Date.now()}>{#if item.priority === 'XHIGH'}<Icon name="alarm" size={13} /> {/if}{formatDue(item)} · {item.priority} · {repeatText(item)}</span>{/if}</button>
                {#if item.imageFileName}<button class="attachment-badge" onclick={(event) => { event.stopPropagation(); void showImagePreview(item.id); }}><Icon name="image" /></button>{/if}
                {#if snapshot.quickDelete}<button class="delete-slot" onclick={(event) => { event.stopPropagation(); selectedTodoId = item.id; void moveSelectedToTrash(); }}>{t('delete')}</button>{/if}
              </article>
            {/each}

            {#if selectedList.items.some((item) => item.completed)}
              <section class="completed-group" aria-label={wt('completedTasks')}>
                <header class="completed-group-header">
                  <span><strong>{wt('completedTasks')}</strong> · {selectedList.items.filter((item) => item.completed).length}</span>
                  <div class="completed-group-actions">
                    <button class="quiet-button" onclick={() => void toggleHideDone()}>{snapshot.hideCompleted ? wt('showCompleted') : wt('hideCompleted')}</button>
                    <button class="quiet-button" onclick={() => void cleanCompleted()}>{t('clean_done')}</button>
                  </div>
                </header>
                {#if !snapshot.hideCompleted}
                  {#each completedItems as item (item.id)}
                    <article class:selected={item.id === selectedTodoId} class:highlighted={highlightedTodoId === item.id} class="task-row completed priority-{item.priority.toLowerCase()}">
                      <button class="completion-control checked" aria-label={t('show')} onclick={() => void toggleTodo(item)}><Icon name="check" size={12} /></button>
                      <button class="task-open task-copy" onclick={(event) => chooseTodo(item, event.currentTarget)}><strong dir="auto">{item.title}</strong></button>
                      {#if item.imageFileName}<button class="attachment-badge" onclick={() => void showImagePreview(item.id)}><Icon name="image" /></button>{/if}
                      {#if snapshot.quickDelete}<button class="delete-slot" onclick={() => { selectedTodoId = item.id; void moveSelectedToTrash(); }}>{t('delete')}</button>{/if}
                    </article>
                  {/each}
                {/if}
              </section>
            {/if}
          {/if}
          </div>
        </div>

        <TodoDock
          actions={snapshot.settings.dock.actions}
          placement={snapshot.settings.dock.plusPlacement}
          active={dockActive}
          enabled={dockEnabled}
          labelFor={dockLabel}
          addLabel={t('new_task')}
          onAction={dockAction}
          onAdd={beginNewTodo}
        />
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
          <div class="settings-grid">
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
                <div class="auth-buttons"><button class="primary-button" disabled={authBusy}>{authMode === 'sign-in' ? t('sign_in') : t('sign_up')}</button></div>
              </form>
            {:else}
              <div class="setting-row">
                <div><strong>{wt('changePassword')}</strong><p>{snapshot.auth.userEmail}</p></div>
                <button class="quiet-button" onclick={() => (passwordEditorOpen = !passwordEditorOpen)}>{wt('changePassword')}</button>
              </div>
              {#if passwordEditorOpen}
                <form class="auth-form" onsubmit={(event) => { event.preventDefault(); void changePassword(); }}>
                  <input type="password" autocomplete="current-password" placeholder={wt('currentPassword')} bind:value={currentPassword} />
                  <input type="password" autocomplete="new-password" placeholder={wt('newPassword')} bind:value={newPassword} />
                  <input type="password" autocomplete="new-password" placeholder={wt('confirmPassword')} bind:value={confirmPassword} />
                  <div class="auth-buttons"><button class="primary-button" disabled={passwordBusy}>{passwordBusy ? wt('changingPassword') : wt('changePassword')}</button></div>
                </form>
              {/if}
            {/if}
            <div class="setting-row"><div><strong>{t('sync')}</strong><p data-testid="sync-detail">{syncDetailMessage()}</p></div><div class="setting-actions"><span class="setting-value">{snapshot.sync.pendingCount} {t('pending')}</span>{#if snapshot.sync.conflictCount}<button class="primary-button" onclick={() => void openConflicts()}>{t('review')} {snapshot.sync.conflictCount}</button>{/if}</div></div>
          </section>

          <section>
            <span class="section-label">{t('settings_display')}</span>
            <div class="setting-row"><div><strong>{t('settings_language')}</strong><p>{languageOptions.find((item) => item.value === snapshot.settings.languageMode)?.label ?? t('language_system')}</p></div></div>
            <div class="language-grid">
              {#each languageOptions as language}
                <button class:selected={snapshot.settings.languageMode === language.value} onclick={() => void setLanguage(language.value)}><span class="pixel-choice"></span><span class="language-label" dir="auto">{language.label ?? t('language_system')}</span></button>
              {/each}
              {#if languageOptions.length % 2}<span aria-hidden="true"></span>{/if}
            </div>
            <div class="setting-row"><div><strong>{t('settings_theme')}</strong><p>{snapshot.settings.darkTheme ? t('theme_dark') : t('theme_light')}</p></div><button aria-label={t('settings_theme')} class:active={snapshot.settings.darkTheme} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, darkTheme: !snapshot.settings.darkTheme })}><span></span></button></div>
          </section>

          <section>
            <span class="section-label">{t('settings_dock')}</span>
            <div class="segmented-row">{#each ['LEFT_EDGE', 'CENTER', 'RIGHT_EDGE'] as placement}<button class="dock-placement-button" data-placement={placement} class:active={snapshot.settings.dock.plusPlacement === placement} onclick={() => void setDockPlacement(placement as DockPlusPlacement)}>{placement}</button>{/each}</div>
            <div class="dock-choice-grid">{#each allDockActions as action}<button class:selected={snapshot.settings.dock.actions.includes(action)} onclick={() => void toggleDockAction(action)}><span class="pixel-choice"></span>{action}</button>{/each}</div>
          </section>

          <section>
            <span class="section-label">{t('settings_updates')}</span>
            <div class="setting-row"><div><strong>{wt('automaticUpdateChecks')}</strong><p>{wt('automaticUpdateChecksDetail')}</p></div><button aria-label={wt('automaticUpdateChecks')} class:active={snapshot.settings.automaticUpdateCheckEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, automaticUpdateCheckEnabled: !snapshot.settings.automaticUpdateCheckEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('currentVersion')}</strong><p>{snapshot.update.currentVersion} · FORMAL · x64 NSIS{#if snapshot.update.message} · {snapshot.update.message}{/if}</p></div><button class="quiet-button" onclick={() => void commit(api.checkForUpdate(snapshot.revision))}>{t('check_update')}</button></div>
            {#if snapshot.update.state === 'AVAILABLE'}<button class="primary-button" onclick={() => { updateProgress = { downloadedBytes: 0, totalBytes: null }; void api.installUpdate(); }}>{t('install_updates')} {snapshot.update.availableVersion}</button>{/if}
            {#if updateProgress}<p class="update-progress">{wt('downloading')} {formatBytes(updateProgress.downloadedBytes)}{#if updateProgress.totalBytes} / {formatBytes(updateProgress.totalBytes)}{/if}</p>{/if}
          </section>

          <section>
            <span class="section-label">{wt('windowsIntegration')}</span>
            <div class="setting-row"><div><strong>{wt('startWithWindows')}</strong><p>{wt('startWithWindowsDetail')}</p></div><button aria-label={wt('startWithWindows')} class:active={snapshot.settings.autostartEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, autostartEnabled: !snapshot.settings.autostartEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('enhancedAlarm')}</strong><p>{wt('enhancedAlarmDetail')}</p></div><button aria-label={wt('enhancedAlarm')} class:active={snapshot.settings.enhancedXhighAlarmEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, enhancedXhighAlarmEnabled: !snapshot.settings.enhancedXhighAlarmEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('reminderQueue')}</strong><p>{snapshot.reminder.scheduledCount} · {snapshot.reminder.state}{#if snapshot.reminder.message} · {snapshot.reminder.message}{/if}</p></div></div>
          </section>

          <section>
            <span class="section-label">{wt('storagePrivacy')}</span>
            {#if storageInfo}
              <div class="setting-row storage-row"><div><strong>{wt('application')}</strong><p><code>{storageInfo.executablePath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>{wt('localData')}</strong><p><code>{storageInfo.dataRoot}</code> · {formatBytes(storageInfo.totalBytes)}</p></div><button class="quiet-button" onclick={() => void api.openDataFolder()}>{wt('openFolder')}</button></div>
              <div class="setting-row storage-row"><div><strong>SQLite</strong><p><code>{storageInfo.databasePath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>WebView2</strong><p><code>{storageInfo.webviewDataPath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>Windows Credential Manager</strong><p><code>{storageInfo.credentialManagerTarget}</code></p></div></div>
              {#if storageInfo.legacyRoamingDatabasePath}<div class="setting-row storage-row"><div><strong>{wt('legacyDataFound')}</strong><p><code>{storageInfo.legacyRoamingDatabasePath}</code> · {formatBytes(storageInfo.legacyRoamingDatabaseBytes ?? 0)}</p></div><button class="danger-button" onclick={() => void deleteLegacyData()}>{t('delete')}</button></div>{/if}
            {/if}
          </section>
          </div>
        </div>
      {/if}
    </section>

  </main>

  {#if selectedList.kind === 'NORMAL' && editorMode.kind !== 'closed'}
    <TodoEditorModal
      mode={editorMode}
      todo={selectedTodo}
      {draft}
      {locale}
      onDraft={(value) => (draft = value)}
      onSave={saveTodo}
      onClose={closeEditor}
      onChooseImage={chooseImage}
      onPreviewImage={() => selectedTodo ? showImagePreview(selectedTodo.id) : undefined}
      onRemoveImage={async () => { if (selectedTodo) await commit(api.deleteImage(snapshot.revision, selectedList.id, selectedTodo.id)); }}
      onDelete={moveSelectedToTrash}
    />
  {/if}

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
