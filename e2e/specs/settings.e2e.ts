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
    const arabicFonts = await browser.execute(() => ({
      sans: getComputedStyle(document.documentElement).getPropertyValue('--font-sans'),
      serif: getComputedStyle(document.documentElement).getPropertyValue('--font-serif'),
      section: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).fontFamily,
      english: getComputedStyle(document.querySelector<HTMLElement>('.language-label[lang="en"]')!).fontFamily,
      chinese: getComputedStyle(document.querySelector<HTMLElement>('.language-label[lang="zh-Hans"]')!).fontFamily,
      arabic: getComputedStyle(document.querySelector<HTMLElement>('.language-label[lang="ar"]')!).fontFamily
    }));
    expect(arabicFonts.sans).toContain('PixelDone Noto Sans Arabic');
    expect(arabicFonts.serif).toContain('PixelDone Noto Naskh Arabic');
    expect(arabicFonts.section).toContain('PixelDone Noto Naskh Arabic');
    expect(arabicFonts.english).toContain('PixelDone Source Sans');
    expect(arabicFonts.chinese).toContain('PixelDone Noto Sans SC');
    expect(arabicFonts.arabic).toContain('PixelDone Noto Sans Arabic');
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
    await $('.cloud-account .cloud-icon-button').click();
    await expect($('.auth-modal')).toBeDisplayed();
    await browser.saveScreenshot('parity/evidence/windows/auth-arabic.png');
    await browser.keys(['Escape']);
    await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));

    snapshot = await bootstrap();
    await invoke('update_settings', {
      expectedRevision: snapshot.revision,
      settings: { ...snapshot.settings, languageMode: 'SIMPLIFIED_CHINESE' }
    });
    await browser.refresh();
    await expect($('.settings-page')).toBeDisplayed();
    const chineseFonts = await browser.execute(() => ({
      sans: getComputedStyle(document.documentElement).getPropertyValue('--font-sans'),
      serif: getComputedStyle(document.documentElement).getPropertyValue('--font-serif'),
      section: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).fontFamily
    }));
    expect(chineseFonts.sans).toContain('PixelDone Noto Sans SC');
    expect(chineseFonts.serif).toContain('PixelDone Noto Serif SC');
    expect(chineseFonts.section).toContain('PixelDone Noto Serif SC');
    await $('.cloud-account .cloud-icon-button').click();
    await expect($('.auth-modal')).toBeDisplayed();
    await browser.saveScreenshot('parity/evidence/windows/auth-chinese.png');
    await browser.keys(['Escape']);
    await browser.waitUntil(async () => !(await $('.auth-modal').isExisting()));

    snapshot = await bootstrap();
    await invoke('update_settings', {
      expectedRevision: snapshot.revision,
      settings: { ...snapshot.settings, languageMode: 'ENGLISH' }
    });
    await browser.refresh();
    await expect($('.language-grid')).toBeDisplayed();
    await expect($('[data-testid="sync-detail"]')).toHaveText('Sign in to sync with Android');
    await expect($('.workspace-status h2')).toHaveText('Options');
    const englishFonts = await browser.execute(() => ({
      sans: getComputedStyle(document.documentElement).getPropertyValue('--font-sans'),
      serif: getComputedStyle(document.documentElement).getPropertyValue('--font-serif'),
      section: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).fontFamily,
      sectionColor: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).color,
      sectionSize: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).fontSize,
      sectionWeight: getComputedStyle(document.querySelector<HTMLElement>('.settings-page .section-title')!).fontWeight
    }));
    expect(englishFonts.sans).toContain('PixelDone Source Sans');
    expect(englishFonts.serif).toContain('PixelDone Source Serif');
    expect(englishFonts.section).toContain('PixelDone Source Serif');
    expect(englishFonts.sectionColor).toBe('rgb(217, 119, 87)');
    expect(englishFonts.sectionSize).toBe('16px');
    expect(englishFonts.sectionWeight).toBe('600');
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
      Array.from(
        document.querySelectorAll<HTMLElement>('.settings-page .setting-icon-button, .settings-page .cloud-icon-button')
      ).map((control) => {
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
