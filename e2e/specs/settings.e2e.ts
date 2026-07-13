import { bootstrap, invoke } from '../helpers';

describe('Settings parity', () => {
  it('renders seven native language names in two columns and preserves RTL semantics', async () => {
    let snapshot = await bootstrap();
    const settingsList = snapshot.checklists.find((list: any) => list.kind === 'SETTINGS');
    await invoke('select_checklist', { expectedRevision: snapshot.revision, checklistId: settingsList.id });
    await browser.refresh();
    const labels = await $$('.language-grid button').map((button) => button.getText());
    expect(labels).toEqual(['System', 'English', '简体中文', 'العربية', 'Français', 'Русский', 'Español']);
    const columns = await browser.execute(() => getComputedStyle(document.querySelector('.language-grid')!).gridTemplateColumns);
    expect(String(columns).trim().split(/\s+/)).toHaveLength(2);

    snapshot = await bootstrap();
    await invoke('update_settings', {
      expectedRevision: snapshot.revision,
      settings: { ...snapshot.settings, languageMode: 'ARABIC' }
    });
    await browser.refresh();
    await expect($('.app-shell.rtl')).toBeDisplayed();
    await expect($('[data-testid="sync-detail"]')).toHaveText('سجّل الدخول للمزامنة مع Android');
    const rtlLabels = await $$('.language-grid button').map((button) => button.getText());
    expect(rtlLabels).toContain('English');
    expect(rtlLabels).toContain('العربية');
    const rtlChoices = await browser.execute(() =>
      Array.from(document.querySelectorAll<HTMLElement>('.language-grid button')).map((button) => {
        const buttonRect = button.getBoundingClientRect();
        const choiceRect = button.querySelector<HTMLElement>('.pixel-choice')!.getBoundingClientRect();
        return Math.round(buttonRect.right - choiceRect.right);
      })
    );
    expect(Math.max(...rtlChoices) - Math.min(...rtlChoices)).toBeLessThanOrEqual(2);
    await browser.pause(300);
    await browser.saveScreenshot('parity/evidence/windows/settings-arabic.png');

    snapshot = await bootstrap();
    await invoke('update_settings', {
      expectedRevision: snapshot.revision,
      settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
    });
    await browser.refresh();
    await expect($('.language-grid')).toBeDisplayed();
    await expect($('[data-testid="sync-detail"]')).toHaveText('Sign in to sync with Android');
    await expect($('.workspace-status h2')).toHaveText('Options');
    const ltrChoices = await browser.execute(() =>
      Array.from(document.querySelectorAll<HTMLElement>('.language-grid button')).map((button) => {
        const buttonRect = button.getBoundingClientRect();
        const choiceRect = button.querySelector<HTMLElement>('.pixel-choice')!.getBoundingClientRect();
        return Math.round(choiceRect.left - buttonRect.left);
      })
    );
    expect(ltrChoices).toHaveLength(7);
    expect(Math.max(...ltrChoices) - Math.min(...ltrChoices)).toBeLessThanOrEqual(2);

    const switchSizes = await browser.execute(() =>
      Array.from(document.querySelectorAll<HTMLElement>('.switch')).map((control) => {
        const rect = control.getBoundingClientRect();
        return { width: rect.width, height: rect.height };
      })
    );
    expect(switchSizes.length).toBeGreaterThan(2);
    expect(switchSizes.every(({ width, height }) => width === 44 && height === 26)).toBe(true);

    const actionButtons = await browser.execute(() =>
      Array.from(document.querySelectorAll<HTMLElement>('.settings-page .setting-icon-button')).map((control) => {
        const rect = control.getBoundingClientRect();
        return {
          width: rect.width,
          height: rect.height,
          text: control.innerText.trim(),
          label: control.getAttribute('aria-label') ?? ''
        };
      })
    );
    expect(actionButtons.length).toBeGreaterThan(2);
    expect(actionButtons.every(({ width, height }) => width === 44 && height === 44)).toBe(true);
    expect(actionButtons.every(({ text, label }) => text === '' && label.length > 0)).toBe(true);

  });
});
