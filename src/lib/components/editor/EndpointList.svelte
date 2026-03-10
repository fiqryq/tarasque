<script lang="ts">
  import { Plus, Trash2, Search, X } from 'lucide-svelte';
  import type { Endpoint } from '$lib/types';

  interface Props {
    endpoints: Endpoint[];
    activeId: string | null;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onDelete: (id: string) => void;
  }

  let { endpoints, activeId, onSelect, onAdd, onDelete }: Props = $props();

  let searchQuery = $state('');
  let methodFilter = $state<string | null>(null);

  const METHOD_BADGE: Record<string, string> = {
    GET:    'bg-emerald-500/10 text-emerald-400',
    POST:   'bg-violet-500/10 text-violet-400',
    PUT:    'bg-sky-500/10 text-sky-400',
    DELETE: 'bg-rose-500/10 text-rose-400',
    PATCH:  'bg-amber-500/10 text-amber-400',
  };

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'];

  let filteredEndpoints = $derived(
    endpoints.filter(ep => {
      const matchesSearch = !searchQuery || 
        ep.path.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesMethod = !methodFilter || ep.method === methodFilter;
      return matchesSearch && matchesMethod;
    })
  );

  function clearFilters() {
    searchQuery = '';
    methodFilter = null;
  }
</script>

<div class="flex flex-col h-full border-r border-line bg-surface">
  <!-- Search + Filter -->
  <div class="px-3 py-2 border-b border-line space-y-2 shrink-0">
    <div class="relative">
      <Search size={12} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-dim" />
      <input
        bind:value={searchQuery}
        placeholder="Search endpoints..."
        class="w-full bg-surface text-ink text-xs pl-7 pr-7 py-1.5 rounded border border-line-strong
               placeholder:text-ink-dim focus:outline-none focus:border-accent transition-colors"
      />
      {#if searchQuery}
        <button
          onclick={() => searchQuery = ''}
          class="absolute right-2 top-1/2 -translate-y-1/2 text-ink-dim hover:text-ink"
        >
          <X size={10} />
        </button>
      {/if}
    </div>
    <div class="flex gap-1">
      {#each METHODS as method}
        <button
          onclick={() => methodFilter = methodFilter === method ? null : method}
          class="px-2 py-1 text-[9px] font-mono font-bold rounded transition-colors
            {methodFilter === method 
              ? METHOD_BADGE[method] + ' ring-1 ring-inset ring-current' 
              : 'bg-surface text-ink-dim hover:text-ink'}"
        >
          {method}
        </button>
      {/each}
    </div>
  </div>

  <!-- Header -->
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
    {#each filteredEndpoints as endpoint (endpoint.id)}
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

    {#if filteredEndpoints.length === 0}
      <p class="text-ink-dim text-[11px] text-center mt-8 px-4 leading-relaxed">
        {#if searchQuery || methodFilter}
          No matching endpoints
        {:else}
          No endpoints yet
        {/if}
      </p>
    {/if}
  </div>
</div>
