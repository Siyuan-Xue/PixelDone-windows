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
    | 'batch-delete'
    | 'key'
    | 'conflict'
    | 'alert'
    | 'update'
    | 'download'
    | 'folder'
    | 'restore'
    | 'export';
  let { name, size = 16, active = false }: { name: IconName; size?: number; active?: boolean } = $props();
  let cloudAction = $derived(name === 'login' || name === 'logout' || name === 'sync');
  let dockAction = $derived(['sort', 'calendar', 'hide', 'trash-check', 'batch-delete', 'export'].includes(name));
</script>

<svg
  class="pixel-icon"
  width={size}
  height={size}
  viewBox={cloudAction || dockAction ? '0 0 22 22' : '0 0 16 16'}
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
    <path d="M2 4h12M2 12h12"></path>
    <rect x="5" y="2" width="3" height="4"></rect>
    <rect x="9" y="10" width="3" height="4"></rect>
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
    <text x="11" y="11.5" fill="currentColor" stroke="none" text-anchor="middle" dominant-baseline="central" font-family="var(--font-ui)" font-size="18" font-weight="700">{active ? 'T' : 'P'}</text>
  {:else if name === 'calendar'}
    <rect x="4" y="5.5" width="14" height="12.5" stroke-width="2.2"></rect>
    <path d="M4 9h14" stroke-width="1.6"></path>
    <path d="M7 3.5v4M15 3.5v4" stroke-width="2.8"></path>
    <circle cx="11" cy="13.6" r="3" stroke-width="1.6"></circle>
    <path d="M11 13.6v-1.9M11 13.6l2.2 1.2" stroke-width="1.6"></path>
  {:else if name === 'hide'}
    <path d="M3 11 7.5 6.5h7L19 11l-4.5 4.5h-7Z" stroke-width="2.2"></path>
    <rect x="9" y="9" width="4" height="4" stroke-width="1.6"></rect>
    <path d="m5 4.5 12 13" stroke-width="2.8"></path>
  {:else if name === 'trash-check'}
    <path d="M5 6h12M8.5 3.5h5" stroke-width="2.8"></path>
    <rect x="6" y="8" width="10" height="10" stroke-width="2.2"></rect>
    <path d="m8.3 13.4 1.8 1.8 3.9-4.4" stroke-width="1.6"></path>
  {:else if name === 'batch-delete'}
    <path d="M3.2 5.5h8M3.2 11h8M3.2 16.5h8" stroke-width="1.6"></path>
    <path d="M13.6 6.5h6.6M15.5 4.3h2.8" stroke-width="2.2"></path>
    <rect x="14.5" y="8.5" width="4.8" height="9.3" stroke-width="1.6"></rect>
    <path d="M16 10.2v5.9M17.8 10.2v5.9" stroke-width="1.6"></path>
  {:else if name === 'export'}
    <rect x="4" y="3.5" width="10.5" height="15" stroke-width="2.2"></rect>
    <path d="M7 8h4.5M7 11.5h4.5M7 15h3" stroke-width="1.6"></path>
    <path d="m12.5 13 6.5-6.5M14.5 6.5H19V11" stroke-width="2.2"></path>
  {:else if name === 'key'}
    <circle cx="5" cy="8" r="3"></circle><path d="M8 8h6M11 8v3M13 8v2"></path>
  {:else if name === 'conflict'}
    <path d="M2 4h5l2 3h5M11 4h3v3M2 12h5l2-3h5M11 12h3V9"></path>
  {:else if name === 'alert'}
    <path d="M8 2v8" stroke-width="2.4"></path><circle cx="8" cy="13" r="1.2" fill="currentColor" stroke="none"></circle>
  {:else if name === 'update'}
    <path d="M13 6a5.5 5.5 0 1 0 .5 5M13 2v4H9"></path>
  {:else if name === 'download'}
    <path d="M8 2v8M4.5 7.5 8 11l3.5-3.5M3 14h10"></path>
  {:else if name === 'folder'}
    <path d="M2 4h5l1.5 2H14v7H2Z"></path>
  {:else if name === 'restore'}
    <path d="M5 5H2v-3M2.5 5A6 6 0 1 1 2 10"></path>
  {/if}
</svg>
