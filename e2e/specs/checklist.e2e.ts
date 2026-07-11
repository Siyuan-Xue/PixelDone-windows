import { bootstrap, invoke } from '../helpers';

describe('Checklist parity', () => {
  it('creates, renames and deletes a normal checklist with revision protection', async () => {
    let snapshot = await bootstrap();
    const created = await invoke('create_checklist', { expectedRevision: snapshot.revision, name: 'E2E LIST' });
    const checklistId = created.changedIds[0];
    const renamed = await invoke('rename_checklist', {
      expectedRevision: created.revision,
      checklistId,
      name: 'RENAMED E2E'
    });
    snapshot = await bootstrap();
    expect(snapshot.checklists.find((list: any) => list.id === checklistId)?.name).toBe('RENAMED E2E');
    await expect(invoke('rename_checklist', {
      expectedRevision: renamed.revision - 1,
      checklistId,
      name: 'STALE'
    })).rejects.toThrow('当前界面状态已过期');
    await invoke('delete_checklist', { expectedRevision: renamed.revision, checklistId });
    snapshot = await bootstrap();
    expect(snapshot.checklists.some((list: any) => list.id === checklistId)).toBe(false);
    expect(snapshot.checklists.filter((list: any) => list.kind === 'NORMAL')).toHaveLength(1);
  });
});
