import type { Options } from '@wdio/types';
import { rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const e2eDataRoot = join(tmpdir(), 'pixeldone-windows-e2e');
process.env.PIXELDONE_DATA_ROOT = e2eDataRoot;
process.env.PIXELDONE_CREDENTIAL_TARGET = 'com.milesxue.pixeldone.windows/e2e-session';
process.env.PIXELDONE_CLEAR_CREDENTIALS_ON_START = 'true';

export const config: Options.Testrunner = {
  runner: 'local',
  specs: ['./e2e/specs/**/*.e2e.ts'],
  maxInstances: 1,
  services: [['@wdio/tauri-service', {
    driverProvider: 'embedded',
    appBinaryPath: './src-tauri/target/debug/PixelDone.exe',
    embeddedStartupTimeout: 60_000
  }]],
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': {
      application: './src-tauri/target/debug/PixelDone.exe'
    }
  }],
  logLevel: 'info',
  bail: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000
  },
  onPrepare() {
    rmSync(e2eDataRoot, { recursive: true, force: true });
  }
};
