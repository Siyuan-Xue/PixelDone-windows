import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const directory = join(root, 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle', 'nsis');
const fileName = 'PixelDone_3.2.1_x64-setup.exe';
const artifact = join(directory, fileName);
const signature = readFileSync(`${artifact}.sig`, 'utf8').trim();
const digest = createHash('sha256').update(readFileSync(artifact)).digest('hex').toUpperCase();

writeFileSync(join(directory, `${fileName}.sha256`), `${digest}  ${fileName}\n`, 'utf8');
writeFileSync(join(directory, 'latest.json'), `${JSON.stringify({
  version: '3.2.1',
  notes: 'PixelDone for Windows 3.2.1 refines the desktop UI and repairs stale Windows notification shortcut identities.',
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': {
      signature,
      url: `https://github.com/Siyuan-Xue/PixelDone-windows/releases/download/v3.2.1/${fileName}`
    }
  }
}, null, 2)}\n`, 'utf8');

console.log(`Generated ${fileName}.sha256 and latest.json in ${dirname(artifact)}`);
