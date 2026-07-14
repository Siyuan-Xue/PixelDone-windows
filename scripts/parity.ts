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
  windowsTarget: {
    product: string;
    version: string;
    stage: 'release_candidate' | 'formal_release';
    evidence: string[];
    authorizedIncompleteRows?: string[];
  };
  rows: ParityRow[];
}

interface ParityBaselineManifest extends Omit<ParityManifest, 'windowsTarget'> {}

interface ParityOverrideGroup {
  id: string;
  rows: string[];
  status: ParityStatus;
  windowsEvidence?: string[];
  windowsTests?: string[];
}

interface ParityReleaseOverlay {
  schemaVersion: number;
  baselineManifest: string;
  windowsTarget: ParityManifest['windowsTarget'];
  overrideGroups: ParityOverrideGroup[];
  rowOverrides?: Record<string, Partial<ParityRow>>;
}

export const CURRENT_PARITY_MANIFEST = 'parity/pixeldone-3.2.3.yaml';

export async function loadManifest(): Promise<ParityManifest> {
  const overlayUrl = new URL(`../${CURRENT_PARITY_MANIFEST}`, import.meta.url);
  const overlay = JSON.parse(await Bun.file(overlayUrl).text()) as ParityReleaseOverlay;
  const baseline = JSON.parse(
    await Bun.file(new URL(overlay.baselineManifest, overlayUrl)).text()
  ) as ParityBaselineManifest;
  const overrides = new Map<string, ParityOverrideGroup>();

  for (const group of overlay.overrideGroups) {
    for (const rowId of group.rows) {
      if (overrides.has(rowId)) throw new Error(`Duplicate parity override for ${rowId}`);
      overrides.set(rowId, group);
    }
  }

  const knownRows = new Set(baseline.rows.map((row) => row.id));
  for (const rowId of overrides.keys()) {
    if (!knownRows.has(rowId)) throw new Error(`Unknown parity override row ${rowId}`);
  }

  return {
    schemaVersion: overlay.schemaVersion,
    baseline: baseline.baseline,
    windowsTarget: overlay.windowsTarget,
    rows: baseline.rows.map((row) => {
      const patch = overlay.rowOverrides?.[row.id];
      const patched = patch ? {
        ...row,
        ...patch,
        android: { ...row.android, ...patch.android },
        windows: { ...row.windows, ...patch.windows },
        evidence: { ...row.evidence, ...patch.evidence }
      } : row;
      const override = overrides.get(row.id);
      if (!override) return patched;
      return {
        ...patched,
        status: override.status,
        windows: {
          ...patched.windows,
          tests: [...new Set([...patched.windows.tests, ...(override.windowsTests ?? [])])]
        },
        evidence: {
          ...patched.evidence,
          windows: [
            ...new Set([...patched.evidence.windows, ...(override.windowsEvidence ?? [])])
          ]
        }
      };
    })
  };
}

export function summarize(manifest: ParityManifest) {
  const required = manifest.rows.filter((row) => row.requiredForRelease);
  const counts = { verified: 0, blocked: 0, in_progress: 0, not_started: 0 };
  for (const row of required) counts[row.status] += 1;
  const percent = required.length === 0 ? 0 : (counts.verified / required.length) * 100;
  return { required: required.length, counts, percent };
}
