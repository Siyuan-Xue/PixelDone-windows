<script lang="ts">
  import {
    scriptRunClass,
    splitScriptRuns,
    type PixelTextRole
  } from '$lib/typography/script-aware';

  let {
    text,
    role = 'sans'
  }: {
    text: string;
    role?: PixelTextRole;
  } = $props();

  let runs = $derived(splitScriptRuns(text));
</script>

{#each runs as run}<span class={scriptRunClass(run.script, role)}>{run.text}</span>{/each}
