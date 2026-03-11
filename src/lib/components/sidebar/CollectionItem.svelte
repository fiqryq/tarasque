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
  class="w-full text-left px-3 py-2 flex items-center justify-between group transition-all duration-200 rounded-hig-sm mx-1
    {active
      ? 'bg-accent/12 text-ink'
      : 'hover:bg-[var(--color-hover)] text-ink-secondary hover:text-ink'}"
  style="width: calc(100% - 8px);"
  onclick={onSelect}
>
  <div class="flex items-center gap-2 min-w-0 flex-1">
    {#if active}
      <FolderOpen size={14} class="shrink-0 text-accent" />
    {:else}
      <Folder size={14} class="shrink-0 text-ink-tertiary group-hover:text-ink-secondary transition-colors" />
    {/if}
    <span class="truncate text-body">{collection.name}</span>
  </div>

  <div class="flex items-center gap-1.5 shrink-0 ml-2">
    {#if running}
      <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse" title="Server running"></span>
    {:else if starting}
      <span class="w-1.5 h-1.5 rounded-full bg-warn animate-pulse" title="Starting..."></span>
    {/if}

    <span class="text-caption font-mono text-ink-tertiary tabular-nums">
      :{collection.port}
    </span>

    <!-- Actions (always visible) -->
    <div class="flex items-center gap-0 shrink-0" style="-webkit-app-region: no-drag;">
      <div
        role="button"
        tabindex="0"
        class="w-5 h-5 flex items-center justify-center rounded-md
               text-ink-tertiary hover:text-run hover:bg-run/10 transition-colors"
        onclick={(e) => { e.stopPropagation(); onEnvironments(); }}
        onkeydown={(e) => e.key === 'Enter' && onEnvironments()}
        title="Environments"
        aria-label="Edit environments"
      >
        <Leaf size={11} />
      </div>
      <div
        role="button"
        tabindex="0"
        class="w-5 h-5 flex items-center justify-center rounded-md
               text-ink-tertiary hover:text-ink hover:bg-[var(--color-hover)] transition-colors"
        onclick={(e) => { e.stopPropagation(); onSettings(); }}
        onkeydown={(e) => e.key === 'Enter' && onSettings()}
        aria-label="Collection settings"
      >
        <Settings size={11} />
      </div>
      <div
        role="button"
        tabindex="0"
        class="w-5 h-5 flex items-center justify-center rounded-md
               text-ink-tertiary hover:text-err hover:bg-err/10 transition-colors"
        onclick={(e) => { e.stopPropagation(); onDelete(); }}
        onkeydown={(e) => e.key === 'Enter' && onDelete()}
        aria-label="Delete collection"
      >
        <Trash2 size={11} />
      </div>
    </div>
  </div>
</button>
