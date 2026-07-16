<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import ScriptAwareText from '$lib/components/common/ScriptAwareText.svelte';
  import type { Locale, MessageKey } from '$lib/generated/i18n';
  import { uiMessage, uiWindowsMessage } from '$lib/i18n/presentation';
  import type { WindowsMessageKey } from '$lib/i18n/windows';

  let {
    locale,
    checklistName,
    busy,
    onConfirm,
    onClose
  }: {
    locale: Locale;
    checklistName: string;
    busy: boolean;
    onConfirm: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  let dialog: HTMLElement;
  let cancelButton: HTMLButtonElement;

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
  }

  function wt(key: WindowsMessageKey): string {
    return uiWindowsMessage(locale, key);
  }

  function close(): void {
    if (!busy) onClose();
  }

  function handleDialogKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    if (event.key !== 'Tab') return;
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [tabindex]:not([tabindex="-1"])'
    ));
    if (!focusable.length) return;
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

  function handleWindowKey(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || event.defaultPrevented) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    close();
  }

  onMount(() => cancelButton.focus());
</script>

<svelte:window onkeydown={handleWindowKey} />

<div class="modal-backdrop delete-checklist-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={t('close')} disabled={busy} onclick={close}></button>
  <div
    bind:this={dialog}
    class="auth-modal delete-checklist-modal"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="delete-checklist-title"
    aria-describedby="delete-checklist-detail"
    aria-busy={busy}
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header auth-modal-header">
      <div>
        <span class="auth-context">{wt('checklists')}</span>
        <h2 id="delete-checklist-title">{t('delete_list_title')}</h2>
      </div>
      <button class="icon-button" type="button" title={t('close')} aria-label={t('close')} disabled={busy} onclick={close}><Icon name="close" /></button>
    </header>

    <div class="delete-checklist-content">
      <div class="delete-checklist-name" dir="auto"><ScriptAwareText text={checklistName} role="serif" /></div>
      <p id="delete-checklist-detail">{wt('deleteChecklistDetail')}</p>
      <div class="form-actions delete-checklist-actions">
        <button bind:this={cancelButton} class="quiet-button" type="button" disabled={busy} onclick={close}>{t('cancel')}</button>
        <button class="danger-button delete-checklist-confirm" type="button" disabled={busy} onclick={() => void onConfirm()}>{busy ? wt('deletingChecklist') : t('delete')}</button>
      </div>
    </div>
  </div>
</div>
