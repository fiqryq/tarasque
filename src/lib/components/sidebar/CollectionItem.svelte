<script lang="ts">
  import { Trash2, FolderOpen, Folder } from 'lucide-svelte';
  import type { Collection } from '$lib/types';

  interface Props {
    collection: Collection;
    active: boolean;
    onSelect: () => void;
    onDelete: () => void;
  }

  let { collection, active, onSelect, onDelete }: Props = $props();
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

  <span class="text-[9px] font-mono text-ink-dim tabular-nums px-1.5 py-0.5 rounded bg-surface shrink-0">
    :{collection.port}
  </span>

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
