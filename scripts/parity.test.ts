import { describe, expect, test } from 'bun:test';
import { loadManifest, summarize } from './parity';

describe('parity manifest', () => {
  test('targets Windows 3.2.4 while preserving the audited Android baseline', async () => {
    const manifest = await loadManifest();
    expect(manifest.windowsTarget).toEqual({
      product: 'PixelDone Windows',
      version: '3.2.4',
      stage: 'formal_release',
      evidence: ['parity/evidence/windows/candidate-3.2.4.md'],
      authorizedIncompleteRows: [
        'IMAGE-LOCAL-001',
        'AUTH-001',
        'SYNC-MANUAL-001',
        'SYNC-CAS-001',
        'SYNC-IMAGE-LOCAL-001',
        'IMAGE-CLOUD-EXCLUDED'
      ]
    });
    expect(manifest.baseline.version).toBe('3.1.0');
    expect(manifest.baseline.commit).toBe('63471218345f6a4efcdbbd32c2d4c42acb25491c');
    expect(manifest.baseline.roomSchema).toBe(5);
    expect(summarize(manifest).required).toBeGreaterThan(35);
  });

  test('records the 3.2.4 installer as verified while cloud exceptions stay explicit', async () => {
    const manifest = await loadManifest();
    for (const id of ['LIST-FIXED-001', 'TODO-CRUD-001', 'SETTINGS-DOCK-001']) {
      const row = manifest.rows.find((candidate) => candidate.id === id);
      expect(row?.status).toBe('verified');
    }
    const installer = manifest.rows.find((candidate) => candidate.id === 'RELEASE-NSIS-001');
    expect(installer?.status).toBe('verified');
    expect(installer?.evidence.windows).toContain('parity/evidence/windows/candidate-3.2.4.md');
    const cloudImage = manifest.rows.find((candidate) => candidate.id === 'IMAGE-CLOUD-EXCLUDED');
    expect(cloudImage?.requiredForRelease).toBeTrue();
    expect(cloudImage?.status).toBe('in_progress');
    expect(summarize(manifest).counts.in_progress).toBeGreaterThan(0);
  });

  test('records source-only batch move as excluded', async () => {
    const manifest = await loadManifest();
    const batchMove = manifest.rows.find((row) => row.id === 'SOURCE-BATCH-MOVE-001');
    expect(batchMove?.requiredForRelease).toBeFalse();
    expect(batchMove?.variance).toBe('excluded_source_only');
  });
});
