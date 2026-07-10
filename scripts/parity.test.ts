import { describe, expect, test } from 'bun:test';
import { loadManifest, summarize } from './parity';

describe('parity manifest', () => {
  test('pins Android 3.0.3 commit and counts required rows', async () => {
    const manifest = await loadManifest();
    expect(manifest.baseline.version).toBe('3.0.3');
    expect(manifest.baseline.commit.startsWith('89763b6')).toBeTrue();
    expect(summarize(manifest).required).toBeGreaterThan(20);
  });

  test('records source-only batch move as excluded', async () => {
    const manifest = await loadManifest();
    const batchMove = manifest.rows.find((row) => row.id === 'SOURCE-BATCH-MOVE-001');
    expect(batchMove?.requiredForRelease).toBeFalse();
    expect(batchMove?.variance).toBe('excluded_source_only');
  });
});
