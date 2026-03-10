<script lang="ts">
  import ScenarioEditor from './ScenarioEditor.svelte';
  import type { Endpoint, Scenario } from '$lib/types';
  import { newScenario } from '$lib/types';

  interface Props {
    endpoint: Endpoint;
    collectionId: string;
    onUpdate: (endpoint: Endpoint) => void;
  }

  let { endpoint, collectionId, onUpdate }: Props = $props();

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'] as const;

  const METHOD_COLORS: Record<string, string> = {
    GET:     'text-emerald-400',
    POST:    'text-violet-400',
    PUT:     'text-sky-400',
    DELETE:  'text-rose-400',
    PATCH:   'text-amber-400',
    HEAD:    'text-teal-400',
    OPTIONS: 'text-orange-400',
  };

  let localScenarioId = $state<string | null>(null);
  let activeScenarioId = $derived(localScenarioId ?? endpoint.active_scenario_id);
  let activeScenario = $derived(
    endpoint.scenarios.find((s) => s.id === activeScenarioId) ?? endpoint.scenarios[0]
  );

  $effect(() => {
    endpoint.id;
    localScenarioId = null;
  });

  function updateScenario(updated: Scenario) {
    onUpdate({
      ...endpoint,
      scenarios: endpoint.scenarios.map((s) => (s.id === updated.id ? updated : s)),
    });
  }

  function addScenario() {
    const s = newScenario({ name: `Scenario ${endpoint.scenarios.length + 1}` });
    onUpdate({ ...endpoint, scenarios: [...endpoint.scenarios, s] });
    localScenarioId = s.id;
  }

  function deleteScenario(id: string) {
    if (endpoint.scenarios.length <= 1) return;
    const remaining = endpoint.scenarios.filter((s) => s.id !== id);
    const newActive = remaining[0].id;
    localScenarioId = newActive;
    onUpdate({ ...endpoint, scenarios: remaining, active_scenario_id: newActive });
  }

  function setActiveScenario(id: string) {
    localScenarioId = id;
    onUpdate({ ...endpoint, active_scenario_id: id });
  }

  function statusColor(status: number): string {
    if (status < 300) return 'text-emerald-400 bg-emerald-500/10';
    if (status < 400) return 'text-amber-400 bg-amber-500/10';
    if (status < 500) return 'text-orange-400 bg-orange-500/10';
    return 'text-rose-400 bg-rose-500/10';
  }
</script>

<div class="flex flex-col h-full">
  <!-- Path bar -->
  <div class="flex flex-col gap-1.5 px-4 py-2 border-b border-line shrink-0">
    <div class="flex items-center gap-2">
      <select
        value={endpoint.method}
        onchange={(e) => onUpdate({ ...endpoint, method: e.currentTarget.value as Endpoint['method'] })}
        class="h-7 bg-surface text-xs font-mono font-bold px-2 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150 cursor-pointer
               {METHOD_COLORS[endpoint.method]}"
      >
        {#each METHODS as m}
          <option value={m} class="bg-surface text-ink">{m}</option>
        {/each}
      </select>
      <input
        value={endpoint.path}
        oninput={(e) => onUpdate({ ...endpoint, path: e.currentTarget.value })}
        placeholder="/your/path/:id"
        class="flex-1 h-7 bg-surface text-ink text-xs font-mono px-3 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150 placeholder:text-ink-dim"
      />
    </div>
    <input
      value={endpoint.description ?? ''}
      oninput={(e) => onUpdate({ ...endpoint, description: e.currentTarget.value || undefined })}
      placeholder="Description (optional)"
      class="w-full h-6 bg-surface text-ink-dim text-[11px] px-3 border border-line rounded
             focus:outline-none focus:border-accent transition-colors duration-150 placeholder:text-ink-dim/40"
    />
  </div>

  <!-- Scenario tabs -->
  <div class="flex items-center px-4 h-9 border-b border-line overflow-x-auto shrink-0 gap-0.5">
    {#each endpoint.scenarios as scenario (scenario.id)}
      <div class="flex items-center shrink-0">
        <button
          onclick={() => setActiveScenario(scenario.id)}
          class="flex items-center gap-1.5 px-3 h-9 text-xs transition-colors duration-150 border-b-2 whitespace-nowrap
            {activeScenarioId === scenario.id
              ? 'text-ink border-accent'
              : 'text-ink-muted border-transparent hover:text-ink hover:border-line-strong'}"
        >
          <span>{scenario.name}</span>
          <span class="text-[9px] font-mono px-1.5 py-0.5 rounded {statusColor(scenario.status)}">
            {scenario.status}
          </span>
        </button>
        {#if endpoint.scenarios.length > 1}
          <button
            onclick={() => deleteScenario(scenario.id)}
            class="w-4 h-4 flex items-center justify-center text-ink-dim hover:text-err
                   text-xs transition-colors duration-150 shrink-0 rounded hover:bg-err/10"
          >×</button>
        {/if}
      </div>
    {/each}
    <button
      onclick={addScenario}
      class="px-3 h-9 text-xs text-ink-dim hover:text-ink transition-colors duration-150
             whitespace-nowrap border-b-2 border-transparent shrink-0"
    >
      + scenario
    </button>
  </div>

  <!-- Scenario Editor -->
  <div class="flex-1 min-h-0">
    {#if activeScenario}
      <ScenarioEditor scenario={activeScenario} onChange={updateScenario} />
    {/if}
  </div>
</div>
