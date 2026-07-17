<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import ScriptAwareText from '$lib/components/common/ScriptAwareText.svelte';
  import type { Locale, MessageKey } from '$lib/generated/i18n';
  import { uiMessage } from '$lib/i18n/presentation';

  let {
    locale,
    context,
    title,
    target,
    detail,
    busy,
    confirmLabel,
    busyLabel,
    onConfirm,
    onClose
  }: {
    locale: Locale;
    context: string;
    title: string;
    target: string;
    detail: string;
    busy: boolean;
    confirmLabel: string;
    busyLabel: string;
    onConfirm: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  let dialog: HTMLElement;
  let cancelButton: HTMLButtonElement;

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
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

<div class="modal-backdrop destructive-confirmation-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={t('close')} disabled={busy} onclick={close}></button>
  <div
    bind:this={dialog}
    class="auth-modal destructive-confirmation-modal"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="destructive-confirmation-title"
    aria-describedby="destructive-confirmation-detail"
    aria-busy={busy}
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header auth-modal-header">
      <div>
        <span class="auth-context">{context}</span>
        <h2 id="destructive-confirmation-title">{title}</h2>
      </div>
      <button class="icon-button" type="button" title={t('close')} aria-label={t('close')} disabled={busy} onclick={close}><Icon name="close" /></button>
    </header>

    <div class="destructive-confirmation-content">
      <div class="destructive-confirmation-target" dir="auto"><ScriptAwareText text={target} role="serif" /></div>
      <p id="destructive-confirmation-detail">{detail}</p>
      <div class="form-actions destructive-confirmation-actions">
        <button bind:this={cancelButton} class="quiet-button" type="button" disabled={busy} onclick={close}>{t('cancel')}</button>
        <button class="danger-button destructive-confirmation-confirm" type="button" disabled={busy} onclick={() => void onConfirm()}>{busy ? busyLabel : confirmLabel}</button>
      </div>
    </div>
  </div>
</div>
