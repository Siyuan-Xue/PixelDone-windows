<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';

  let {
    title,
    detail,
    detailedLabel,
    simpleLabel,
    closeLabel,
    onDetailed,
    onSimple,
    onClose
  }: {
    title: string;
    detail: string;
    detailedLabel: string;
    simpleLabel: string;
    closeLabel: string;
    onDetailed: () => void | Promise<void>;
    onSimple: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  let dialog: HTMLElement;
  let detailedButton: HTMLButtonElement;

  function handleDialogKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onClose();
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

  onMount(() => detailedButton.focus());
</script>

<div class="modal-backdrop export-markdown-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={closeLabel} onclick={onClose}></button>
  <div
    bind:this={dialog}
    class="auth-modal export-markdown-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="export-markdown-title"
    aria-describedby="export-markdown-detail"
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header">
      <h2 id="export-markdown-title">{title}</h2>
      <button class="icon-button" type="button" title={closeLabel} aria-label={closeLabel} onclick={onClose}><Icon name="close" /></button>
    </header>
    <div class="export-markdown-content">
      <p id="export-markdown-detail">{detail}</p>
      <div class="form-actions export-markdown-actions">
        <button class="quiet-button" type="button" onclick={() => void onSimple()}>{simpleLabel}</button>
        <button bind:this={detailedButton} class="primary-button" type="button" onclick={() => void onDetailed()}>{detailedLabel}</button>
      </div>
    </div>
  </div>
</div>
