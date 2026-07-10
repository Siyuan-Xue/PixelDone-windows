import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadManifest, summarize } from './parity';

const manifest = await loadManifest();
const allowed = new Set(['not_started', 'in_progress', 'blocked', 'verified']);
const failures: string[] = [];
const root = fileURLToPath(new URL('..', import.meta.url));

for (const row of manifest.rows) {
  if (!allowed.has(row.status)) failures.push(`${row.id}: invalid status ${row.status}`);
  if (!row.android.source.length) failures.push(`${row.id}: missing Android source evidence`);
  if (row.status === 'verified') {
    if (!row.windows.rustSource.length && !row.windows.uiEntry.length) failures.push(`${row.id}: missing Windows implementation evidence`);
    if (!row.windows.tests.length) failures.push(`${row.id}: missing Windows test evidence`);
    for (const path of [...row.windows.rustSource, ...row.windows.uiEntry]) {
      const filePath = path.split('#')[0];
      if (!existsSync(`${root}/${filePath}`)) failures.push(`${row.id}: evidence file not found ${filePath}`);
    }
  }
}

const summary = summarize(manifest);
if (summary.percent !== 100 || summary.counts.blocked || summary.counts.in_progress || summary.counts.not_started) {
  failures.push(
    `release gate: ${summary.percent.toFixed(2)}%, ${summary.counts.blocked} blocked, ${summary.counts.in_progress} in_progress, ${summary.counts.not_started} not_started`
  );
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log('PixelDone parity gate: 100.00%');
