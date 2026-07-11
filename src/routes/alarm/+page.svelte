<script lang="ts">
  import { onMount } from 'svelte';
  import { api, applyMutation } from '$lib/ipc/client';
  import type { AppSnapshot } from '$lib/generated/ipc';

  let snapshot = $state<AppSnapshot | null>(null);
  let error = $state('');
  let todoIds = $state<string[]>([]);

  onMount(async () => {
    todoIds = new URLSearchParams(location.search).get('todoIds')?.split(',').filter(Boolean) ?? [];
    snapshot = await api.bootstrap();
  });

  let todos = $derived(
    snapshot?.checklists.flatMap((list) => list.items).filter((item) => todoIds.includes(item.id)) ?? []
  );

  async function act(kind: 'stop' | 'snooze') {
    if (!snapshot) return;
    try {
      const result = kind === 'stop'
        ? await api.stopReminder(snapshot.revision, todoIds)
        : await api.snoozeReminder(snapshot.revision, todoIds);
      snapshot = applyMutation(snapshot, result);
    } catch (value) {
      error = value instanceof Error ? value.message : String(value);
    }
  }
</script>

<svelte:head><title>PixelDone XHIGH</title></svelte:head>

<main class="alarm-shell">
  <span class="alarm-label">XHIGH REMINDER</span>
  <h1>PixelDone</h1>
  <div class="alarm-list">
    {#each todos as todo}
      <p>{todo.title}</p>
    {/each}
  </div>
  {#if error}<strong class="alarm-error">{error}</strong>{/if}
  <div class="alarm-actions">
    <button onclick={() => void act('stop')}>STOP</button>
    <button class="snooze" onclick={() => void act('snooze')}>SNOOZE 10 MIN</button>
  </div>
</main>

<style>
  :global(html, body) { margin: 0; min-height: 100%; background: #141413; color: #faf9f5; font-family: "Cascadia Mono", Consolas, monospace; }
  .alarm-shell { min-height: 100vh; box-sizing: border-box; border: 6px solid #ea4335; padding: 32px; display: flex; flex-direction: column; gap: 16px; }
  .alarm-label { color: #ea4335; font-weight: 800; letter-spacing: .08em; }
  h1 { margin: 0; font-size: 32px; }
  .alarm-list { flex: 1; border: 2px solid #5e5d59; padding: 16px; }
  .alarm-list p { margin: 0 0 12px; font-size: 18px; }
  .alarm-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  button { min-height: 52px; border: 2px solid #faf9f5; border-radius: 0; background: #ea4335; color: #141413; font: inherit; font-weight: 800; }
  button.snooze { background: transparent; color: #faf9f5; }
  .alarm-error { color: #df6666; }
</style>
