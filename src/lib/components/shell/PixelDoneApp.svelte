<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import type {
    AppError,
    AppSettings,
    AppSnapshot,
    Checklist,
    MutationResult,
    SortMode,
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

  let selectedList = $derived(
    snapshot?.checklists.find((list) => list.id === snapshot?.selectedChecklistId) ?? null
  );
  let selectedTodo = $derived(
    selectedList?.items.find((item) => item.id === selectedTodoId) ?? null
  );
  let normalLists = $derived(snapshot?.checklists.filter((list) => list.kind === 'NORMAL') ?? []);
  let specialLists = $derived(snapshot?.checklists.filter((list) => list.kind !== 'NORMAL') ?? []);
  let displayItems = $derived.by(() => {
    if (!selectedList || !snapshot) return [];
    const items = selectedList.items.filter((item) => !snapshot?.hideCompleted || !item.completed);
    return [...items].sort((left, right) => compareTodo(left, right, snapshot.sortMode));
  });

  onMount(async () => {
    try {
      snapshot = await api.bootstrap();
      document.documentElement.dataset.theme = snapshot.settings.darkTheme ? 'dark' : 'light';
    } catch (error) {
      errorMessage = errorText(error);
    } finally {
      loading = false;
    }
  });

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

  async function commit(promise: Promise<MutationResult>): Promise<void> {
    if (!snapshot) return;
    errorMessage = '';
    try {
      const result = await promise;
      snapshot = applyMutation(snapshot, result);
    } catch (error) {
      const appError = error as Partial<AppError>;
      if (appError.code === 'STALE_REVISION') snapshot = await api.bootstrap();
      errorMessage = errorText(error);
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
    if (isNewTodo) {
      await commit(api.createTodo(snapshot.revision, selectedList.id, { ...draft }));
    } else if (selectedTodoId) {
      await commit(api.updateTodo(snapshot.revision, selectedList.id, selectedTodoId, { ...draft }));
    }
    if (!errorMessage) {
      isNewTodo = false;
      selectedTodoId = null;
      inspectorOpen = false;
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
    }
    await commit(api.toggleTodo(snapshot.revision, selectedList.id, item.id));
  }

  async function moveSelectedToTrash(): Promise<void> {
    if (!snapshot || !selectedList || !selectedTodoId) return;
    await commit(api.moveTodoToTrash(snapshot.revision, selectedList.id, selectedTodoId));
    selectedTodoId = null;
    inspectorOpen = false;
  }

  async function submitNewList(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!snapshot || !newListName.trim()) return;
    await commit(api.createChecklist(snapshot.revision, newListName));
    newListName = '';
    creatingList = false;
  }

  function beginRename(list: Checklist): void {
    editingListId = list.id;
    listNameDraft = list.name;
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>('#rename-list')?.select());
  }

  async function saveListName(): Promise<void> {
    if (!snapshot || !editingListId) return;
    await commit(api.renameChecklist(snapshot.revision, editingListId, listNameDraft));
    editingListId = null;
  }

  async function deleteList(list: Checklist): Promise<void> {
    if (!snapshot || !confirm(`删除清单 ${list.name}？任务将移入回收站。`)) return;
    await commit(api.deleteChecklist(snapshot.revision, list.id));
  }

  async function toggleSort(): Promise<void> {
    if (!snapshot) return;
    await commit(api.setSortMode(snapshot.revision, snapshot.sortMode === 'PRIORITY' ? 'TIME' : 'PRIORITY'));
  }

  async function toggleHideDone(): Promise<void> {
    if (!snapshot) return;
    await commit(api.setHideCompleted(snapshot.revision, !snapshot.hideCompleted));
  }

  async function toggleQuickDelete(): Promise<void> {
    if (!snapshot) return;
    await commit(api.setQuickDelete(snapshot.revision, !snapshot.quickDelete));
  }

  async function cleanCompleted(): Promise<void> {
    if (!snapshot || selectedList?.kind !== 'NORMAL') return;
    await commit(api.cleanCompleted(snapshot.revision, selectedList.id));
  }

  async function setDarkTheme(darkTheme: boolean): Promise<void> {
    if (!snapshot) return;
    const settings: AppSettings = { ...snapshot.settings, darkTheme };
    await commit(api.updateSettings(snapshot.revision, settings));
    document.documentElement.dataset.theme = darkTheme ? 'dark' : 'light';
  }

  async function trashAction(item: TodoItem, action: 'restore' | 'purge'): Promise<void> {
    if (!snapshot) return;
    await commit(
      action === 'restore'
        ? api.restoreTodo(snapshot.revision, item.id)
        : api.purgeTodo(snapshot.revision, item.id)
    );
    selectedTodoId = null;
  }

  function errorText(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error && typeof error === 'object' && 'message' in error) return String(error.message);
    return '操作失败，请重试';
  }

  function formatDue(millis: number): string {
    if (millis <= 0) return '无截止时间';
    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(millis);
  }
