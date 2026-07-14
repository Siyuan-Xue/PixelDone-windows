<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import type { Locale, MessageKey } from '$lib/generated/i18n';
  import { uiMessage } from '$lib/i18n/presentation';
  import type { AuthMode, AuthValidationError } from './auth';
  import { passwordAutocomplete } from './auth';

  let {
    locale,
    mode,
    email,
    password,
    busy,
    error,
    errorKind,
    onModeChange,
    onEmailChange,
    onPasswordChange,
    onSubmit,
    onClose
  }: {
    locale: Locale;
    mode: AuthMode;
    email: string;
    password: string;
    busy: boolean;
    error: string;
    errorKind: AuthValidationError | null;
    onModeChange: (mode: AuthMode) => void;
    onEmailChange: (value: string) => void;
    onPasswordChange: (value: string) => void;
    onSubmit: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  let dialog: HTMLElement;
  let emailInput: HTMLInputElement;
  let passwordVisible = $state(false);

  function t(key: MessageKey): string {
    return uiMessage(locale, key);
  }

  function changeMode(nextMode: AuthMode): void {
    if (busy || nextMode === mode) return;
    passwordVisible = false;
    onModeChange(nextMode);
  }

  function handleModeKey(event: KeyboardEvent): void {
    if (busy) return;
    if (event.key === 'Home') {
      event.preventDefault();
      changeMode('sign-in');
    } else if (event.key === 'End') {
      event.preventDefault();
      changeMode('sign-up');
    } else if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault();
      changeMode(mode === 'sign-in' ? 'sign-up' : 'sign-in');
    }
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

  function handleWindowKey(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || event.defaultPrevented) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    onClose();
  }

  onMount(() => emailInput.focus());
</script>

<svelte:window onkeydown={handleWindowKey} />

<div class="modal-backdrop auth-backdrop">
  <button class="modal-dismiss-layer" type="button" aria-label={t('close')} onclick={onClose}></button>
  <div
    bind:this={dialog}
    class="auth-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="auth-dialog-title"
    tabindex="-1"
    onkeydown={handleDialogKey}
  >
    <header class="modal-header auth-modal-header">
      <div>
        <span class="auth-context">{t('account')}</span>
        <h2 id="auth-dialog-title">{t(mode === 'sign-in' ? 'sync_sign_in' : 'sync_sign_up')}</h2>
      </div>
      <button class="icon-button" type="button" title={t('close')} aria-label={t('close')} onclick={onClose}><Icon name="close" /></button>
    </header>

    <form class="auth-modal-form" novalidate onsubmit={(event) => { event.preventDefault(); void onSubmit(); }}>
      <div class="auth-mode-tabs" role="tablist" aria-label={t('account')} tabindex="-1" onkeydown={handleModeKey}>
        <button
          type="button"
          role="tab"
          class:active={mode === 'sign-in'}
          aria-selected={mode === 'sign-in'}
          tabindex={mode === 'sign-in' ? 0 : -1}
          disabled={busy}
          onclick={() => changeMode('sign-in')}
        >{t('sign_in')}</button>
        <button
          type="button"
          role="tab"
          class:active={mode === 'sign-up'}
          aria-selected={mode === 'sign-up'}
          tabindex={mode === 'sign-up' ? 0 : -1}
          disabled={busy}
          onclick={() => changeMode('sign-up')}
        >{t('sign_up')}</button>
      </div>

      <label class="auth-field">
        <span>{t('email')}</span>
        <input
          bind:this={emailInput}
          id="auth-email"
          type="email"
          autocomplete="email"
          inputmode="email"
          value={email}
          aria-invalid={errorKind === 'required' || errorKind === 'invalid-email'}
          disabled={busy}
          oninput={(event) => onEmailChange(event.currentTarget.value)}
        />
      </label>

      <div class="auth-field">
        <label for="auth-password">{t('password')}</label>
        <span class="auth-password-field">
          <input
            id="auth-password"
            type={passwordVisible ? 'text' : 'password'}
            autocomplete={passwordAutocomplete(mode)}
            value={password}
            aria-invalid={errorKind === 'required' || errorKind === 'password-short'}
            disabled={busy}
            oninput={(event) => onPasswordChange(event.currentTarget.value)}
          />
          <button
            class="auth-password-toggle"
            type="button"
            disabled={busy}
            aria-label={passwordVisible ? t('hide') : t('show')}
            onclick={() => (passwordVisible = !passwordVisible)}
          >{passwordVisible ? t('hide') : t('show')}</button>
        </span>
      </div>

      {#if error}
        <p class="auth-error" role="alert" aria-live="assertive">{error}</p>
      {/if}

      <button class="primary-button auth-submit" type="submit" disabled={busy}>
        {busy ? t(mode === 'sign-in' ? 'signing_in' : 'signing_up') : t(mode === 'sign-in' ? 'sign_in' : 'sign_up')}
      </button>
    </form>
  </div>
</div>
