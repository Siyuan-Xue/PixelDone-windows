<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import ScriptAwareText from '$lib/components/common/ScriptAwareText.svelte';
  import { type Locale, type MessageKey } from '$lib/generated/i18n';
  import { uiMessage } from '$lib/i18n/presentation';
  import type { TodoDraft, TodoItem, TodoPriority } from '$lib/generated/ipc';
  import { dateTimeLocalValue, millisFromDateTimeLocal } from '$lib/ipc/client';
  import type { TodoEditorMode } from './editor';

  let {
    mode,
    todo,
    draft,
    locale,
    onDraft,
    onSave,
    onClose,
    onChooseImage,
    onPreviewImage,
    onRemoveImage,
    onDelete
  }: {
    mode: Exclude<TodoEditorMode, { kind: 'closed' }>;
    todo: TodoItem | null;
    draft: TodoDraft;
    locale: Locale;
    onDraft: (draft: TodoDraft) => void;
    onSave: () => void | Promise<void>;
    onClose: () => void;
    onChooseImage: () => void | Promise<void>;
    onPreviewImage: () => void | Promise<void>;
    onRemoveImage: () => void | Promise<void>;
    onDelete: (trigger: HTMLElement) => void | Promise<void>;
  } = $props();

  let dialog: HTMLElement;
  let form: HTMLFormElement;
  let titleInput: HTMLInputElement;

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
  }

  function update(patch: Partial<TodoDraft>): void {
    onDraft({ ...draft, ...patch });
  }

  function priorityLabel(priority: TodoPriority): string {
    return t(priority === 'XHIGH' ? 'priority_xhigh' : priority === 'HIGH' ? 'priority_high' : priority === 'MEDIUM' ? 'priority_medium' : 'priority_low');
  }

  function handleDialogKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.ctrlKey && event.key === 'Enter') {
      event.preventDefault();
      form.requestSubmit();
      return;
    }
    if (event.key !== 'Tab') return;
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
    ));
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  onMount(() => titleInput.focus());
</script>

<div class="modal-backdrop editor-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={t('close')} onclick={onClose}></button>
  <div
    bind:this={dialog}
    class="todo-editor-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="todo-editor-title"
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header">
      <div>
        <span class="eyebrow">{mode.kind === 'new' ? t('new_task') : t('edit_task')}</span>
        <h2 id="todo-editor-title"><ScriptAwareText text={todo?.title ?? t('new_task')} role="serif" /></h2>
      </div>
      <button class="icon-button" type="button" title={t('close')} aria-label={t('close')} onclick={onClose}><Icon name="close" /></button>
    </header>

    <form bind:this={form} class="todo-editor-form" onsubmit={(event) => { event.preventDefault(); void onSave(); }}>
      <label>
        <span>{t('name')}</span>
        <input bind:this={titleInput} id="todo-title" dir="auto" value={draft.title} placeholder={t('new_task')} oninput={(event) => update({ title: event.currentTarget.value })} />
      </label>

      <fieldset>
        <legend>{t('field_priority')}</legend>
        <div class="priority-segments">
          {#each ['LOW', 'MEDIUM', 'HIGH', 'XHIGH'] as priority}
            <button
              type="button"
              class:active={draft.priority === priority}
              class="priority-{priority.toLowerCase()}"
              aria-pressed={draft.priority === priority}
              onclick={() => update({ priority: priority as TodoPriority })}
            >{priorityLabel(priority as TodoPriority)}</button>
          {/each}
        </div>
      </fieldset>

      <div class="editor-field-grid">
        <label>
          <span>{t('time_alarm')}</span>
          <input type="datetime-local" value={dateTimeLocalValue(draft.dueAtMillis)} onchange={(event) => update({ dueAtMillis: millisFromDateTimeLocal(event.currentTarget.value) })} />
        </label>
        <label>
          <span>{t('field_repeat')}</span>
          <select value={draft.reminderRepeat} onchange={(event) => update({ reminderRepeat: event.currentTarget.value as TodoDraft['reminderRepeat'] })}>
            <option value="NONE">{t('repeat_none')}</option>
            <option value="DAILY">{t('repeat_daily')}</option>
            <option value="WEEKLY">{t('repeat_weekly')}</option>
          </select>
        </label>
      </div>

      {#if todo}
        <section class="attachment-panel">
          <span class="section-label">{t('task_image')}</span>
          <div class="image-actions">
            <button type="button" class="quiet-button" onclick={() => void onChooseImage()}>{todo.imageFileName ? t('change') : t('add')}</button>
            {#if todo.imageFileName}
              <button type="button" class="quiet-button" onclick={() => void onPreviewImage()}>{t('preview')}</button>
              <button type="button" class="danger-button" onclick={() => void onRemoveImage()}>{t('remove')}</button>
            {/if}
          </div>
        </section>
      {/if}

      <footer class="modal-actions">
        {#if todo}<button type="button" class="danger-button editor-delete" aria-haspopup="dialog" onclick={(event) => void onDelete(event.currentTarget)}>{t('delete_task')}</button>{/if}
        <span class="keyboard-hint"><kbd>Ctrl</kbd> + <kbd>Enter</kbd></span>
        <button type="button" class="quiet-button" onclick={onClose}>{t('cancel')}</button>
        <button type="submit" class="primary-button">{t('save')}</button>
      </footer>
    </form>
  </div>
</div>