</script>

<svelte:head><title>PixelDone</title></svelte:head>

{#if loading}
  <main class="launch-state"><span class="launch-mark">PD</span><p>正在打开本地任务库…</p></main>
{:else if snapshot && selectedList}
  <main class:dark={snapshot.settings.darkTheme} class="app-shell">
    <aside class="sidebar" aria-label="清单导航">
      <div class="brand-row">
        <div><span class="eyebrow">PIXEL UTILITY</span><h1>PixelDone</h1></div>
        <button class="icon-button" title="新建清单" onclick={() => (creatingList = !creatingList)}><Icon name="plus" /></button>
      </div>

      <nav class="list-nav" aria-label="普通清单">
        <span class="section-label">清单</span>
        {#each normalLists as list (list.id)}
          <div class:active={list.id === snapshot.selectedChecklistId} class="nav-row">
            <button
              class="nav-main"
              oncontextmenu={(event) => { event.preventDefault(); beginRename(list); }}
              onclick={() => void chooseChecklist(list.id)}
            >
              <span class="nav-icon"><Icon name="list" /></span>
              {#if editingListId === list.id}
                <input
                  id="rename-list"
                  aria-label="清单名称"
                  bind:value={listNameDraft}
                  onblur={() => void saveListName()}
                  onkeydown={(event) => event.key === 'Enter' && void saveListName()}
                  onclick={(event) => event.stopPropagation()}
                />
              {:else}
                <span class="nav-name">{list.name}</span><span class="nav-count">{list.items.filter((item) => !item.completed).length}</span>
              {/if}
            </button>
            {#if editingListId !== list.id}
              <button class="row-more" title="清单菜单" onclick={() => beginRename(list)}><Icon name="more" /></button>
            {/if}
          </div>
        {/each}
        {#if creatingList}
          <form class="inline-create" onsubmit={submitNewList}>
            <input aria-label="新清单名称" placeholder="NEW LIST" bind:value={newListName} />
            <button type="submit">添加</button>
          </form>
        {/if}
      </nav>

      <nav class="special-nav" aria-label="特殊页面">
        {#each specialLists as list (list.id)}
          <button class:active={list.id === snapshot.selectedChecklistId} class="special-row" onclick={() => void chooseChecklist(list.id)}>
            <span class="nav-icon"><Icon name={list.kind === 'TRASH' ? 'trash' : 'settings'} /></span>
            <span>{list.name}</span>
            {#if list.kind === 'TRASH' && list.items.length}<span class="nav-count">{list.items.length}</span>{/if}
          </button>
        {/each}
      </nav>

      <div class="cloud-state"><Icon name="cloud" /><span><strong>LOCAL</strong> 云同步协议待升级</span></div>
    </aside>

    <section class="workspace">
      <header class="workspace-header">
        <div class="title-stack">
          <span class="eyebrow">{selectedList.kind === 'NORMAL' ? 'CHECKLIST' : 'SYSTEM'}</span>
          <div class="title-line"><h2>{selectedList.name}</h2><span>{selectedList.items.filter((item) => !item.completed).length} active</span></div>
        </div>
        <div class="header-actions">
          {#if selectedList.kind === 'NORMAL'}
            <button class="quiet-button" onclick={() => void toggleSort()}>{snapshot.sortMode === 'PRIORITY' ? 'PRIORITY' : 'TIME'} SORT</button>
          {/if}
          <button class="icon-button" title="更多"><Icon name="more" /></button>
        </div>
      </header>

      {#if selectedList.kind === 'NORMAL'}
        <div class="task-list" aria-label={`${selectedList.name} 任务`}>
          {#if displayItems.length === 0}
            <div class="empty-state"><span class="empty-glyph">□</span><h3>这里很安静</h3><p>创建一个任务，或关闭 HIDE DONE 查看已完成项。</p><button class="primary-button" onclick={beginNewTodo}>新建任务</button></div>
          {:else}
            {#each displayItems as item (item.id)}
              <div
                role="button"
                class:completed={item.completed}
                class:selected={item.id === selectedTodoId}
                class:held={completionHold[item.id]}
                class="task-row priority-{item.priority.toLowerCase()}"
                onclick={() => chooseTodo(item)}
                onkeydown={(event) => event.key === 'Enter' && chooseTodo(item)}
                tabindex="0"
              >
                <button class="completion-control" class:checked={item.completed} aria-label={item.completed ? '重新激活任务' : '完成任务'} onclick={(event) => { event.stopPropagation(); void toggleTodo(item); }}>
                  {#if item.completed}<Icon name="check" size={12} />{/if}
                </button>
                <div class="task-copy">
                  <strong>{item.title}</strong>
                  <span>{formatDue(item.dueAtMillis)} · {item.priority} · {repeatLabel(item.reminderRepeat)}</span>
                </div>
                {#if item.imageFileName}<span class="attachment-badge">IMG</span>{/if}
                {#if snapshot.quickDelete}
                  <button class="delete-slot" onclick={(event) => { event.stopPropagation(); selectedTodoId = item.id; void moveSelectedToTrash(); }}>DELETE</button>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

        <div class="dock" data-placement={snapshot.settings.dock.plusPlacement}>
          <button class:active={snapshot.sortMode === 'TIME'} onclick={() => void toggleSort()}>SORT</button>
          <button class:active={snapshot.hideCompleted} onclick={() => void toggleHideDone()}>HIDE DONE</button>
          <button class="dock-add" title="新建任务" onclick={beginNewTodo}><Icon name="plus" size={18} /></button>
          <button onclick={() => void cleanCompleted()}>CLEAN DONE</button>
          <button class:active={snapshot.quickDelete} onclick={() => void toggleQuickDelete()}>QUICK DELETE</button>
        </div>
      {:else if selectedList.kind === 'TRASH'}
        <div class="task-list trash-list">
          {#if selectedList.items.length === 0}
            <div class="empty-state"><span class="empty-glyph">×</span><h3>回收站为空</h3><p>软删除任务会保留原清单来源，可随时恢复。</p></div>
          {:else}
            {#each selectedList.items as item (item.id)}
              <article class="task-row trash-row">
                <div class="task-copy"><strong>{item.title}</strong><span>来自 {item.trashedFromChecklistName ?? 'MAIN'} · {formatDue(item.trashedAtMillis ?? 0)}</span></div>
                <button class="quiet-button" onclick={() => void trashAction(item, 'restore')}>RESTORE</button>
                <button class="danger-button" onclick={() => void trashAction(item, 'purge')}>DELETE</button>
              </article>
            {/each}
          {/if}
        </div>
      {:else}
        <div class="settings-page">
          <section><span class="section-label">APPEARANCE</span><div class="setting-row"><div><strong>深色主题</strong><p>使用完整的 PixelDone 深色语义 Token。</p></div><button aria-label="切换深色主题" class:active={snapshot.settings.darkTheme} class="switch" onclick={() => void setDarkTheme(!snapshot.settings.darkTheme)}><span></span></button></div></section>
          <section><span class="section-label">DOCK</span><div class="setting-row"><div><strong>底部工具栏</strong><p>保留 Android Dock 语义；最多四个动作。</p></div><span class="setting-value">{snapshot.settings.dock.plusPlacement}</span></div></section>
          <section><span class="section-label">CLOUD</span><div class="setting-row"><div><strong>Supabase 同步</strong><p>CAS、原子 mutation RPC 与 Storage 完成前保持关闭。</p></div><span class="status-pill blocked">BLOCKED</span></div></section>
          <section><span class="section-label">UPDATES</span><div class="setting-row"><div><strong>更新渠道</strong><p>GitHub 权威源，Gitee 签名镜像；stable 与 beta 隔离。</p></div><span class="setting-value">BETA / RC</span></div></section>
        </div>
      {/if}
    </section>

    <aside class:open={inspectorOpen || selectedList.kind !== 'NORMAL'} class="inspector" aria-label="任务详情">
      <div class="inspector-head"><div><span class="eyebrow">INSPECTOR</span><h2>{isNewTodo ? 'NEW TASK' : selectedTodo?.title ?? selectedList.name}</h2></div><button class="icon-button inspector-close" title="关闭详情" onclick={() => (inspectorOpen = false)}><Icon name="close" /></button></div>
      {#if selectedList.kind === 'NORMAL' && (selectedTodo || isNewTodo)}
        <form class="inspector-form" onsubmit={(event) => { event.preventDefault(); void saveTodo(); }}>
          <label>任务标题<input id="todo-title" bind:value={draft.title} placeholder="输入任务" /></label>
          <fieldset><legend>优先级</legend><div class="priority-segments">
            {#each ['XHIGH', 'HIGH', 'MEDIUM', 'LOW'] as priority}
              <button type="button" class:active={draft.priority === priority} class="priority-{priority.toLowerCase()}" onclick={() => (draft.priority = priority as TodoPriority)}>{priority}</button>
            {/each}
          </div></fieldset>
          <label>日期与时间<input type="datetime-local" value={dateTimeLocalValue(draft.dueAtMillis)} onchange={(event) => (draft.dueAtMillis = millisFromDateTimeLocal(event.currentTarget.value))} /></label>
          <label>重复提醒<select bind:value={draft.reminderRepeat}><option value="NONE">NONE</option><option value="DAILY">DAILY</option><option value="WEEKLY">WEEKLY</option></select></label>
          <div class="attachment-panel"><span class="section-label">IMAGE</span><div class="image-placeholder">单张图片支持将在 Storage parity 完成后解锁</div></div>
          <div class="form-actions"><button type="submit" class="primary-button">SAVE</button>{#if selectedTodo}<button type="button" class="danger-button" onclick={() => void moveSelectedToTrash()}>MOVE TO TRASH</button>{/if}</div>
        </form>
      {:else}
        <div class="inspector-summary"><span class="summary-number">{selectedList.items.length}</span><p>项目总数</p><dl><div><dt>ACTIVE</dt><dd>{selectedList.items.filter((item) => !item.completed).length}</dd></div><div><dt>COMPLETED</dt><dd>{selectedList.items.filter((item) => item.completed).length}</dd></div></dl>{#if selectedList.kind === 'NORMAL'}<button class="primary-button" onclick={beginNewTodo}>新建任务</button>{/if}<div class="shortcut-card"><span class="section-label">SHORTCUTS</span><p><kbd>Ctrl</kbd> + <kbd>N</kbd> 新建任务</p><p><kbd>Esc</kbd> 关闭详情</p></div></div>
      {/if}
    </aside>

    <footer class="status-strip"><span class="status-ready">● LOCAL READY</span><span>REV {snapshot.revision}</span><span>REMINDER IDLE</span><span>UPDATE RC</span>{#if errorMessage}<strong class="status-error">{errorMessage}</strong>{/if}</footer>
  </main>
{:else}
  <main class="launch-state error"><span class="launch-mark">!</span><p>{errorMessage || '无法载入 PixelDone'}</p></main>
{/if}
