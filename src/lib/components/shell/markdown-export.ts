import type {
  Checklist,
  ReminderRepeat,
  SortMode,
  TodoItem,
  TodoPriority
} from '$lib/generated/ipc';

export type MarkdownExportMode = 'simple' | 'detailed';

export interface MarkdownExportLabels {
  priority: string;
  due: string;
  repeat: string;
  none: string;
}

export interface MarkdownExportFormatters {
  priority: (priority: TodoPriority) => string;
  due: (dueAtMillis: number) => string;
  repeat: (repeat: ReminderRepeat) => string;
}

export function exportChecklistToMarkdown(
  checklist: Checklist,
  sortMode: SortMode,
  mode: MarkdownExportMode,
  labels: MarkdownExportLabels,
  formatters: MarkdownExportFormatters
): string {
  const heading = `# ${markdownPlainText(checklist.name)}`;
  const items = [...checklist.items].sort((left, right) => compareTodo(left, right, sortMode));
  if (!items.length) return heading;

  return [
    heading,
    ...items.map((item) => {
      const lines = [`- [${item.completed ? 'x' : ' '}] ${markdownPlainText(item.title)}`];
      if (mode === 'detailed') {
        lines.push(`  - ${labels.priority}：${formatters.priority(item.priority)}`);
        lines.push(`  - ${labels.due}：${item.dueAtMillis > 0 ? formatters.due(item.dueAtMillis) : labels.none}`);
        lines.push(`  - ${labels.repeat}：${item.reminderRepeat === 'NONE' ? labels.none : formatters.repeat(item.reminderRepeat)}`);
      }
      return lines.join('\n');
    })
  ].join('\n');
}

function compareTodo(left: TodoItem, right: TodoItem, mode: SortMode): number {
  const completed = Number(left.completed) - Number(right.completed);
  if (completed !== 0) return completed;
  if (mode === 'TIME' && left.dueAtMillis !== right.dueAtMillis) {
    return left.dueAtMillis - right.dueAtMillis;
  }
  const priority = priorityRank(left.priority) - priorityRank(right.priority);
  if (priority !== 0) return priority;
  if (mode === 'PRIORITY' && left.dueAtMillis !== right.dueAtMillis) {
    return left.dueAtMillis - right.dueAtMillis;
  }
  return left.createdAtMillis - right.createdAtMillis;
}

function priorityRank(priority: TodoPriority): number {
  return { XHIGH: 0, HIGH: 1, MEDIUM: 2, LOW: 3 }[priority];
}

export function markdownPlainText(value: string): string {
  const flattened = value.replace(/[\r\n]+/g, ' ').replace(/\s+/g, ' ').trim();
  return flattened.replace(/[\\`*_{}\[\]()#+\-.!>|]/g, '\\$&');
}
