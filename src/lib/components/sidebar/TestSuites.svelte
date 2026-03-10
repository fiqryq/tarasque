<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import { testSuites, addTestSuite, deleteTestSuite } from '$lib/stores/testSuites';
  import { activeCollectionId } from '$lib/stores/collections';
  import { get } from 'svelte/store';

  interface Props {
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { selectedId, onSelect }: Props = $props();

  let showNewSuite = $state(false);
  let newSuiteName = $state('');

  function handleCreateSuite() {
    const activeCid = get(activeCollectionId);
    if (newSuiteName.trim() && activeCid) {
      addTestSuite(newSuiteName.trim(), activeCid);
      newSuiteName = '';
      showNewSuite = false;
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-3 h-9 border-b border-line shrink-0">
    <span class="font-krub text-[10px] font-semibold text-ink-muted uppercase tracking-widest">Test Suites</span>
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
        placeholder="Suite name..."
        class="w-full bg-surface text-ink text-xs px-2 py-1 rounded border border-line-strong focus:outline-none focus:border-accent"
        onkeydown={(e) => { if (e.key === 'Enter') handleCreateSuite(); if (e.key === 'Escape') showNewSuite = false; }}
      />
    </div>
  {/if}

  <div class="flex-1 overflow-y-auto py-0.5">
    {#each $testSuites as suite (suite.id)}
      <button
        onclick={() => onSelect(suite.id)}
        class="w-full text-left px-3 py-2 flex items-center gap-2 group transition-colors
               {selectedId === suite.id
                 ? 'bg-hover text-ink border-l-[2px] border-accent'
                 : 'border-l-[2px] border-transparent hover:bg-hover/50'}"
      >
        <span class="flex-1 text-xs truncate {selectedId === suite.id ? 'text-ink' : 'text-ink-muted group-hover:text-ink'}">{suite.name}</span>
        <span class="text-[9px] text-ink-dim shrink-0">{suite.test_cases.length}</span>
        <div
          role="button"
          tabindex="0"
          onclick={(e) => { e.stopPropagation(); deleteTestSuite(suite.id); }}
          onkeydown={(e) => e.key === 'Enter' && deleteTestSuite(suite.id)}
          class="opacity-0 group-hover:opacity-100 w-4 h-4 flex items-center justify-center rounded text-ink-dim hover:text-err transition-all shrink-0"
        >
          <Trash2 size={9} />
        </div>
      </button>
    {/each}

    {#if $testSuites.length === 0 && !showNewSuite}
      <p class="text-ink-dim text-[10px] text-center py-4">No test suites yet</p>
    {/if}
  </div>
</div>