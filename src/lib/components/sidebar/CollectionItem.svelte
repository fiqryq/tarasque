<script lang="ts">
  import { Trash2, FolderOpen, Folder, Settings, Leaf } from 'lucide-svelte';
  import { serverStatuses } from '$lib/stores/server';
  import type { Collection } from '$lib/types';

  interface Props {
    collection: Collection;
    active: boolean;
    onSelect: () => void;
    onDelete: () => void;
    onSettings: () => void;
    onEnvironments: () => void;
  }

  let { collection, active, onSelect, onDelete, onSettings, onEnvironments }: Props = $props();

  let running = $derived($serverStatuses[collection.id] === 'running');
  let starting = $derived($serverStatuses[collection.id] === 'starting');
</script>

<button
  class="w-full text-left px-4 py-2.5 flex items-center gap-3 group transition-all duration-200 rounded-hig-sm mx-1
    {active
      ? 'bg-accent/12 text-ink'
      : 'hover:bg-[var(--color-hover)] text-ink-secondary hover:text-ink'}"
  style="width: calc(100% - 8px);"
  onclick={onSelect}
>
  {#if active}
    <FolderOpen size={16} class="shrink-0 text-accent" />
  {:else}
    <Folder size={16} class="shrink-0 text-ink-tertiary group-hover:text-ink-secondary transition-colors" />
  {/if}

  <span class="flex-1 truncate text-body">{collection.name}</span>

  {#if running}
    <span class="w-2 h-2 rounded-full bg-run animate-pulse shrink-0" title="Server running"></span>
  {:else if starting}
    <span class="w-2 h-2 rounded-full bg-warn animate-pulse shrink-0" title="Starting..."></span>
  {/if}

  <span class="text-caption font-mono text-ink-tertiary tabular-nums shrink-0">
    :{collection.port}
  </span>

  {#if collection.proxy_enabled}
    <span class="text-caption font-mono text-warn shrink-0" title="Proxy enabled">↗</span>
  {/if}

  <!-- Actions (hover reveal) -->
  <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 shrink-0 transition-opacity duration-150" style="-webkit-app-region: no-drag;">
    <div
      role="button"
      tabindex="0"
      class="w-6 h-6 flex items-center justify-center rounded-md
             text-ink-tertiary hover:text-run hover:bg-run/10 transition-colors"
      onclick={(e) => { e.stopPropagation(); onEnvironments(); }}
      onkeydown={(e) => e.key === 'Enter' && onEnvironments()}
      title="Environments"
      aria-label="Edit environments"
    >
      <Leaf size={12} />
    </div>
    <div
      role="button"
      tabindex="0"
      class="w-6 h-6 flex items-center justify-center rounded-md
             text-ink-tertiary hover:text-ink hover:bg-[var(--color-hover)] transition-colors"
      onclick={(e) => { e.stopPropagation(); onSettings(); }}
      onkeydown={(e) => e.key === 'Enter' && onSettings()}
      aria-label="Collection settings"
    >
      <Settings size={12} />
    </div>
    <div
      role="button"
      tabindex="0"
      class="w-6 h-6 flex items-center justify-center rounded-md
             text-ink-tertiary hover:text-err hover:bg-err/10 transition-colors"
      onclick={(e) => { e.stopPropagation(); onDelete(); }}
      onkeydown={(e) => e.key === 'Enter' && onDelete()}
      aria-label="Delete collection"
    >
      <Trash2 size={12} />
    </div>
  </div>
</button>
