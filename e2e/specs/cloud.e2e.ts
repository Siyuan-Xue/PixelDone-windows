import { bootstrap, invoke } from '../helpers';

describe('Cloud boundary', () => {
  it('uses shared HTTP configuration without exposing credentials to the WebView', async () => {
    const snapshot = await bootstrap();
    expect(snapshot.auth).toMatchObject({ cloudAvailable: true, insecureHttp: true, signedIn: false });
    expect(snapshot.sync.insecureHttp).toBe(true);
    expect(snapshot.auth).not.toHaveProperty('accessToken');
    expect(snapshot.auth).not.toHaveProperty('refreshToken');
  });

  if (process.env.PIXELDONE_RUN_LIVE_CLOUD === 'true') {
    it('registers, pushes and pulls through the deployed 3.1 HTTP RPC', async () => {
      const email = `pixeldone.windows.e2e.${Date.now()}@example.com`;
      const password = `Pd-${crypto.randomUUID()}-3.1!`;
      let snapshot = await bootstrap();
      await invoke('auth_sign_up', { expectedRevision: snapshot.revision, email, password });
      snapshot = await bootstrap();
      expect(snapshot.auth.signedIn).toBe(true);

      const createdList = await invoke('create_checklist', { expectedRevision: snapshot.revision, name: 'CLOUD E2E' });
      snapshot = await bootstrap();
      const checklistId = snapshot.selectedChecklistId;
      await invoke('create_todo', {
        expectedRevision: createdList.revision,
        checklistId,
        draft: { title: 'CLOUD ROUNDTRIP', priority: 'HIGH', dueAtMillis: Date.now() + 60_000, reminderRepeat: 'WEEKLY' }
      });
      snapshot = await bootstrap();
      await invoke('sync_now', { expectedRevision: snapshot.revision });
      snapshot = await bootstrap();
      expect(snapshot.sync.state).toBe('SYNCED');
      expect(snapshot.sync.remoteVersion).toBeGreaterThan(0);
      expect(snapshot.sync.pendingCount).toBe(0);
      await invoke('auth_sign_out', { expectedRevision: snapshot.revision });
    });
  }
});
