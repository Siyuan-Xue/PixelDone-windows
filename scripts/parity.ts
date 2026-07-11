export type ParityStatus = 'not_started' | 'in_progress' | 'blocked' | 'verified';

export interface ParityRow {
  id: string;
  area: string;
  title: string;
  requiredForRelease: boolean;
  android: { source: string[]; tests: string[] };
  windows: { rustSource: string[]; uiEntry: string[]; tests: string[] };
  evidence: { android: string[]; windows: string[] };
  status: ParityStatus;
  variance: string;
}

export interface ParityManifest {
  schemaVersion: number;
  baseline: { product: string; version: string; versionCode: number; commit: string; roomSchema: number };
  rows: ParityRow[];
}

export async function loadManifest(): Promise<ParityManifest> {
  const file = Bun.file(new URL('../parity/pixeldone-3.1.0.yaml', import.meta.url));
  return JSON.parse(await file.text()) as ParityManifest;
}

export function summarize(manifest: ParityManifest) {
  const required = manifest.rows.filter((row) => row.requiredForRelease);
  const counts = { verified: 0, blocked: 0, in_progress: 0, not_started: 0 };
  for (const row of required) counts[row.status] += 1;
  const percent = required.length === 0 ? 0 : (counts.verified / required.length) * 100;
  return { required: required.length, counts, percent };
}
