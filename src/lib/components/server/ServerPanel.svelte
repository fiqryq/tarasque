<script lang="ts">
  import { Play, Square, Loader2 } from 'lucide-svelte';
  import { serverStatus, serverError, startServer, stopServer } from '$lib/stores/server';
  import { activeCollection, activeCollectionId, updateCollectionPort } from '$lib/stores/collections';

  async function toggle() {
    if ($serverStatus === 'running') {
      await stopServer();
    } else {
      await startServer();
    }
  }
</script>

<div class="flex items-center gap-4 h-full w-full font-inter">
  <!-- Port input -->
  <div class="flex items-center gap-2">
    <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Port</span>
    <input
      id="server-port"
      type="number"
      min="1024"
      max="65535"
      value={$activeCollection?.port ?? 3000}
      onchange={(e) => {
        if ($activeCollectionId) updateCollectionPort($activeCollectionId, Number(e.currentTarget.value));
      }}
      disabled={$serverStatus === 'running'}
      class="w-16 bg-surface text-ink text-xs font-mono px-2 py-1 border border-line-strong rounded
             focus:outline-none focus:border-accent transition-colors duration-150
             disabled:opacity-30 disabled:cursor-not-allowed"
    />
  </div>

  <!-- Divider -->
  <div class="w-px h-4 bg-line-strong"></div>

  <!-- Run / Stop button -->
  <button
    onclick={toggle}
    disabled={$serverStatus === 'starting' || !$activeCollection}
    class="flex items-center gap-1.5 h-7 px-3 rounded text-xs font-krub font-medium
           transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed
      {$serverStatus === 'running'
        ? 'bg-err/10 text-err border border-err/20 hover:bg-err/20'
        : 'bg-run/10 text-run border border-run/20 hover:bg-run/20'}"
  >
    {#if $serverStatus === 'starting'}
      <Loader2 size={11} class="animate-spin" />
      <span>starting</span>
    {:else if $serverStatus === 'running'}
      <Square size={11} />
      <span>stop</span>
    {:else}
      <Play size={11} />
      <span>run</span>
    {/if}
  </button>

  <!-- Running indicator -->
  {#if $serverStatus === 'running' && $activeCollection}
    <div class="flex items-center gap-1.5 text-[11px] font-mono text-run">
      <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse inline-block"></span>
      <span>localhost:{$activeCollection.port}</span>
    </div>
  {/if}

  <!-- Error -->
  {#if $serverStatus === 'error' && $serverError}
    <span class="text-[11px] text-err font-mono truncate max-w-xs">{$serverError}</span>
  {/if}
</div>
