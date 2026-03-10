<script lang="ts">
  import { Play, Plus, Trash2, ChevronDown, ChevronRight } from 'lucide-svelte';
  import {
    addTestCase,
    deleteTestCase,
    updateTestCase,
    runTestSuite,
    testResults,
    runningTests,
  } from '$lib/stores/testSuites';
  import { collections } from '$lib/stores/collections';
  import type { TestSuite, TestCase, TestAssertion } from '$lib/types';
  import { newTestAssertion } from '$lib/types';

  interface Props {
    suite: TestSuite;
  }

  let { suite }: Props = $props();

  let expandedCases = $state<Set<string>>(new Set());

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];
  const METHOD_COLORS: Record<string, string> = {
    GET:     'bg-emerald-500/10 text-emerald-400',
    POST:    'bg-violet-500/10 text-violet-400',
    PUT:     'bg-sky-500/10 text-sky-400',
    DELETE:  'bg-rose-500/10 text-rose-400',
    PATCH:   'bg-amber-500/10 text-amber-400',
    HEAD:    'bg-teal-500/10 text-teal-400',
    OPTIONS: 'bg-orange-500/10 text-orange-400',
  };

  function toggleCase(id: string) {
    if (expandedCases.has(id)) expandedCases.delete(id);
    else expandedCases.add(id);
    expandedCases = new Set(expandedCases);
  }

  function getPort(): number {
    return $collections.find((c) => c.id === suite.collection_id)?.port ?? 3000;
  }

  function handleRun() {
    runTestSuite(suite, `http://localhost:${getPort()}`);
  }

  function patchCase(tc: TestCase, patch: Partial<TestCase>) {
    updateTestCase(suite.id, { ...tc, ...patch });
  }

  function addAssertion(tc: TestCase) {
    patchCase(tc, { assertions: [...tc.assertions, newTestAssertion()] });
  }

  function updateAssertion(tc: TestCase, idx: number, patch: Partial<TestAssertion>) {
    const updated = tc.assertions.map((a, i) => i === idx ? { ...a, ...patch } : a);
    patchCase(tc, { assertions: updated });
  }

  function removeAssertion(tc: TestCase, idx: number) {
    patchCase(tc, { assertions: tc.assertions.filter((_, i) => i !== idx) });
  }

  function resultFor(caseId: string) {
    return $testResults?.results.find((r) => r.test_case_id === caseId);
  }
</script>

