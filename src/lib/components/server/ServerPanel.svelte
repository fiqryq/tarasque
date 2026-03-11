<script lang="ts">
  import { Play, Square, Loader2, Shuffle } from "lucide-svelte";
  import {
    activeServerStatus,
    activeServerError,
    startServer,
    stopServer,
    findFreePort,
  } from "$lib/stores/server";
  import {
    activeCollection,
    activeCollectionId,
    updateCollectionPort,
  } from "$lib/stores/collections";

  async function toggle() {
    const id = $activeCollectionId;
    if (!id) return;
    if ($activeServerStatus === "running") {
      await stopServer(id);
    } else {
      await startServer(id);
    }
  }

  async function autoPort() {
    if (!$activeCollectionId || $activeServerStatus === "running") return;
    try {
      const port = await findFreePort();
      updateCollectionPort($activeCollectionId, port);
    } catch {
      /* ignore */
    }
  }
</script>

<div class="flex items-center gap-3 h-full w-full mb-3 font-sf">
  <!-- Start / Stop capsule -->
  <button
    onclick={toggle}
    disabled={$activeServerStatus === "starting" || !$activeCollection}
    class="btn-capsule disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none
           {$activeServerStatus === 'running'
      ? 'bg-err/12 text-err hover:bg-err/18'
      : 'btn-capsule-primary'}"
  >
    {#if $activeServerStatus === "starting"}
      <Loader2 size={14} class="animate-spin" />
      <span>Starting</span>
    {:else if $activeServerStatus === "running"}
      <Square size={10} fill="currentColor" />
      <span>Stop</span>
    {:else}
      <Play size={12} fill="currentColor" />
      <span>Start</span>
    {/if}
  </button>

  <!-- Port config -->
  <div class="flex items-center gap-1.5 bg-surface rounded-full px-3 py-1.5">
    <span class="text-caption text-ink-tertiary">Port</span>
    <input
      id="server-port"
      type="number"
      min="1024"
      max="65535"
      value={$activeCollection?.port ?? 3000}
      onchange={(e) => {
        if ($activeCollectionId)
          updateCollectionPort(
            $activeCollectionId,
            Number(e.currentTarget.value),
          );
      }}
      disabled={$activeServerStatus === "running"}
      aria-label="Server port"
      class="w-14 bg-transparent text-ink text-body font-mono text-center
             focus:outline-none disabled:opacity-40 disabled:cursor-not-allowed"
    />
    <button
      onclick={autoPort}
      disabled={$activeServerStatus === "running" || !$activeCollection}
      class="w-5 h-5 flex items-center justify-center rounded-full text-ink-tertiary hover:text-ink-secondary
             transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
      title="Find a free port"
      aria-label="Find a free port"
    >
      <Shuffle size={12} />
    </button>
  </div>

  <div class="flex-1"></div>

  <!-- Running status -->
  {#if $activeServerStatus === "running" && $activeCollection}
    <div class="flex items-center gap-2 text-caption font-mono text-run">
      <span class="w-2 h-2 rounded-full bg-run animate-pulse"></span>
      localhost:{$activeCollection.port}
    </div>
  {/if}

  {#if $activeServerStatus === "error" && $activeServerError}
    <span
      class="text-caption text-err font-mono truncate max-w-[200px]"
      title={$activeServerError}
    >
      {$activeServerError}
    </span>
  {/if}
</div>
