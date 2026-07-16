import { describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = join(import.meta.dir, '..');

describe('3.2.7 version contract', () => {
  test('keeps frontend, Tauri, Cargo, updater and E2E versions aligned', () => {
    const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
    const tauriConfig = JSON.parse(readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'));
    const cargoToml = readFileSync(join(root, 'src-tauri', 'Cargo.toml'), 'utf8');
    const cargoLock = readFileSync(join(root, 'src-tauri', 'Cargo.lock'), 'utf8');
    const releaseManifest = readFileSync(join(root, 'scripts', 'generate-release-manifest.ts'), 'utf8');
    const previewClient = readFileSync(join(root, 'src', 'lib', 'ipc', 'client.ts'), 'utf8');
    const releaseIntegrity = readFileSync(join(root, 'src-tauri', 'tests', 'release_integrity.rs'), 'utf8');
    const releaseWorkflow = readFileSync(
      join(root, '.github', 'workflows', 'release-windows.yml'),
      'utf8'
    );
    const parityManifest = JSON.parse(
      readFileSync(join(root, 'parity', 'pixeldone-3.2.7.yaml'), 'utf8')
    );
    const updateE2e = readFileSync(join(root, 'e2e', 'specs', 'update.e2e.ts'), 'utf8');
    const candidateEvidence = readFileSync(
      join(root, 'parity', 'evidence', 'windows', 'candidate-3.2.7.md'),
      'utf8'
    );

    expect(packageJson.version).toBe('3.2.7');
    expect(packageJson.scripts.build).toBe('svelte-kit sync && vite build');
    expect(packageJson.scripts['build:e2e']).toBe('svelte-kit sync && vite build --mode e2e');
    expect(packageJson.scripts.test).toBe('svelte-kit sync && bun test');
    expect(tauriConfig.version).toBe('3.2.7');
    expect(cargoToml).toMatch(/^version = "3\.2\.7"$/m);
    expect(cargoLock).toMatch(/name = "pixeldone-windows"\r?\nversion = "3\.2\.7"/);
    expect(releaseManifest).toContain('const version = packageJson.version');
    expect(releaseManifest).toContain('latest-gitee.json');
    expect(releaseManifest).toContain('gitee.com/milesxue/pixel-done-windows/releases/download');
    expect(previewClient).toContain("currentVersion: '3.2.7'");
    expect(releaseIntegrity).toContain('env!("CARGO_PKG_VERSION")');
    expect(releaseWorkflow).toContain('- run: bun run test');
    expect(releaseWorkflow).toContain('GITEE_ACCESS_TOKEN');
    expect(releaseWorkflow).toContain('Publish-GiteeRelease.ps1');
    expect(releaseWorkflow).toContain("'latest.json', 'latest-gitee.json'");
    expect(releaseWorkflow).not.toContain('--clobber');
    expect(parityManifest.baselineManifest).toBe('pixeldone-3.1.0.yaml');
    expect(parityManifest.windowsTarget.version).toBe('3.2.7');
    expect(parityManifest.windowsTarget.stage).toBe('formal_release');
    expect(updateE2e).toContain("currentVersion).toBe('3.2.7')");
    expect(candidateEvidence).toContain('# Windows 3.2.7 formal release evidence');
  });

  test('preserves the migration 7 checksum deployed by 3.2.0', () => {
    const attributes = readFileSync(join(root, '.gitattributes'), 'utf8');
    const migration = readFileSync(
      join(root, 'src-tauri', 'migrations', '0007_attachment_sync.sql')
    );

    expect(attributes).toContain(
      'src-tauri/migrations/0007_attachment_sync.sql text eol=lf'
    );
    expect(createHash('sha384').update(migration).digest('hex').toUpperCase()).toBe(
      '9606CFB487A71F9661010578B3E0D527A5A44B0B9D554650F1E6D524D086594560669F8548893FC6E85DECC900BE1CC6'
    );
  });
});
