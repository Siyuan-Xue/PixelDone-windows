import { bootstrap } from '../helpers';
import { mkdirSync } from 'node:fs';

describe('PixelDone desktop bootstrap', () => {
  it('opens the real Tauri window with the Rust snapshot', async () => {
    const snapshot = await bootstrap();
    expect(snapshot.revision).toBe(0);
    expect(snapshot.checklists.map((list: any) => list.kind)).toEqual(['NORMAL', 'TRASH', 'SETTINGS']);
    expect(snapshot.auth.insecureHttp).toBe(true);
    expect(snapshot.sync.insecureHttp).toBe(true);
    await expect($('.app-shell')).toBeDisplayed();
    await expect($('.sidebar')).toBeDisplayed();
    await expect($('.workspace')).toBeDisplayed();
    await expect($('.inspector')).toBeDisplayed();
    await expect($('.status-strip')).toHaveText(expect.stringContaining('REV 0'));
    mkdirSync('parity/evidence/windows', { recursive: true });
    await browser.saveScreenshot('parity/evidence/windows/main-1180x780.png');
  });
});
