import { describe, expect, it } from 'bun:test';
import {
  exportChecklistToMarkdown,
  type MarkdownExportFormatters,
  type MarkdownExportLabels
} from '../src/lib/components/shell/markdown-export';
import type { Checklist, TodoItem } from '../src/lib/generated/ipc';

const labels: MarkdownExportLabels = {
  priority: 'Priority',
  due: 'Due',
  repeat: 'Repeat',
  none: 'None'
};
const formatters: MarkdownExportFormatters = {
  priority: (priority) => priority,
  due: (due) => `DUE-${due}`,
  repeat: (repeat) => repeat
};

describe('exportChecklistToMarkdown', () => {
  it('exports all tasks with completion state in the selected sort order', () => {
    const markdown = exportChecklistToMarkdown(
      checklist('Ship #3', [
        todo('done', { completed: true, priority: 'XHIGH', dueAtMillis: 1 }),
        todo('active', { priority: 'LOW', dueAtMillis: 2 })
      ]),
      'PRIORITY',
      'simple',
      labels,
      formatters
    );

    expect(markdown).toBe('# Ship \\#3\n- [ ] active\n- [x] done');
  });

  it('adds detailed metadata and flattens Markdown-sensitive multiline titles', () => {
    const markdown = exportChecklistToMarkdown(
      checklist('List', [
        todo('Write *notes*\nfor [team]', {
          priority: 'HIGH',
          dueAtMillis: 42,
          reminderRepeat: 'WEEKLY'
        })
      ]),
      'TIME',
      'detailed',
      labels,
      formatters
    );

    expect(markdown).toBe(
      '# List\n'
      + '- [ ] Write \\*notes\\* for \\[team\\]\n'
      + '  - Priority：HIGH\n'
      + '  - Due：DUE-42\n'
      + '  - Repeat：WEEKLY'
    );
  });

  it('uses none for absent details and only a heading for an empty list', () => {
    expect(exportChecklistToMarkdown(
      checklist('Plain', [todo('Task', { dueAtMillis: 0 })]),
      'PRIORITY',
      'detailed',
      labels,
      formatters
    )).toContain('- Due：None\n  - Repeat：None');
    expect(exportChecklistToMarkdown(
      checklist('Empty', []),
      'PRIORITY',
      'simple',
      labels,
      formatters
    )).toBe('# Empty');
  });
});

function checklist(name: string, items: TodoItem[]): Checklist {
  return {
    id: 'list',
    name,
    kind: 'NORMAL',
    items,
    createdAtMillis: 0,
    updatedAtMillis: 0,
    remoteVersion: null
  };
}

function todo(title: string, patch: Partial<TodoItem> = {}): TodoItem {
  return {
    id: title,
    title,
    priority: 'MEDIUM',
    dueAtMillis: 0,
    completed: false,
    createdAtMillis: 0,
    updatedAtMillis: 0,
    reminderRepeat: 'NONE',
    imageFileName: null,
    trashedFromChecklistId: null,
    trashedFromChecklistName: null,
    trashedAtMillis: null,
    remoteVersion: null,
    ...patch
  };
}
