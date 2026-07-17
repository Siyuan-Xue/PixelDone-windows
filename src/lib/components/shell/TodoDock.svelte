<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import type { DockAction, DockPlusPlacement } from '$lib/generated/ipc';
  import { orderedDockItems } from './dock';

  let {
    actions,
    placement,
    active,
    enabled,
    labelFor,
    addLabel,
    onAction,
    onAdd
  }: {
    actions: DockAction[];
    placement: DockPlusPlacement;
    active: (action: DockAction) => boolean;
    enabled: (action: DockAction) => boolean;
    labelFor: (action: DockAction) => string;
    addLabel: string;
    onAction: (action: DockAction, trigger: HTMLElement) => void;
    onAdd: (trigger: HTMLElement) => void;
  } = $props();

  const actionIcons = {
    SORT: 'sort',
    DEADLINE: 'calendar',
    HIDE_DONE: 'hide',
    DELETE_DONE: 'trash-check',
    BATCH_DELETE: 'batch-delete'
  } as const;
</script>

<div class="dock" data-placement={placement} aria-label="Dock">
  {#each orderedDockItems(actions, placement) as item, index (`${item.kind}-${item.kind === 'action' ? item.action : index}`)}
    {#if item.kind === 'plus'}
      <button
        class="dock-add"
        type="button"
        title={`${addLabel} · Ctrl+N`}
        aria-label={addLabel}
        onclick={(event) => onAdd(event.currentTarget)}
      ><span class="dock-plus-symbol" aria-hidden="true">+</span></button>
    {:else}
      <button
        class:active={active(item.action)}
        class="dock-action"
        data-action={item.action}
        type="button"
        disabled={!enabled(item.action)}
        title={labelFor(item.action)}
        aria-label={labelFor(item.action)}
        aria-pressed={active(item.action)}
        onclick={(event) => onAction(item.action, event.currentTarget)}
      ><Icon name={actionIcons[item.action]} size={22} active={item.action === 'SORT' && active(item.action)} /></button>
    {/if}
  {/each}
</div>
