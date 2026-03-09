<script lang="ts">
  import MonacoEditor from './MonacoEditor.svelte';
  import type { Scenario } from '$lib/types';

  interface Props {
    scenario: Scenario;
    onChange: (scenario: Scenario) => void;
  }

  let { scenario, onChange }: Props = $props();

  function update(partial: Partial<Scenario>) {
    onChange({ ...scenario, ...partial });
  }

  let newHeaderKey = $state('');
  let newHeaderVal = $state('');

  function addHeader() {
    if (newHeaderKey.trim()) {
      update({ headers: { ...scenario.headers, [newHeaderKey.trim()]: newHeaderVal } });
      newHeaderKey = '';
      newHeaderVal = '';
    }
  }

  function removeHeader(key: string) {
    const { [key]: _, ...rest } = scenario.headers;
    update({ headers: rest });
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Status + Delay row -->
  <div class="flex gap-4 px-4 py-3 border-b border-line shrink-0">
    <div class="flex flex-col gap-1.5">
      <label for="status-code" class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Status</label>
      <input
        id="status-code"
        type="number"
        value={scenario.status}
        onchange={(e) => update({ status: Number(e.currentTarget.value) })}
        class="w-20 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150"
      />
    </div>
    <div class="flex flex-col gap-1.5">
      <label for="delay-ms" class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Delay ms</label>
      <input
        id="delay-ms"
        type="number"
        min="0"
        value={scenario.delay_ms}
        onchange={(e) => update({ delay_ms: Number(e.currentTarget.value) })}
        class="w-24 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150"
      />
    </div>
  </div>

  <!-- Headers -->
  <div class="px-4 py-3 border-b border-line shrink-0">
    <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest block mb-2">Headers</span>
    <div class="space-y-1 mb-2">
      {#each Object.entries(scenario.headers) as [key, val] (key)}
        <div class="flex gap-2 items-center group">
          <span class="text-[11px] font-mono text-ink px-2 py-1 bg-surface border border-line rounded flex-1">{key}</span>
          <span class="text-[11px] font-mono text-ink-muted px-2 py-1 bg-surface border border-line rounded flex-1 truncate">{val}</span>
          <button
            onclick={() => removeHeader(key)}
            class="w-6 h-6 flex items-center justify-center text-ink-dim hover:text-err hover:bg-err/10
                   rounded text-sm transition-colors duration-150 shrink-0"
          >×</button>
        </div>
      {/each}
    </div>
    <div class="flex gap-1.5">
      <input
        bind:value={newHeaderKey}
        placeholder="key"
        class="flex-1 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               placeholder:text-ink-dim focus:outline-none focus:border-accent transition-colors duration-150"
      />
      <input
        bind:value={newHeaderVal}
        placeholder="value"
        class="flex-1 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               placeholder:text-ink-dim focus:outline-none focus:border-accent transition-colors duration-150"
      />
      <button
        onclick={addHeader}
        class="px-3 py-1.5 text-[11px] font-krub text-ink-muted hover:text-ink bg-surface border border-line-strong
               hover:border-accent rounded transition-colors duration-150"
      >add</button>
    </div>
  </div>

  <!-- Body -->
  <div class="flex-1 min-h-0 flex flex-col">
    <div class="px-4 h-8 flex items-center border-b border-line shrink-0">
      <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Body</span>
    </div>
    <div class="flex-1 min-h-0">
      <MonacoEditor
        value={scenario.body}
        onChange={(val) => update({ body: val })}
      />
    </div>
  </div>
</div>
