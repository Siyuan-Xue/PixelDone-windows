import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = join(import.meta.dir, '..');

describe('3.2.1 version contract', () => {
  test('keeps frontend, Tauri, Cargo, updater and E2E versions aligned', () => {
    const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
    const tauriConfig = JSON.parse(readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'));
    const cargoToml = readFileSync(join(root, 'src-tauri', 'Cargo.toml'), 'utf8');
    const cargoLock = readFileSync(join(root, 'src-tauri', 'Cargo.lock'), 'utf8');
    const releaseManifest = readFileSync(join(root, 'scripts', 'generate-release-manifest.ts'), 'utf8');
    const previewClient = readFileSync(join(root, 'src', 'lib', 'ipc', 'client.ts'), 'utf8');
    const releaseIntegrity = readFileSync(join(root, 'src-tauri', 'tests', 'release_integrity.rs'), 'utf8');
    const parityManifest = JSON.parse(
      readFileSync(join(root, 'parity', 'pixeldone-3.2.1.yaml'), 'utf8')
    );
    const updateE2e = readFileSync(join(root, 'e2e', 'specs', 'update.e2e.ts'), 'utf8');
    const candidateEvidence = readFileSync(
      join(root, 'parity', 'evidence', 'windows', 'candidate-3.2.1.md'),
      'utf8'
    );

    expect(packageJson.version).toBe('3.2.1');
    expect(packageJson.scripts.build).toBe('svelte-kit sync && vite build');
    expect(packageJson.scripts['build:e2e']).toBe('svelte-kit sync && vite build --mode e2e');
    expect(tauriConfig.version).toBe('3.2.1');
    expect(cargoToml).toMatch(/^version = "3\.2\.1"$/m);
    expect(cargoLock).toMatch(/name = "pixeldone-windows"\r?\nversion = "3\.2\.1"/);
    expect(releaseManifest).toContain("version: '3.2.1'");
    expect(releaseManifest).toContain('PixelDone_3.2.1_x64-setup.exe');
    expect(previewClient).toContain("currentVersion: '3.2.1'");
    expect(releaseIntegrity).toContain('PixelDone_3.2.1_x64-setup.exe');
    expect(parityManifest.baselineManifest).toBe('pixeldone-3.1.0.yaml');
    expect(parityManifest.windowsTarget.version).toBe('3.2.1');
    expect(parityManifest.windowsTarget.stage).toBe('formal_release');
    expect(updateE2e).toContain("currentVersion).toBe('3.2.1')");
    expect(candidateEvidence).toContain('# Windows 3.2.1 candidate verification evidence');
  });
});
