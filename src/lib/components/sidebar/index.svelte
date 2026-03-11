<script lang="ts">
  import { Plus, Upload, Download } from 'lucide-svelte';
  import CollectionItem from './CollectionItem.svelte';
  import CollectionSettingsModal from '$lib/components/CollectionSettingsModal.svelte';
  import CollectionEnvironmentModal from '$lib/components/CollectionEnvironmentModal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
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
  let environmentCollection = $state<Collection | null>(null);
  let deleteCollectionId = $state<string | null>(null);
  let deleteCollectionName = $state('');
  let importExportRef: { triggerImport: () => void; triggerExport: () => void } | undefined;

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

  function confirmDeleteCollection(id: string, name: string) {
    deleteCollectionId = id;
    deleteCollectionName = name;
  }

  function executeDelete() {
    if (deleteCollectionId) {
      deleteCollection(deleteCollectionId);
      deleteCollectionId = null;
      deleteCollectionName = '';
    }
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="px-4 h-toolbar flex items-center justify-between shrink-0">
    <span class="text-title text-ink">Collections</span>
    <div class="flex items-center gap-1">
      <button
        onclick={() => importExportRef?.triggerImport()}
        class="w-7 h-7 flex items-center justify-center rounded-md text-ink-tertiary
               hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-colors"
        title="Import"
        aria-label="Import collection"
      >
        <Upload size={14} />
      </button>
      <button
        onclick={() => importExportRef?.triggerExport()}
        class="w-7 h-7 flex items-center justify-center rounded-md text-ink-tertiary
               hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-colors"
        title="Export"
        aria-label="Export collection"
      >
        <Download size={14} />
      </button>
      <button
        onclick={() => { creating = true; }}
        class="w-7 h-7 flex items-center justify-center rounded-md text-accent
               hover:bg-accent/10 transition-colors"
        title="New collection"
        aria-label="Create new collection"
      >
        <Plus size={16} />
      </button>
    </div>
  </div>

  <!-- Separator -->
  <div class="hig-separator"></div>

  <!-- New collection input -->
  {#if creating}
    <div class="px-4 py-3 shrink-0">
      <input
        bind:value={newName}
        onkeydown={handleKeydown}
        placeholder="Collection name..."
        aria-label="New collection name"
        class="w-full bg-surface text-ink text-body px-3 py-2 rounded-hig-sm border border-line-strong
               placeholder:text-ink-tertiary focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/30
               transition-all duration-200"
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
        onDelete={() => confirmDeleteCollection(collection.id, collection.name)}
        onSettings={() => settingsCollection = collection}
        onEnvironments={() => environmentCollection = collection}
      />
    {/each}

    {#if $collections.length === 0 && !creating}
      <div class="px-4 mt-12 text-center">
        <p class="text-body text-ink-tertiary">No collections yet</p>
        <button
          onclick={() => creating = true}
          class="mt-3 text-body text-accent hover:underline"
        >
          Create your first collection
        </button>
      </div>
    {/if}
  </div>

  <ImportExport bind:this={importExportRef} />

  <!-- Nginx Proxy Status -->
  <div class="hig-separator"></div>
  <div class="shrink-0 px-4 py-3 flex items-center gap-2">
    <span class="w-2 h-2 rounded-full shrink-0 {
      $proxyStatus === 'running' ? 'bg-run' :
      $proxyStatus === 'stopped' ? 'bg-ink-tertiary' : 'bg-warn'
    }"></span>
    <span class="text-caption text-ink-tertiary flex-1 uppercase tracking-widest">Nginx</span>
    {#if $proxyLoading}
      <span class="text-caption text-ink-tertiary">...</span>
    {:else if $proxyStatus === 'running'}
      <button
        onclick={stopProxy}
        class="text-caption text-ink-tertiary hover:text-err transition-colors"
      >Stop</button>
    {:else}
      <button
        onclick={startProxy}
        class="text-caption text-ink-tertiary hover:text-run transition-colors"
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

{#if environmentCollection}
  <CollectionEnvironmentModal
    collection={environmentCollection}
    onclose={() => environmentCollection = null}
  />
{/if}

{#if deleteCollectionId}
  <ConfirmModal
    title="Delete Collection"
    message={`Are you sure you want to delete "${deleteCollectionName}"? This will also delete all endpoints and scenarios. This action cannot be undone.`}
    confirmLabel="Delete Collection"
    onConfirm={executeDelete}
    onCancel={() => { deleteCollectionId = null; deleteCollectionName = ''; }}
  />
{/if}
