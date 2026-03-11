<script lang="ts">
  import { Copy, ChevronDown, ChevronRight, ExternalLink, Upload, Check } from 'lucide-svelte';
  import type { Collection, Endpoint } from '$lib/types';
  import { addEndpointsFromOpenApi } from '$lib/stores/collections';
  import { toasts } from '$lib/stores/toasts';
  import { createHighlighter, type Highlighter } from 'shiki';

  let highlighter = $state<Highlighter | null>(null);
  const highlightCache = $state<Record<string, string>>({});

  async function initHighlighter() {
    highlighter = await createHighlighter({
      themes: ['vitesse-dark'],
      langs: ['bash', 'javascript', 'python', 'go', 'php', 'rust'],
    });
  }

  initHighlighter();

  const LANG_MAP: Record<Lang, string> = {
    curl: 'bash',
    javascript: 'javascript',
    python: 'python',
    go: 'go',
    php: 'php',
    rust: 'rust',
  };

  function highlight(code: string, lang: Lang): string {
    const cacheKey = `${lang}:${code}`;
    if (highlightCache[cacheKey]) return highlightCache[cacheKey];
    if (!highlighter) return '';
    const html = highlighter.codeToHtml(code, {
      lang: LANG_MAP[lang],
      theme: 'vitesse-dark',
    });
    highlightCache[cacheKey] = html;
    return html;
  }

  interface Props {
    collection: Collection;
  }

  let { collection }: Props = $props();

  let fileInput: HTMLInputElement;

  async function handleOpenApiImport(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const spec = JSON.parse(text);
      const count = addEndpointsFromOpenApi(collection.id, spec);
      toasts.show('success', `Imported ${count} endpoint${count !== 1 ? 's' : ''}`);
    } catch (err) {
      toasts.show('error', `Import failed: ${err}`);
    }
    input.value = '';
  }

  const baseUrl = $derived(`http://localhost:${collection.port}`);

  const METHOD_STYLE: Record<string, { text: string; bg: string; border: string }> = {
    GET:    { text: 'text-emerald-300', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30' },
    POST:   { text: 'text-blue-300',    bg: 'bg-blue-500/10',    border: 'border-blue-500/30'    },
    PUT:    { text: 'text-amber-300',   bg: 'bg-amber-500/10',   border: 'border-amber-500/30'   },
    DELETE: { text: 'text-rose-300',    bg: 'bg-rose-500/10',    border: 'border-rose-500/30'    },
    PATCH:  { text: 'text-teal-300',    bg: 'bg-teal-500/10',    border: 'border-teal-500/30'    },
  };

  const STATUS_STYLE: Record<number, string> = {};
  function statusStyle(code: number): string {
    if (code < 300) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    if (code < 400) return 'text-amber-400 bg-amber-500/10 border-amber-500/20';
    if (code < 500) return 'text-orange-400 bg-orange-500/10 border-orange-500/20';
    return 'text-rose-400 bg-rose-500/10 border-rose-500/20';
  }

  const STATUS_TEXT: Record<number, string> = {
    200: 'OK', 201: 'Created', 202: 'Accepted', 204: 'No Content',
    400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden',
    404: 'Not Found', 409: 'Conflict', 422: 'Unprocessable Entity',
    500: 'Internal Server Error', 502: 'Bad Gateway', 503: 'Service Unavailable',
  };

  let expanded = $state<Record<string, boolean>>({});

  function toggle(id: string) {
    expanded[id] = !expanded[id];
  }

  function copyUrl(endpoint: Endpoint) {
    navigator.clipboard.writeText(`${baseUrl}${endpoint.path}`);
  }

  function openUrl(endpoint: Endpoint) {
    const url = `${baseUrl}${endpoint.path}`;
    window.open(url, '_blank');
  }

  function prettyBody(body: string): string {
    if (!body.trim()) return '';
    try { return JSON.stringify(JSON.parse(body), null, 2); }
    catch { return body; }
  }

  // Group endpoints by first path segment
  let groups = $derived((() => {
    const map = new Map<string, Endpoint[]>();
    for (const ep of collection.endpoints) {
      const tag = ep.path.split('/').filter(Boolean)[0] ?? 'default';
      if (!map.has(tag)) map.set(tag, []);
      map.get(tag)!.push(ep);
    }
    return [...map.entries()].map(([tag, endpoints]) => ({ tag, endpoints }));
  })());

  let expandedGroups = $state<Record<string, boolean>>({});
  function isGroupExpanded(tag: string) {
    return expandedGroups[tag] !== false;
  }
  function toggleGroup(tag: string) {
    expandedGroups[tag] = !isGroupExpanded(tag);
  }

  // Code snippets
  type Lang = 'curl' | 'javascript' | 'python' | 'go' | 'php' | 'rust';
  const LANGS: { key: Lang; label: string }[] = [
    { key: 'curl', label: 'cURL' },
    { key: 'javascript', label: 'JavaScript' },
    { key: 'python', label: 'Python' },
    { key: 'go', label: 'Go' },
    { key: 'php', label: 'PHP' },
    { key: 'rust', label: 'Rust' },
  ];

  let snippetLang = $state<Record<string, Lang>>({});
  let copiedSnippet = $state<string | null>(null);

  function getLang(id: string): Lang {
    return snippetLang[id] ?? 'curl';
  }

  function generateSnippet(endpoint: Endpoint, lang: Lang): string {
    const url = `${baseUrl}${endpoint.path}`;
    const method = endpoint.method;
    const hasBody = ['POST', 'PUT', 'PATCH'].includes(method);
    const scenario = endpoint.scenarios.find(s => s.id === endpoint.active_scenario_id) ?? endpoint.scenarios[0];
    const bodyExample = hasBody && scenario?.body?.trim() ? scenario.body.trim() : null;

    switch (lang) {
      case 'curl': {
        let cmd = `curl -X ${method} "${url}"`;
        if (hasBody) cmd += ` \\\n  -H "Content-Type: application/json"`;
        if (bodyExample) cmd += ` \\\n  -d '${bodyExample}'`;
        return cmd;
      }
      case 'javascript': {
        if (method === 'GET') return `const res = await fetch("${url}");\nconst data = await res.json();`;
        let code = `const res = await fetch("${url}", {\n  method: "${method}",`;
        if (hasBody) {
          code += `\n  headers: { "Content-Type": "application/json" },`;
          if (bodyExample) code += `\n  body: JSON.stringify(${bodyExample}),`;
        }
        code += `\n});\nconst data = await res.json();`;
        return code;
      }
      case 'python': {
        let code = `import requests\n\n`;
        if (method === 'GET') {
          code += `res = requests.get("${url}")\ndata = res.json()`;
        } else {
          code += `res = requests.${method.toLowerCase()}(\n    "${url}",`;
          if (bodyExample) code += `\n    json=${bodyExample},`;
          code += `\n)\ndata = res.json()`;
        }
        return code;
      }
      case 'go': {
        let code = `package main\n\nimport (\n    "fmt"\n    "net/http"\n    "io"\n`;
        if (hasBody) code += `    "bytes"\n`;
        code += `)\n\nfunc main() {\n`;
        if (hasBody && bodyExample) {
          code += `    body := bytes.NewBufferString(\`${bodyExample}\`)\n`;
          code += `    req, _ := http.NewRequest("${method}", "${url}", body)\n`;
          code += `    req.Header.Set("Content-Type", "application/json")\n`;
        } else {
          code += `    req, _ := http.NewRequest("${method}", "${url}", nil)\n`;
        }
        code += `    res, _ := http.DefaultClient.Do(req)\n    defer res.Body.Close()\n    data, _ := io.ReadAll(res.Body)\n    fmt.Println(string(data))\n}`;
        return code;
      }
      case 'php': {
        let code = `$ch = curl_init("${url}");\ncurl_setopt($ch, CURLOPT_RETURNTRANSFER, true);\n`;
        if (method !== 'GET') {
          code += `curl_setopt($ch, CURLOPT_CUSTOMREQUEST, "${method}");\n`;
          if (hasBody) {
            code += `curl_setopt($ch, CURLOPT_HTTPHEADER, ["Content-Type: application/json"]);\n`;
            if (bodyExample) code += `curl_setopt($ch, CURLOPT_POSTFIELDS, '${bodyExample}');\n`;
          }
        }
        code += `$response = curl_exec($ch);\ncurl_close($ch);\n$data = json_decode($response, true);`;
        return code;
      }
      case 'rust': {
        let code = `// reqwest with tokio\nlet client = reqwest::Client::new();\nlet res = client\n    .${method.toLowerCase()}("${url}")`;
        if (hasBody) {
          code += `\n    .header("Content-Type", "application/json")`;
          if (bodyExample) code += `\n    .body(r#"${bodyExample}"#)`;
        }
        code += `\n    .send()\n    .await?;\nlet body = res.text().await?;`;
        return code;
      }
    }
  }

  function copySnippet(id: string, code: string) {
    navigator.clipboard.writeText(code);
    copiedSnippet = id;
    setTimeout(() => { if (copiedSnippet === id) copiedSnippet = null; }, 1500);
  }
</script>

<input
  type="file"
  accept=".json,.yaml,.yml"
  bind:this={fileInput}
  onchange={handleOpenApiImport}
  class="hidden"
/>

<div class="h-full overflow-y-auto bg-app">
  <div class="w-full px-6 py-6 space-y-6">

    <!-- API Header -->
    <div class="border border-line rounded-xl p-5 bg-surface">
      <div class="flex items-start justify-between gap-4">
        <div>
          <h1 class="text-lg font-krub font-semibold text-ink">{collection.name}</h1>
          <div class="flex items-center gap-2 mt-1.5">
            <code class="text-xs font-mono text-ink-muted bg-app px-2 py-0.5 rounded border border-line">
              {baseUrl}
            </code>
            <span class="flex items-center gap-1 text-[10px] font-mono text-run bg-run/10 px-2 py-0.5 rounded border border-run/20">
              <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse inline-block"></span>
              running
            </span>
          </div>
        </div>
        <div class="flex flex-col items-end gap-2 shrink-0">
          <span class="text-[10px] font-mono text-ink-dim">{collection.endpoints.length} endpoint{collection.endpoints.length !== 1 ? 's' : ''}</span>
          <button
            onclick={() => fileInput.click()}
            class="flex items-center gap-1.5 px-2.5 py-1 text-[10px] font-krub
                   bg-surface border border-line-strong rounded text-ink-muted hover:text-ink hover:border-accent transition-colors"
            title="Import OpenAPI / Swagger spec"
          >
            <Upload size={10} />
            Import OpenAPI
          </button>
        </div>
      </div>
    </div>

    <!-- Endpoint groups -->
    {#if collection.endpoints.length === 0}
      <div class="text-center py-12">
        <p class="text-ink-dim text-sm">No endpoints defined in this collection.</p>
      </div>
    {:else}
      {#each groups as { tag, endpoints } (tag)}
        <div class="border border-line rounded-xl overflow-hidden">
          <!-- Group header -->
          <button
            onclick={() => toggleGroup(tag)}
            class="w-full flex items-center gap-3 px-4 py-3 bg-surface hover:bg-hover/30 transition-colors duration-150 border-b border-line"
          >
            {#if isGroupExpanded(tag)}
              <ChevronDown size={13} class="text-ink-dim shrink-0" />
            {:else}
              <ChevronRight size={13} class="text-ink-dim shrink-0" />
            {/if}
            <span class="text-xs font-krub font-semibold text-ink uppercase tracking-widest">/{tag}</span>
            <span class="ml-auto text-[10px] font-mono text-ink-dim">{endpoints.length}</span>
          </button>

          <!-- Endpoints -->
          {#if isGroupExpanded(tag)}
            <div class="divide-y divide-line">
              {#each endpoints as endpoint (endpoint.id)}
                {@const style = METHOD_STYLE[endpoint.method] ?? METHOD_STYLE.GET}
                {@const isOpen = !!expanded[endpoint.id]}
                {@const activeScenario = endpoint.scenarios.find(s => s.id === endpoint.active_scenario_id) ?? endpoint.scenarios[0]}

                <!-- Endpoint row -->
                <div class="bg-app/30">
                  <button
                    onclick={() => toggle(endpoint.id)}
                    class="w-full flex items-center gap-3 px-4 py-3 hover:bg-hover/20 transition-colors duration-150 group"
                  >
                    <!-- Method badge -->
                    <span class="text-[10px] font-mono font-bold w-14 text-center shrink-0 py-0.5 rounded border
                                 {style.text} {style.bg} {style.border}">
                      {endpoint.method}
                    </span>

                    <!-- Path -->
                    <span class="flex-1 text-sm font-mono text-ink text-left">{endpoint.path}</span>

                    <!-- Status hint -->
                    {#if activeScenario}
                      <span class="text-[10px] font-mono px-1.5 py-0.5 rounded border {statusStyle(activeScenario.status)} shrink-0">
                        {activeScenario.status}
                      </span>
                    {/if}

                    <!-- Actions (copy, open) -->
                    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                      <span
                        role="button"
                        tabindex="0"
                        onclick={(e) => { e.stopPropagation(); copyUrl(endpoint); }}
                        onkeydown={(e) => e.key === 'Enter' && copyUrl(endpoint)}
                        class="w-6 h-6 flex items-center justify-center rounded text-ink-dim hover:text-ink hover:bg-hover transition-colors"
                        title="Copy URL"
                      >
                        <Copy size={11} />
                      </span>
                      {#if endpoint.method === 'GET'}
                        <span
                          role="button"
                          tabindex="0"
                          onclick={(e) => { e.stopPropagation(); openUrl(endpoint); }}
                          onkeydown={(e) => e.key === 'Enter' && openUrl(endpoint)}
                          class="w-6 h-6 flex items-center justify-center rounded text-ink-dim hover:text-ink hover:bg-hover transition-colors"
                          title="Open in browser"
                        >
                          <ExternalLink size={11} />
                        </span>
                      {/if}
                    </div>

                    <ChevronDown
                      size={12}
                      class="text-ink-dim shrink-0 transition-transform duration-150 {isOpen ? 'rotate-0' : '-rotate-90'}"
                    />
                  </button>

                  <!-- Expanded detail -->
                  {#if isOpen}
                    {@const paramsScenario = endpoint.scenarios.find(s => Object.keys(s.query_params ?? {}).length > 0)}
                    <div class="border-t border-line bg-app/60 px-4 py-4 space-y-4">

                      <!-- Full URL -->
                      <div class="flex items-center gap-2">
                        <span class="text-[9px] font-krub text-ink-dim uppercase tracking-widest w-20 shrink-0">URL</span>
                        <code class="text-[11px] font-mono text-accent flex-1">{baseUrl}{endpoint.path}</code>
                      </div>

                      <!-- Query params (from first scenario that has them) -->
                      {#if paramsScenario}
                        <div>
                          <span class="text-[9px] font-krub text-ink-dim uppercase tracking-widest block mb-2">Parameters</span>
                          <div class="border border-line rounded-lg overflow-hidden">
                            <div class="flex items-center px-3 py-1 bg-surface/80 border-b border-line">
                              <span class="text-[9px] font-krub text-ink-dim uppercase w-28">Name</span>
                              <span class="text-[9px] font-krub text-ink-dim uppercase w-20">In</span>
                              <span class="text-[9px] font-krub text-ink-dim uppercase flex-1">Example</span>
                            </div>
                            {#each Object.entries(paramsScenario.query_params ?? {}) as [key, val] (key)}
                              <div class="flex items-center px-3 py-1.5 border-b border-line last:border-0 hover:bg-hover/20">
                                <span class="text-[11px] font-mono text-accent w-28 truncate">{key}</span>
                                <span class="text-[10px] font-mono text-ink-dim w-20">query</span>
                                <span class="text-[11px] font-mono text-ink-muted flex-1">{val || '—'}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/if}

                      <!-- Responses (all scenarios) -->
                      <div>
                        <span class="text-[9px] font-krub text-ink-dim uppercase tracking-widest block mb-2">Responses</span>
                        <div class="space-y-2">
                          {#each endpoint.scenarios as scenario (scenario.id)}
                            <div class="border border-line rounded-lg overflow-hidden">
                              <!-- Scenario header -->
                              <div class="flex items-center gap-3 px-3 py-2 bg-surface/60 border-b border-line">
                                <span class="text-[10px] font-mono font-semibold px-2 py-0.5 rounded border {statusStyle(scenario.status)}">
                                  {scenario.status} {STATUS_TEXT[scenario.status] ?? ''}
                                </span>
                                <span class="text-[11px] font-inter text-ink-muted flex-1">{scenario.name}</span>
                                {#if scenario.id === endpoint.active_scenario_id}
                                  <span class="text-[9px] font-krub text-accent bg-accent/10 px-1.5 py-0.5 rounded border border-accent/20">active</span>
                                {/if}
                                {#if scenario.delay_ms > 0}
                                  <span class="text-[9px] font-mono text-ink-dim">{scenario.delay_ms}ms</span>
                                {/if}
                              </div>

                              <!-- Response headers -->
                              {#if Object.keys(scenario.headers).length > 0}
                                <div class="px-3 py-2 border-b border-line">
                                  {#each Object.entries(scenario.headers) as [k, v] (k)}
                                    <div class="flex items-center gap-2 text-[10px] font-mono py-0.5">
                                      <span class="text-ink-dim w-40 shrink-0 truncate">{k}</span>
                                      <span class="text-ink-muted">{v}</span>
                                    </div>
                                  {/each}
                                </div>
                              {/if}

                              <!-- Response body -->
                              {#if scenario.body.trim()}
                                <pre class="px-3 py-3 text-[11px] font-mono text-ink-muted leading-relaxed
                                            max-h-48 overflow-y-auto whitespace-pre-wrap break-words
                                            bg-app/40">{prettyBody(scenario.body)}</pre>
                              {:else}
                                <div class="px-3 py-2 text-[11px] text-ink-dim italic">Empty body</div>
                              {/if}
                            </div>
                          {/each}
                        </div>
                      </div>

                      <!-- Code snippets -->
                      <div>
                        <div class="flex items-center justify-between mb-2">
                          <span class="text-[9px] font-krub text-ink-dim uppercase tracking-widest">Code Examples</span>
                          <button
                            onclick={() => copySnippet(endpoint.id, generateSnippet(endpoint, getLang(endpoint.id)))}
                            class="flex items-center gap-1 text-[9px] font-krub text-ink-dim hover:text-ink transition-colors"
                          >
                            {#if copiedSnippet === endpoint.id}
                              <Check size={10} class="text-run" />
                              <span class="text-run">Copied</span>
                            {:else}
                              <Copy size={10} />
                              Copy
                            {/if}
                          </button>
                        </div>
                        <div class="border border-line rounded-lg overflow-hidden">
                          <div class="flex items-center gap-0.5 px-2 py-1.5 bg-surface/80 border-b border-line overflow-x-auto">
                            {#each LANGS as { key, label }}
                              <button
                                onclick={() => snippetLang[endpoint.id] = key}
                                class="px-2 py-0.5 text-[9px] font-mono rounded transition-colors shrink-0
                                  {getLang(endpoint.id) === key
                                    ? 'bg-accent/15 text-accent'
                                    : 'text-ink-dim hover:text-ink hover:bg-hover/50'}"
                              >
                                {label}
                              </button>
                            {/each}
                          </div>
                          {#if highlighter}
                            <div class="snippet-highlight px-3 py-3 text-[11px] leading-relaxed max-h-64 overflow-y-auto">
                              {@html highlight(generateSnippet(endpoint, getLang(endpoint.id)), getLang(endpoint.id))}
                            </div>
                          {:else}
                            <pre class="px-3 py-3 text-[11px] font-mono text-ink-muted leading-relaxed
                                        max-h-64 overflow-y-auto whitespace-pre-wrap break-words
                                        bg-app/40">{generateSnippet(endpoint, getLang(endpoint.id))}</pre>
                          {/if}
                        </div>
                      </div>

                    </div>
                  {/if}
                </div>

              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}

  </div>
</div>

<style>
  .snippet-highlight :global(pre) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 11px;
    line-height: 1.625;
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Monaco, Consolas, monospace;
  }
  .snippet-highlight :global(code) {
    font-family: inherit;
    font-size: inherit;
  }
</style>
