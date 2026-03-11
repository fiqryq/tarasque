<script lang="ts">
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import { collections } from '$lib/stores/collections';
  import { serverStatuses } from '$lib/stores/server';

  interface Props {
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { selectedId, onSelect }: Props = $props();

  const METHOD_COLORS: Record<string, string> = {
    GET:    'text-emerald-400',
    POST:   'text-violet-400',
    PUT:    'text-sky-400',
    DELETE: 'text-rose-400',
    PATCH:  'text-amber-400',
  };

  const METHOD_BG: Record<string, string> = {
    GET:    'bg-emerald-500/10',
    POST:   'bg-violet-500/10',
    PUT:    'bg-sky-500/10',
    DELETE: 'bg-rose-500/10',
    PATCH:  'bg-amber-500/10',
  };

  let runningCollections = $derived(
    $collections.filter((c) => $serverStatuses[c.id] === 'running')
  );

  // Track which collections are expanded (all expanded by default)
  let expanded = $state<Record<string, boolean>>({});

  function isExpanded(id: string): boolean {
    return expanded[id] !== false; // default open
  }

  function toggle(id: string) {
    expanded[id] = !isExpanded(id);
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="px-3 h-11 flex items-center gap-2 border-b border-line shrink-0">
    <span class="font-semibold text-body text-ink-secondary uppercase tracking-widest flex-1">
      Running APIs
    </span>
    {#if runningCollections.length > 0}
      <span class="text-caption font-mono text-run bg-run/10 px-1.5 py-0.5 rounded-hig-sm">
        {runningCollections.length} live
      </span>
    {/if}
  </div>

  <!-- List -->
  <div class="flex-1 overflow-y-auto">
    {#if runningCollections.length === 0}
      <div class="px-4 mt-10 text-center">
        <div class="w-8 h-8 rounded-hig-sm bg-surface flex items-center justify-center mx-auto mb-3">
          <span class="text-ink-tertiary text-base">○</span>
        </div>
        <p class="text-ink-tertiary text-body leading-relaxed">
          No servers running.<br />Start one from Collections.
        </p>
      </div>
    {:else}
      {#each runningCollections as collection (collection.id)}
        <div class="border-b border-line last:border-0">
          <!-- Collection row -->
          <button
            onclick={() => { toggle(collection.id); onSelect(collection.id); }}
            class="w-full flex items-center gap-2 px-3 py-2 hover:bg-[var(--color-hover)] transition-colors duration-100 group
                   {selectedId === collection.id ? 'bg-accent/12' : ''}"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse shrink-0"></span>

            {#if isExpanded(collection.id)}
              <ChevronDown size={11} class="text-ink-tertiary shrink-0" />
            {:else}
              <ChevronRight size={11} class="text-ink-tertiary shrink-0" />
            {/if}

            <span class="flex-1 text-body font-sf font-medium {selectedId === collection.id ? 'text-accent' : 'text-ink'} truncate text-left">
              {collection.name}
            </span>
            <span class="text-caption font-mono text-ink-tertiary shrink-0">:{collection.port}</span>
          </button>

          <!-- Endpoints list -->
          {#if isExpanded(collection.id)}
            <div class="pb-1">
              {#if collection.endpoints.length === 0}
                <div class="pl-8 pr-3 py-1.5 text-caption text-ink-tertiary italic">
                  No endpoints defined
                </div>
              {:else}
                {#each collection.endpoints as endpoint (endpoint.id)}
                  <div
                    class="w-full flex items-center gap-2 pl-8 pr-3 py-1.5
                           transition-colors duration-100"
                  >
                    <span class="text-caption font-mono font-bold px-1.5 py-0.5 rounded-hig-sm shrink-0
                                 {METHOD_COLORS[endpoint.method] ?? 'text-ink-secondary'}
                                 {METHOD_BG[endpoint.method] ?? 'bg-surface'}">
                      {endpoint.method}
                    </span>
                    <span class="flex-1 text-body font-mono text-ink-secondary truncate text-left">
                      {endpoint.path}
                    </span>
                  </div>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Footer: total endpoint count -->
  {#if runningCollections.length > 0}
    <div class="px-3 py-2 border-t border-line shrink-0">
      <span class="text-caption font-mono text-ink-tertiary">
        {runningCollections.reduce((n, c) => n + c.endpoints.length, 0)} endpoints across {runningCollections.length} server{runningCollections.length !== 1 ? 's' : ''}
      </span>
    </div>
  {/if}
</div>
