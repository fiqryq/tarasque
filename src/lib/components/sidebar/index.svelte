<script lang="ts">
  import { Plus, Search } from 'lucide-svelte';
  import CollectionItem from './CollectionItem.svelte';
  import {
    collections,
    activeCollectionId,
    addCollection,
    deleteCollection,
  } from '$lib/stores/collections';

  let newName = $state('');
  let creating = $state(false);

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
      />
    {/each}

    {#if $collections.length === 0 && !creating}
      <div class="px-4 mt-8 text-center">
        <p class="text-ink-dim text-[11px] leading-relaxed">No collections yet.<br/>Press + to create one.</p>
      </div>
    {/if}
  </div>
</div>
