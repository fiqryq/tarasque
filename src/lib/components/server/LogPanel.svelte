<script lang="ts">
  import { logs, clearLogs } from '$lib/stores/logs';
  import { logPanel } from '$lib/stores/logPanel';
  import { Trash2, ChevronDown, ChevronUp, ChevronRight } from 'lucide-svelte';
  import type { LogEntry } from '$lib/types';
  import StatsPanel from './StatsPanel.svelte';

  let activeTab = $state<'logs' | 'stats'>('logs');

  function exportHar(logEntries: LogEntry[]) {
    const entries = logEntries.map((log) => ({
      startedDateTime: log.timestamp,
      time: log.duration,
      request: {
        method: log.method,
        url: `http://localhost${log.path}`,
        httpVersion: 'HTTP/1.1',
        headers: Object.entries(log.headers ?? {}).map(([name, value]) => ({ name, value })),
        queryString: Object.entries(log.query ?? {}).map(([name, value]) => ({ name, value })),
        postData: log.body ? { mimeType: 'application/json', text: log.body } : undefined,
        cookies: [],
        headersSize: -1,
        bodySize: log.body?.length ?? 0,
      },
      response: {
        status: log.status,
        statusText: '',
        headers: [],
        content: {
          size: log.response_body?.length ?? 0,
          mimeType: 'application/json',
          text: log.response_body ?? '',
        },
        cookies: [],
        redirectURL: '',
        headersSize: -1,
        bodySize: log.response_body?.length ?? -1,
      },
      cache: {},
      timings: { send: 0, wait: log.duration, receive: 0 },
    }));

    const har = {
      log: {
        version: '1.2',
        creator: { name: 'Tarasque', version: '2.0' },
        entries,
      },
    };

    const blob = new Blob([JSON.stringify(har, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `tarasque-${new Date().toISOString().slice(0, 10)}.har`;
    a.click();
    URL.revokeObjectURL(url);
  }

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
  let startY = 0;
  let startHeight = 0;
  let isResizing = false;
  let expandedLogs = $state<Set<string>>(new Set());

  function toggleExpand(timestamp: string) {
    if (expandedLogs.has(timestamp)) {
      expandedLogs.delete(timestamp);
    } else {
      expandedLogs.add(timestamp);
    }
    expandedLogs = new Set(expandedLogs);
  }

  $effect(() => {
    if ($logs.length > 0 && logContainer && !$logPanel.collapsed) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  function startResize(e: MouseEvent) {
    isResizing = true;
    startY = e.clientY;
    startHeight = $logPanel.height;
    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
  }

  function handleResize(e: MouseEvent) {
    if (!isResizing) return;
    const delta = startY - e.clientY;
    logPanel.setHeight(startHeight + delta);
  }

  function stopResize() {
    isResizing = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
  }
</script>

{#if $logPanel.collapsed}
  <div class="h-9 border-t border-line flex items-center justify-between px-4 bg-surface">
    <button
      onclick={() => logPanel.toggle()}
      class="flex items-center gap-2 text-ink-muted hover:text-ink transition-colors"
    >
      <ChevronUp size={14} />
      <span class="font-krub text-[11px] font-semibold uppercase tracking-widest">Logs</span>
      <span class="text-[11px] text-ink-dim">({$logs.length})</span>
    </button>
    <button
      onclick={clearLogs}
      class="w-5 h-5 flex items-center justify-center rounded text-ink-dim hover:text-ink-muted
             hover:bg-hover transition-colors duration-150"
      title="Clear logs"
    >
      <Trash2 size={10} />
    </button>
  </div>
{:else}
  <div class="flex flex-col border-t border-line" style="height: {$logPanel.height}px">
    <!-- Resize handle -->
    <div 
      class="h-1 w-full cursor-ns-resize hover:bg-accent/50 transition-colors shrink-0"
      onmousedown={startResize}
      role="separator"
    ></div>
    
    <div class="flex flex-col flex-1 min-h-0 bg-surface">
      <div class="flex items-center px-4 h-8 border-b border-line shrink-0 gap-3">
        <button
          onclick={() => logPanel.toggle()}
          class="flex items-center gap-1 text-ink-muted hover:text-ink transition-colors shrink-0"
        >
          <ChevronDown size={14} />
        </button>

        <!-- Tabs -->
        <div class="flex items-center gap-1">
          <button
            onclick={() => activeTab = 'logs'}
            class="flex items-center gap-1.5 px-2 h-6 text-[11px] font-krub font-semibold uppercase tracking-widest rounded transition-colors
                   {activeTab === 'logs' ? 'text-ink bg-hover' : 'text-ink-dim hover:text-ink'}"
          >
            Logs
            <span class="text-[9px] font-mono font-normal">({$logs.length})</span>
          </button>
          <button
            onclick={() => activeTab = 'stats'}
            class="px-2 h-6 text-[11px] font-krub font-semibold uppercase tracking-widest rounded transition-colors
                   {activeTab === 'stats' ? 'text-ink bg-hover' : 'text-ink-dim hover:text-ink'}"
          >
            Stats
          </button>
        </div>

        <div class="flex-1"></div>
        {#if activeTab === 'logs'}
          {#if $logs.length > 0}
            <button
              onclick={() => exportHar($logs)}
              class="px-2 h-5 text-[9px] font-krub text-ink-dim hover:text-ink border border-line rounded
                     hover:border-line-strong transition-colors"
              title="Export as HAR"
            >
              HAR
            </button>
          {/if}
          <button
            onclick={clearLogs}
            class="w-5 h-5 flex items-center justify-center rounded text-ink-dim hover:text-ink-muted
                   hover:bg-hover transition-colors duration-150"
            title="Clear logs"
          >
            <Trash2 size={10} />
          </button>
        {/if}
      </div>

      {#if activeTab === 'stats'}
        <StatsPanel />
      {:else}
      <div bind:this={logContainer} class="flex-1 overflow-y-auto font-mono text-[11px]">
        {#each $logs as log (log.timestamp + log.path)}
          <div class="border-b border-line/50">
            <!-- Log Row -->
            <div 
              class="flex items-center gap-2 px-3 py-1.5 hover:bg-hover/40 transition-colors cursor-pointer group"
              onclick={() => toggleExpand(log.timestamp + log.path)}
            >
              <ChevronRight 
                size={12} 
                class="text-ink-dim transition-transform {expandedLogs.has(log.timestamp + log.path) ? 'rotate-90' : ''}" 
              />
              <span class="text-ink-dim shrink-0 tabular-nums w-[72px] text-[11px]">
                {new Date(log.timestamp).toLocaleTimeString()}
              </span>
              <span class="font-bold w-12 shrink-0 text-[11px] {METHOD_TEXT[log.method] ?? 'text-ink-muted'}">
                {log.method}
              </span>
              <span class="flex-1 text-ink-muted truncate group-hover:text-ink transition-colors">
                {log.path}
              </span>
              {#if log.proxied}
                <span class="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 shrink-0" title="Proxied">
                  ↗
                </span>
              {/if}
              <span class="shrink-0 tabular-nums text-[11px] px-1.5 py-0.5 rounded font-bold {statusBadge(log.status)}">
                {log.status}
              </span>
              <span class="text-ink-dim shrink-0 tabular-nums w-14 text-right text-[11px]">
                {log.duration}ms
              </span>
            </div>

            <!-- Expanded Details -->
            {#if expandedLogs.has(log.timestamp + log.path)}
              <div class="px-4 pb-3 bg-[#151515] space-y-2 text-[11px]">
                <!-- Query -->
                {#if log.query && Object.keys(log.query).length > 0}
                  <div>
                    <span class="text-ink-dim">Query:</span>
                    <div class="mt-1 pl-2 font-mono text-ink-muted">
                      {#each Object.entries(log.query) as [key, val]}
                        <div>{key}={val}</div>
                      {/each}
                    </div>
                  </div>
                {/if}

                <!-- Headers -->
                {#if log.headers && Object.keys(log.headers).length > 0}
                  <div>
                    <span class="text-ink-dim">Headers:</span>
                    <div class="mt-1 pl-2 font-mono text-ink-muted max-h-24 overflow-y-auto">
                      {#each Object.entries(log.headers) as [key, val]}
                        <div><span class="text-violet-400">{key}</span>: {val}</div>
                      {/each}
                    </div>
                  </div>
                {/if}

                <!-- Request Body -->
                {#if log.body}
                  <div>
                    <span class="text-ink-dim">Request Body:</span>
                    <pre class="mt-1 pl-2 py-1 bg-surface rounded text-ink-muted overflow-x-auto max-h-32">{log.body}</pre>
                  </div>
                {/if}

                <!-- Response Body -->
                {#if log.response_body}
                  <div>
                    <span class="text-ink-dim">Response Body:</span>
                    <pre class="mt-1 pl-2 py-1 bg-surface rounded text-emerald-400 overflow-x-auto max-h-32">{log.response_body}</pre>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}

        {#if $logs.length === 0}
          <p class="text-ink-dim text-center py-3 text-[11px]">No requests yet</p>
        {/if}
      </div>
      {/if}
    </div>
  </div>
{/if}