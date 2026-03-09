<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import type { Endpoint } from '$lib/types';

  interface Props {
    endpoints: Endpoint[];
    activeId: string | null;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onDelete: (id: string) => void;
  }

  let { endpoints, activeId, onSelect, onAdd, onDelete }: Props = $props();

  const METHOD_BADGE: Record<string, string> = {
    GET:    'bg-emerald-500/10 text-emerald-400',
    POST:   'bg-violet-500/10 text-violet-400',
    PUT:    'bg-sky-500/10 text-sky-400',
    DELETE: 'bg-rose-500/10 text-rose-400',
    PATCH:  'bg-amber-500/10 text-amber-400',
  };
</script>

<div class="flex flex-col h-full border-r border-line bg-[#191919]">
  <div class="flex items-center justify-between px-3 h-9 border-b border-line shrink-0">
    <span class="font-krub text-[10px] font-semibold text-ink-muted uppercase tracking-widest">Endpoints</span>
    <button
      onclick={onAdd}
      class="w-5 h-5 flex items-center justify-center rounded text-ink-dim hover:text-ink hover:bg-hover transition-colors duration-150"
      title="Add endpoint"
    >
      <Plus size={12} />
    </button>
  </div>

  <div class="flex-1 overflow-y-auto py-0.5">
    {#each endpoints as endpoint (endpoint.id)}
      <button
        onclick={() => onSelect(endpoint.id)}
        class="w-full text-left px-3 py-2 flex items-center gap-2.5 group transition-colors duration-150
          {activeId === endpoint.id
            ? 'bg-hover text-ink border-l-[2px] border-accent'
            : 'border-l-[2px] border-transparent hover:bg-hover/50'}"
      >
        <span class="text-[9px] font-mono font-bold w-11 shrink-0 text-center rounded px-1 py-0.5
          {METHOD_BADGE[endpoint.method] ?? 'bg-surface text-ink-muted'}">
          {endpoint.method}
        </span>
        <span class="flex-1 text-xs font-mono truncate
          {activeId === endpoint.id ? 'text-ink' : 'text-ink-muted group-hover:text-ink'} transition-colors duration-150">
          {endpoint.path}
        </span>
        <div
          role="button"
          tabindex="0"
          onclick={(e) => { e.stopPropagation(); onDelete(endpoint.id); }}
          onkeydown={(e) => e.key === 'Enter' && onDelete(endpoint.id)}
          class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded
                 text-ink-dim hover:text-err hover:bg-err/10 transition-all duration-150 shrink-0"
        >
          <Trash2 size={10} />
        </div>
      </button>
    {/each}

    {#if endpoints.length === 0}
      <p class="text-ink-dim text-[11px] text-center mt-8 px-4 leading-relaxed">No endpoints yet</p>
    {/if}
  </div>
</div>
