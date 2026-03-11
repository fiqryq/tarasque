<script lang="ts">
  import { Search, X, Trash2, Download } from 'lucide-svelte';
  import { logs, clearLogs } from '$lib/stores/logs';
  import type { LogEntry } from '$lib/types';

  interface Props {
    selectedLog: LogEntry | null;
    onSelect: (log: LogEntry) => void;
  }

  let { selectedLog, onSelect }: Props = $props();

  let searchQuery = $state('');
  let methodFilter = $state<string | null>(null);
  let statusFilter = $state<string | null>(null);

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'];
  const STATUS_GROUPS = [
    { key: '2xx', label: '2xx', min: 200, max: 299 },
    { key: '3xx', label: '3xx', min: 300, max: 399 },
    { key: '4xx', label: '4xx', min: 400, max: 499 },
    { key: '5xx', label: '5xx', min: 500, max: 599 },
  ];

  const METHOD_TEXT: Record<string, string> = {
    GET:    'text-emerald-400',
    POST:   'text-violet-400',
    PUT:    'text-sky-400',
    DELETE: 'text-rose-400',
    PATCH:  'text-amber-400',
  };

  const METHOD_BG: Record<string, string> = {
    GET:    'bg-emerald-500/10',
    POST:   'bg-violet-500/10',
    PUT:    'bg-sky-500/10',
    DELETE: 'bg-rose-500/10',
    PATCH:  'bg-amber-500/10',
  };

  const STATUS_COLOR: Record<string, string> = {
    '2xx': 'text-emerald-400 bg-emerald-500/10',
    '3xx': 'text-amber-400 bg-amber-500/10',
    '4xx': 'text-orange-400 bg-orange-500/10',
    '5xx': 'text-rose-400 bg-rose-500/10',
  };

  function statusBadge(status: number): string {
    if (status < 300) return 'text-emerald-400 bg-emerald-500/10';
    if (status < 400) return 'text-amber-400 bg-amber-500/10';
    if (status < 500) return 'text-orange-400 bg-orange-500/10';
    return 'text-rose-400 bg-rose-500/10';
  }

  function statusGroup(status: number): string {
    if (status < 300) return '2xx';
    if (status < 400) return '3xx';
    if (status < 500) return '4xx';
    return '5xx';
  }

  let filteredLogs = $derived(
    [...$logs].reverse().filter(log => {
      if (searchQuery && !log.path.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      if (methodFilter && log.method !== methodFilter) return false;
      if (statusFilter && statusGroup(log.status) !== statusFilter) return false;
      return true;
    })
  );

  let activeFilters = $derived(
    (searchQuery ? 1 : 0) + (methodFilter ? 1 : 0) + (statusFilter ? 1 : 0)
  );

  function clearFilters() {
    searchQuery = '';
    methodFilter = null;
    statusFilter = null;
  }

  function isSelected(log: LogEntry): boolean {
    return selectedLog?.timestamp === log.timestamp && selectedLog?.path === log.path;
  }

  function exportHar() {
    const entries = filteredLogs.map((log) => ({
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
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="px-3 h-11 flex items-center gap-2 border-b border-line shrink-0">
    <span class="font-semibold text-body text-ink-secondary uppercase tracking-widest flex-1">
      Logs
    </span>
    {#if $logs.length > 0}
      <span class="text-caption font-mono text-ink-tertiary">
        {$logs.length}
      </span>
    {/if}
  </div>

  <!-- Search -->
  <div class="px-3 py-2 border-b border-line space-y-2 shrink-0">
    <div class="relative">
      <Search size={12} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-tertiary" />
      <input
        bind:value={searchQuery}
        placeholder="Filter by path..."
        aria-label="Filter logs by path"
        class="w-full bg-surface text-ink text-xs pl-7 pr-7 py-1.5 rounded border border-line-strong
               placeholder:text-ink-tertiary focus:outline-none focus:border-accent transition-colors"
      />
      {#if searchQuery}
        <button
          onclick={() => searchQuery = ''}
          class="absolute right-2 top-1/2 -translate-y-1/2 text-ink-tertiary hover:text-ink"
        >
          <X size={10} />
        </button>
      {/if}
    </div>

    <!-- Method filter -->
    <div class="flex flex-wrap gap-1">
      {#each METHODS as method}
        <button
          onclick={() => methodFilter = methodFilter === method ? null : method}
          class="px-1.5 py-0.5 text-caption font-mono font-bold rounded-hig-sm transition-colors
            {methodFilter === method
              ? (METHOD_TEXT[method] ?? '') + ' ' + (METHOD_BG[method] ?? '') + ' ring-1 ring-inset ring-current'
              : 'text-ink-tertiary hover:text-ink'}"
        >
          {method}
        </button>
      {/each}
    </div>

    <!-- Status filter -->
    <div class="flex gap-1">
      {#each STATUS_GROUPS as { key, label }}
        <button
          onclick={() => statusFilter = statusFilter === key ? null : key}
          class="px-1.5 py-0.5 text-caption font-mono font-bold rounded-hig-sm transition-colors
            {statusFilter === key
              ? (STATUS_COLOR[key] ?? '') + ' ring-1 ring-inset ring-current'
              : 'text-ink-tertiary hover:text-ink'}"
        >
          {label}
        </button>
      {/each}
    </div>

    {#if activeFilters > 0}
      <button
        onclick={clearFilters}
        class="text-caption text-accent hover:underline"
      >
        Clear {activeFilters} filter{activeFilters > 1 ? 's' : ''}
      </button>
    {/if}
  </div>

  <!-- Log list -->
  <div class="flex-1 overflow-y-auto">
    {#if filteredLogs.length === 0}
      <div class="px-4 mt-10 text-center">
        <p class="text-ink-tertiary text-body leading-relaxed">
          {#if $logs.length === 0}
            No requests yet.<br />Start a server and make requests.
          {:else}
            No logs match filters.
          {/if}
        </p>
      </div>
    {:else}
      {#each filteredLogs as log (log.timestamp + log.method + log.path)}
        <button
          onclick={() => onSelect(log)}
          class="w-full text-left px-3 py-2 flex items-center gap-2 border-b border-line
                 transition-colors duration-100 group
                 {isSelected(log) ? 'bg-accent/12 border-l-2 border-l-accent' : 'border-l-2 border-l-transparent hover:bg-[var(--color-hover)]'}"
        >
          <span class="text-caption font-mono font-bold w-10 shrink-0 text-center rounded-hig-sm px-1 py-0.5
                       {METHOD_TEXT[log.method] ?? 'text-ink-secondary'} {METHOD_BG[log.method] ?? 'bg-surface'}">
            {log.method}
          </span>
          <span class="flex-1 min-w-0 flex flex-col">
            <span class="text-body font-mono truncate {isSelected(log) ? 'text-ink' : 'text-ink-secondary'}">
              {log.path}
            </span>
            <span class="text-caption text-ink-tertiary font-mono">
              {new Date(log.timestamp).toLocaleTimeString()}
            </span>
          </span>
          <div class="flex flex-col items-end gap-0.5 shrink-0">
            <span class="text-caption font-mono font-bold px-1.5 py-0.5 rounded-hig-sm {statusBadge(log.status)}">
              {log.status}
            </span>
            <span class="text-caption font-mono text-ink-tertiary">{log.duration}ms</span>
          </div>
        </button>
      {/each}
    {/if}
  </div>

  <!-- Footer actions -->
  {#if $logs.length > 0}
    <div class="px-3 py-2 border-t border-line shrink-0 flex items-center gap-2">
      <button
        onclick={exportHar}
        class="flex items-center gap-1 text-caption text-ink-tertiary hover:text-ink transition-colors"
        title="Export filtered logs as HAR"
      >
        <Download size={10} />
        HAR
      </button>
      <div class="flex-1"></div>
      <button
        onclick={clearLogs}
        class="flex items-center gap-1 text-caption text-ink-tertiary hover:text-err transition-colors"
        title="Clear all logs"
      >
        <Trash2 size={10} />
        Clear
      </button>
    </div>
  {/if}
</div>
