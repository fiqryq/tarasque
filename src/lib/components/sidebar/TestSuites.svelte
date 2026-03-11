<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import { testSuites, addTestSuite, deleteTestSuite } from '$lib/stores/testSuites';
  import { activeCollectionId } from '$lib/stores/collections';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import { get } from 'svelte/store';

  interface Props {
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { selectedId, onSelect }: Props = $props();

  let showNewSuite = $state(false);
  let newSuiteName = $state('');
  let deleteSuiteId = $state<string | null>(null);
  let deleteSuiteName = $state('');

  function handleCreateSuite() {
    const activeCid = get(activeCollectionId);
    if (newSuiteName.trim() && activeCid) {
      addTestSuite(newSuiteName.trim(), activeCid);
      newSuiteName = '';
      showNewSuite = false;
    }
  }

  function confirmDeleteSuite(id: string, name: string) {
    deleteSuiteId = id;
    deleteSuiteName = name;
  }

  function executeDelete() {
    if (deleteSuiteId) {
      deleteTestSuite(deleteSuiteId);
      deleteSuiteId = null;
      deleteSuiteName = '';
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-3 h-9 border-b border-line shrink-0">
    <span class="text-caption font-semibold text-ink-secondary uppercase tracking-widest">Test Suites</span>
    <button
      onclick={() => showNewSuite = true}
      class="w-5 h-5 flex items-center justify-center rounded-hig-sm text-ink-tertiary hover:text-ink hover:bg-[var(--color-hover)] transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
      title="New test suite"
      aria-label="Create new test suite"
    >
      <Plus size={12} />
    </button>
  </div>

  {#if showNewSuite}
    <div class="px-3 py-2 border-b border-line">
      <input
        bind:value={newSuiteName}
        placeholder="Suite name…"
        aria-label="New test suite name"
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
                 ? 'bg-accent/12 text-ink border-l-[2px] border-accent'
                 : 'border-l-[2px] border-transparent hover:bg-[var(--color-hover)]'}"
      >
        <span class="flex-1 text-xs truncate {selectedId === suite.id ? 'text-ink' : 'text-ink-secondary group-hover:text-ink'}">{suite.name}</span>
        <span class="text-caption text-ink-tertiary shrink-0">{suite.test_cases.length}</span>
        <div
          role="button"
          tabindex="0"
          onclick={(e) => { e.stopPropagation(); confirmDeleteSuite(suite.id, suite.name); }}
          onkeydown={(e) => e.key === 'Enter' && confirmDeleteSuite(suite.id, suite.name)}
          class="opacity-0 group-hover:opacity-100 w-4 h-4 flex items-center justify-center rounded-hig-sm text-ink-tertiary hover:text-err transition-all shrink-0
                 focus:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-err"
          aria-label="Delete test suite"
        >
          <Trash2 size={9} />
        </div>
      </button>
    {/each}

    {#if $testSuites.length === 0 && !showNewSuite}
      <div class="text-center py-4">
        <p class="text-ink-tertiary text-caption">No test suites yet</p>
        <button
          onclick={() => showNewSuite = true}
          class="mt-2 text-caption text-accent hover:underline"
        >
          Create your first test suite
        </button>
      </div>
    {/if}
  </div>
</div>

{#if deleteSuiteId}
  <ConfirmModal
    title="Delete Test Suite"
    message={`Are you sure you want to delete "${deleteSuiteName}"? All test cases will be removed. This action cannot be undone.`}
    confirmLabel="Delete Suite"
    onConfirm={executeDelete}
    onCancel={() => { deleteSuiteId = null; deleteSuiteName = ''; }}
  />
{/if}
