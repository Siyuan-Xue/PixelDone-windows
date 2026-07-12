import type { DockAction, DockPlusPlacement } from '$lib/generated/ipc';

export type DockItem =
  | { kind: 'action'; action: DockAction }
  | { kind: 'plus' };

/** Mirrors Android's orderedDockItems contract: placement changes plus insertion, not dock position. */
export function orderedDockItems(actions: DockAction[], placement: DockPlusPlacement): DockItem[] {
  const actionItems = actions.map((action) => ({ kind: 'action', action }) as const);
  if (placement === 'LEFT_EDGE') return [{ kind: 'plus' }, ...actionItems];
  if (placement === 'RIGHT_EDGE') return [...actionItems, { kind: 'plus' }];
  const leftCount = Math.ceil(actionItems.length / 2);
  return [...actionItems.slice(0, leftCount), { kind: 'plus' }, ...actionItems.slice(leftCount)];
}

