<script lang="ts">
  import { parseCurl, addEndpointFromCurl } from '$lib/stores/collections';
  import { X } from 'lucide-svelte';

  let { collectionId, onclose }: { collectionId: string; onclose: () => void } = $props();

  let curlText = $state('');
  let error = $state('');

  function handleImport() {
    error = '';
    const trimmed = curlText.trim();
    if (!trimmed) {
      error = 'Paste a cURL command first.';
      return;
    }
    if (!trimmed.startsWith('curl')) {
      error = 'Command must start with "curl".';
      return;
    }
    try {
      const parsed = parseCurl(trimmed);
      addEndpointFromCurl(collectionId, parsed);
      onclose();
    } catch (e) {
      error = String(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  onclick={onclose}
>
  <div
    class="bg-surface border border-line rounded-xl w-[520px] shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between px-4 py-3 border-b border-line">
      <span class="text-sm font-krub text-ink">Import cURL</span>
      <button onclick={onclose} class="text-ink-dim hover:text-ink transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded" aria-label="Close">
        <X size={14} />
      </button>
    </div>

    <div class="p-4 space-y-3">
      <textarea
        bind:value={curlText}
        placeholder={'curl -X POST https://api.example.com/users \n  -H \'Content-Type: application/json\' \n  -d \'{"name": "John"}\''}
        aria-label="cURL command"
        class="w-full h-32 px-3 py-2 bg-app border border-line rounded text-xs text-ink font-mono
               placeholder:text-ink-dim/50 resize-none focus:outline-none focus:border-accent transition-colors"
      ></textarea>

      {#if error}
        <p class="text-xs text-err">{error}</p>
      {/if}

      <div class="flex gap-2 justify-end">
        <button
          onclick={onclose}
          class="px-3 py-1.5 text-xs font-krub text-ink-dim hover:text-ink transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleImport}
          class="px-4 py-1.5 text-xs font-krub bg-accent text-white rounded hover:opacity-90 transition-opacity focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface"
        >
          Import cURL
        </button>
      </div>
    </div>
  </div>
</div>
