<script lang="ts">
  import { Trash2, FolderOpen, Folder, Settings } from 'lucide-svelte';
  import { serverStatuses } from '$lib/stores/server';
  import type { Collection } from '$lib/types';

  interface Props {
    collection: Collection;
    active: boolean;
    onSelect: () => void;
    onDelete: () => void;
    onSettings: () => void;
  }

  let { collection, active, onSelect, onDelete, onSettings }: Props = $props();

  let running = $derived($serverStatuses[collection.id] === 'running');
  let starting = $derived($serverStatuses[collection.id] === 'starting');
</script>

<button
  class="w-full text-left px-3 py-2 flex items-center gap-2 group transition-colors duration-150 border-l-[2px]
    {active
      ? 'border-accent bg-hover text-ink'
      : 'border-transparent hover:bg-hover/60 text-ink-muted hover:text-ink'}"
  onclick={onSelect}
>
  {#if active}
    <FolderOpen size={13} class="shrink-0 text-accent" />
  {:else}
    <Folder size={13} class="shrink-0 text-ink-dim group-hover:text-ink-muted transition-colors duration-150" />
  {/if}

  <span class="flex-1 truncate text-xs font-inter">{collection.name}</span>

  <!-- Running / starting indicator -->
  {#if running}
    <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse shrink-0" title="Server running"></span>
  {:else if starting}
    <span class="w-1.5 h-1.5 rounded-full bg-warn animate-pulse shrink-0" title="Starting…"></span>
  {/if}

  <span class="text-[9px] font-mono text-ink-dim tabular-nums px-1.5 py-0.5 rounded bg-surface shrink-0">
    :{collection.port}
  </span>

  {#if collection.proxy_enabled}
    <span class="text-[9px] font-mono text-amber-400 shrink-0" title="Proxy enabled">↗</span>
  {/if}

  <div
    role="button"
    tabindex="0"
    class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded
           text-ink-dim hover:text-ink hover:bg-hover transition-all duration-150 shrink-0"
    onclick={(e) => { e.stopPropagation(); onSettings(); }}
    onkeydown={(e) => e.key === 'Enter' && onSettings()}
  >
    <Settings size={10} />
  </div>

  <div
    role="button"
    tabindex="0"
    class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded
           text-ink-dim hover:text-err hover:bg-err/10 transition-all duration-150 shrink-0"
    onclick={(e) => { e.stopPropagation(); onDelete(); }}
    onkeydown={(e) => e.key === 'Enter' && onDelete()}
  >
    <Trash2 size={10} />
  </div>
</button>
