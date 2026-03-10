<script lang="ts">
  import { Plus, Play, Trash2, ChevronDown, ChevronRight } from 'lucide-svelte';
  import { 
    testSuites, 
    testResults, 
    runningTests,
    addTestSuite, 
    deleteTestSuite, 
    addTestCase,
    deleteTestCase,
    runTestSuite,
    loadTestSuites 
  } from '$lib/stores/testSuites';
  import { collections, activeCollectionId } from '$lib/stores/collections';
  import { get } from 'svelte/store';
  import type { TestCase } from '$lib/types';
  import { onMount } from 'svelte';

  let expandedSuites = $state<Set<string>>(new Set());
  let showNewSuite = $state(false);
  let newSuiteName = $state('');

  const METHOD_COLORS: Record<string, string> = {
    GET:    'bg-emerald-500/10 text-emerald-400',
    POST:   'bg-violet-500/10 text-violet-400',
    PUT:    'bg-sky-500/10 text-sky-400',
    DELETE: 'bg-rose-500/10 text-rose-400',
    PATCH:  'bg-amber-500/10 text-amber-400',
  };

  onMount(() => {
    loadTestSuites();
  });

  function toggleSuite(id: string) {
    if (expandedSuites.has(id)) {
      expandedSuites.delete(id);
    } else {
      expandedSuites.add(id);
    }
    expandedSuites = new Set(expandedSuites);
  }

  function handleCreateSuite() {
    const activeCid = get(activeCollectionId);
    if (newSuiteName.trim() && activeCid) {
      addTestSuite(newSuiteName.trim(), activeCid);
      newSuiteName = '';
      showNewSuite = false;
    }
  }

  function getCollectionPort(collectionId: string): number {
    const col = $collections.find(c => c.id === collectionId);
    return col?.port ?? 3000;
  }
</script>

<div class="border-t border-line mt-auto">
  <div class="flex items-center justify-between px-3 h-9 border-b border-line shrink-0">
    <span class="font-krub text-[10px] font-semibold text-ink-muted uppercase tracking-widest">Tests</span>
    <button
      onclick={() => showNewSuite = true}
      class="w-5 h-5 flex items-center justify-center rounded text-ink-dim hover:text-ink hover:bg-hover transition-colors"
      title="New test suite"
    >
      <Plus size={12} />
    </button>
  </div>

  {#if showNewSuite}
    <div class="px-3 py-2 border-b border-line">
      <input
        bind:value={newSuiteName}
        placeholder="Test suite name..."
        class="w-full bg-surface text-ink text-xs px-2 py-1 rounded border border-line-strong focus:outline-none focus:border-accent"
        onkeydown={(e) => e.key === 'Enter' && handleCreateSuite()}
      />
    </div>
  {/if}

  <div class="flex-1 overflow-y-auto py-1 max-h-48">
    {#each $testSuites as suite (suite.id)}
      <div>
        <button
          onclick={() => toggleSuite(suite.id)}
          class="w-full text-left px-3 py-1.5 flex items-center gap-1 group hover:bg-hover/50"
        >
          <ChevronRight size={12} class="text-ink-dim transition-transform {expandedSuites.has(suite.id) ? 'rotate-90' : ''}" />
          <span class="flex-1 text-xs truncate">{suite.name}</span>
          <span class="text-[9px] text-ink-dim">{suite.test_cases.length}</span>
        </button>
        <div class="flex gap-1 pr-2">
          <button
            onclick={(e) => { e.stopPropagation(); runTestSuite(suite, `http://localhost:${getCollectionPort(suite.collection_id)}`); }}
            class="text-emerald-400 hover:text-emerald-300"
            title="Run tests"
          >
            <Play size={10} />
          </button>
          <button
            onclick={(e) => { e.stopPropagation(); deleteTestSuite(suite.id); }}
            class="text-ink-dim hover:text-err"
            title="Delete"
          >
            <Trash2 size={10} />
          </button>
        </div>

        {#if expandedSuites.has(suite.id)}
          <div class="pl-6 pr-2 space-y-1">
            {#each suite.test_cases as tc (tc.id)}
              <div class="group flex items-center gap-1 px-2 py-1 bg-surface/50 rounded text-[10px]">
                <span class="font-mono font-bold {METHOD_COLORS[tc.method]}">{tc.method}</span>
                <span class="flex-1 truncate text-ink-muted">{tc.path}</span>
                <button
                  onclick={() => deleteTestCase(suite.id, tc.id)}
                  class="text-ink-dim hover:text-err opacity-0 group-hover:opacity-100"
                >
                  <Trash2 size={8} />
                </button>
              </div>
            {/each}
            <button
              onclick={() => addTestCase(suite.id)}
              class="w-full text-left px-2 py-1 text-[10px] text-ink-dim hover:text-ink"
            >
              + Add test case
            </button>
          </div>
        {/if}
      </div>
    {/each}

    {#if $testSuites.length === 0 && !showNewSuite}
      <p class="text-ink-dim text-[10px] text-center py-2">No test suites</p>
    {/if}
  </div>

  {#if $testResults}
    <div class="border-t border-line px-3 py-2 bg-surface/50">
      <div class="flex items-center justify-between text-[10px] mb-1">
        <span class="font-semibold">Results</span>
        <span class="text-emerald-400">{$testResults.passed} passed</span>
        {#if $testResults.failed > 0}
          <span class="text-rose-400">{$testResults.failed} failed</span>
        {/if}
      </div>
      <div class="space-y-1 max-h-24 overflow-y-auto">
        {#each $testResults.results as result}
          <div class="flex items-center gap-1 text-[9px]">
            {#if result.passed}
              <span class="text-emerald-400">✓</span>
            {:else}
              <span class="text-rose-400">✗</span>
            {/if}
            <span class="text-ink-muted truncate">{result.duration}ms</span>
            {#if result.error}
              <span class="text-rose-400 truncate">{result.error}</span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if $runningTests}
    <div class="border-t border-line px-3 py-2 text-center">
      <span class="text-[10px] text-ink-dim">Running tests...</span>
    </div>
  {/if}
</div>