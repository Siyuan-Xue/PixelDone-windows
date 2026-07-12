import { describe, expect, test } from 'bun:test';
import { loadManifest, summarize } from './parity';

describe('parity manifest', () => {
  test('targets Windows 3.1.2 while preserving the Android 3.1.0 functional baseline', async () => {
    const manifest = await loadManifest();
    expect(manifest.windowsTarget).toEqual({
      product: 'PixelDone Windows',
      version: '3.1.2',
      stage: 'released',
      evidence: ['parity/evidence/windows/release-3.1.2.md']
    });
    expect(manifest.baseline.version).toBe('3.1.0');
    expect(manifest.baseline.commit).toBe('63471218345f6a4efcdbbd32c2d4c42acb25491c');
    expect(manifest.baseline.roomSchema).toBe(5);
    expect(summarize(manifest).required).toBeGreaterThan(35);
  });

  test('accepts verified desktop and release-integrity rows for 3.1.2', async () => {
    const manifest = await loadManifest();
    for (const id of ['LIST-FIXED-001', 'TODO-CRUD-001', 'SETTINGS-DOCK-001']) {
      const row = manifest.rows.find((candidate) => candidate.id === id);
      expect(row?.status).toBe('verified');
      expect(row?.evidence.windows).toContain('parity/evidence/windows/release-3.1.2.md');
    }
    const installer = manifest.rows.find((candidate) => candidate.id === 'RELEASE-NSIS-001');
    expect(installer?.status).toBe('verified');
    expect(installer?.evidence.windows).toContain('parity/evidence/windows/release-3.1.2.md');
    expect(summarize(manifest).counts.in_progress).toBe(0);
  });

  test('records source-only batch move as excluded', async () => {
    const manifest = await loadManifest();
    const batchMove = manifest.rows.find((row) => row.id === 'SOURCE-BATCH-MOVE-001');
    expect(batchMove?.requiredForRelease).toBeFalse();
    expect(batchMove?.variance).toBe('excluded_source_only');
  });
});
