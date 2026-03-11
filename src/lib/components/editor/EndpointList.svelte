<script lang="ts">
  import { Plus, Trash2, Search, X, Copy, Terminal } from 'lucide-svelte';
  import CurlImportModal from './CurlImportModal.svelte';
  import type { Endpoint } from '$lib/types';

  interface Props {
    endpoints: Endpoint[];
    activeId: string | null;
    collectionId: string;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onDelete: (id: string) => void;
    onDuplicate: (id: string) => void;
  }

  let { endpoints, activeId, collectionId, onSelect, onAdd, onDelete, onDuplicate }: Props = $props();

  let showCurlModal = $state(false);
  let searchQuery = $state('');
  let methodFilter = $state<string | null>(null);

  const METHOD_BADGE: Record<string, string> = {
    GET:     'bg-emerald-500/12 text-emerald-400',
    POST:    'bg-violet-500/12 text-violet-400',
    PUT:     'bg-sky-500/12 text-sky-400',
    DELETE:  'bg-rose-500/12 text-rose-400',
    PATCH:   'bg-amber-500/12 text-amber-400',
    HEAD:    'bg-teal-500/12 text-teal-400',
    OPTIONS: 'bg-orange-500/12 text-orange-400',
  };

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];

  let filteredEndpoints = $derived(
    endpoints.filter(ep => {
      const matchesSearch = !searchQuery ||
        ep.path.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesMethod = !methodFilter || ep.method === methodFilter;
      return matchesSearch && matchesMethod;
    })
  );
</script>

<div class="flex flex-col h-full border-r border-line bg-surface">
  <!-- Search -->
  <div class="px-3 py-3 space-y-2 shrink-0">
    <div class="relative">
      <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-ink-tertiary" />
      <input
        bind:value={searchQuery}
        placeholder="Search endpoints..."
        aria-label="Search endpoints"
        class="w-full bg-surface-alt text-ink text-body pl-8 pr-8 py-2 rounded-hig-sm
               placeholder:text-ink-tertiary focus:outline-none focus:ring-2 focus:ring-accent/40 transition-all"
      />
      {#if searchQuery}
        <button
          onclick={() => searchQuery = ''}
          class="absolute right-2.5 top-1/2 -translate-y-1/2 text-ink-tertiary hover:text-ink"
        >
          <X size={12} />
        </button>
      {/if}
    </div>
    <div class="flex flex-wrap gap-1">
      {#each METHODS as method}
        <button
          onclick={() => methodFilter = methodFilter === method ? null : method}
          class="px-2 py-1 text-caption font-mono font-bold rounded-md transition-all duration-200
            {methodFilter === method
              ? METHOD_BADGE[method] + ' ring-1 ring-inset ring-current'
              : 'text-ink-tertiary hover:text-ink-secondary hover:bg-[var(--color-hover)]'}"
        >
          {method}
        </button>
      {/each}
    </div>
  </div>

  <div class="hig-separator"></div>

  <!-- Header -->
  <div class="flex items-center justify-between px-4 h-9 shrink-0">
    <span class="text-caption text-ink-tertiary uppercase tracking-widest">Endpoints</span>
    <div class="flex items-center gap-0.5">
      <button
        onclick={() => showCurlModal = true}
        class="w-6 h-6 flex items-center justify-center rounded-md text-ink-tertiary
               hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-colors"
        title="Import cURL"
      >
        <Terminal size={13} />
      </button>
      <button
        onclick={onAdd}
        class="w-6 h-6 flex items-center justify-center rounded-md text-accent
               hover:bg-accent/10 transition-colors"
        title="Add endpoint"
      >
        <Plus size={14} />
      </button>
    </div>
  </div>

  {#if showCurlModal}
    <CurlImportModal {collectionId} onclose={() => showCurlModal = false} />
  {/if}

  <div class="flex-1 overflow-y-auto py-1">
    {#each filteredEndpoints as endpoint (endpoint.id)}
      <button
        onclick={() => onSelect(endpoint.id)}
        class="w-full text-left px-3 py-2.5 flex items-center gap-3 group transition-all duration-200
               rounded-hig-sm mx-1
          {activeId === endpoint.id
            ? 'bg-accent/12 text-ink'
            : 'hover:bg-[var(--color-hover)] text-ink-secondary'}"
        style="width: calc(100% - 8px);"
      >
        <span class="text-caption font-mono font-bold w-12 shrink-0 text-center rounded-md px-1.5 py-1
          {METHOD_BADGE[endpoint.method] ?? 'bg-surface text-ink-tertiary'}">
          {endpoint.method}
        </span>
        <span class="flex-1 min-w-0 flex flex-col">
          <span class="text-body font-mono truncate
            {activeId === endpoint.id ? 'text-ink' : 'text-ink-secondary group-hover:text-ink'} transition-colors">
            {endpoint.path}
          </span>
          {#if endpoint.description}
            <span class="text-caption text-ink-tertiary truncate leading-tight">{endpoint.description}</span>
          {/if}
        </span>
        <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 shrink-0 transition-opacity">
          <div
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); onDuplicate(endpoint.id); }}
            onkeydown={(e) => e.key === 'Enter' && onDuplicate(endpoint.id)}
            class="w-6 h-6 flex items-center justify-center rounded-md text-ink-tertiary
                   hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-colors"
            title="Duplicate"
            aria-label="Duplicate endpoint"
          >
            <Copy size={12} />
          </div>
          <div
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); onDelete(endpoint.id); }}
            onkeydown={(e) => e.key === 'Enter' && onDelete(endpoint.id)}
            class="w-6 h-6 flex items-center justify-center rounded-md text-ink-tertiary
                   hover:text-err hover:bg-err/10 transition-colors"
            title="Delete"
            aria-label="Delete endpoint"
          >
            <Trash2 size={12} />
          </div>
        </div>
      </button>
    {/each}

    {#if filteredEndpoints.length === 0}
      <p class="text-body text-ink-tertiary text-center mt-12 px-4">
        {#if searchQuery || methodFilter}
          No matching endpoints
        {:else}
          No endpoints yet
        {/if}
      </p>
    {/if}
  </div>
</div>
