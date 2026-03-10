<script lang="ts">
  import MonacoEditor from './MonacoEditor.svelte';
  import ScenarioRules from './ScenarioRules.svelte';
  import type { Scenario, MatchRule } from '$lib/types';
  import { HelpCircle, X } from 'lucide-svelte';

  interface Props {
    scenario: Scenario;
    onChange: (scenario: Scenario) => void;
  }

  let { scenario, onChange }: Props = $props();

  function update(partial: Partial<Scenario>) {
    onChange({ ...scenario, ...partial });
  }

  type Tab = 'params' | 'headers' | 'body' | 'response' | 'rules';
  let activeTab = $state<Tab>('body');

  const matchRules = $derived(scenario.match_rules ?? []);

  function updateRules(rules: MatchRule[]) {
    update({ match_rules: rules });
  }

  // Local mirrors for controlled inputs — synced from prop so external updates (e.g. template load) reflect immediately
  let localStatus  = $state(scenario.status);
  let localDelayMs = $state(scenario.delay_ms);
  $effect(() => { localStatus  = scenario.status; });
  $effect(() => { localDelayMs = scenario.delay_ms; });

  let newHeaderKey = $state('');
  let newHeaderVal = $state('');
  let newParamKey   = $state('');
  let newParamVal   = $state('');
  let showTemplateHelp = $state(false);

  const queryParams = $derived(scenario.query_params ?? {});

  function addParam() {
    if (newParamKey.trim()) {
      update({ query_params: { ...queryParams, [newParamKey.trim()]: newParamVal } });
      newParamKey = '';
      newParamVal = '';
    }
  }

  function removeParam(key: string) {
    const { [key]: _, ...rest } = queryParams;
    update({ query_params: rest });
  }

  function renameParamKey(oldKey: string, newKey: string) {
    if (!newKey.trim() || newKey === oldKey) return;
    const updated: Record<string, string> = {};
    for (const [k, v] of Object.entries(queryParams)) {
      updated[k === oldKey ? newKey.trim() : k] = v;
    }
    update({ query_params: updated });
  }

  function updateParamVal(key: string, val: string) {
    update({ query_params: { ...queryParams, [key]: val } });
  }

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

  function renameHeaderKey(oldKey: string, newKey: string) {
    if (!newKey.trim() || newKey === oldKey) return;
    const entries = Object.entries(scenario.headers);
    const updated: Record<string, string> = {};
    for (const [k, v] of entries) {
      updated[k === oldKey ? newKey.trim() : k] = v;
    }
    update({ headers: updated });
  }

  function updateHeaderVal(key: string, val: string) {
    update({ headers: { ...scenario.headers, [key]: val } });
  }

  interface ResponseTemplate {
    label: string;
    status: number;
    body: string;
  }

  const RESPONSE_TEMPLATES: { group: string; items: ResponseTemplate[] }[] = [
    {
      group: 'GET — Single resource',
      items: [
        {
          label: 'User',
          status: 200,
          body: JSON.stringify({ id: '{{uuid}}', name: 'John Doe', email: 'john@example.com', avatar: 'https://i.pravatar.cc/150', created_at: '{{iso_timestamp}}' }, null, 2),
        },
        {
          label: 'Product',
          status: 200,
          body: JSON.stringify({ id: '{{uuid}}', name: 'Sample Product', description: 'A great product.', price: 29.99, currency: 'USD', stock: 100, created_at: '{{iso_timestamp}}' }, null, 2),
        },
        {
          label: 'Article / Post',
          status: 200,
          body: JSON.stringify({ id: '{{uuid}}', title: 'Hello World', content: 'Lorem ipsum dolor sit amet.', author: 'Jane Doe', tags: ['news', 'general'], published_at: '{{iso_timestamp}}' }, null, 2),
        },
      ],
    },
    {
      group: 'GET — List',
      items: [
        {
          label: 'Array list',
          status: 200,
          body: JSON.stringify([
            { id: '{{uuid}}', name: 'Item 1', status: 'active' },
            { id: '{{uuid}}', name: 'Item 2', status: 'inactive' },
          ], null, 2),
        },
        {
          label: 'Paginated list',
          status: 200,
          body: JSON.stringify({
            data: [{ id: '{{uuid}}', name: 'Item 1' }, { id: '{{uuid}}', name: 'Item 2' }],
            meta: { total: 100, page: 1, per_page: 20, total_pages: 5 },
          }, null, 2),
        },
        {
          label: 'Empty list',
          status: 200,
          body: JSON.stringify({ data: [], meta: { total: 0, page: 1, per_page: 20, total_pages: 0 } }, null, 2),
        },
      ],
    },
    {
      group: 'POST / PUT / PATCH',
      items: [
        {
          label: 'Created (201)',
          status: 201,
          body: JSON.stringify({ id: '{{uuid}}', message: 'Resource created successfully', created_at: '{{iso_timestamp}}' }, null, 2),
        },
        {
          label: 'Updated (200)',
          status: 200,
          body: JSON.stringify({ id: '{{request.params.id}}', message: 'Resource updated successfully', updated_at: '{{iso_timestamp}}' }, null, 2),
        },
        {
          label: 'Accepted (202)',
          status: 202,
          body: JSON.stringify({ message: 'Request accepted and being processed', job_id: '{{uuid}}', status: 'processing' }, null, 2),
        },
      ],
    },
    {
      group: 'DELETE',
      items: [
        { label: 'No content (204)', status: 204, body: '' },
        { label: 'Deleted with message', status: 200, body: JSON.stringify({ message: 'Resource deleted successfully' }, null, 2) },
      ],
    },
    {
      group: 'Errors',
      items: [
        { label: '400 Bad Request', status: 400, body: JSON.stringify({ error: 'bad_request', message: 'The request is invalid', details: [] }, null, 2) },
        { label: '401 Unauthorized', status: 401, body: JSON.stringify({ error: 'unauthorized', message: 'Authentication is required' }, null, 2) },
        { label: '403 Forbidden', status: 403, body: JSON.stringify({ error: 'forbidden', message: 'You do not have permission to access this resource' }, null, 2) },
        { label: '404 Not Found', status: 404, body: JSON.stringify({ error: 'not_found', message: 'Resource not found' }, null, 2) },
        { label: '409 Conflict', status: 409, body: JSON.stringify({ error: 'conflict', message: 'Resource already exists' }, null, 2) },
        {
          label: '422 Validation Error',
          status: 422,
          body: JSON.stringify({ error: 'validation_error', message: 'The given data was invalid', errors: { email: ['The email field is required'], name: ['The name must be at least 2 characters'] } }, null, 2),
        },
        { label: '500 Server Error', status: 500, body: JSON.stringify({ error: 'internal_server_error', message: 'An unexpected error occurred' }, null, 2) },
      ],
    },
  ];

  function applyTemplate(tpl: ResponseTemplate) {
    update({ body: tpl.body, status: tpl.status });
  }

  const STATUS_TEXT: Record<number, string> = {
    200: 'OK', 201: 'Created', 202: 'Accepted', 204: 'No Content',
    301: 'Moved Permanently', 302: 'Found', 304: 'Not Modified',
    400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden',
    404: 'Not Found', 405: 'Method Not Allowed', 409: 'Conflict',
    422: 'Unprocessable Entity', 429: 'Too Many Requests',
    500: 'Internal Server Error', 502: 'Bad Gateway', 503: 'Service Unavailable',
  };

  function statusText(code: number): string {
    return STATUS_TEXT[code] ?? 'Unknown';
  }

  function statusColor(code: number): string {
    if (code < 300) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    if (code < 400) return 'text-amber-400 bg-amber-500/10 border-amber-500/20';
    if (code < 500) return 'text-orange-400 bg-orange-500/10 border-orange-500/20';
    return 'text-rose-400 bg-rose-500/10 border-rose-500/20';
  }

  let prettyBody = $derived(
    (() => {
      if (!scenario.body.trim()) return '';
      try { return JSON.stringify(JSON.parse(scenario.body), null, 2); }
      catch { return scenario.body; }
    })()
  );

  const TEMPLATE_VARS = [
    { var: '{{uuid}}', desc: 'Random UUID v4' },
    { var: '{{timestamp}}', desc: 'Unix timestamp' },
    { var: '{{iso_timestamp}}', desc: 'ISO 8601 timestamp' },
    { var: '{{random.int}}', desc: 'Random integer 0-999' },
    { var: '{{random.string}}', desc: 'Random 6-char string' },
    { var: '{{random.bool}}', desc: 'Random true/false' },
    { var: '{{request.params.id}}', desc: 'Path param "id"' },
    { var: '{{request.query.page}}', desc: 'Query param "page"' },
    { var: '{{request.body.name}}', desc: 'Body field "name"' },
    { var: '{{request.headers.Auth}}', desc: 'Header "Auth"' },
  ];
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Status + Delay row -->
  <div class="flex gap-4 px-4 py-3 border-b border-line shrink-0">
    <div class="flex flex-col gap-1.5">
      <label for="status-code" class="text-[11px] font-krub text-ink-muted uppercase tracking-widest">Status</label>
      <input
        id="status-code"
        type="number"
        bind:value={localStatus}
        onchange={() => update({ status: localStatus })}
        class="w-20 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150"
      />
    </div>
    <div class="flex flex-col gap-1.5">
      <label for="delay-ms" class="text-[11px] font-krub text-ink-muted uppercase tracking-widest">Delay ms</label>
      <input
        id="delay-ms"
        type="number"
        min="0"
        bind:value={localDelayMs}
        onchange={() => update({ delay_ms: localDelayMs })}
        class="w-24 bg-surface text-ink text-xs font-mono px-2.5 py-1.5 border border-line-strong rounded
               focus:outline-none focus:border-accent transition-colors duration-150"
      />
    </div>
  </div>

  <!-- Tab bar: Params | Headers | Body | Response | Rules -->
  <div class="flex items-center px-4 border-b border-line shrink-0 gap-0">
    {#each ([['params', 'Params', Object.keys(queryParams).length], ['headers', 'Headers', Object.keys(scenario.headers).length], ['body', 'Body', null], ['response', 'Response', null], ['rules', 'Rules', matchRules.length || null]] as const) as [tab, label, count]}
      <button
        onclick={() => activeTab = tab}
        class="flex items-center gap-1.5 px-3 h-8 text-[11px] font-krub border-b-2 transition-colors duration-150
          {activeTab === tab
            ? 'text-ink border-accent'
            : 'text-ink-muted border-transparent hover:text-ink hover:border-line-strong'}"
      >
        {label}
        {#if count !== null && count > 0}
          <span class="text-[9px] font-mono px-1.5 py-0.5 rounded
            {activeTab === tab ? 'bg-accent/15 text-accent' : 'bg-surface text-ink-dim'}">
            {count}
          </span>
        {/if}
      </button>
    {/each}

    <!-- Body tab actions (template pickers) — shown only when body tab active -->
    {#if activeTab === 'body'}
      <div class="ml-auto flex items-center gap-2">
        <select
          onchange={(e) => {
            const [gi, ti] = e.currentTarget.value.split('-').map(Number);
            if (!isNaN(gi) && !isNaN(ti)) applyTemplate(RESPONSE_TEMPLATES[gi].items[ti]);
            e.currentTarget.value = '';
          }}
          class="h-6 text-[11px] font-krub bg-surface text-ink-muted border border-line-strong rounded
                 px-2 focus:outline-none focus:border-accent cursor-pointer hover:border-accent
                 transition-colors duration-150"
        >
          <option value="" disabled selected>Load template…</option>
          {#each RESPONSE_TEMPLATES as group, gi}
            <optgroup label={group.group}>
              {#each group.items as item, ti}
                <option value="{gi}-{ti}">{item.label}</option>
              {/each}
            </optgroup>
          {/each}
        </select>

        <div class="relative">
          <button
            onclick={() => showTemplateHelp = !showTemplateHelp}
            class="flex items-center gap-1 text-ink-dim hover:text-ink transition-colors"
            title="Template variables"
          >
            <HelpCircle size={12} />
            <span class="text-[11px]">Vars</span>
          </button>
          {#if showTemplateHelp}
            <div class="absolute right-0 top-7 z-50 w-72 bg-surface border border-line rounded-lg shadow-xl p-3">
              <div class="text-[11px] font-krub text-ink-muted uppercase tracking-widest mb-2">Template Variables</div>
              <div class="space-y-1 max-h-48 overflow-y-auto">
                {#each TEMPLATE_VARS as item}
                  <div class="flex items-start gap-2">
                    <code class="text-[11px] font-mono text-accent bg-accent/10 px-1.5 py-0.5 rounded shrink-0">{item.var}</code>
                    <span class="text-[11px] text-ink-dim">{item.desc}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Tab content -->
  <div class="flex-1 min-h-0 flex flex-col overflow-hidden">

    <!-- Params tab -->
    {#if activeTab === 'params'}
      <div class="flex flex-col h-full overflow-hidden">
        <div class="flex items-center px-4 py-1 bg-app/40 border-b border-line/40 shrink-0">
          <div class="w-5 shrink-0"></div>
          <span class="flex-1 text-[9px] font-krub text-ink-dim uppercase tracking-wider">Key</span>
          <span class="flex-1 text-[9px] font-krub text-ink-dim uppercase tracking-wider">Example value</span>
          <div class="w-6 shrink-0"></div>
        </div>
        <div class="flex-1 overflow-y-auto">
          {#each Object.entries(queryParams) as [key, val] (key)}
            <div class="flex items-center px-4 border-b border-line/30 group hover:bg-hover/20">
              <div class="w-5 shrink-0 flex items-center">
                <span class="w-1 h-1 rounded-full bg-accent/60 shrink-0"></span>
              </div>
              <input
                value={key}
                onchange={(e) => renameParamKey(key, e.currentTarget.value)}
                class="flex-1 bg-transparent text-ink text-[11px] font-mono py-1.5 px-1
                       border border-transparent rounded hover:border-line
                       focus:border-accent focus:bg-surface focus:outline-none transition-colors duration-100"
              />
              <input
                value={val}
                onchange={(e) => updateParamVal(key, e.currentTarget.value)}
                class="flex-1 bg-transparent text-ink-muted text-[11px] font-mono py-1.5 px-1
                       border border-transparent rounded hover:border-line
                       focus:border-accent focus:bg-surface focus:outline-none transition-colors duration-100"
              />
              <button
                onclick={() => removeParam(key)}
                class="w-6 h-6 flex items-center justify-center text-ink-dim opacity-0
                       group-hover:opacity-100 hover:text-err rounded transition-all duration-100 shrink-0"
              ><X size={11} /></button>
            </div>
          {/each}

          <!-- Add new row -->
          <div class="flex items-center px-4">
            <div class="w-5 shrink-0 text-ink-dim/40 text-[11px]">+</div>
            <input
              bind:value={newParamKey}
              placeholder="Key"
              onkeydown={(e) => e.key === 'Enter' && addParam()}
              class="flex-1 bg-transparent text-ink text-[11px] font-mono py-1.5 px-1
                     border border-transparent rounded focus:border-accent focus:bg-surface focus:outline-none
                     placeholder:text-ink-dim/40 transition-colors duration-100"
            />
            <input
              bind:value={newParamVal}
              placeholder="Example value"
              onkeydown={(e) => e.key === 'Enter' && addParam()}
              onblur={() => newParamKey.trim() && addParam()}
              class="flex-1 bg-transparent text-ink-muted text-[11px] font-mono py-1.5 px-1
                     border border-transparent rounded focus:border-accent focus:bg-surface focus:outline-none
                     placeholder:text-ink-dim/40 transition-colors duration-100"
            />
            <div class="w-6 shrink-0"></div>
          </div>
        </div>

        <!-- Footer hint -->
        {#if Object.keys(queryParams).length > 0}
          <div class="px-4 py-2 border-t border-line/40 shrink-0">
            <p class="text-[11px] text-ink-dim">
              Access in body: <code class="text-accent">{"{{request.query.key}}"}</code>
            </p>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Headers tab -->
    {#if activeTab === 'headers'}
      <div class="flex flex-col h-full overflow-hidden">
        <div class="flex items-center px-4 py-1 bg-app/40 border-b border-line/40 shrink-0">
          <div class="w-5 shrink-0"></div>
          <span class="flex-1 text-[9px] font-krub text-ink-dim uppercase tracking-wider">Key</span>
          <span class="flex-1 text-[9px] font-krub text-ink-dim uppercase tracking-wider">Value</span>
          <div class="w-6 shrink-0"></div>
        </div>
        <div class="flex-1 overflow-y-auto">
          {#each Object.entries(scenario.headers) as [key, val] (key)}
            <div class="flex items-center px-4 border-b border-line/30 group hover:bg-hover/20">
              <div class="w-5 shrink-0 flex items-center">
                <span class="w-1 h-1 rounded-full bg-run/60 shrink-0"></span>
              </div>
              <input
                value={key}
                onchange={(e) => renameHeaderKey(key, e.currentTarget.value)}
                class="flex-1 bg-transparent text-ink text-[11px] font-mono py-1.5 px-1
                       border border-transparent rounded hover:border-line
                       focus:border-accent focus:bg-surface focus:outline-none transition-colors duration-100"
              />
              <input
                value={val}
                onchange={(e) => updateHeaderVal(key, e.currentTarget.value)}
                class="flex-1 bg-transparent text-ink-muted text-[11px] font-mono py-1.5 px-1
                       border border-transparent rounded hover:border-line
                       focus:border-accent focus:bg-surface focus:outline-none transition-colors duration-100"
              />
              <button
                onclick={() => removeHeader(key)}
                class="w-6 h-6 flex items-center justify-center text-ink-dim opacity-0
                       group-hover:opacity-100 hover:text-err rounded transition-all duration-100 shrink-0"
              ><X size={11} /></button>
            </div>
          {/each}

          <!-- Add new row -->
          <div class="flex items-center px-4">
            <div class="w-5 shrink-0 text-ink-dim/40 text-[11px]">+</div>
            <input
              bind:value={newHeaderKey}
              placeholder="Key"
              onkeydown={(e) => e.key === 'Enter' && addHeader()}
              class="flex-1 bg-transparent text-ink text-[11px] font-mono py-1.5 px-1
                     border border-transparent rounded focus:border-accent focus:bg-surface focus:outline-none
                     placeholder:text-ink-dim/40 transition-colors duration-100"
            />
            <input
              bind:value={newHeaderVal}
              placeholder="Value"
              onkeydown={(e) => e.key === 'Enter' && addHeader()}
              onblur={() => newHeaderKey.trim() && addHeader()}
              class="flex-1 bg-transparent text-ink-muted text-[11px] font-mono py-1.5 px-1
                     border border-transparent rounded focus:border-accent focus:bg-surface focus:outline-none
                     placeholder:text-ink-dim/40 transition-colors duration-100"
            />
            <div class="w-6 shrink-0"></div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Body tab -->
    {#if activeTab === 'body'}
      <div class="flex-1 min-h-0">
        <MonacoEditor
          value={scenario.body}
          onChange={(val) => update({ body: val })}
        />
      </div>
    {/if}

    <!-- Rules tab -->
    {#if activeTab === 'rules'}
      <div class="flex-1 overflow-y-auto">
        <ScenarioRules
          rules={matchRules}
          onchange={(rules) => updateRules(rules)}
        />
      </div>
    {/if}

    <!-- Response tab -->
    {#if activeTab === 'response'}
      <div class="flex-1 overflow-y-auto px-4 py-3 space-y-4">

        <!-- Status line -->
        <div class="flex items-center gap-3">
          <span class="text-[11px] font-krub text-ink-dim uppercase tracking-widest w-16 shrink-0">Status</span>
          <span class="inline-flex items-center gap-2 px-3 py-1 rounded border text-xs font-mono font-semibold {statusColor(scenario.status)}">
            {scenario.status} {statusText(scenario.status)}
          </span>
          {#if scenario.delay_ms > 0}
            <span class="text-[11px] font-mono text-ink-dim">{scenario.delay_ms}ms delay</span>
          {/if}
        </div>

        <!-- Response headers -->
        <div>
          <div class="flex items-center gap-3 mb-1.5">
            <span class="text-[11px] font-krub text-ink-dim uppercase tracking-widest w-16 shrink-0">Headers</span>
            <span class="text-[11px] text-ink-dim">{Object.keys(scenario.headers).length} header{Object.keys(scenario.headers).length !== 1 ? 's' : ''}</span>
          </div>
          <div class="rounded border border-line overflow-hidden">
            {#if Object.keys(scenario.headers).length === 0}
              <div class="px-3 py-2 text-[11px] text-ink-dim font-mono italic">No response headers defined</div>
            {:else}
              {#each Object.entries(scenario.headers) as [key, val], i (key)}
                <div class="flex items-start px-3 py-1.5 font-mono text-[11px]
                  {i % 2 === 0 ? 'bg-surface' : 'bg-app/40'}">
                  <span class="text-accent w-48 shrink-0 truncate">{key}</span>
                  <span class="text-ink-muted flex-1 break-all">{val}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Response body -->
        <div>
          <div class="flex items-center gap-3 mb-1.5">
            <span class="text-[11px] font-krub text-ink-dim uppercase tracking-widest w-16 shrink-0">Body</span>
            {#if scenario.body.trim()}
              <span class="text-[11px] text-ink-dim">{scenario.body.trim().length} chars</span>
            {/if}
          </div>
          {#if !scenario.body.trim()}
            <div class="px-3 py-2 rounded border border-line text-[11px] text-ink-dim font-mono italic bg-surface">
              Empty body
            </div>
          {:else}
            <pre class="rounded border border-line bg-surface text-[11px] font-mono text-ink-muted
                        px-3 py-2.5 overflow-x-auto whitespace-pre-wrap break-words leading-relaxed
                        max-h-72 overflow-y-auto">{prettyBody}</pre>
          {/if}
        </div>

        <!-- Query params note -->
        {#if Object.keys(queryParams).length > 0}
          <div>
            <div class="flex items-center gap-3 mb-1.5">
              <span class="text-[11px] font-krub text-ink-dim uppercase tracking-widest w-16 shrink-0">Params</span>
              <span class="text-[11px] text-ink-dim">{Object.keys(queryParams).length} expected</span>
            </div>
            <div class="rounded border border-line overflow-hidden">
              {#each Object.entries(queryParams) as [key, val], i (key)}
                <div class="flex items-center px-3 py-1.5 font-mono text-[11px]
                  {i % 2 === 0 ? 'bg-surface' : 'bg-app/40'}">
                  <span class="text-accent w-48 shrink-0">?{key}</span>
                  <span class="text-ink-dim flex-1">{val || '(any)'}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

      </div>
    {/if}

  </div>
</div>
