import { bootstrap } from '../helpers';

describe('Update channel', () => {
  it('reports the formal version from Rust', async () => {
    const snapshot = await bootstrap();
    expect(snapshot.update.currentVersion).toBe('3.2.1');
    expect(['IDLE', 'CURRENT', 'AVAILABLE', 'CHECKING']).toContain(snapshot.update.state);
  });
});
