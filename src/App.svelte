<script lang="ts">
  import { onMount } from 'svelte';
  import { Database, Settings } from 'lucide-svelte';
  import Sidebar from '$lib/components/sidebar/index.svelte';
  import Editor from '$lib/components/editor/index.svelte';
  import ServerPanel from '$lib/components/server/ServerPanel.svelte';
  import LogPanel from '$lib/components/server/LogPanel.svelte';
  import { loadCollections, activeCollection } from '$lib/stores/collections';
  import { initLogListener } from '$lib/stores/logs';

  onMount(async () => {
    await loadCollections();
    await initLogListener();
  });
</script>

<div class="h-screen bg-app text-ink flex overflow-hidden font-inter">
  <!-- Icon nav strip -->
  <div class="w-10 shrink-0 bg-[#141414] flex flex-col items-center border-r border-line">
    <div class="w-10 h-11 flex items-center justify-center border-b border-line shrink-0">
      <span class="font-krub font-bold text-sm text-accent">T</span>
    </div>
    <div class="flex-1 flex flex-col items-center pt-2 gap-0.5">
      <button
        class="w-8 h-8 flex items-center justify-center rounded text-accent bg-accent/10"
        title="Collections"
      >
        <Database size={14} />
      </button>
    </div>
    <div class="pb-3">
      <button
        class="w-8 h-8 flex items-center justify-center rounded text-ink-dim hover:text-ink-muted transition-colors duration-150"
        title="Settings"
      >
        <Settings size={13} />
      </button>
    </div>
  </div>

  <!-- Collection sidebar -->
  <div class="w-56 shrink-0 flex flex-col bg-[#191919] border-r border-line">
    <Sidebar />
  </div>

  <!-- Main -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Top bar -->
    <div class="shrink-0 h-11 px-4 flex items-center border-b border-line">
      <ServerPanel />
    </div>

    <!-- Editor -->
    <div class="flex-1 min-h-0">
      {#if $activeCollection}
        <Editor />
      {:else}
        <div class="flex flex-col items-center justify-center h-full gap-2 text-center">
          <div class="w-10 h-10 rounded-xl bg-surface flex items-center justify-center mb-1">
            <Database size={18} class="text-ink-dim" />
          </div>
          <span class="font-krub text-sm text-ink-muted">No collection selected</span>
          <span class="text-xs text-ink-dim">Create or select one from the sidebar</span>
        </div>
      {/if}
    </div>

    <!-- Log panel -->
    <div class="h-36 shrink-0 border-t border-line">
      <LogPanel />
    </div>
  </div>
</div>