<div class="h-full flex flex-col overflow-hidden bg-app">
  <!-- Header -->
  <div class="shrink-0 px-4 h-11 flex items-center justify-between border-b border-line bg-surface">
    <span class="font-krub font-semibold text-sm text-ink">{suite.name}</span>
    <div class="flex items-center gap-3">
      {#if $testResults && $testResults.suite_id === suite.id}
        <span class="text-xs text-ink-dim">
          <span class="text-emerald-400 font-semibold">{$testResults.passed}</span>
          / {$testResults.total} passed
          {#if $testResults.failed > 0}
            · <span class="text-rose-400">{$testResults.failed} failed</span>
          {/if}
        </span>
      {/if}
      <button
        onclick={handleRun}
        disabled={$runningTests}
        class="flex items-center gap-1.5 px-3 py-1 text-xs font-krub rounded border
               bg-run/10 text-run border-run/30 hover:bg-run/20 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <Play size={10} />
        {$runningTests ? 'Running…' : 'Run Suite'}
      </button>
    </div>
  </div>

  <!-- Test cases -->
  <div class="flex-1 overflow-y-auto p-4 space-y-2">
    {#each suite.test_cases as tc (tc.id)}
      {@const result = resultFor(tc.id)}
      <div class="border border-line rounded-lg overflow-hidden">
        <!-- Case header row -->
        <div class="flex items-center gap-2 px-3 py-2 bg-surface hover:bg-hover/20 transition-colors">
          <button onclick={() => toggleCase(tc.id)} class="shrink-0 text-ink-dim">
            {#if expandedCases.has(tc.id)}
              <ChevronDown size={12} />
            {:else}
              <ChevronRight size={12} />
            {/if}
          </button>

          <!-- Method select -->
          <select
            value={tc.method}
            onchange={(e) => patchCase(tc, { method: (e.target as HTMLSelectElement).value as TestCase['method'] })}
            class="text-[9px] font-mono font-bold rounded px-1 py-0.5 border-0 outline-none cursor-pointer
                   {METHOD_COLORS[tc.method] ?? 'bg-surface text-ink-muted'}"
          >
            {#each METHODS as m}
              <option value={m}>{m}</option>
            {/each}
          </select>

          <!-- Path input -->
          <input
            value={tc.path}
            onchange={(e) => patchCase(tc, { path: (e.target as HTMLInputElement).value })}
            class="flex-1 bg-transparent text-xs font-mono text-ink border-0 outline-none min-w-0"
            placeholder="/path"
          />

          <!-- Name input -->
          <input
            value={tc.name}
            onchange={(e) => patchCase(tc, { name: (e.target as HTMLInputElement).value })}
            class="w-32 bg-transparent text-xs text-ink-muted border-0 outline-none text-right"
            placeholder="name"
          />

          <!-- Result indicator -->
          {#if result}
            {#if result.passed}
              <span class="text-emerald-400 text-[10px] font-mono shrink-0">✓ {result.duration}ms</span>
            {:else}
              <span class="text-rose-400 text-[10px] font-mono shrink-0">{result.error ? '✗ error' : `✗ ${result.actual_status}`}</span>
            {/if}
          {/if}

          <button onclick={() => deleteTestCase(suite.id, tc.id)} class="text-ink-dim hover:text-err shrink-0">
            <Trash2 size={10} />
          </button>
        </div>

        <!-- Expanded detail -->
        {#if expandedCases.has(tc.id)}
          <div class="border-t border-line/50 px-4 py-3 space-y-3 bg-app/40">
            <!-- Assertions -->
            <div class="space-y-1.5">
              <span class="text-[9px] font-krub text-ink-dim uppercase tracking-widest">Assertions</span>
              {#each tc.assertions as assertion, i (assertion.id)}
                {@const assertResult = result?.assertion_results[i]}
                <div class="flex items-center gap-2">
                  {#if assertResult}
                    <span class="{assertResult.passed ? 'text-emerald-400' : 'text-rose-400'} shrink-0 text-[10px]">
                      {assertResult.passed ? '✓' : '✗'}
                    </span>
                  {/if}

                  <!-- Type -->
                  <select
                    value={assertion.type}
                    onchange={(e) => updateAssertion(tc, i, { type: (e.target as HTMLSelectElement).value as TestAssertion['type'] })}
                    class="text-[10px] bg-surface border border-line rounded px-1.5 py-0.5 text-ink outline-none"
                  >
                    <option value="status">status</option>
                    <option value="body">body</option>
                  </select>

                  <!-- Path (body only) -->
                  {#if assertion.type === 'body'}
                    <input
                      value={assertion.path ?? ''}
                      onchange={(e) => updateAssertion(tc, i, { path: (e.target as HTMLInputElement).value || undefined })}
                      placeholder="json.path"
                      class="w-28 text-[10px] font-mono bg-surface border border-line rounded px-1.5 py-0.5 text-ink outline-none"
                    />
                  {/if}

                  <!-- Operator -->
                  <select
                    value={assertion.operator}
                    onchange={(e) => updateAssertion(tc, i, { operator: (e.target as HTMLSelectElement).value as TestAssertion['operator'] })}
                    class="text-[10px] bg-surface border border-line rounded px-1.5 py-0.5 text-ink outline-none"
                  >
                    <option value="eq">eq</option>
                    <option value="contains">contains</option>
                    <option value="exists">exists</option>
                    <option value="not_exists">not_exists</option>
                  </select>

                  <!-- Expected -->
                  {#if assertion.operator !== 'exists' && assertion.operator !== 'not_exists'}
                    <input
                      value={String(assertion.expected)}
                      onchange={(e) => updateAssertion(tc, i, { expected: (e.target as HTMLInputElement).value })}
                      placeholder="expected"
                      class="flex-1 text-[10px] font-mono bg-surface border border-line rounded px-1.5 py-0.5 text-ink outline-none min-w-0"
                    />
                  {/if}

                  {#if assertResult && !assertResult.passed}
                    <span class="text-[9px] text-rose-400 font-mono shrink-0">got: {assertResult.actual}</span>
                  {/if}

                  <button onclick={() => removeAssertion(tc, i)} class="text-ink-dim hover:text-err shrink-0">
                    <Trash2 size={9} />
                  </button>
                </div>
              {/each}
              <button
                onclick={() => addAssertion(tc)}
                class="text-[10px] text-ink-dim hover:text-ink transition-colors"
              >
                + Add assertion
              </button>
            </div>

            <!-- Error message -->
            {#if result?.error}
              <p class="text-[10px] text-rose-400 font-mono">{result.error}</p>
            {/if}
          </div>
        {/if}
      </div>
    {/each}

    <button
      onclick={() => addTestCase(suite.id)}
      class="w-full py-2 text-xs text-ink-dim hover:text-ink border border-dashed border-line hover:border-line-strong rounded-lg transition-colors"
    >
      + Add test case
    </button>
  </div>
</div>
