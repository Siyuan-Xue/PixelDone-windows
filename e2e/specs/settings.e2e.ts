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
    const rtlLabels = await $$('.language-grid button').map((button) => button.getText());
    expect(rtlLabels).toContain('English');
    expect(rtlLabels).toContain('العربية');
    await browser.pause(300);
    await browser.saveScreenshot('parity/evidence/windows/settings-arabic.png');
  });
});
