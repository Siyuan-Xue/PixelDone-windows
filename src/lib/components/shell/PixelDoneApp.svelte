<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import AuthModal from '$lib/components/auth/AuthModal.svelte';
  import PasswordModal from '$lib/components/auth/PasswordModal.svelte';
  import {
    authEmailForSubmission,
    validateAuthInput,
    type AuthMode,
    type AuthValidationError
  } from '$lib/components/auth/auth';
  import {
    validatePasswordChange,
    type PasswordValidationError
  } from '$lib/components/auth/password';
  import Icon from '$lib/components/common/Icon.svelte';
  import ScriptAwareText from '$lib/components/common/ScriptAwareText.svelte';
  import DestructiveActionModal from '$lib/components/shell/DestructiveActionModal.svelte';
  import TodoDock from '$lib/components/shell/TodoDock.svelte';
  import TodoEditorModal from '$lib/components/shell/TodoEditorModal.svelte';
  import {
    checklistConfirmation,
    completedConfirmation,
    destructivePolicy,
    legacyDataConfirmation,
    todoConfirmation,
    trashAllConfirmation,
    trashItemConfirmation,
    type DestructiveConfirmation,
    type DestructivePolicy
  } from '$lib/components/shell/destructive';
  import { reconcileEditorMode, type TodoEditorMode } from '$lib/components/shell/editor';
  import {
    filterTrashItems,
    trashSourceName,
    trashSourceOptions,
    type TrashPriorityFilter
  } from '$lib/components/shell/trash';
  import { localeFor, type MessageKey } from '$lib/generated/i18n';
  import type { WindowsMessageKey, WindowsReliabilityMessageKey } from '$lib/i18n/windows';
  import {
    uiMessage,
    uiText,
    uiWindowsAuthValidationMessage,
    uiWindowsReliabilityMessage,
    uiWindowsMessage
  } from '$lib/i18n/presentation';
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
    SyncConflictFieldView,
    SyncConflictValueView,
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
  import { mutationDisposition } from '$lib/ipc/revision';

  let snapshot = $state<AppSnapshot>(null!);
  let loading = $state(true);
  let launchErrorMessage = $state('');
  let workspaceNotice = $state<{ tone: 'success' | 'warning' | 'error'; message: string } | null>(null);
  let workspaceNoticeTimer: number | null = null;
  let selectedTodoId = $state<string | null>(null);
  let draft = $state<TodoDraft>(emptyDraft());
  let editorMode = $state<TodoEditorMode>({ kind: 'closed' });
  let editorTrigger = $state<HTMLElement | null>(null);
  let creatingList = $state(false);
  let newListName = $state('');
  let editingListId = $state<string | null>(null);
  let listNameDraft = $state('');
  let destructiveConfirmation = $state<DestructiveConfirmation | null>(null);
  let destructiveConfirmationTrigger = $state<HTMLElement | null>(null);
  let destructiveConfirmationBusy = $state(false);
  let completionHold = $state<Record<string, boolean>>({});
  let highlightedTodoId = $state<string | null>(null);
  let authEmail = $state('');
  let authPassword = $state('');
  let authMode = $state<AuthMode>('sign-in');
  let authBusy = $state(false);
  let authModalOpen = $state(false);
  let authError = $state('');
  let authErrorKind = $state<AuthValidationError | null>(null);
  let authTrigger = $state<HTMLElement | null>(null);
  let authModalGeneration = 0;
  let authSubmissionId = 0;
  let passwordModalOpen = $state(false);
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let passwordBusy = $state(false);
  let passwordError = $state('');
  let passwordErrorKind = $state<PasswordValidationError | null>(null);
  let passwordTrigger = $state<HTMLElement | null>(null);
  let passwordModalGeneration = 0;
  let passwordSubmissionId = 0;
  let conflicts = $state<SyncConflictView[]>([]);
  let conflictOpen = $state(false);
  let resolvingConflictKey = $state<string | null>(null);
  let conflictActionError = $state('');
  let previewData = $state<string | null>(null);
  let previewZoom = $state(1);
  let previewX = $state(0);
  let previewY = $state(0);
  let draggingPreview = $state(false);
  let dragStart = $state({ x: 0, y: 0, offsetX: 0, offsetY: 0 });
  let storageInfo = $state<StorageInfo | null>(null);
  let updateProgress = $state<{ downloadedBytes: number; totalBytes: number | null } | null>(null);
  let sidebarWidth = $state(320);
  let resizingSidebar = $state(false);
  let trashSearchQuery = $state('');
  let trashPriorityFilter = $state<TrashPriorityFilter>('');
  let trashChecklistFilter = $state('');
  let wasTrashSelected = false;

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
  let filteredTrashItems = $derived.by(() => selectedList?.kind === 'TRASH'
    ? filterTrashItems(selectedList.items, {
        query: trashSearchQuery,
        priority: trashPriorityFilter,
        checklistName: trashChecklistFilter
      }, locale)
    : []);
  let availableTrashSources = $derived.by(() => selectedList?.kind === 'TRASH'
    ? trashSourceOptions(selectedList.items, locale)
    : []);
  let activeDestructivePolicy = $derived(
    destructiveConfirmation ? destructivePolicy(destructiveConfirmation) : null
  );

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
  const sidebarMinimumWidth = 200;
  const sidebarMaximumWidth = 720;
  const workspaceMinimumWidth = 440;

  onMount(() => {
    const cleanups: Array<() => void> = [];
    const clampForViewport = () => {
      if (!resizingSidebar) sidebarWidth = clampSidebarWidth(sidebarWidth);
    };
    window.addEventListener('resize', clampForViewport);
    cleanups.push(() => window.removeEventListener('resize', clampForViewport));
    void (async () => {
      try {
        snapshot = await api.bootstrap();
        storageInfo = await api.getStorageInfo();
        applyPresentationSettings();
        cleanups.push(await listen<MutationResult>('snapshot://delta', ({ payload }) => {
          void acceptMutation(payload);
        }));
        cleanups.push(await listen<{ downloadedBytes: number; totalBytes: number | null }>('update://progress', ({ payload }) => {
          updateProgress = payload;
        }));
      } catch (error) {
        launchErrorMessage = errorText(error);
      } finally {
        loading = false;
      }
    })();
    return () => {
      cleanups.forEach((cleanup) => cleanup());
      if (workspaceNoticeTimer !== null) window.clearTimeout(workspaceNoticeTimer);
    };
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

  $effect(() => {
    if (snapshot?.auth.signedIn && authModalOpen) closeAuthModal();
  });

  $effect(() => {
    const inTrash = selectedList?.kind === 'TRASH';
    if ((wasTrashSelected && !inTrash) || (inTrash && selectedList.items.length === 0)) {
      resetTrashFilters();
    }
    wasTrashSelected = inTrash;
  });

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
  }

  function wt(key: WindowsMessageKey): string {
    return uiWindowsMessage(locale, key);
  }

  function formatConfirmationMessage(key: WindowsMessageKey, action: DestructiveConfirmation): string {
    const count = 'count' in action ? action.count : 0;
    return wt(key).replaceAll('{count}', String(count));
  }

  function confirmationTitle(policy: DestructivePolicy): string {
    return policy.title.source === 'shared' ? t(policy.title.key) : wt(policy.title.key);
  }

  function confirmationContext(policy: DestructivePolicy): string {
    if (policy.context === 'trash') return t('field_trash');
    if (policy.context === 'storage') return wt('storagePrivacy');
    return wt('checklists');
  }

  function confirmationTarget(policy: DestructivePolicy): string {
    return policy.target.kind === 'label'
      ? policy.target.value
      : wt(policy.target.count === 1 ? 'taskCountOne' : 'taskCount')
          .replace('{count}', String(policy.target.count));
  }

  function rt(key: WindowsReliabilityMessageKey): string {
    return uiWindowsReliabilityMessage(locale, key);
  }

  function resetTrashFilters(): void {
    trashSearchQuery = '';
    trashPriorityFilter = '';
    trashChecklistFilter = '';
  }

  function languageTagFor(language: AppLanguage): string {
    if (language === 'SYSTEM') return locale;
    return {
      ENGLISH: 'en',
      SIMPLIFIED_CHINESE: 'zh-Hans',
      ARABIC: 'ar',
      FRENCH: 'fr',
      RUSSIAN: 'ru',
      SPANISH: 'es'
    }[language];
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
    const issueKeys: Record<string, WindowsReliabilityMessageKey> = {
      NETWORK_RETRYING: 'syncNetworkRetrying',
      AUTH_EXPIRED: 'syncAuthExpired',
      SERVER_UPDATE_REQUIRED: 'syncServerUpdateRequired',
      LOCAL_STORAGE_ERROR: 'syncLocalStorageError',
      REMOTE_DATA_INVALID: 'syncRemoteDataInvalid',
      UNKNOWN: 'syncUnknown'
    };
    if (snapshot.sync.issueCode) return rt(issueKeys[snapshot.sync.issueCode] ?? 'syncUnknown');
    if (snapshot.sync.state === 'SIGNED_OUT') return wt('signInToSyncAndroid');
    return uiText(snapshot.sync.message ?? syncStateLabel());
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  function applyPresentationSettings(): void {
    if (!snapshot) return;
    if (!resizingSidebar) sidebarWidth = clampSidebarWidth(snapshot.settings.sidebarWidthPx);
    document.documentElement.dataset.theme = snapshot.settings.darkTheme ? 'dark' : 'light';
    document.documentElement.lang = localeFor(snapshot.settings.languageMode);
    document.documentElement.dir = localeFor(snapshot.settings.languageMode) === 'ar' ? 'rtl' : 'ltr';
  }

  function clampSidebarWidth(value: number): number {
    const viewportMaximum = Math.max(sidebarMinimumWidth, window.innerWidth - workspaceMinimumWidth);
    return Math.min(sidebarMaximumWidth, viewportMaximum, Math.max(sidebarMinimumWidth, Math.round(value)));
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
    if (event.key === 'Home') next = 320;
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
    const doubleClick = () => void persistSidebarWidth(320);
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

  type MutationFactory = (revision: number) => Promise<MutationResult>;
  type MutationSurface = 'workspace' | 'sync' | 'update' | 'notification' | 'silent';

  function dismissWorkspaceNotice(): void {
    if (workspaceNoticeTimer !== null) window.clearTimeout(workspaceNoticeTimer);
    workspaceNoticeTimer = null;
    workspaceNotice = null;
  }

  function showWorkspaceNotice(
    tone: 'success' | 'warning' | 'error',
    message: string,
    autoDismiss = false
  ): void {
    dismissWorkspaceNotice();
    workspaceNotice = { tone, message };
    if (autoDismiss) {
      workspaceNoticeTimer = window.setTimeout(() => {
        workspaceNotice = null;
        workspaceNoticeTimer = null;
      }, 6_000);
    }
  }

  async function refreshSnapshot(): Promise<void> {
    const latest = await api.bootstrap();
    if (!snapshot || latest.revision >= snapshot.revision) snapshot = latest;
    applyPresentationSettings();
  }

  async function acceptMutation(result: MutationResult): Promise<void> {
    if (!snapshot) return;
    const disposition = mutationDisposition(snapshot.revision, result.revision);
    if (disposition === 'ignore') return;
    if (disposition === 'reload') {
      await refreshSnapshot();
      return;
    }
    snapshot = applyMutation(snapshot, result);
    applyPresentationSettings();
  }

  async function executeMutation(
    factory: MutationFactory
  ): Promise<{ result: MutationResult | null; error: unknown | null }> {
    if (!snapshot) return { result: null, error: null };
    for (let attempt = 0; attempt < 2; attempt += 1) {
      try {
        const result = await factory(snapshot.revision);
        await acceptMutation(result);
        return { result, error: null };
      } catch (error) {
        const appError = error as Partial<AppError>;
        if (appError.code === 'STALE_REVISION') {
          try {
            await refreshSnapshot();
          } catch (bootstrapError) {
            return { result: null, error: bootstrapError };
          }
          if (attempt === 0) continue;
          return {
            result: null,
            error: { code: 'STALE_REVISION_REPEATED', message: rt('staleRevisionAgain') }
          };
        }
        if (appError.code === 'NOT_FOUND') {
          try {
            await refreshSnapshot();
          } catch (bootstrapError) {
            return { result: null, error: bootstrapError };
          }
        }
        return { result: null, error };
      }
    }
    return { result: null, error: null };
  }

  function mutationErrorText(error: unknown): string {
    const code = (error as Partial<AppError> | null)?.code;
    if (code === 'STALE_REVISION_REPEATED') return rt('staleRevisionAgain');
    if (code === 'NOT_FOUND') return rt('targetChanged');
    return errorText(error);
  }

  async function routeMutationError(error: unknown, surface: MutationSurface): Promise<void> {
    const code = (error as Partial<AppError> | null)?.code;
    if (surface === 'sync' || surface === 'update' || surface === 'notification') {
      try {
        await refreshSnapshot();
      } catch (bootstrapError) {
        showWorkspaceNotice('error', errorText(bootstrapError));
      }
      return;
    }
    if (surface === 'silent') return;
    const fatal = ['DATABASE_ERROR', 'PLATFORM_ERROR', 'INITIALIZATION_ERROR'].includes(code ?? '');
    showWorkspaceNotice(fatal ? 'error' : 'warning', mutationErrorText(error));
  }

  async function commit(
    factory: MutationFactory,
    surface: MutationSurface = 'workspace'
  ): Promise<MutationResult | null> {
    if (workspaceNotice?.tone !== 'success') dismissWorkspaceNotice();
    const outcome = await executeMutation(factory);
    if (outcome.error) await routeMutationError(outcome.error, surface);
    return outcome.result;
  }

  async function chooseChecklist(id: string): Promise<void> {
    if (!snapshot || id === snapshot.selectedChecklistId) return;
    selectedTodoId = null;
    closeEditor(false);
    await commit((revision) => api.selectChecklist(revision, id));
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
    const listId = selectedList.id;
    const mode = editorMode;
    const submittedDraft = { ...draft };
    const result = mode.kind === 'new'
      ? await commit((revision) => api.createTodo(revision, listId, submittedDraft))
      : mode.kind === 'edit'
        ? await commit((revision) => api.updateTodo(revision, listId, mode.todoId, submittedDraft))
        : null;
    if (result) {
      const todoId = result.changedIds.find((id) => id !== listId) ?? selectedTodoId;
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
    await commit((revision) => api.toggleTodo(revision, selectedList.id, item.id));
  }

  function highlight(todoId: string | null): void {
    highlightedTodoId = todoId;
    if (todoId) window.setTimeout(() => highlightedTodoId === todoId && (highlightedTodoId = null), 1600);
  }

  async function moveSelectedToTrash(): Promise<void> {
    if (!snapshot || !selectedList || !selectedTodoId) return;
    if (await commit((revision) => api.moveTodoToTrash(revision, selectedList.id, selectedTodoId!))) {
      closeEditor();
    }
  }

  function openDestructiveConfirmation(action: DestructiveConfirmation, trigger: HTMLElement): void {
    destructiveConfirmation = action;
    destructiveConfirmationTrigger = trigger;
    destructiveConfirmationBusy = false;
  }

  function closeDestructiveConfirmation(restoreFocus = true): void {
    if (destructiveConfirmationBusy) return;
    const trigger = destructiveConfirmationTrigger;
    destructiveConfirmation = null;
    destructiveConfirmationTrigger = null;
    if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
  }

  function focusAfterDestructiveAction(action: DestructiveConfirmation, trigger: HTMLElement | null): void {
    const triggerButton = trigger instanceof HTMLButtonElement ? trigger : null;
    if (trigger?.isConnected && !triggerButton?.disabled) {
      trigger.focus();
      return;
    }
    const selector = action.kind === 'checklist'
      ? '.new-list-button'
      : action.kind === 'trash-item'
        ? '.trash-search'
        : action.kind === 'trash-all'
          ? '.special-row.active'
          : action.kind === 'legacy-data'
            ? '.storage-row .setting-icon-button'
            : '.dock-add';
    document.querySelector<HTMLElement>(selector)?.focus();
  }

  function requestDeleteSelected(trigger: HTMLElement): void {
    if (!selectedList || !selectedTodo) return;
    openDestructiveConfirmation(
      todoConfirmation(selectedList.id, selectedTodo.id, selectedTodo.title),
      trigger
    );
  }

  function requestDeleteCompleted(trigger: HTMLElement): void {
    if (selectedList?.kind !== 'NORMAL') return;
    const count = selectedList.items.filter((item) => item.completed).length;
    if (count > 0) openDestructiveConfirmation(completedConfirmation(selectedList.id, count), trigger);
  }

  function requestDeleteChecklist(list: Checklist, trigger: HTMLElement): void {
    openDestructiveConfirmation(checklistConfirmation(list.id, list.name), trigger);
  }

  function requestPurgeTrashItem(item: TodoItem, trigger: HTMLElement): void {
    openDestructiveConfirmation(trashItemConfirmation(item.id, item.title), trigger);
  }

  function requestPurgeAllTrash(trigger: HTMLElement): void {
    if (selectedList?.kind !== 'TRASH' || selectedList.items.length === 0) return;
    openDestructiveConfirmation(trashAllConfirmation(selectedList.items.length), trigger);
  }

  function requestDeleteLegacyData(trigger: HTMLElement): void {
    const path = storageInfo?.legacyRoamingDatabasePath;
    if (path) openDestructiveConfirmation(legacyDataConfirmation(path), trigger);
  }

  async function confirmDestructiveAction(): Promise<void> {
    if (!snapshot || !destructiveConfirmation || destructiveConfirmationBusy) return;
    const action = destructiveConfirmation;
    const trigger = destructiveConfirmationTrigger;
    destructiveConfirmationBusy = true;
    let succeeded = false;
    try {
      if (action.kind === 'todo') {
        succeeded = Boolean(await commit((revision) => api.moveTodoToTrash(revision, action.checklistId, action.todoId)));
      } else if (action.kind === 'completed') {
        succeeded = Boolean(await commit((revision) => api.cleanCompleted(revision, action.checklistId)));
      } else if (action.kind === 'checklist') {
        succeeded = Boolean(await commit((revision) => api.deleteChecklist(revision, action.checklistId)));
      } else if (action.kind === 'trash-item') {
        succeeded = Boolean(await commit((revision) => api.purgeTodo(revision, action.todoId)));
      } else if (action.kind === 'trash-all') {
        succeeded = Boolean(await commit((revision) => api.purgeAllTrash(revision)));
      } else {
        try {
          await api.deleteLegacyRoamingData(true);
          storageInfo = await api.getStorageInfo();
          succeeded = true;
        } catch (error) {
          showWorkspaceNotice('error', errorText(error));
        }
      }

      if (!succeeded) return;
      destructiveConfirmation = null;
      destructiveConfirmationTrigger = null;
      if (action.kind === 'todo') closeEditor(false);
      if (action.kind === 'trash-item' || action.kind === 'trash-all') selectedTodoId = null;
      requestAnimationFrame(() => focusAfterDestructiveAction(action, trigger));
    } finally {
      destructiveConfirmationBusy = false;
    }
  }

  async function submitNewList(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!snapshot || !newListName.trim()) return;
    if (await commit((revision) => api.createChecklist(revision, newListName))) {
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
    if (await commit((revision) => api.renameChecklist(revision, editingListId!, listNameDraft))) editingListId = null;
  }

  async function toggleSort(): Promise<void> {
    if (snapshot) {
      const next = snapshot.sortMode === 'PRIORITY' ? 'TIME' : 'PRIORITY';
      await commit((revision) => api.setSortMode(revision, next));
    }
  }
  async function toggleHideDone(): Promise<void> {
    if (snapshot) {
      const next = !snapshot.hideCompleted;
      await commit((revision) => api.setHideCompleted(revision, next));
    }
  }
  async function toggleQuickDelete(): Promise<void> {
    if (snapshot) {
      const next = !snapshot.quickDelete;
      await commit((revision) => api.setQuickDelete(revision, next));
    }
  }
  async function toggleDeadline(): Promise<void> {
    if (snapshot) {
      const next = !snapshot.showDeadlineCountdown;
      await commit((revision) => api.setDeadlineCountdown(revision, next));
    }
  }
  async function updateSettings(settings: AppSettings): Promise<void> {
    if (snapshot) await commit((revision) => api.updateSettings(revision, settings));
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

  async function restoreTrashItem(item: TodoItem): Promise<void> {
    if (!snapshot) return;
    await commit((revision) => api.restoreTodo(revision, item.id));
    selectedTodoId = null;
  }

  function openAuthModal(trigger: HTMLElement): void {
    authTrigger = trigger;
    authMode = 'sign-in';
    authPassword = '';
    authError = '';
    authErrorKind = null;
    authModalGeneration += 1;
    authModalOpen = true;
  }

  function closeAuthModal(): void {
    if (!authModalOpen) return;
    const trigger = authTrigger;
    authModalOpen = false;
    authModalGeneration += 1;
    authPassword = '';
    authError = '';
    authErrorKind = null;
    authTrigger = null;
    requestAnimationFrame(() => trigger?.focus());
  }

  function setAuthMode(mode: AuthMode): void {
    authMode = mode;
    authError = '';
    authErrorKind = null;
  }

  function updateAuthEmail(value: string): void {
    authEmail = value;
    authError = '';
    authErrorKind = null;
  }

  function updateAuthPassword(value: string): void {
    authPassword = value;
    authError = '';
    authErrorKind = null;
  }

  function authValidationText(error: AuthValidationError): string {
    if (error === 'required') return t('auth_email_password_required');
    return uiWindowsAuthValidationMessage(
      locale,
      error === 'invalid-email' ? 'invalidEmail' : 'passwordTooShort'
    );
  }

  async function authAction(): Promise<void> {
    if (!snapshot || authBusy) return;
    const validationError = validateAuthInput(authEmail, authPassword);
    if (validationError) {
      authErrorKind = validationError;
      authError = authValidationText(validationError);
      return;
    }

    const generation = authModalGeneration;
    const submissionId = ++authSubmissionId;
    const email = authEmailForSubmission(authEmail);
    authBusy = true;
    authError = '';
    authErrorKind = null;
    try {
      const outcome = await executeMutation((revision) => authMode === 'sign-in'
        ? api.signIn(revision, email, authPassword)
        : api.signUp(revision, email, authPassword));
      if (outcome.result) {
        authEmail = email;
        if (authModalOpen && generation === authModalGeneration) closeAuthModal();
      } else if (outcome.error && authModalOpen && generation === authModalGeneration) {
        authError = mutationErrorText(outcome.error);
        authErrorKind = null;
      }
    } finally {
      if (submissionId === authSubmissionId) authBusy = false;
    }
  }

  function openPasswordModal(trigger: HTMLElement): void {
    passwordTrigger = trigger;
    currentPassword = '';
    newPassword = '';
    confirmPassword = '';
    passwordError = '';
    passwordErrorKind = null;
    passwordModalGeneration += 1;
    passwordModalOpen = true;
  }

  function closePasswordModal(restoreFocus = true): void {
    if (!passwordModalOpen) return;
    const trigger = passwordTrigger;
    passwordModalOpen = false;
    passwordModalGeneration += 1;
    currentPassword = '';
    newPassword = '';
    confirmPassword = '';
    passwordError = '';
    passwordErrorKind = null;
    passwordTrigger = null;
    if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
  }

  function updatePasswordField(
    field: 'current' | 'new' | 'confirmation',
    value: string
  ): void {
    if (field === 'current') currentPassword = value;
    else if (field === 'new') newPassword = value;
    else confirmPassword = value;
    passwordError = '';
    passwordErrorKind = null;
  }

  function passwordValidationText(error: PasswordValidationError): string {
    return rt(error === 'required' ? 'passwordFieldsRequired'
      : error === 'password-short' ? 'passwordTooShort'
        : error === 'same-password' ? 'passwordMustDiffer'
          : 'passwordsDoNotMatch');
  }

  async function changePassword(): Promise<void> {
    if (!snapshot || passwordBusy) return;
    const validationError = validatePasswordChange(currentPassword, newPassword, confirmPassword);
    if (validationError) {
      passwordErrorKind = validationError;
      passwordError = passwordValidationText(validationError);
      return;
    }
    const generation = passwordModalGeneration;
    const submissionId = ++passwordSubmissionId;
    const submittedCurrent = currentPassword;
    const submittedNew = newPassword;
    const submittedConfirmation = confirmPassword;
    passwordBusy = true;
    passwordError = '';
    passwordErrorKind = null;
    try {
      const outcome = await executeMutation((revision) => api.changePassword(
        revision,
        submittedCurrent,
        submittedNew,
        submittedConfirmation
      ));
      if (outcome.result) {
        if (passwordModalOpen) closePasswordModal(false);
        else {
          currentPassword = '';
          newPassword = '';
          confirmPassword = '';
        }
        showWorkspaceNotice('success', rt('passwordChanged'), true);
        requestAnimationFrame(() => document.querySelector<HTMLElement>('.cloud-account .cloud-icon-button')?.focus());
      } else if (outcome.error && passwordModalOpen && generation === passwordModalGeneration) {
        passwordError = mutationErrorText(outcome.error);
        passwordErrorKind = null;
      }
    } finally {
      if (submissionId === passwordSubmissionId) passwordBusy = false;
    }
  }

  async function syncNow(): Promise<void> {
    if (!snapshot) return;
    if (await commit((revision) => api.syncNow(revision), 'sync') && snapshot.sync.conflictCount > 0) await openConflicts();
  }

  async function openConflicts(): Promise<void> {
    conflicts = await api.loadConflicts();
    conflictActionError = '';
    conflictOpen = true;
  }

  async function resolveConflict(conflict: SyncConflictView, choice: ConflictResolutionChoice): Promise<void> {
    if (!snapshot || resolvingConflictKey) return;
    const key = `${conflict.recordType}:${conflict.localId}`;
    resolvingConflictKey = key;
    conflictActionError = '';
    try {
      const outcome = await executeMutation((revision) => api.resolveConflict(revision, conflict.recordType, conflict.localId, choice));
      if (outcome.result) {
        conflicts = await api.loadConflicts();
        conflictOpen = conflicts.length > 0;
      } else if (outcome.error) {
        conflictActionError = mutationErrorText(outcome.error);
      }
    } finally {
      if (resolvingConflictKey === key) resolvingConflictKey = null;
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
    if (await commit((revision) => api.attachImage(revision, selectedList.id, selectedTodoId!, selected))) {
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
      showWorkspaceNotice('error', errorText(error));
    }
  }

  function dockAction(action: DockAction, trigger: HTMLElement): void {
    if (action === 'SORT') void toggleSort();
    if (action === 'DEADLINE') void toggleDeadline();
    if (action === 'HIDE_DONE') void toggleHideDone();
    if (action === 'DELETE_DONE') requestDeleteCompleted(trigger);
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

  function priorityLabel(priority: TodoPriority): string {
    return t(priority === 'XHIGH' ? 'priority_xhigh'
      : priority === 'HIGH' ? 'priority_high'
        : priority === 'MEDIUM' ? 'priority_medium'
          : 'priority_low');
  }

  function conflictTitle(conflict: SyncConflictView): string {
    return conflict.recordType === 'settings'
      ? `${t('options')} · ${t('settings_language')}`
      : conflict.title;
  }

  function conflictDescription(conflict: SyncConflictView): string {
    if (conflict.messageCode === 'cloud_record_deleted') return wt('conflictCloudDeleted');
    if (conflict.messageCode === 'local_record_deleted') return wt('conflictLocalDeleted');
    if (conflict.messageCode === 'overlapping_fields_changed' || conflict.messageCode === 'remote_version_changed') return wt('conflictOverlapping');
    return wt('conflictUnknown');
  }

  function keepLocalConflictLabel(conflict: SyncConflictView): string {
    return conflict.recordType === 'checklist' && conflict.messageCode === 'cloud_record_deleted'
      ? wt('keepAsNewChecklist')
      : wt('useThisDevice');
  }

  function keepCloudConflictLabel(conflict: SyncConflictView): string {
    return conflict.messageCode === 'cloud_record_deleted'
      ? wt('acceptCloudDeletion')
      : wt('useCloudVersion');
  }

  function conflictFieldLabel(field: SyncConflictFieldView): string {
    const keys: Record<string, MessageKey> = {
      name: 'field_name', sort_index: 'field_sort', checklist_local_id: 'field_checklist',
      title: 'field_title', priority: 'field_priority', due_at_millis: 'field_due',
      completed: 'field_completed', reminder_repeat: 'field_repeat', language_mode: 'field_language',
      trashed_from_checklist_id: 'field_trash', trashed_from_checklist_name: 'field_trash',
      trashed_at_millis: 'field_trash'
    };
    return t(keys[field.key] ?? 'conflict');
  }

  function conflictValueText(value: SyncConflictValueView): string {
    if (value.kind === 'empty') return wt('emptyValue');
    if (value.kind === 'checklist') return value.label ?? String(value.value);
    if (value.kind === 'position') return wt('conflictPosition').replace('{value}', String(value.value));
    if (value.kind === 'status') return wt(value.value === 'completed' ? 'statusCompleted' : 'statusActive');
    if (value.kind === 'priority') return priorityLabel(String(value.value).toUpperCase() as TodoPriority);
    if (value.kind === 'repeat') {
      const repeat = String(value.value).toUpperCase();
      return t(repeat === 'NONE' ? 'repeat_none' : repeat === 'DAILY' ? 'repeat_daily' : 'repeat_weekly');
    }
    if (value.kind === 'language') {
      const languageKeys: Record<string, MessageKey> = {
        system: 'language_system', en: 'language_english', 'zh-Hans': 'language_chinese',
        ar: 'language_arabic', fr: 'language_french', ru: 'language_russian', es: 'language_spanish'
      };
      return t(languageKeys[String(value.value)] ?? 'language_system');
    }
    if (value.kind === 'timestamp' && typeof value.value === 'number') {
      return new Intl.DateTimeFormat(locale, {
        year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit'
      }).format(value.value);
    }
    return String(value.value);
  }

  function checklistDisplayName(list: Checklist): string {
    if (list.kind === 'SETTINGS') return t('options');
    if (list.kind === 'TRASH') return t('field_trash');
    return list.name;
  }

  function dockSettingsLabel(action: DockAction): string {
    return t(action === 'SORT' ? 'sort'
      : action === 'DEADLINE' ? 'deadline_short'
        : action === 'HIDE_DONE' ? 'hide_done'
          : action === 'DELETE_DONE' ? 'clean_done'
            : 'quick_delete');
  }

  function errorText(error: unknown): string {
    if (typeof error === 'string') return uiText(error);
    if (error && typeof error === 'object' && 'message' in error) return uiText(String(error.message));
    return t('error');
  }

  function handleKeys(event: KeyboardEvent): void {
    if (destructiveConfirmation) return;
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
      void commit((revision) => api.backChecklist(revision));
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
        <span class="section-title">{wt('checklists')}</span>
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
                <span class="nav-name"><ScriptAwareText text={list.name} role="serif" /></span>
              </button>
            {/if}
            {#if editingListId !== list.id}
              <div class="nav-actions">
                <button class="row-more" title={t('edit_list')} aria-label={`${t('edit_list')}: ${list.name}`} onclick={() => beginRename(list)}><Icon name="edit" /></button>
                {#if normalLists.length > 1}<button class="row-delete" title={t('delete_list')} aria-label={`${t('delete_list')}: ${list.name}`} aria-haspopup="dialog" onclick={(event) => requestDeleteChecklist(list, event.currentTarget)}><Icon name="trash" /></button>{/if}
              </div>
            {/if}
          </div>
        {/each}
      </nav>

      <nav class="special-nav" aria-label={t('app_options')}>
        {#each specialLists as list (list.id)}
          <button class:active={list.id === snapshot.selectedChecklistId} class="special-row" onclick={() => void chooseChecklist(list.id)}>
            <span class="nav-icon"><Icon name={list.kind === 'TRASH' ? 'trash' : 'settings'} /></span><span class="special-name"><ScriptAwareText text={checklistDisplayName(list)} role="serif" /></span>
            {#if list.kind === 'TRASH' && list.items.length}<span class="nav-count">{list.items.length}</span>{/if}
          </button>
        {/each}
      </nav>

      <footer class="sidebar-footer">
        <div class="cloud-state">
          <button class="sidebar-account-button" aria-label={t('account')} onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}><Icon name="cloud" /><span><strong>{syncStateLabel()}</strong>{snapshot.auth.userEmail ?? t('signed_out')}</span></button>
          {#if snapshot.auth.signedIn}<button class="sidebar-signout" title={t('sign_out')} aria-label={t('sign_out')} onclick={() => void commit((revision) => api.signOut(revision))}><Icon name="logout" size={20} /></button>{/if}
        </div>
      </footer>
      <div
        use:sidebarResize
        class="sidebar-resizer"
        role="separator"
        aria-label={wt('checklists')}
        aria-orientation="vertical"
        aria-valuemin="200"
        aria-valuemax="720"
        aria-valuenow={sidebarWidth}
      ></div>
    </aside>

    <section class="workspace">
      <header class="workspace-header workspace-status">
        <div class="title-stack status-title">
          <div class="title-line">
            {#if snapshot.checklistHistory.length}<button class="icon-button" title="Alt+Left" onclick={() => void commit((revision) => api.backChecklist(revision))}><Icon name="back" /></button>{/if}
            <h2 dir="auto"><ScriptAwareText text={checklistDisplayName(selectedList)} role="serif" /></h2>
            {#if selectedList.kind === 'NORMAL'}<span class="status-counts"><span class="status-count">{wt('active')} {selectedList.items.filter((item) => !item.completed).length}</span><span class="status-count">{wt('done')} {selectedList.items.filter((item) => item.completed).length}</span></span>{/if}
          </div>
        </div>
        <div class="header-actions status-actions status-signals">
          {#if snapshot.update.state === 'AVAILABLE'}<button class="status-chip status-signal update-chip" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{wt('updateReady')}</button>{/if}
          {#if reminderNeedsAttention()}<button class="status-chip status-signal error" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{wt('notificationIssue')}</button>{/if}
        </div>
      </header>

      {#if snapshot.settings.enhancedXhighAlarmEnabled && snapshot.reminder.activeTodoIds.length}
        <div class="workspace-alert"><strong>{wt('alarmRinging')}</strong><button class="quiet-button" onclick={() => void chooseChecklist(specialLists.find((list) => list.kind === 'SETTINGS')?.id ?? snapshot.selectedChecklistId)}>{t('options')}</button></div>
      {/if}

      {#if workspaceNotice}
        <div class="workspace-alert operation-notice {workspaceNotice.tone}" class:operation-error={workspaceNotice.tone === 'error'} role={workspaceNotice.tone === 'error' ? 'alert' : 'status'} aria-live={workspaceNotice.tone === 'error' ? 'assertive' : 'polite'}>
          <span>{workspaceNotice.message}</span>
          <button class="icon-button" type="button" aria-label={t('close')} onclick={dismissWorkspaceNotice}><Icon name="close" /></button>
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
                <button class="task-open task-copy" onclick={(event) => chooseTodo(item, event.currentTarget)}><strong dir="auto"><ScriptAwareText text={item.title} /></strong>{#if !item.completed}<span class:overdue={item.dueAtMillis > 0 && item.dueAtMillis <= Date.now()}>{#if item.priority === 'XHIGH'}<Icon name="alarm" size={13} /> {/if}{formatDue(item)} · {priorityLabel(item.priority)} · {repeatText(item)}</span>{/if}</button>
                {#if item.imageFileName}<button class="attachment-badge" onclick={(event) => { event.stopPropagation(); void showImagePreview(item.id); }}><Icon name="image" /></button>{/if}
                {#if snapshot.quickDelete}<button class="delete-slot" onclick={(event) => { event.stopPropagation(); selectedTodoId = item.id; void moveSelectedToTrash(); }}>{t('delete')}</button>{/if}
              </article>
            {/each}

            {#if !snapshot.hideCompleted}
              {#each completedItems as item (item.id)}
                <article class:selected={item.id === selectedTodoId} class:highlighted={highlightedTodoId === item.id} class="task-row completed priority-{item.priority.toLowerCase()}">
                  <button class="completion-control checked" aria-label={t('show')} onclick={() => void toggleTodo(item)}><Icon name="check" size={12} /></button>
                  <button class="task-open task-copy" onclick={(event) => chooseTodo(item, event.currentTarget)}><strong dir="auto"><ScriptAwareText text={item.title} /></strong></button>
                  {#if item.imageFileName}<button class="attachment-badge" onclick={() => void showImagePreview(item.id)}><Icon name="image" /></button>{/if}
                  {#if snapshot.quickDelete}<button class="delete-slot" onclick={() => { selectedTodoId = item.id; void moveSelectedToTrash(); }}>{t('delete')}</button>{/if}
                </article>
              {/each}
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
            <div class="trash-toolbar">
              <div class="trash-filters" role="search" aria-label={wt('trashSearchPlaceholder')}>
                <input
                  class="trash-search"
                  data-testid="trash-search"
                  type="search"
                  aria-label={wt('trashSearchPlaceholder')}
                  placeholder={wt('trashSearchPlaceholder')}
                  bind:value={trashSearchQuery}
                />
                <select
                  class="trash-filter"
                  data-testid="trash-priority-filter"
                  aria-label={t('field_priority')}
                  bind:value={trashPriorityFilter}
                >
                  <option value="">{wt('allPriorities')}</option>
                  {#each ['XHIGH', 'HIGH', 'MEDIUM', 'LOW'] as priority}
                    <option value={priority}>{priorityLabel(priority as TodoPriority)}</option>
                  {/each}
                </select>
                <select
                  class="trash-filter"
                  data-testid="trash-checklist-filter"
                  aria-label={t('field_checklist')}
                  bind:value={trashChecklistFilter}
                >
                  <option value="">{wt('allChecklists')}</option>
                  {#each availableTrashSources as checklistName}
                    <option value={checklistName}>{checklistName}</option>
                  {/each}
                </select>
              </div>
              <button
                class="danger-button trash-delete-all"
                data-testid="trash-delete-all"
                aria-haspopup="dialog"
                onclick={(event) => requestPurgeAllTrash(event.currentTarget)}
              >{t('delete_all')}</button>
            </div>
            {#if filteredTrashItems.length === 0}
              <div class="trash-no-results" data-testid="trash-no-results" role="status">{wt('trashNoMatches')}</div>
            {:else}
              {#each filteredTrashItems as item (item.id)}
                <article
                  class="task-row trash-row priority-{item.priority.toLowerCase()}"
                  data-testid="trash-row"
                  data-todo-id={item.id}
                >
                  <div class="task-copy">
                    <strong dir="auto"><ScriptAwareText text={item.title} /></strong>
                    <span class="trash-source"><ScriptAwareText text={trashSourceName(item)} role="serif" /></span>
                  </div>
                  <button
                    class="trash-action trash-restore"
                    title={`${t('restore_task')}: ${item.title}`}
                    aria-label={`${t('restore_task')}: ${item.title}`}
                    onclick={() => void restoreTrashItem(item)}
                  ><Icon name="restore" size={20} /></button>
                  <button
                    class="trash-action trash-delete"
                    title={`${t('delete')}: ${item.title}`}
                    aria-label={`${t('delete')}: ${item.title}`}
                    aria-haspopup="dialog"
                    onclick={(event) => requestPurgeTrashItem(item, event.currentTarget)}
                  ><Icon name="trash" size={20} /></button>
                </article>
              {/each}
            {/if}
          {/if}
        </div>
      {:else}
        <div class="settings-page">
          <div class="settings-grid">
          <section>
            <span class="section-title">{t('settings_cloud')}</span>
            <div class="setting-row cloud-account">
              <div><strong>{t('account')}</strong><p>{snapshot.auth.userEmail ?? t('signed_out')}</p></div>
              <div class="cloud-actions">
                {#if snapshot.auth.signedIn}
                  <button class="cloud-icon-button" title={t('sign_out')} onclick={() => void commit((revision) => api.signOut(revision))}><Icon name="logout" size={22} /></button>
                {:else}
                  <button
                    class="cloud-icon-button"
                    title={t('sign_in')}
                    aria-label={t('sign_in')}
                    aria-haspopup="dialog"
                    aria-expanded={authModalOpen}
                    disabled={authBusy}
                    onclick={(event) => openAuthModal(event.currentTarget)}
                  ><Icon name="login" size={22} /></button>
                {/if}
              </div>
            </div>
            {#if snapshot.auth.signedIn}
              <div class="setting-row">
                <div><strong>{wt('changePassword')}</strong><p>{snapshot.auth.userEmail}</p></div>
                <button class="setting-icon-button password-modal-trigger" title={wt('changePassword')} aria-label={wt('changePassword')} aria-haspopup="dialog" aria-expanded={passwordModalOpen} disabled={passwordBusy} onclick={(event) => openPasswordModal(event.currentTarget)}><Icon name="key" size={20} /></button>
              </div>
            {/if}
            <div class="setting-row sync-setting-row"><div><strong>{t('sync')}</strong><p data-testid="sync-detail">{syncDetailMessage()}</p></div><div class="setting-actions"><span class="setting-value">{snapshot.sync.pendingCount} {t('pending')}</span>{#if snapshot.auth.signedIn}{#if snapshot.sync.conflictCount}<button class="setting-icon-button danger sync-conflict-button" title={`${wt('reviewSyncConflicts')} · ${snapshot.sync.conflictCount}`} aria-label={`${wt('reviewSyncConflicts')} · ${snapshot.sync.conflictCount}`} onclick={() => void openConflicts()}><Icon name="alert" size={20} /></button>{:else}<button class="setting-icon-button sync-now-button" title={t('sync_now')} aria-label={t('sync_now')} aria-busy={snapshot.sync.state === 'SYNCING'} disabled={snapshot.sync.state === 'SYNCING'} onclick={() => void syncNow()}><Icon name="sync" size={20} /></button>{/if}{/if}</div></div>
          </section>

          <section>
            <span class="section-title">{t('settings_display')}</span>
            <div class="setting-row"><div><strong>{t('settings_language')}</strong><p>{languageOptions.find((item) => item.value === snapshot.settings.languageMode)?.label ?? t('language_system')}</p></div></div>
            <div class="language-grid">
              {#each languageOptions as language}
                <button class:selected={snapshot.settings.languageMode === language.value} onclick={() => void setLanguage(language.value)}><span class="pixel-choice"></span><span class="language-label" lang={languageTagFor(language.value)} dir={languageTagFor(language.value) === 'ar' ? 'rtl' : 'ltr'}>{language.label ?? t('language_system')}</span></button>
              {/each}
              {#if languageOptions.length % 2}<span aria-hidden="true"></span>{/if}
            </div>
            <div class="setting-row"><div><strong>{t('settings_theme')}</strong><p>{snapshot.settings.darkTheme ? t('theme_dark') : t('theme_light')}</p></div><button aria-label={t('settings_theme')} class:active={snapshot.settings.darkTheme} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, darkTheme: !snapshot.settings.darkTheme })}><span></span></button></div>
          </section>

          <section>
            <span class="section-title">{t('settings_dock')}</span>
            <div class="segmented-row">{#each ['LEFT_EDGE', 'CENTER', 'RIGHT_EDGE'] as placement}<button class="dock-placement-button" data-placement={placement} class:active={snapshot.settings.dock.plusPlacement === placement} title={t(placement === 'LEFT_EDGE' ? 'left' : placement === 'CENTER' ? 'center' : 'right')} aria-label={t(placement === 'LEFT_EDGE' ? 'left' : placement === 'CENTER' ? 'center' : 'right')} onclick={() => void setDockPlacement(placement as DockPlusPlacement)}><span class="placement-track"><span class="placement-plus">+</span></span></button>{/each}</div>
            <div class="dock-choice-grid">{#each allDockActions as action}<button class:selected={snapshot.settings.dock.actions.includes(action)} title={dockSettingsLabel(action)} aria-label={dockSettingsLabel(action)} aria-pressed={snapshot.settings.dock.actions.includes(action)} onclick={() => void toggleDockAction(action)}><Icon name={action === 'SORT' ? 'sort' : action === 'DEADLINE' ? 'calendar' : action === 'HIDE_DONE' ? 'hide' : action === 'DELETE_DONE' ? 'trash-check' : 'batch-delete'} size={20} active={action === 'SORT' && snapshot.sortMode === 'TIME'} /></button>{/each}</div>
          </section>

          <section>
            <span class="section-title">{t('settings_updates')}</span>
            <div class="setting-row"><div><strong>{wt('automaticUpdateChecks')}</strong><p>{wt('automaticUpdateChecksDetail')}</p></div><button aria-label={wt('automaticUpdateChecks')} class:active={snapshot.settings.automaticUpdateCheckEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, automaticUpdateCheckEnabled: !snapshot.settings.automaticUpdateCheckEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('currentVersion')}</strong><p>{snapshot.update.currentVersion} · FORMAL · x64 NSIS{#if snapshot.update.message} · {uiText(snapshot.update.message)}{/if}</p></div><button class="setting-icon-button" title={t('check_update')} aria-label={t('check_update')} onclick={() => void commit((revision) => api.checkForUpdate(revision), 'update')}><Icon name="update" size={20} /></button></div>
            {#if snapshot.update.state === 'AVAILABLE'}<button class="setting-icon-button primary section-action" title={`${t('install_updates')} ${snapshot.update.availableVersion}`} aria-label={`${t('install_updates')} ${snapshot.update.availableVersion}`} onclick={() => { updateProgress = { downloadedBytes: 0, totalBytes: null }; void api.installUpdate(); }}><Icon name="download" size={20} /></button>{/if}
            {#if updateProgress}<p class="update-progress">{wt('downloading')} {formatBytes(updateProgress.downloadedBytes)}{#if updateProgress.totalBytes} / {formatBytes(updateProgress.totalBytes)}{/if}</p>{/if}
          </section>

          <section>
            <span class="section-title">{wt('windowsIntegration')}</span>
            <div class="setting-row"><div><strong>{wt('startWithWindows')}</strong><p>{wt('startWithWindowsDetail')}</p></div><button aria-label={wt('startWithWindows')} class:active={snapshot.settings.autostartEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, autostartEnabled: !snapshot.settings.autostartEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('enhancedAlarm')}</strong><p>{wt('enhancedAlarmDetail')}</p></div><button aria-label={wt('enhancedAlarm')} class:active={snapshot.settings.enhancedXhighAlarmEnabled} class="switch" onclick={() => void updateSettings({ ...snapshot.settings, enhancedXhighAlarmEnabled: !snapshot.settings.enhancedXhighAlarmEnabled })}><span></span></button></div>
            <div class="setting-row"><div><strong>{wt('reminderQueue')}</strong><p>{snapshot.reminder.scheduledCount} · {snapshot.reminder.state}{#if snapshot.reminder.message} · {uiText(snapshot.reminder.message)}{/if}</p></div></div>
          </section>

          <section>
            <span class="section-title">{wt('storagePrivacy')}</span>
            {#if storageInfo}
              <div class="setting-row storage-row"><div><strong>{wt('application')}</strong><p><code>{storageInfo.executablePath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>{wt('localData')}</strong><p><code>{storageInfo.dataRoot}</code> · {formatBytes(storageInfo.totalBytes)}</p></div><button class="setting-icon-button" title={wt('openFolder')} aria-label={wt('openFolder')} onclick={() => void api.openDataFolder()}><Icon name="folder" size={20} /></button></div>
              <div class="setting-row storage-row"><div><strong>SQLite</strong><p><code>{storageInfo.databasePath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>WebView2</strong><p><code>{storageInfo.webviewDataPath}</code></p></div></div>
              <div class="setting-row storage-row"><div><strong>Windows Credential Manager</strong><p><code>{storageInfo.credentialManagerTarget}</code></p></div></div>
              {#if storageInfo.legacyRoamingDatabasePath}<div class="setting-row storage-row"><div><strong>{wt('legacyDataFound')}</strong><p><code>{storageInfo.legacyRoamingDatabasePath}</code> · {formatBytes(storageInfo.legacyRoamingDatabaseBytes ?? 0)}</p></div><button class="setting-icon-button danger" title={t('delete')} aria-label={t('delete')} aria-haspopup="dialog" onclick={(event) => requestDeleteLegacyData(event.currentTarget)}><Icon name="trash" size={20} /></button></div>{/if}
            {/if}
          </section>
          </div>
        </div>
      {/if}
    </section>

  </main>

  {#if destructiveConfirmation && activeDestructivePolicy}
    <DestructiveActionModal
      {locale}
      context={confirmationContext(activeDestructivePolicy)}
      title={confirmationTitle(activeDestructivePolicy)}
      target={confirmationTarget(activeDestructivePolicy)}
      detail={formatConfirmationMessage(activeDestructivePolicy.detailKey, destructiveConfirmation)}
      busy={destructiveConfirmationBusy}
      confirmLabel={t(activeDestructivePolicy.confirmKey)}
      busyLabel={wt('deleting')}
      onConfirm={confirmDestructiveAction}
      onClose={closeDestructiveConfirmation}
    />
  {/if}

  {#if authModalOpen && !snapshot.auth.signedIn}
    <AuthModal
      {locale}
      mode={authMode}
      email={authEmail}
      password={authPassword}
      busy={authBusy}
      error={authError}
      errorKind={authErrorKind}
      onModeChange={setAuthMode}
      onEmailChange={updateAuthEmail}
      onPasswordChange={updateAuthPassword}
      onSubmit={authAction}
      onClose={closeAuthModal}
    />
  {/if}

  {#if passwordModalOpen && snapshot.auth.signedIn}
    <PasswordModal
      {locale}
      {currentPassword}
      {newPassword}
      confirmation={confirmPassword}
      busy={passwordBusy}
      error={passwordError}
      errorKind={passwordErrorKind}
      onCurrentPasswordChange={(value) => updatePasswordField('current', value)}
      onNewPasswordChange={(value) => updatePasswordField('new', value)}
      onConfirmationChange={(value) => updatePasswordField('confirmation', value)}
      onSubmit={changePassword}
      onClose={closePasswordModal}
    />
  {/if}

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
      onRemoveImage={async () => { if (selectedTodo) await commit((revision) => api.deleteImage(revision, selectedList.id, selectedTodo.id)); }}
      onDelete={requestDeleteSelected}
    />
  {/if}

  {#if conflictOpen}
    <div class="modal-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && resolvingConflictKey === null && (conflictOpen = false)}>
      <div class="conflict-modal" role="dialog" aria-modal="true" aria-labelledby="conflict-dialog-title" aria-busy={resolvingConflictKey !== null} tabindex="-1">
        <header><h2 id="conflict-dialog-title">{t('sync_conflicts')}</h2><button class="icon-button" aria-label={t('close')} disabled={resolvingConflictKey !== null} onclick={() => (conflictOpen = false)}><Icon name="close" /></button></header>
        {#if conflicts.length === 0}<p>{t('no_conflicts')}</p>{/if}
        {#if conflictActionError}<p class="conflict-action-error" role="alert">{conflictActionError}</p>{/if}
        {#each conflicts as conflict}
          <article class="conflict-card">
            <h3><ScriptAwareText text={conflictTitle(conflict)} /></h3>
            <p class="conflict-explanation">{conflictDescription(conflict)}</p>
            <div class="conflict-table">
              <div class="conflict-table-head"><span></span><strong>{wt('thisDevice')}</strong><strong>{wt('cloudVersion')}</strong></div>
              {#each conflict.fields as field}
                <div class="conflict-field-row"><strong>{conflictFieldLabel(field)}</strong><span><ScriptAwareText text={conflictValueText(field.localValue)} /></span><span><ScriptAwareText text={conflictValueText(field.cloudValue)} /></span></div>
              {/each}
            </div>
            <div class="form-actions"><button class="quiet-button" disabled={resolvingConflictKey !== null} onclick={() => void resolveConflict(conflict, 'KEEP_LOCAL')}>{resolvingConflictKey === `${conflict.recordType}:${conflict.localId}` ? wt('resolvingConflict') : keepLocalConflictLabel(conflict)}</button><button class="primary-button" disabled={resolvingConflictKey !== null} onclick={() => void resolveConflict(conflict, 'KEEP_CLOUD')}>{resolvingConflictKey === `${conflict.recordType}:${conflict.localId}` ? wt('resolvingConflict') : keepCloudConflictLabel(conflict)}</button></div>
          </article>
        {/each}
      </div>
    </div>
  {/if}

  {#if previewData}
    <div class="modal-backdrop image-backdrop" role="presentation" onclick={() => (previewData = null)} onwheel={(event) => { event.preventDefault(); previewZoom = Math.min(5, Math.max(.25, previewZoom + (event.deltaY < 0 ? .15 : -.15))); }} onpointermove={(event) => { if (draggingPreview) { previewX = dragStart.offsetX + event.clientX - dragStart.x; previewY = dragStart.offsetY + event.clientY - dragStart.y; } }} onpointerup={() => (draggingPreview = false)}>
      <button class="icon-button preview-close" onclick={() => (previewData = null)}><Icon name="close" /></button>
      <img src={previewData} alt={t('task_image_preview')} style:transform={`translate(${previewX}px, ${previewY}px) scale(${previewZoom})`} onpointerdown={(event) => { event.stopPropagation(); draggingPreview = true; dragStart = { x: event.clientX, y: event.clientY, offsetX: previewX, offsetY: previewY }; }} ondblclick={(event) => { event.stopPropagation(); previewZoom = 1; previewX = 0; previewY = 0; }} />
    </div>
  {/if}
{:else}
  <main class="launch-state error"><span class="launch-mark">!</span><p>{launchErrorMessage || 'PixelDone'}</p></main>
{/if}
