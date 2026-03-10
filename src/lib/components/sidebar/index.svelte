<script lang="ts">
  import { Plus, Search } from 'lucide-svelte';
  import CollectionItem from './CollectionItem.svelte';
  import CollectionSettingsModal from '$lib/components/CollectionSettingsModal.svelte';
  import ImportExport from './ImportExport.svelte';
  import {
    collections,
    activeCollectionId,
    addCollection,
    deleteCollection,
  } from '$lib/stores/collections';
  import type { Collection } from '$lib/types';
  import { proxyStatus, proxyLoading, startProxy, stopProxy, refreshProxyStatus } from '$lib/stores/proxy';
  import { onMount } from 'svelte';

  let newName = $state('');
  let creating = $state(false);
  let settingsCollection = $state<Collection | null>(null);

  onMount(() => {
    refreshProxyStatus();
    const interval = setInterval(refreshProxyStatus, 5000);
    return () => clearInterval(interval);
  });

  function handleCreate() {
    if (newName.trim()) {
      addCollection(newName.trim());
      newName = '';
      creating = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleCreate();
    if (e.key === 'Escape') { creating = false; newName = ''; }
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="px-3 h-11 flex items-center justify-between border-b border-line shrink-0">
    <span class="font-krub font-semibold text-[11px] text-ink-muted uppercase tracking-widest">Collections</span>
    <button
      onclick={() => { creating = true; }}
      class="w-6 h-6 flex items-center justify-center rounded text-ink-dim hover:text-ink hover:bg-hover transition-colors duration-150"
      title="New collection"
    >
      <Plus size={13} />
    </button>
  </div>

  <!-- New collection input (inline) -->
  {#if creating}
    <div class="px-3 py-2 border-b border-line shrink-0">
      <input
        bind:value={newName}
        onkeydown={handleKeydown}
        placeholder="Collection name…"
        class="w-full bg-surface text-ink text-xs px-2.5 py-1.5 rounded border border-line-strong
               placeholder:text-ink-dim focus:outline-none focus:border-accent transition-colors duration-150"
      />
    </div>
  {/if}

  <!-- Collection list -->
  <div class="flex-1 overflow-y-auto py-1">
    {#each $collections as collection (collection.id)}
      <CollectionItem
        {collection}
        active={$activeCollectionId === collection.id}
        onSelect={() => activeCollectionId.set(collection.id)}
        onDelete={() => deleteCollection(collection.id)}
        onSettings={() => settingsCollection = collection}
      />
    {/each}

    {#if $collections.length === 0 && !creating}
      <div class="px-4 mt-8 text-center">
        <p class="text-ink-dim text-[11px] leading-relaxed">No collections yet.<br/>Press + to create one.</p>
      </div>
    {/if}
  </div>

  <!-- Import/Export -->
  <ImportExport />

  <!-- Nginx Proxy Status -->
  <div class="shrink-0 border-t border-line px-3 py-2.5 flex items-center gap-2">
    <span class="w-1.5 h-1.5 rounded-full shrink-0 {
      $proxyStatus === 'running' ? 'bg-run' :
      $proxyStatus === 'stopped' ? 'bg-ink-dim' : 'bg-warn'
    }"></span>
    <span class="text-[10px] font-krub text-ink-dim flex-1 uppercase tracking-widest">nginx</span>
    {#if $proxyLoading}
      <span class="text-[10px] text-ink-dim">…</span>
    {:else if $proxyStatus === 'running'}
      <button
        onclick={stopProxy}
        class="text-[10px] font-krub text-ink-dim hover:text-err transition-colors"
      >Stop</button>
    {:else}
      <button
        onclick={startProxy}
        class="text-[10px] font-krub text-ink-dim hover:text-run transition-colors"
      >Start</button>
    {/if}
  </div>
</div>

{#if settingsCollection}
  <CollectionSettingsModal 
    collection={settingsCollection} 
    onClose={() => settingsCollection = null} 
  />
{/if}
