<script lang="ts">
  import type { MatchRule } from '$lib/types';
  import { Plus, X } from 'lucide-svelte';

  let { rules, onchange }: { rules: MatchRule[]; onchange: (rules: MatchRule[]) => void } = $props();

  const sources = ['header', 'query', 'body', 'path_param'] as const;
  const operators = ['eq', 'contains', 'exists', 'not_exists'] as const;

  function addRule() {
    onchange([...rules, { source: 'header', key: '', operator: 'eq', value: '' }]);
  }

  function removeRule(index: number) {
    onchange(rules.filter((_, i) => i !== index));
  }

  function updateRule(index: number, patch: Partial<MatchRule>) {
    onchange(rules.map((r, i) => (i === index ? { ...r, ...patch } : r)));
  }

  const needsValue = (op: string) => op !== 'exists' && op !== 'not_exists';
</script>

<div class="space-y-2 p-3">
  {#if rules.length === 0}
    <p class="text-[11px] text-ink-dim leading-relaxed">
      No rules — this scenario always responds when it's the active scenario.
      Add rules to auto-select it based on incoming request data.
    </p>
  {/if}

  {#each rules as rule, i}
    <div class="flex items-center gap-1.5">
      <select
        value={rule.source}
        onchange={(e) => updateRule(i, { source: e.currentTarget.value as MatchRule['source'] })}
        class="flex-none w-24 px-1.5 py-1 bg-app border border-line rounded text-[11px] text-ink focus:outline-none focus:border-accent"
      >
        {#each sources as s}
          <option value={s} class="bg-app">{s}</option>
        {/each}
      </select>

      <input
        type="text"
        placeholder="key"
        value={rule.key}
        oninput={(e) => updateRule(i, { key: e.currentTarget.value })}
        class="flex-1 min-w-0 px-1.5 py-1 bg-app border border-line rounded text-[11px] text-ink
               placeholder:text-ink-dim/50 focus:outline-none focus:border-accent"
      />

      <select
        value={rule.operator}
        onchange={(e) => updateRule(i, { operator: e.currentTarget.value as MatchRule['operator'] })}
        class="flex-none w-28 px-1.5 py-1 bg-app border border-line rounded text-[11px] text-ink focus:outline-none focus:border-accent"
      >
        {#each operators as op}
          <option value={op} class="bg-app">{op}</option>
        {/each}
      </select>

      {#if needsValue(rule.operator)}
        <input
          type="text"
          placeholder="value"
          value={rule.value ?? ''}
          oninput={(e) => updateRule(i, { value: e.currentTarget.value })}
          class="flex-1 min-w-0 px-1.5 py-1 bg-app border border-line rounded text-[11px] text-ink
                 placeholder:text-ink-dim/50 focus:outline-none focus:border-accent"
        />
      {:else}
        <div class="flex-1"></div>
      {/if}

      <button
        onclick={() => removeRule(i)}
        class="flex-none p-0.5 text-ink-dim hover:text-err transition-colors"
        title="Remove rule"
      >
        <X size={12} />
      </button>
    </div>
  {/each}

  <button
    onclick={addRule}
    class="flex items-center gap-1 text-[11px] text-ink-dim hover:text-ink transition-colors mt-1"
  >
    <Plus size={11} />
    Add Rule
  </button>
</div>
