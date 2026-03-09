<script lang="ts">
  import { logs, clearLogs } from '$lib/stores/logs';
  import { Trash2 } from 'lucide-svelte';

  function statusBadge(status: number): string {
    if (status < 300) return 'text-emerald-400 bg-emerald-500/10';
    if (status < 400) return 'text-amber-400 bg-amber-500/10';
    if (status < 500) return 'text-orange-400 bg-orange-500/10';
    return 'text-rose-400 bg-rose-500/10';
  }

  const METHOD_TEXT: Record<string, string> = {
    GET:    'text-emerald-400',
    POST:   'text-violet-400',
    PUT:    'text-sky-400',
    DELETE: 'text-rose-400',
    PATCH:  'text-amber-400',
  };

  let logContainer: HTMLDivElement;

  $effect(() => {
    if ($logs.length > 0 && logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-4 h-8 border-b border-line shrink-0">
    <span class="font-krub text-[10px] font-semibold text-ink-muted uppercase tracking-widest">Logs</span>
    <button
      onclick={clearLogs}
      class="w-5 h-5 flex items-center justify-center rounded text-ink-dim hover:text-ink-muted
             hover:bg-hover transition-colors duration-150"
      title="Clear logs"
    >
      <Trash2 size={10} />
    </button>
  </div>

  <div bind:this={logContainer} class="flex-1 overflow-y-auto font-mono text-[11px] px-2 py-1">
    {#each $logs as log (log.timestamp + log.path)}
      <div class="flex gap-3 px-2 py-1 rounded hover:bg-hover/40 transition-colors duration-75 group items-center">
        <span class="text-ink-dim shrink-0 tabular-nums w-[72px] text-[10px]">
          {new Date(log.timestamp).toLocaleTimeString()}
        </span>
        <span class="font-bold w-12 shrink-0 text-[10px] {METHOD_TEXT[log.method] ?? 'text-ink-muted'}">
          {log.method}
        </span>
        <span class="flex-1 text-ink-muted truncate group-hover:text-ink transition-colors duration-75">
          {log.path}
        </span>
        <span class="shrink-0 tabular-nums text-[10px] px-1.5 py-0.5 rounded font-bold {statusBadge(log.status)}">
          {log.status}
        </span>
        <span class="text-ink-dim shrink-0 tabular-nums w-14 text-right text-[10px]">
          {log.duration}ms
        </span>
      </div>
    {/each}

    {#if $logs.length === 0}
      <p class="text-ink-dim text-center py-3 text-[11px]">No requests yet</p>
    {/if}
  </div>
</div>
