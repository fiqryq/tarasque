<script lang="ts">
  import { X, Plus, Trash2, Check } from 'lucide-svelte';
  import type { Collection, Environment } from '$lib/types';
  import { updateCollectionEnvironments } from '$lib/stores/collections';

  interface Props {
    collection: Collection;
    onclose: () => void;
  }

  let { collection, onclose }: Props = $props();

  let envs = $state<Environment[]>(collection.environments ? [...collection.environments] : []);
  let activeEnvId = $state<string | undefined>(collection.active_env_id);
  let selectedEnvId = $state<string | null>(envs[0]?.id ?? null);

  let selectedEnv = $derived(envs.find((e) => e.id === selectedEnvId) ?? null);

  function addEnv() {
    const id = crypto.randomUUID();
    envs = [...envs, { id, name: 'New Environment', variables: {} }];
    selectedEnvId = id;
  }

  function deleteEnv(id: string) {
    envs = envs.filter((e) => e.id !== id);
    if (activeEnvId === id) activeEnvId = undefined;
    if (selectedEnvId === id) selectedEnvId = envs[0]?.id ?? null;
  }

  function renameEnv(id: string, name: string) {
    envs = envs.map((e) => e.id === id ? { ...e, name } : e);
  }

  function addVar() {
    if (!selectedEnv) return;
    envs = envs.map((e) =>
      e.id === selectedEnv!.id
        ? { ...e, variables: { ...e.variables, '': '' } }
        : e
    );
  }

  function updateVar(oldKey: string, newKey: string, value: string) {
    if (!selectedEnv) return;
    envs = envs.map((e) => {
      if (e.id !== selectedEnv!.id) return e;
      const vars = { ...e.variables };
      if (oldKey !== newKey) delete vars[oldKey];
      vars[newKey] = value;
      return { ...e, variables: vars };
    });
  }

  function deleteVar(key: string) {
    if (!selectedEnv) return;
    envs = envs.map((e) => {
      if (e.id !== selectedEnv!.id) return e;
      const vars = { ...e.variables };
      delete vars[key];
      return { ...e, variables: vars };
    });
  }

  function save() {
    updateCollectionEnvironments(collection.id, {
      environments: envs,
      active_env_id: activeEnvId,
    });
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50"
  onclick={onclose}
>
  <div
    class="bg-surface border border-line rounded-xl w-[440px] shadow-hig-dark max-h-[85vh] flex flex-col"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-line shrink-0">
      <span class="text-sm font-krub text-ink">Environments — {collection.name}</span>
      <button onclick={onclose} class="text-ink-dim hover:text-ink transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded" aria-label="Close">
        <X size={14} />
      </button>
    </div>

    <div class="flex flex-1 min-h-0">
      <!-- Env list (left) -->
      <div class="w-48 shrink-0 border-r border-line flex flex-col">
        <div class="flex-1 overflow-y-auto py-1">
          {#each envs as env (env.id)}
            <div
              class="group flex items-center gap-1 px-2 py-1.5 cursor-pointer transition-colors
                     {selectedEnvId === env.id ? 'bg-hover text-ink' : 'text-ink-muted hover:bg-hover/50'}"
              onclick={() => selectedEnvId = env.id}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && (selectedEnvId = env.id)}
            >
              {#if activeEnvId === env.id}
                <Check size={10} class="text-emerald-400 shrink-0" />
              {:else}
                <span class="w-2.5 shrink-0"></span>
              {/if}
              <span class="flex-1 text-xs truncate">{env.name}</span>
              <button
                onclick={(e) => { e.stopPropagation(); deleteEnv(env.id); }}
                class="opacity-0 group-hover:opacity-100 text-ink-dim hover:text-err shrink-0 focus:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-err rounded"
                aria-label="Delete environment"
              >
                <Trash2 size={9} />
              </button>
            </div>
          {/each}
        </div>
        <div class="border-t border-line p-2">
          <button
            onclick={addEnv}
            class="w-full flex items-center justify-center gap-1 py-1 text-[10px] text-ink-dim hover:text-ink transition-colors"
          >
            <Plus size={10} /> New Environment
          </button>
        </div>
      </div>

      <!-- Env editor (right) -->
      <div class="flex-1 flex flex-col min-w-0 p-4 overflow-y-auto">
        {#if selectedEnv}
          <!-- Name + active toggle -->
          <div class="flex items-center gap-3 mb-4">
            <input
              value={selectedEnv.name}
              onchange={(e) => renameEnv(selectedEnv!.id, (e.target as HTMLInputElement).value)}
              aria-label="Environment name"
              class="flex-1 bg-app border border-line rounded px-2 py-1 text-sm text-ink focus:outline-none focus:border-accent"
            />
            <button
              onclick={() => activeEnvId = activeEnvId === selectedEnv?.id ? undefined : selectedEnv?.id}
              class="flex items-center gap-1.5 px-3 py-1 text-xs font-krub rounded border transition-colors
                     {activeEnvId === selectedEnv?.id
                       ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                       : 'bg-surface border-line text-ink-muted hover:border-accent'}"
            >
              {activeEnvId === selectedEnv?.id ? '✓ Active' : 'Set Active'}
            </button>
          </div>

          <!-- Variables table -->
          <div class="space-y-1.5 mb-3">
            {#each Object.entries(selectedEnv.variables) as [key, value] (key)}
              <div class="flex items-center gap-2">
                <input
                  value={key}
                  onchange={(e) => updateVar(key, (e.target as HTMLInputElement).value, value)}
                  placeholder="KEY"
                  class="w-36 shrink-0 bg-app border border-line rounded px-2 py-1 text-xs font-mono text-ink focus:outline-none focus:border-accent"
                />
                <input
                  value={value}
                  onchange={(e) => updateVar(key, key, (e.target as HTMLInputElement).value)}
                  placeholder="value"
                  class="flex-1 bg-app border border-line rounded px-2 py-1 text-xs font-mono text-ink-muted focus:outline-none focus:border-accent min-w-0"
                />
                <button onclick={() => deleteVar(key)} class="text-ink-dim hover:text-err shrink-0 focus:outline-none focus-visible:ring-2 focus-visible:ring-err rounded" aria-label="Delete variable">
                  <Trash2 size={10} />
                </button>
              </div>
            {/each}
          </div>
          <button
            onclick={addVar}
            class="text-[10px] text-ink-dim hover:text-ink transition-colors text-left"
          >
            + Add variable
          </button>
        {:else}
          <div class="text-center text-xs text-ink-dim mt-8">
            Create or select an environment on the left.
          </div>
        {/if}
      </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-line shrink-0">
      <button
        onclick={onclose}
        class="px-3 py-1.5 text-xs font-krub text-ink-dim hover:text-ink transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={save}
        class="px-4 py-1.5 text-xs font-krub bg-accent text-white rounded hover:opacity-90 transition-opacity"
      >
        Save
      </button>
    </div>
  </div>
</div>
