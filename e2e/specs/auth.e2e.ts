import { bootstrap, invoke } from '../helpers';

describe('Authentication modal', () => {
  it('opens from Account, validates locally, switches modes lightly and restores focus', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    const originalSettings = initial.settings;
    const settingsList = initial.checklists.find((list: any) => list.kind === 'SETTINGS');

    try {
      let snapshot = initial;
      if (snapshot.settings.languageMode !== 'ENGLISH') {
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
        });
        snapshot = await bootstrap();
      }
      if (snapshot.selectedChecklistId !== settingsList.id) {
        await invoke('select_checklist', {
          expectedRevision: snapshot.revision,
          checklistId: settingsList.id
        });
      }
      await browser.refresh();

      expect(await $('.auth-form').isExisting()).toBe(false);
      const trigger = await $('.cloud-account .cloud-icon-button');
      await trigger.click();
      await expect($('.auth-modal')).toBeDisplayed();
      expect(await browser.execute(() => document.activeElement?.id)).toBe('auth-email');
      await expect($('.auth-mode-tabs button.active')).toHaveText('Sign in');
      expect(await $('#auth-password').getAttribute('autocomplete')).toBe('current-password');

      await $('.auth-submit').click();
      await expect($('.auth-error')).toBeDisplayed();
      await $('#auth-email').setValue('not-an-email');
      expect(await $('.auth-error').isExisting()).toBe(false);
      await $('#auth-password').setValue('secret');
      await $('.auth-submit').click();
      await expect($('.auth-error')).toBeDisplayed();

      await $('#auth-email').setValue('person@example.com');
      await $('#auth-password').setValue('short');
      await $('.auth-submit').click();
      await expect($('.auth-error')).toBeDisplayed();

      await $$('.auth-mode-tabs button')[1].click();
      await expect($('.auth-mode-tabs button.active')).toHaveText('Sign up');
      expect(await $('#auth-password').getAttribute('autocomplete')).toBe('new-password');
      expect(await $('.auth-error').isExisting()).toBe(false);
      await $('#auth-password').setValue('secret');
      await $('.auth-password-toggle').click();
      expect(await $('#auth-password').getAttribute('type')).toBe('text');

      const focusWrap = await browser.execute(() => {
        const close = document.querySelector<HTMLElement>('.auth-modal-header .icon-button')!;
        const submit = document.querySelector<HTMLElement>('.auth-submit')!;
        close.focus();
        close.dispatchEvent(new KeyboardEvent('keydown', {
          key: 'Tab', shiftKey: true, bubbles: true, cancelable: true
        }));
        const wrappedBackward = document.activeElement === submit;
        submit.dispatchEvent(new KeyboardEvent('keydown', {
          key: 'Tab', bubbles: true, cancelable: true
        }));
        return { wrappedBackward, wrappedForward: document.activeElement === close };
      });
      expect(focusWrap).toEqual({ wrappedBackward: true, wrappedForward: true });

      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));
      expect(await browser.execute(() => document.activeElement?.classList.contains('cloud-icon-button'))).toBe(true);

      await trigger.click();
      await expect($('.auth-modal')).toBeDisplayed();
      await expect($('.auth-mode-tabs button.active')).toHaveText('Sign in');
      expect(await $('#auth-email').getValue()).toBe('person@example.com');
      expect(await $('#auth-password').getValue()).toBe('');
      await $('#auth-email').setValue('');
      await browser.saveScreenshot('parity/evidence/windows/auth-english.png');
      await $('.modal-dismiss-layer').click();
      await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));
      expect(await browser.execute(() => document.activeElement?.classList.contains('cloud-icon-button'))).toBe(true);

      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_SIGN_IN__?: () => Promise<never>;
          __pixeldoneRejectAuth?: () => void;
        };
        scope.__PIXELDONE_E2E_AUTH_SIGN_IN__ = () => new Promise((_, reject) => {
          scope.__pixeldoneRejectAuth = () => reject(new Error('obsolete authentication error'));
        });
      });
      await trigger.click();
      await $('#auth-email').setValue('person@example.com');
      await $('#auth-password').setValue('secret!');
      await $('.auth-submit').click();
      await browser.waitUntil(async () => browser.execute(
        () => typeof (globalThis as typeof globalThis & { __pixeldoneRejectAuth?: () => void }).__pixeldoneRejectAuth === 'function'
      ));
      await expect($('.auth-submit')).toBeDisabled();
      await expect($('.auth-submit')).toHaveText('Signing in');
      expect(await $$('.auth-mode-tabs button')[0].isEnabled()).toBe(false);
      expect(await $('#auth-email').isEnabled()).toBe(false);
      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & { __pixeldoneRejectAuth?: () => void };
        scope.__pixeldoneRejectAuth?.();
        delete scope.__pixeldoneRejectAuth;
      });
      await browser.waitUntil(async () => trigger.isEnabled(), { timeout: 7_000 });
      expect(await $('.operation-error').isExisting()).toBe(false);
      await trigger.click();
      await expect($('.auth-modal')).toBeDisplayed();
      expect(await $('.auth-error').isExisting()).toBe(false);
      expect(await $('#auth-password').getValue()).toBe('');
      await browser.keys(['Escape']);
    } finally {
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_SIGN_IN__?: unknown;
          __pixeldoneRejectAuth?: unknown;
        };
        delete scope.__PIXELDONE_E2E_AUTH_SIGN_IN__;
        delete scope.__pixeldoneRejectAuth;
      });
      let snapshot = await bootstrap();
      if (snapshot.settings.languageMode !== originalSettings.languageMode) {
        await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
        snapshot = await bootstrap();
      }
      if (
        snapshot.selectedChecklistId !== originalChecklistId &&
        snapshot.checklists.some((list: any) => list.id === originalChecklistId)
      ) {
        await invoke('select_checklist', {
          expectedRevision: snapshot.revision,
          checklistId: originalChecklistId
        });
      }
      await browser.refresh();
    }
  });

  it('changes passwords in an isolated modal and keeps Sync actions in the Sync row', async () => {
    const initial = await bootstrap();
    const originalChecklistId = initial.selectedChecklistId;
    const originalSettings = initial.settings;
    const settingsList = initial.checklists.find((list: any) => list.kind === 'SETTINGS');

    try {
      let snapshot = initial;
      if (snapshot.settings.languageMode !== 'ENGLISH') {
        await invoke('update_settings', {
          expectedRevision: snapshot.revision,
          settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
        });
        snapshot = await bootstrap();
      }
      if (snapshot.selectedChecklistId !== settingsList.id) {
        await invoke('select_checklist', { expectedRevision: snapshot.revision, checklistId: settingsList.id });
      }
      await browser.refresh();

      await browser.execute(() => {
        const delta = (auth: unknown, sync: unknown) => ({
          upsertedChecklists: [], removedChecklistIds: [], selectedChecklistId: null,
          sortMode: null, hideCompleted: null, quickDelete: null, showDeadlineCountdown: null,
          checklistHistory: null, settings: null, auth, sync, reminder: null, update: null
        });
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_SIGN_IN__?: (input: { expectedRevision: number }) => Promise<unknown>;
        };
        scope.__PIXELDONE_E2E_AUTH_SIGN_IN__ = async ({ expectedRevision }) => ({
          revision: expectedRevision + 1,
          changedIds: ['auth'],
          snapshotDelta: delta(
            { cloudAvailable: true, signedIn: true, userId: 'e2e-user', userEmail: 'person@example.com', insecureHttp: true },
            { state: 'ERROR', message: 'raw network diagnostics', issueCode: 'NETWORK_RETRYING', nextRetryAtMillis: Date.now() + 1_000, remoteVersion: null, pendingCount: 0, conflictCount: 0, insecureHttp: true }
          )
        });
      });

      await $('.cloud-account .cloud-icon-button').click();
      await $('#auth-email').setValue('person@example.com');
      await $('#auth-password').setValue('secret!');
      await $('.auth-submit').click();
      await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));
      await expect($('.password-modal-trigger')).toBeDisplayed();
      expect(await $('.cloud-account .sync-now-button').isExisting()).toBe(false);
      await expect($('.sync-setting-row .sync-now-button')).toBeDisplayed();
      await expect($('[data-testid="sync-detail"]')).toHaveText('The connection was interrupted. PixelDone is retrying automatically');
      expect(await $('.operation-error').isExisting()).toBe(false);
      expect(await $('.password-form').isExisting()).toBe(false);

      const trigger = await $('.password-modal-trigger');
      await trigger.click();
      await expect($('.password-modal')).toBeDisplayed();
      expect(await browser.execute(() => document.activeElement?.id)).toBe('password-current');
      await $('.password-submit').click();
      await expect($('.password-error')).toHaveText('All password fields are required');
      await $('#password-current').setValue('old-secret');
      await $('#password-new').setValue('short');
      await $('#password-confirmation').setValue('short');
      await $('.password-submit').click();
      await expect($('.password-error')).toHaveText('The new password must contain at least 6 characters');
      await $('#password-new').setValue('new-secret');
      await $('#password-confirmation').setValue('different');
      await $('.password-submit').click();
      await expect($('.password-error')).toHaveText('The new passwords do not match');
      await $$('.password-modal .auth-password-toggle')[1].click();
      expect(await $('#password-new').getAttribute('type')).toBe('text');

      const focusWrap = await browser.execute(() => {
        const close = document.querySelector<HTMLElement>('.password-modal-header .icon-button')!;
        const submit = document.querySelector<HTMLElement>('.password-submit')!;
        close.focus();
        close.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }));
        const backward = document.activeElement === submit;
        submit.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        return { backward, forward: document.activeElement === close };
      });
      expect(focusWrap).toEqual({ backward: true, forward: true });
      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.password-modal').isExisting()));
      expect(await browser.execute(() => document.activeElement?.classList.contains('password-modal-trigger'))).toBe(true);

      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__?: () => Promise<never>;
          __pixeldoneRejectPassword?: () => void;
        };
        scope.__PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__ = () => new Promise((_, reject) => {
          scope.__pixeldoneRejectPassword = () => reject(new Error('obsolete password error'));
        });
      });
      await trigger.click();
      await $('#password-current').setValue('old-secret');
      await $('#password-new').setValue('new-secret');
      await $('#password-confirmation').setValue('new-secret');
      await $('.password-submit').click();
      await expect($('.password-submit')).toBeDisabled();
      expect(await $('.password-submit').getText()).toContain('Changing password');
      await browser.keys(['Escape']);
      await browser.waitUntil(async () => !(await $('.password-modal').isExisting()));
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & { __pixeldoneRejectPassword?: () => void };
        scope.__pixeldoneRejectPassword?.();
        delete scope.__pixeldoneRejectPassword;
      });
      await browser.waitUntil(async () => trigger.isEnabled(), { timeout: 7_000 });
      expect(await $('.operation-error').isExisting()).toBe(false);

      await browser.execute(() => {
        const delta = (auth: unknown, sync: unknown) => ({
          upsertedChecklists: [], removedChecklistIds: [], selectedChecklistId: null,
          sortMode: null, hideCompleted: null, quickDelete: null, showDeadlineCountdown: null,
          checklistHistory: null, settings: null, auth, sync, reminder: null, update: null
        });
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__?: (input: { expectedRevision: number }) => Promise<unknown>;
        };
        scope.__PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__ = async ({ expectedRevision }) => ({
          revision: expectedRevision + 1,
          changedIds: ['auth'],
          snapshotDelta: delta(
            { cloudAvailable: true, signedIn: false, userId: null, userEmail: null, insecureHttp: true },
            { state: 'SIGNED_OUT', message: 'Password changed', issueCode: null, nextRetryAtMillis: null, remoteVersion: null, pendingCount: 0, conflictCount: 0, insecureHttp: true }
          )
        });
      });
      await trigger.click();
      expect(await $('#password-current').getValue()).toBe('');
      expect(await $('.password-error').isExisting()).toBe(false);
      await $('#password-current').setValue('old-secret');
      await $('#password-new').setValue('new-secret');
      await $('#password-confirmation').setValue('new-secret');
      await $('.password-submit').click();
      await browser.waitUntil(async () => !(await $('.password-modal').isExisting()));
      await expect($('.operation-notice.success')).toHaveText('Password changed. Sign in again');
      expect(await browser.execute(() => document.activeElement?.classList.contains('cloud-icon-button'))).toBe(true);
      expect(await $('.sync-setting-row .sync-now-button').isExisting()).toBe(false);
    } finally {
      await browser.execute(() => {
        const scope = globalThis as typeof globalThis & {
          __PIXELDONE_E2E_AUTH_SIGN_IN__?: unknown;
          __PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__?: unknown;
          __pixeldoneRejectPassword?: unknown;
        };
        delete scope.__PIXELDONE_E2E_AUTH_SIGN_IN__;
        delete scope.__PIXELDONE_E2E_AUTH_CHANGE_PASSWORD__;
        delete scope.__pixeldoneRejectPassword;
      });
      let snapshot = await bootstrap();
      if (snapshot.settings.languageMode !== originalSettings.languageMode) {
        await invoke('update_settings', { expectedRevision: snapshot.revision, settings: originalSettings });
        snapshot = await bootstrap();
      }
      if (
        snapshot.selectedChecklistId !== originalChecklistId &&
        snapshot.checklists.some((list: any) => list.id === originalChecklistId)
      ) {
        await invoke('select_checklist', { expectedRevision: snapshot.revision, checklistId: originalChecklistId });
      }
      await browser.refresh();
    }
  });
});
