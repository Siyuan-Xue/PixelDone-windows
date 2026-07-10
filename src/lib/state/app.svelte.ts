import type { AppSnapshot } from '$lib/generated/ipc';

export class AppProjection {
  snapshot = $state<AppSnapshot | null>(null);
  selectedTodoId = $state<string | null>(null);
  inspectorOpen = $state(false);
}
