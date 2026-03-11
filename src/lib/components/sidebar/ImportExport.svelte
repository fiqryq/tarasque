<script lang="ts">
  import { Upload, Download } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { importCollections, collections } from '$lib/stores/collections';
  import { toasts } from '$lib/stores/toasts';
  import type { Collection } from '$lib/types';

  let fileInput: HTMLInputElement;
  let exportingCollection = $state<Collection | null>(null);

  async function handleImport(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const content = await file.text();
    const isTarasqueJson = file.name.endsWith('.tarasque.json');

    try {
      if (isTarasqueJson) {
        // Import as a Tarasque collection JSON directly
        const col: Collection = JSON.parse(content);
        col.id = crypto.randomUUID(); // fresh ID to avoid collisions
        importCollections([col]);
        toasts.show('success', `Imported collection: ${col.name}`);
      } else {
        // OpenAPI / Swagger import
        const imported = await invoke<Collection[]>('import_openapi', { content });
        importCollections(imported);
        toasts.show('success', `Imported ${imported.length} collection(s)`);
      }
    } catch (err) {
      toasts.show('error', `Import failed: ${err}`);
    }

    input.value = '';
  }

  async function handleExportJson() {
    if (!exportingCollection) return;

    try {
      const json = await invoke<string>('export_collection_json', {
        collection: exportingCollection,
      });
      const safeName = exportingCollection.name.replace(/\s+/g, '-');
      downloadFile(`${safeName}.tarasque.json`, json, 'application/json');
      toasts.show('success', 'Exported as .tarasque.json');
    } catch (err) {
      toasts.show('error', `Export failed: ${err}`);
    }

    exportingCollection = null;
  }

  async function handleExportPostman() {
    if (!exportingCollection) return;

    try {
      const json = await invoke<string>('export_collection_postman', {
        collection: exportingCollection,
      });
      downloadFile(`${exportingCollection.name}.postman.json`, json, 'application/json');
      toasts.show('success', 'Exported as Postman collection');
    } catch (err) {
      toasts.show('error', `Export failed: ${err}`);
    }

    exportingCollection = null;
  }

  function downloadFile(filename: string, content: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleExportModalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') exportingCollection = null;
  }

  export function triggerImport() {
    fileInput.click();
  }

  export function triggerExport() {
    if ($collections.length === 1) {
      exportingCollection = $collections[0];
    } else if ($collections.length > 1) {
      showExportPicker = true;
    }
  }

  let showExportPicker = $state(false);
</script>

<svelte:window onkeydown={handleExportModalKeydown} />

<input
  type="file"
  accept=".json,.yaml,.yml,.tarasque.json"
  bind:this={fileInput}
  onchange={handleImport}
  class="hidden"
/>

{#if showExportPicker}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={() => showExportPicker = false}
  >
    <div
      class="w-[280px] bg-surface border border-line rounded-xl shadow-2xl p-4"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="text-xs font-krub text-ink-muted uppercase tracking-widest mb-3">Select collection</div>
      <div class="space-y-0.5 max-h-60 overflow-y-auto">
        {#each $collections as col}
          <button
            onclick={() => { exportingCollection = col; showExportPicker = false; }}
            class="w-full px-3 py-2 text-left text-xs text-ink-muted hover:text-ink hover:bg-hover rounded transition-colors"
          >
            {col.name}
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}

{#if exportingCollection}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={() => exportingCollection = null}
  >
    <div
      class="w-[300px] bg-surface border border-line rounded-xl shadow-2xl p-4"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="text-sm font-krub text-ink mb-3">Export: {exportingCollection.name}</div>
      <div class="flex gap-2">
        <button
          onclick={handleExportJson}
          class="flex-1 px-3 py-2 text-xs font-krub bg-surface border border-line-strong rounded text-ink-muted hover:text-ink hover:border-accent transition-colors"
        >
          JSON
        </button>
        <button
          onclick={handleExportPostman}
          class="flex-1 px-3 py-2 text-xs font-krub bg-surface border border-line-strong rounded text-ink-muted hover:text-ink hover:border-accent transition-colors"
        >
          Postman
        </button>
      </div>
    </div>
  </div>
{/if}
