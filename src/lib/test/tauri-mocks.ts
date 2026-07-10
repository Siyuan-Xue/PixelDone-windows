import type { AppSnapshot } from '$lib/generated/ipc';

export function cloneFixture(snapshot: AppSnapshot): AppSnapshot {
  return structuredClone(snapshot);
}
