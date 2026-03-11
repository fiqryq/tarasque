<script lang="ts">
  import { onMount } from 'svelte';
  import { Database, Activity, Settings, FlaskConical, ScrollText, PanelLeftClose, PanelLeftOpen } from 'lucide-svelte';
  import Sidebar from '$lib/components/sidebar/index.svelte';
  import RunningApis from '$lib/components/sidebar/RunningApis.svelte';
  import LogsSidebar from '$lib/components/sidebar/LogsSidebar.svelte';
  import Editor from '$lib/components/editor/index.svelte';
  import SwaggerView from '$lib/components/SwaggerView.svelte';
  import LogDetailView from '$lib/components/LogDetailView.svelte';
  import ServerPanel from '$lib/components/server/ServerPanel.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import CollectionSettingsModal from '$lib/components/CollectionSettingsModal.svelte';
  import CollectionEnvironmentModal from '$lib/components/CollectionEnvironmentModal.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import TestSuiteEditor from '$lib/components/editor/TestSuiteEditor.svelte';
  import TestSuites from '$lib/components/sidebar/TestSuites.svelte';
  import type { LogEntry, Collection } from '$lib/types';
  import { get } from 'svelte/store';
  import { loadCollections, activeCollection, collections } from '$lib/stores/collections';
  import { initLogListener, logs } from '$lib/stores/logs';
  import { syncRunningServers } from '$lib/stores/server';
  import { settings, applyTheme } from '$lib/stores/settings';
  import { testSuites, loadTestSuites } from '$lib/stores/testSuites';

  let showSettings = $state(false);
  let settingsCollection = $state<Collection | null>(null);
  let environmentCollection = $state<Collection | null>(null);
  type NavView = 'collections' | 'running' | 'tests' | 'logs';
  let activeNav = $state<NavView>('collections');

  // Sidebar collapse & resize
  let sidebarCollapsed = $state(false);
  let sidebarWidth = $state(240);
  let isResizing = $state(false);
  const SIDEBAR_MIN = 180;
  const SIDEBAR_MAX = 400;

  function startResize(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    function onMouseMove(e: MouseEvent) {
      const delta = e.clientX - startX;
      sidebarWidth = Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, startWidth + delta));
    }

    function onMouseUp() {
      isResizing = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }
  let activeRunningCollectionId = $state<string | null>(null);
  let activeSuiteId = $state<string | null>(null);
  let selectedLog = $state<LogEntry | null>(null);

  let activeSuite = $derived($testSuites.find((s) => s.id === activeSuiteId) ?? null);

  let selectedRunningCollection = $derived(
    activeRunningCollectionId ? $collections.find(c => c.id === activeRunningCollectionId) ?? null : null
  );

  const navItems: { key: NavView; icon: typeof Database; label: string }[] = [
    { key: 'collections', icon: Database, label: 'Collections' },
    { key: 'running', icon: Activity, label: 'Running' },
    { key: 'tests', icon: FlaskConical, label: 'Tests' },
    { key: 'logs', icon: ScrollText, label: 'Logs' },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === ',') {
      e.preventDefault();
      showSettings = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
      e.preventDefault();
      activeNav = 'logs';
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
      e.preventDefault();
      sidebarCollapsed = !sidebarCollapsed;
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

<div class="h-screen bg-app text-ink flex flex-col overflow-hidden font-sf {isResizing ? 'select-none cursor-col-resize' : ''}">
  <!-- Titlebar drag region (macOS traffic lights) -->
  <div data-tauri-drag-region class="h-12 shrink-0 flex items-end" style="-webkit-app-region: drag;">
    <!-- Empty drag area — traffic lights sit here natively -->
  </div>

  <div class="flex-1 flex overflow-hidden">
    <!-- Icon nav strip -->
    <div class="w-[44px] shrink-0 glass border-r border-line flex flex-col items-center">
      <div class="flex-1 flex flex-col items-center pt-1 gap-0.5">
        {#each navItems as { key, icon: Icon, label }}
          <button
            onclick={() => activeNav = key}
            class="w-9 h-9 flex flex-col items-center justify-center rounded-hig-sm transition-all duration-200 relative group
                   {activeNav === key
                     ? 'bg-accent/12 text-accent'
                     : 'text-ink-tertiary hover:text-ink-secondary hover:bg-[var(--color-hover)]'}"
            title={label}
          >
            <Icon size={18} />
            {#if key === 'logs' && $logs.length > 0 && activeNav !== 'logs'}
              <span class="absolute top-1.5 right-1.5 w-[6px] h-[6px] rounded-full bg-accent"></span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="pb-4 flex flex-col items-center gap-1">
        <button
          onclick={() => sidebarCollapsed = !sidebarCollapsed}
          class="w-9 h-9 flex items-center justify-center rounded-hig-sm text-ink-tertiary
                 hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-all duration-200"
          title={sidebarCollapsed ? 'Show sidebar (⌘B)' : 'Hide sidebar (⌘B)'}
        >
          {#if sidebarCollapsed}
            <PanelLeftOpen size={18} />
          {:else}
            <PanelLeftClose size={18} />
          {/if}
        </button>
        <button
          onclick={() => showSettings = true}
          class="w-9 h-9 flex items-center justify-center rounded-hig-sm text-ink-tertiary
                 hover:text-ink-secondary hover:bg-[var(--color-hover)] transition-all duration-200"
          title="Settings"
        >
          <Settings size={18} />
        </button>
      </div>
    </div>

    <!-- Sidebar panel -->
    {#if !sidebarCollapsed}
      <div
        class="shrink-0 flex flex-col glass border-r border-line relative"
        style="width: {sidebarWidth}px;"
      >
        {#if activeNav === 'collections'}
          <Sidebar
            onOpenSettings={(c) => settingsCollection = c}
            onOpenEnvironments={(c) => environmentCollection = c}
          />
        {:else if activeNav === 'running'}
          <RunningApis
            selectedId={activeRunningCollectionId}
            onSelect={(id) => activeRunningCollectionId = id}
          />
        {:else if activeNav === 'logs'}
          <LogsSidebar
            {selectedLog}
            onSelect={(log) => selectedLog = log}
          />
        {:else}
          <TestSuites
            selectedId={activeSuiteId}
            onSelect={(id) => activeSuiteId = id}
          />
        {/if}

        <!-- Resize handle -->
        <div
          class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent/30 transition-colors z-10
                 {isResizing ? 'bg-accent/40' : ''}"
          onmousedown={startResize}
          role="separator"
          aria-orientation="vertical"
          tabindex="-1"
        ></div>
      </div>
    {/if}

    <!-- Main content -->
    <div class="flex-1 flex flex-col min-w-0 bg-app">
      <!-- Toolbar (collections only) -->
      {#if activeNav === 'collections'}
        <div class="shrink-0 h-toolbar px-4 flex items-center border-b border-line">
          <ServerPanel />
        </div>
      {/if}

      <!-- Content area -->
      <div class="flex-1 min-h-0">
        {#if activeNav === 'running'}
          {#if selectedRunningCollection}
            <SwaggerView collection={selectedRunningCollection} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center">
              <div class="w-12 h-12 rounded-hig bg-surface flex items-center justify-center">
                <Activity size={24} class="text-ink-tertiary" />
              </div>
              <span class="text-title text-ink-secondary">No Running APIs</span>
              <span class="text-body text-ink-tertiary">Start a server from Collections to see its docs here</span>
            </div>
          {/if}
        {:else if activeNav === 'logs'}
          {#if selectedLog}
            <LogDetailView log={selectedLog} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center">
              <div class="w-12 h-12 rounded-hig bg-surface flex items-center justify-center">
                <ScrollText size={24} class="text-ink-tertiary" />
              </div>
              <span class="text-title text-ink-secondary">No Log Selected</span>
              <span class="text-body text-ink-tertiary">Select a request from the sidebar to inspect it</span>
            </div>
          {/if}
        {:else if activeNav === 'tests'}
          {#if activeSuite}
            <TestSuiteEditor suite={activeSuite} />
          {:else}
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center">
              <div class="w-12 h-12 rounded-hig bg-surface flex items-center justify-center">
                <FlaskConical size={24} class="text-ink-tertiary" />
              </div>
              <span class="text-title text-ink-secondary">No Test Suite Selected</span>
              <span class="text-body text-ink-tertiary">Create or select one from the sidebar</span>
            </div>
          {/if}
        {:else if $activeCollection}
          <Editor />
        {:else}
          <div class="flex flex-col items-center justify-center h-full gap-3 text-center">
            <div class="w-12 h-12 rounded-hig bg-surface flex items-center justify-center">
              <Database size={24} class="text-ink-tertiary" />
            </div>
            <span class="text-title text-ink-secondary">No Collection Selected</span>
            <span class="text-body text-ink-tertiary">Select one from the sidebar or create a new one</span>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

{#if showSettings}
  <SettingsModal onClose={() => showSettings = false} />
{/if}

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

<Toast />
