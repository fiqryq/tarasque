<script lang="ts">
  import { Copy, Check, FileText } from 'lucide-svelte';
  import type { LogEntry } from '$lib/types';

  interface Props {
    log: LogEntry;
  }

  let { log }: Props = $props();

  let activeTab = $state<'headers' | 'query' | 'request' | 'response'>('response');
  let copied = $state<string | null>(null);

  const METHOD_STYLE: Record<string, string> = {
    GET:    'text-emerald-300 bg-emerald-500/10 border-emerald-500/30',
    POST:   'text-violet-300 bg-violet-500/10 border-violet-500/30',
    PUT:    'text-sky-300 bg-sky-500/10 border-sky-500/30',
    DELETE: 'text-rose-300 bg-rose-500/10 border-rose-500/30',
    PATCH:  'text-amber-300 bg-amber-500/10 border-amber-500/30',
  };

  function statusBadge(status: number): string {
    if (status < 300) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
    if (status < 400) return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
    if (status < 500) return 'text-orange-400 bg-orange-500/10 border-orange-500/30';
    return 'text-rose-400 bg-rose-500/10 border-rose-500/30';
  }

  function statusText(status: number): string {
    const map: Record<number, string> = {
      200: 'OK', 201: 'Created', 204: 'No Content',
      301: 'Moved', 302: 'Found', 304: 'Not Modified',
      400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden',
      404: 'Not Found', 422: 'Unprocessable', 429: 'Rate Limited',
      500: 'Server Error', 502: 'Bad Gateway', 503: 'Unavailable',
    };
    return map[status] ?? '';
  }

  function prettyJson(str: string): string {
    if (!str?.trim()) return '';
    try { return JSON.stringify(JSON.parse(str), null, 2); }
    catch { return str; }
  }

  function copyText(key: string, text: string) {
    navigator.clipboard.writeText(text);
    copied = key;
    setTimeout(() => { if (copied === key) copied = null; }, 1500);
  }

  let hasQuery = $derived(log.query && Object.keys(log.query).length > 0);
  let hasHeaders = $derived(log.headers && Object.keys(log.headers).length > 0);
  let hasBody = $derived(!!log.body?.trim());
  let hasResponse = $derived(!!log.response_body?.trim());
</script>

