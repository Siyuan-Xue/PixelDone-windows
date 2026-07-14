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
});
