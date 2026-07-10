<script lang="ts">
  type IconName = 'list' | 'trash' | 'settings' | 'plus' | 'back' | 'more' | 'close' | 'cloud' | 'check';
  let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<svg
  class="pixel-icon"
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="square"
  stroke-linejoin="miter"
  aria-hidden="true"
>
  {#if name === 'list'}
    <rect x="2" y="2" width="12" height="12" rx="1"></rect>
    <path d="M5 5h6M5 8h6M5 11h4"></path>
  {:else if name === 'trash'}
    <path d="M3 5h10M6 2h4l1 2H5l1-2ZM5 7v5M8 7v5M11 7v5M4 5l1 9h6l1-9"></path>
  {:else if name === 'settings'}
    <circle cx="8" cy="8" r="2.5"></circle>
    <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.5 1.5M11.5 11.5 13 13M13 3l-1.5 1.5M4.5 11.5 3 13"></path>
  {:else if name === 'plus'}
    <path d="M8 2v12M2 8h12"></path>
  {:else if name === 'back'}
    <path d="M9 3 4 8l5 5M4 8h9"></path>
  {:else if name === 'more'}
    <circle cx="3" cy="8" r=".7" fill="currentColor" stroke="none"></circle>
    <circle cx="8" cy="8" r=".7" fill="currentColor" stroke="none"></circle>
    <circle cx="13" cy="8" r=".7" fill="currentColor" stroke="none"></circle>
  {:else if name === 'close'}
    <path d="m3 3 10 10M13 3 3 13"></path>
  {:else if name === 'cloud'}
    <path d="M4 12h8a3 3 0 0 0 .4-6A4.5 4.5 0 0 0 4 6.5 2.8 2.8 0 0 0 4 12Z"></path>
  {:else if name === 'check'}
    <path d="m3 8 3 3 7-7"></path>
  {/if}
</svg>
