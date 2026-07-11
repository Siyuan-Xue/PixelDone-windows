<script lang="ts">
  import PixelDoneApp from '$lib/components/shell/PixelDoneApp.svelte';
  import '$lib/styles/tokens.css';
  import '$lib/styles/base.css';
  import '$lib/styles/layout.css';
  import '$lib/styles/motion.css';

  if (import.meta.env.MODE === 'e2e') {
    void import('@wdio/tauri-plugin');
  }
</script>

<PixelDoneApp />
