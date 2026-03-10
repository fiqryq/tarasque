<script lang="ts">
  import { onMount } from 'svelte';
  import { Database, Activity, Settings, FlaskConical } from 'lucide-svelte';
  import Sidebar from '$lib/components/sidebar/index.svelte';
  import RunningApis from '$lib/components/sidebar/RunningApis.svelte';
  import Editor from '$lib/components/editor/index.svelte';
  import SwaggerView from '$lib/components/SwaggerView.svelte';
  import ServerPanel from '$lib/components/server/ServerPanel.svelte';
  import LogPanel from '$lib/components/server/LogPanel.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import TestSuiteEditor from '$lib/components/editor/TestSuiteEditor.svelte';
  import TestSuites from '$lib/components/sidebar/TestSuites.svelte';
  import { get } from 'svelte/store';
  import { loadCollections, activeCollection, collections } from '$lib/stores/collections';
  import { initLogListener } from '$lib/stores/logs';
  import { syncRunningServers } from '$lib/stores/server';
  import { logPanel } from '$lib/stores/logPanel';
  import { settings, applyTheme } from '$lib/stores/settings';
  import { testSuites, loadTestSuites } from '$lib/stores/testSuites';

  let showSettings = $state(false);
  type NavView = 'collections' | 'running' | 'tests';
  let activeNav = $state<NavView>('collections');
  let activeRunningCollectionId = $state<string | null>(null);
  let activeSuiteId = $state<string | null>(null);

  let activeSuite = $derived($testSuites.find((s) => s.id === activeSuiteId) ?? null);

  let selectedRunningCollection = $derived(
    activeRunningCollectionId ? $collections.find(c => c.id === activeRunningCollectionId) ?? null : null
  );

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === ',') {
      e.preventDefault();
      showSettings = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
      e.preventDefault();
      logPanel.toggle();
    }
  }

  onMount(async () => {
    await loadCollections();
    await initLogListener();
    await syncRunningServers();
    settings.load();
    applyTheme(get(settings).theme);
    loadTestSuites();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-screen bg-app text-ink flex flex-col overflow-hidden font-inter">
  <div class="flex-1 flex overflow-hidden">
    <!-- Icon nav strip -->
    <div class="w-10 shrink-0 bg-app flex flex-col items-center border-r border-line">
      <div class="w-10 h-11 flex items-center justify-center border-b border-line shrink-0">
        <span class="font-krub font-bold text-sm text-accent">T</span>
      </div>
      <div class="flex-1 flex flex-col items-center pt-2 gap-0.5">
        <button
          onclick={() => activeNav = 'collections'}
          class="w-8 h-8 flex items-center justify-center rounded transition-colors duration-150
                 {activeNav === 'collections' ? 'text-accent bg-accent/10' : 'text-ink-dim hover:text-ink hover:bg-hover'}"
          title="Collections"
        >
          <Database size={14} />
        </button>
        <button
          onclick={() => activeNav = 'running'}
          class="w-8 h-8 flex items-center justify-center rounded transition-colors duration-150
                 {activeNav === 'running' ? 'text-run bg-run/10' : 'text-ink-dim hover:text-ink hover:bg-hover'}"
          title="Running APIs"
        >
          <Activity size={14} />
        </button>
        <button
          onclick={() => activeNav = 'tests'}
          class="w-8 h-8 flex items-center justify-center rounded transition-colors duration-150
                 {activeNav === 'tests' ? 'text-accent bg-accent/10' : 'text-ink-dim hover:text-ink hover:bg-hover'}"
          title="Test Suites"
        >
          <FlaskConical size={14} />
        </button>
      </div>
      <div class="pb-3">
        <button
          onclick={() => showSettings = true}
          class="w-8 h-8 flex items-center justify-center rounded text-ink-dim hover:text-ink-muted transition-colors duration-150"
          title="Settings"
        >
          <Settings size={13} />
        </button>
      </div>
    </div>

    <!-- Sidebar panel (switches based on activeNav) -->
    <div class="w-56 shrink-0 flex flex-col bg-surface border-r border-line">
      {#if activeNav === 'collections'}
        <Sidebar />
      {:else if activeNav === 'running'}
        <RunningApis
          selectedId={activeRunningCollectionId}
          onSelect={(id) => activeRunningCollectionId = id}
        />
      {:else}
        <TestSuites
          selectedId={activeSuiteId}
          onSelect={(id) => activeSuiteId = id}
        />
      {/if}
    </div>

    <!-- Main -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Top bar -->
      <div class="shrink-0 h-11 px-4 flex items-center border-b border-line">
        <ServerPanel />
      </div>

      <!-- Editor / Swagger view / Test suite editor -->
      <div class="flex-1 min-h-0">
        {#if activeNav === 'running'}
          {#if selectedRunningCollection}
            <SwaggerView collection={selectedRunningCollection} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full gap-2 text-center">
              <div class="w-10 h-10 rounded-xl bg-surface flex items-center justify-center mb-1">
                <Activity size={18} class="text-ink-dim" />
              </div>
              <span class="font-krub text-sm text-ink-muted">Select a running API</span>
              <span class="text-xs text-ink-dim">Click a collection in the sidebar to view its docs</span>
            </div>
          {/if}
        {:else if activeNav === 'tests'}
          {#if activeSuite}
            <TestSuiteEditor suite={activeSuite} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full gap-2 text-center">
              <div class="w-10 h-10 rounded-xl bg-surface flex items-center justify-center mb-1">
                <FlaskConical size={18} class="text-ink-dim" />
              </div>
              <span class="font-krub text-sm text-ink-muted">No test suite selected</span>
              <span class="text-xs text-ink-dim">Create a suite in the sidebar</span>
            </div>
          {/if}
        {:else if $activeCollection}
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
      <div class="shrink-0 border-t border-line" style="height: {$logPanel.collapsed ? 'auto' : $logPanel.height + 'px'}">
        <LogPanel />
      </div>
    </div>
  </div>
</div>

{#if showSettings}
  <SettingsModal onClose={() => showSettings = false} />
{/if}

<Toast />
