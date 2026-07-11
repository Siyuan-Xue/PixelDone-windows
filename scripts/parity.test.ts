import { describe, expect, test } from 'bun:test';
import { loadManifest, summarize } from './parity';

describe('parity manifest', () => {
  test('pins the final Android 3.1.0 commit and counts atomic required rows', async () => {
    const manifest = await loadManifest();
    expect(manifest.baseline.version).toBe('3.1.0');
    expect(manifest.baseline.commit).toBe('63471218345f6a4efcdbbd32c2d4c42acb25491c');
    expect(manifest.baseline.roomSchema).toBe(5);
    expect(summarize(manifest).required).toBeGreaterThan(35);
  });

  test('records source-only batch move as excluded', async () => {
    const manifest = await loadManifest();
    const batchMove = manifest.rows.find((row) => row.id === 'SOURCE-BATCH-MOVE-001');
    expect(batchMove?.requiredForRelease).toBeFalse();
    expect(batchMove?.variance).toBe('excluded_source_only');
  });
});
