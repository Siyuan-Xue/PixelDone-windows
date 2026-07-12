<script lang="ts">
  type IconName =
    | 'list'
    | 'trash'
    | 'settings'
    | 'plus'
    | 'back'
    | 'more'
    | 'close'
    | 'cloud'
    | 'check'
    | 'login'
    | 'logout'
    | 'sync'
    | 'image'
    | 'edit'
    | 'alarm'
    | 'sort'
    | 'calendar'
    | 'hide'
    | 'trash-check'
    | 'batch-delete';
  let { name, size = 16 }: { name: IconName; size?: number } = $props();
  let cloudAction = $derived(name === 'login' || name === 'logout' || name === 'sync');
</script>

<svg
  class="pixel-icon"
  width={size}
  height={size}
  viewBox={cloudAction ? '0 0 22 22' : '0 0 16 16'}
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
  {:else if name === 'login'}
    <path d="M11 3H19V19H11M3 11H16M11 6 16 11 11 16"></path>
  {:else if name === 'logout'}
    <g transform="translate(22 0) scale(-1 1)"><path d="M11 3H19V19H11M3 11H16M11 6 16 11 11 16"></path></g>
  {:else if name === 'sync'}
    <path d="M18 12C18 7 15 4 11 4M11 1 7 5l4 4M4 10c0 5 3 8 7 8M11 13l4 4-4 4"></path>
  {:else if name === 'image'}
    <rect x="2" y="3" width="12" height="10"></rect><path d="m3 11 3-3 2 2 2-3 3 4M5 6h.01"></path>
  {:else if name === 'edit'}
    <path d="M3 13h3l7-7-3-3-7 7v3ZM9 4l3 3M3 13h10"></path>
  {:else if name === 'alarm'}
    <circle cx="8" cy="8.5" r="4.5"></circle><path d="M4 3 2.5 4.5M12 3l1.5 1.5M8 8.5V5.8M8 8.5l2.2 1.2M5 13l-1 1M11 13l1 1"></path>
  {:else if name === 'sort'}
    <path d="M3 4h8M3 8h6M3 12h4M12 7v6M10 11l2 2 2-2"></path>
  {:else if name === 'calendar'}
    <rect x="2" y="3.5" width="12" height="10.5"></rect><path d="M2 7h12M5 2v3M11 2v3M8 9v2.5l2 1"></path>
  {:else if name === 'hide'}
    <path d="M2 8s2-3.5 6-3.5S14 8 14 8s-2 3.5-6 3.5S2 8 2 8ZM3 3l10 10"></path>
  {:else if name === 'trash-check'}
    <path d="M3 5h10M6 2h4l1 2H5l1-2ZM5 5l1 9h6l1-9M7 9l1.5 1.5L11 8"></path>
  {:else if name === 'batch-delete'}
    <path d="M2 4h6M2 8h6M2 12h6M10 5h4M11 3h2M10.5 7l.5 7h2l.5-7"></path>
  {/if}
</svg>
