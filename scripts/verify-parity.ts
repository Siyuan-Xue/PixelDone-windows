import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadManifest, summarize } from './parity';

const manifest = await loadManifest();
const allowed = new Set(['not_started', 'in_progress', 'blocked', 'verified']);
const failures: string[] = [];
const root = fileURLToPath(new URL('..', import.meta.url));

if (manifest.windowsTarget.version !== '3.2.0') {
  failures.push(`release target: expected Windows 3.2.0, found ${manifest.windowsTarget.version}`);
}
for (const path of manifest.windowsTarget.evidence) {
  if (!existsSync(`${root}/${path}`)) failures.push(`release target: evidence file not found ${path}`);
}

for (const row of manifest.rows) {
  if (!allowed.has(row.status)) failures.push(`${row.id}: invalid status ${row.status}`);
  if (!row.android.source.length) failures.push(`${row.id}: missing Android source evidence`);
  if (row.status === 'verified') {
    if (!row.windows.rustSource.length && !row.windows.uiEntry.length) failures.push(`${row.id}: missing Windows implementation evidence`);
    if (!row.windows.tests.length) failures.push(`${row.id}: missing Windows test evidence`);
    if (!row.evidence.android.length) failures.push(`${row.id}: missing Android release evidence`);
    if (!row.evidence.windows.length) failures.push(`${row.id}: missing Windows release evidence`);
    for (const path of [...row.windows.rustSource, ...row.windows.uiEntry]) {
      const filePath = path.split('#')[0];
      if (!existsSync(`${root}/${filePath}`)) failures.push(`${row.id}: evidence file not found ${filePath}`);
    }
    for (const path of [...row.windows.tests, ...row.evidence.android, ...row.evidence.windows]) {
      const filePath = path.split('#')[0];
      if (!existsSync(`${root}/${filePath}`)) failures.push(`${row.id}: test/evidence file not found ${filePath}`);
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
