<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import type { Locale, MessageKey } from '$lib/generated/i18n';
  import { uiMessage, uiWindowsMessage } from '$lib/i18n/presentation';
  import type { WindowsMessageKey } from '$lib/i18n/windows';
  import type { PasswordValidationError } from './password';

  let {
    locale,
    currentPassword,
    newPassword,
    confirmation,
    busy,
    error,
    errorKind,
    onCurrentPasswordChange,
    onNewPasswordChange,
    onConfirmationChange,
    onSubmit,
    onClose
  }: {
    locale: Locale;
    currentPassword: string;
    newPassword: string;
    confirmation: string;
    busy: boolean;
    error: string;
    errorKind: PasswordValidationError | null;
    onCurrentPasswordChange: (value: string) => void;
    onNewPasswordChange: (value: string) => void;
    onConfirmationChange: (value: string) => void;
    onSubmit: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  let dialog: HTMLElement;
  let currentInput: HTMLInputElement;
  let currentVisible = $state(false);
  let newVisible = $state(false);
  let confirmationVisible = $state(false);

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
  }

  function wt(key: WindowsMessageKey): string {
    return uiWindowsMessage(locale, key);
  }

  function handleDialogKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key !== 'Tab') return;
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])'
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
    onClose();
  }

  onMount(() => currentInput.focus());
</script>

<svelte:window onkeydown={handleWindowKey} />

<div class="modal-backdrop password-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={t('close')} onclick={onClose}></button>
  <div
    bind:this={dialog}
    class="auth-modal password-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="password-dialog-title"
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header auth-modal-header password-modal-header">
      <div>
        <span class="auth-context">{t('account')}</span>
        <h2 id="password-dialog-title">{wt('changePassword')}</h2>
      </div>
      <button class="icon-button" type="button" title={t('close')} aria-label={t('close')} onclick={onClose}><Icon name="close" /></button>
    </header>

    <form class="auth-modal-form password-modal-form" novalidate onsubmit={(event) => { event.preventDefault(); void onSubmit(); }}>
      <div class="auth-field">
        <label for="password-current">{wt('currentPassword')}</label>
        <span class="auth-password-field">
          <input
            bind:this={currentInput}
            id="password-current"
            type={currentVisible ? 'text' : 'password'}
            autocomplete="current-password"
            value={currentPassword}
            aria-invalid={errorKind === 'required'}
            disabled={busy}
            oninput={(event) => onCurrentPasswordChange(event.currentTarget.value)}
          />
          <button class="auth-password-toggle" type="button" disabled={busy} aria-label={currentVisible ? t('hide') : t('show')} onclick={() => (currentVisible = !currentVisible)}>{currentVisible ? t('hide') : t('show')}</button>
        </span>
      </div>

      <div class="auth-field">
        <label for="password-new">{wt('newPassword')}</label>
        <span class="auth-password-field">
          <input
            id="password-new"
            type={newVisible ? 'text' : 'password'}
            autocomplete="new-password"
            value={newPassword}
            aria-invalid={errorKind === 'required' || errorKind === 'password-short' || errorKind === 'same-password'}
            disabled={busy}
            oninput={(event) => onNewPasswordChange(event.currentTarget.value)}
          />
          <button class="auth-password-toggle" type="button" disabled={busy} aria-label={newVisible ? t('hide') : t('show')} onclick={() => (newVisible = !newVisible)}>{newVisible ? t('hide') : t('show')}</button>
        </span>
      </div>

      <div class="auth-field">
        <label for="password-confirmation">{wt('confirmPassword')}</label>
        <span class="auth-password-field">
          <input
            id="password-confirmation"
            type={confirmationVisible ? 'text' : 'password'}
            autocomplete="new-password"
            value={confirmation}
            aria-invalid={errorKind === 'required' || errorKind === 'confirmation-mismatch'}
            disabled={busy}
            oninput={(event) => onConfirmationChange(event.currentTarget.value)}
          />
          <button class="auth-password-toggle" type="button" disabled={busy} aria-label={confirmationVisible ? t('hide') : t('show')} onclick={() => (confirmationVisible = !confirmationVisible)}>{confirmationVisible ? t('hide') : t('show')}</button>
        </span>
      </div>

      {#if error}
        <p class="auth-error password-error" role="alert" aria-live="assertive">{error}</p>
      {/if}

      <button class="primary-button auth-submit password-submit" type="submit" disabled={busy}>
        {busy ? wt('changingPassword') : wt('changePassword')}
      </button>
    </form>
  </div>
</div>
