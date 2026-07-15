import { describe, expect, test } from 'bun:test';
import { mutationDisposition } from '../src/lib/ipc/revision';

describe('revision ordering', () => {
  test('ignores old results, applies the next delta and reloads on gaps', () => {
    expect(mutationDisposition(8, 7)).toBe('ignore');
    expect(mutationDisposition(8, 8)).toBe('ignore');
    expect(mutationDisposition(8, 9)).toBe('apply');
    expect(mutationDisposition(8, 10)).toBe('reload');
  });
});
