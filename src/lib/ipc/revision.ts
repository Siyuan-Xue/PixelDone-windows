export type MutationDisposition = 'apply' | 'ignore' | 'reload';

export function mutationDisposition(
  currentRevision: number,
  incomingRevision: number
): MutationDisposition {
  if (incomingRevision <= currentRevision) return 'ignore';
  if (incomingRevision === currentRevision + 1) return 'apply';
  return 'reload';
}