<div class="h-full overflow-y-auto bg-app">
  <div class="w-full px-6 py-6 space-y-5">

    <!-- Request summary -->
    <div class="border border-line rounded-hig p-5 bg-surface">
      <div class="flex items-center gap-3 flex-wrap">
        <span class="text-body font-mono font-bold px-2.5 py-1 rounded-hig-sm border {METHOD_STYLE[log.method] ?? 'text-ink-secondary bg-surface border-line'}">
          {log.method}
        </span>
        <code class="text-sm font-mono text-ink flex-1 min-w-0 truncate">{log.path}</code>
        <span class="text-body font-mono font-bold px-2.5 py-1 rounded-hig-sm border {statusBadge(log.status)}">
          {log.status} {statusText(log.status)}
        </span>
      </div>

      <div class="flex items-center gap-4 mt-3 text-caption font-mono text-ink-tertiary">
        <span>{new Date(log.timestamp).toLocaleString()}</span>
        <span>{log.duration}ms</span>
        {#if log.proxied}
          <span class="text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded">Proxied</span>
        {/if}
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-1 border-b border-line">
      {#if hasResponse}
        <button
          onclick={() => activeTab = 'response'}
          class="px-3 py-2 text-caption font-sf font-semibold uppercase tracking-widest transition-colors border-b-2
                 {activeTab === 'response' ? 'text-ink border-accent' : 'text-ink-tertiary border-transparent hover:text-ink'}"
        >
          Response
        </button>
      {/if}
      {#if hasHeaders}
        <button
          onclick={() => activeTab = 'headers'}
          class="px-3 py-2 text-caption font-sf font-semibold uppercase tracking-widest transition-colors border-b-2
                 {activeTab === 'headers' ? 'text-ink border-accent' : 'text-ink-tertiary border-transparent hover:text-ink'}"
        >
          Headers
          <span class="text-caption font-mono font-normal ml-1 text-ink-tertiary">({Object.keys(log.headers ?? {}).length})</span>
        </button>
      {/if}
      {#if hasQuery}
        <button
          onclick={() => activeTab = 'query'}
          class="px-3 py-2 text-caption font-sf font-semibold uppercase tracking-widest transition-colors border-b-2
                 {activeTab === 'query' ? 'text-ink border-accent' : 'text-ink-tertiary border-transparent hover:text-ink'}"
        >
          Query
          <span class="text-caption font-mono font-normal ml-1 text-ink-tertiary">({Object.keys(log.query ?? {}).length})</span>
        </button>
      {/if}
      {#if hasBody}
        <button
          onclick={() => activeTab = 'request'}
          class="px-3 py-2 text-caption font-sf font-semibold uppercase tracking-widest transition-colors border-b-2
                 {activeTab === 'request' ? 'text-ink border-accent' : 'text-ink-tertiary border-transparent hover:text-ink'}"
        >
          Request Body
        </button>
      {/if}
    </div>

    <!-- Tab content -->
    <div class="border border-line rounded-hig overflow-hidden bg-surface">

      {#if activeTab === 'response' && hasResponse}
        <div class="flex items-center justify-between px-4 py-2 border-b border-line">
          <span class="text-caption text-ink-tertiary uppercase tracking-widest">Response Body</span>
          <button
            onclick={() => copyText('response', log.response_body ?? '')}
            class="flex items-center gap-1 text-caption text-ink-tertiary hover:text-ink transition-colors"
          >
            {#if copied === 'response'}
              <Check size={10} class="text-run" />
              <span class="text-run">Copied</span>
            {:else}
              <Copy size={10} />
              Copy
            {/if}
          </button>
        </div>
        <pre class="px-4 py-3 text-body font-mono text-emerald-300/90 leading-relaxed
                    overflow-auto whitespace-pre-wrap break-words bg-app/40">{prettyJson(log.response_body ?? '')}</pre>
      {/if}

      {#if activeTab === 'headers' && hasHeaders}
        <div class="flex items-center justify-between px-4 py-2 border-b border-line">
          <span class="text-caption text-ink-tertiary uppercase tracking-widest">Request Headers</span>
          <button
            onclick={() => copyText('headers', Object.entries(log.headers).map(([k, v]) => `${k}: ${v}`).join('\n'))}
            class="flex items-center gap-1 text-caption text-ink-tertiary hover:text-ink transition-colors"
          >
            {#if copied === 'headers'}
              <Check size={10} class="text-run" />
            {:else}
              <Copy size={10} />
            {/if}
          </button>
        </div>
        <div class="divide-y divide-line">
          {#each Object.entries(log.headers) as [key, val]}
            <div class="flex items-start gap-3 px-4 py-2 hover:bg-[var(--color-hover)] transition-colors">
              <span class="text-body font-mono text-accent shrink-0 w-48 truncate" title={key}>{key}</span>
              <span class="text-body font-mono text-ink-secondary flex-1 break-all">{val}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if activeTab === 'query' && hasQuery}
        <div class="px-4 py-2 border-b border-line">
          <span class="text-caption text-ink-tertiary uppercase tracking-widest">Query Parameters</span>
        </div>
        <div class="divide-y divide-line">
          {#each Object.entries(log.query) as [key, val]}
            <div class="flex items-start gap-3 px-4 py-2 hover:bg-[var(--color-hover)] transition-colors">
              <span class="text-body font-mono text-accent shrink-0 w-32">{key}</span>
              <span class="text-body font-mono text-ink-secondary flex-1 break-all">{val}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if activeTab === 'request' && hasBody}
        <div class="flex items-center justify-between px-4 py-2 border-b border-line">
          <span class="text-caption text-ink-tertiary uppercase tracking-widest">Request Body</span>
          <button
            onclick={() => copyText('request', log.body)}
            class="flex items-center gap-1 text-caption text-ink-tertiary hover:text-ink transition-colors"
          >
            {#if copied === 'request'}
              <Check size={10} class="text-run" />
            {:else}
              <Copy size={10} />
            {/if}
          </button>
        </div>
        <pre class="px-4 py-3 text-body font-mono text-ink-secondary leading-relaxed
                    overflow-auto whitespace-pre-wrap break-words bg-app/40">{prettyJson(log.body)}</pre>
      {/if}

      {#if (activeTab === 'response' && !hasResponse) || (activeTab === 'headers' && !hasHeaders) || (activeTab === 'query' && !hasQuery) || (activeTab === 'request' && !hasBody)}
        <div class="px-4 py-8 text-center text-body text-ink-tertiary">
          No data available
        </div>
      {/if}
    </div>

  </div>
</div>
