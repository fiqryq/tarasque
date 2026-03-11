<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';

  interface EndpointStats {
    count: number;
    error_count: number;
    total_duration: number;
    durations: number[];
    last_hit?: string;
  }

  let stats = $state<Record<string, EndpointStats>>({});
  let unlisten: (() => void) | undefined;

  onMount(async () => {
    unlisten = await listen<Record<string, EndpointStats>>('server:stats', (e) => {
      stats = e.payload;
    });
  });

  onDestroy(() => unlisten?.());

  function p95(durations: number[]): number {
    if (!durations.length) return 0;
    const sorted = [...durations].sort((a, b) => a - b);
    return sorted[Math.floor(sorted.length * 0.95)] ?? sorted[sorted.length - 1];
  }

  function avg(s: EndpointStats): number {
    return s.count ? Math.round(s.total_duration / s.count) : 0;
  }
</script>

<div class="overflow-auto h-full font-mono text-[11px]">
  {#if Object.keys(stats).length === 0}
    <p class="text-ink-dim p-4 font-inter text-xs">No requests yet. Start the server and make requests.</p>
  {:else}
    <table class="w-full">
      <thead class="sticky top-0 bg-surface">
        <tr class="text-ink-dim border-b border-line text-[10px] font-krub uppercase tracking-wide">
          <th class="text-left px-3 py-1.5">Endpoint</th>
          <th class="text-right px-3 py-1.5">Hits</th>
          <th class="text-right px-3 py-1.5">Errors</th>
          <th class="text-right px-3 py-1.5">Avg</th>
          <th class="text-right px-3 py-1.5">p95</th>
          <th class="text-right px-3 py-1.5">Last Hit</th>
        </tr>
      </thead>
      <tbody>
        {#each Object.entries(stats) as [key, s]}
          <tr class="border-b border-line hover:bg-hover/30 transition-colors">
            <td class="px-3 py-1 text-ink">{key}</td>
            <td class="px-3 py-1 text-right text-ink">{s.count}</td>
            <td class="px-3 py-1 text-right {s.error_count > 0 ? 'text-err' : 'text-ink-dim'}">{s.error_count}</td>
            <td class="px-3 py-1 text-right text-ink-dim">{avg(s)}ms</td>
            <td class="px-3 py-1 text-right text-ink-dim">{p95(s.durations)}ms</td>
            <td class="px-3 py-1 text-right text-ink-dim">
              {s.last_hit ? new Date(s.last_hit).toLocaleTimeString() : '—'}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
