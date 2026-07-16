import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')) as { version: string };
const version = packageJson.version;
const directory = join(root, 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle', 'nsis');
const fileName = `PixelDone_${version}_x64-setup.exe`;
const artifact = join(directory, fileName);
const signature = readFileSync(`${artifact}.sig`, 'utf8').trim();
const digest = createHash('sha256').update(readFileSync(artifact)).digest('hex').toUpperCase();
const releaseNotes = readFileSync(join(root, 'RELEASE_NOTES.md'), 'utf8');
const notes = releaseNotes
  .split(/\r?\n\r?\n/)
  .map((paragraph) => paragraph.replace(/^#+\s+.*$/gm, '').trim())
  .find(Boolean) ?? `PixelDone for Windows ${version}`;

function manifest(url: string): string {
  return `${JSON.stringify({
    version,
    notes,
    pub_date: new Date().toISOString(),
    platforms: {
      'windows-x86_64': { signature, url }
    }
  }, null, 2)}\n`;
}

writeFileSync(join(directory, `${fileName}.sha256`), `${digest}  ${fileName}\n`, 'utf8');
writeFileSync(
  join(directory, 'latest.json'),
  manifest(`https://github.com/Siyuan-Xue/PixelDone-windows/releases/download/v${version}/${fileName}`),
  'utf8'
);
writeFileSync(
  join(directory, 'latest-gitee.json'),
  manifest(`https://gitee.com/milesxue/pixel-done-windows/releases/download/v${version}/${fileName}`),
  'utf8'
);

console.log(`Generated ${fileName}.sha256, latest.json and latest-gitee.json in ${dirname(artifact)}`);
