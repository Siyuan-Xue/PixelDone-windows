import { describe, expect, it } from 'bun:test';
import { orderedDockItems } from '../src/lib/components/shell/dock';

const actions = ['SORT', 'DEADLINE', 'HIDE_DONE', 'DELETE_DONE'] as const;

function names(placement: 'LEFT_EDGE' | 'CENTER' | 'RIGHT_EDGE'): string[] {
  return orderedDockItems([...actions], placement).map((item) =>
    item.kind === 'plus' ? 'PLUS' : item.action
  );
}

describe('orderedDockItems', () => {
  it('inserts plus at the physical left edge', () => {
    expect(names('LEFT_EDGE')).toEqual(['PLUS', ...actions]);
  });

  it('inserts plus between the two Android-style action groups', () => {
    expect(names('CENTER')).toEqual(['SORT', 'DEADLINE', 'PLUS', 'HIDE_DONE', 'DELETE_DONE']);
  });

  it('inserts plus at the physical right edge', () => {
    expect(names('RIGHT_EDGE')).toEqual([...actions, 'PLUS']);
  });

  it('keeps the extra action on the left side for an odd action count', () => {
    expect(orderedDockItems(['SORT', 'DEADLINE', 'HIDE_DONE'], 'CENTER')).toEqual([
      { kind: 'action', action: 'SORT' },
      { kind: 'action', action: 'DEADLINE' },
      { kind: 'plus' },
      { kind: 'action', action: 'HIDE_DONE' }
    ]);
  });
});

