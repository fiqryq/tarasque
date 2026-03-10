import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Collection, Endpoint, AppData } from '$lib/types';
import { newCollection, newEndpoint, newScenario } from '$lib/types';

export interface CurlParsed {
  method: string;
  path: string;
  headers: Record<string, string>;
  body: string;
}

export function parseCurl(curlCommand: string): CurlParsed {
  const result: CurlParsed = { method: 'GET', path: '/', headers: {}, body: '' };

  // Extract first http(s):// URL
  const urlMatch = curlCommand.match(/https?:\/\/[^\s'"]+/);
  if (urlMatch) {
    try {
      const u = new URL(urlMatch[0]);
      result.path = u.pathname + (u.search ?? '');
    } catch { /* ignore */ }
  }

  // Method: -X POST or --request POST
  const methodMatch = curlCommand.match(/(?:-X|--request)\s+([A-Z]+)/i);
  if (methodMatch) result.method = methodMatch[1].toUpperCase();

  // Headers: -H 'Key: Value' (multiple)
  const headerRe = /(?:-H|--header)\s+(?:'([^']+)'|"([^"]+)")/g;
  let hm: RegExpExecArray | null;
  while ((hm = headerRe.exec(curlCommand)) !== null) {
    const raw = hm[1] ?? hm[2];
    const colon = raw.indexOf(':');
    if (colon > 0) {
      result.headers[raw.slice(0, colon).trim()] = raw.slice(colon + 1).trim();
    }
  }

  // Body: -d or --data or --data-raw
  const bodyMatch = curlCommand.match(/(?:-d|--data(?:-raw)?)\s+(?:'([^']+)'|"([^"]+)"|(\S+))/);
  if (bodyMatch) {
    result.body = bodyMatch[1] ?? bodyMatch[2] ?? bodyMatch[3] ?? '';
    if (result.method === 'GET') result.method = 'POST';
  }

  return result;
}

export function addEndpointFromCurl(collectionId: string, parsed: CurlParsed) {
  const scenario = newScenario({
    headers: parsed.headers['Content-Type']
      ? parsed.headers
      : { 'Content-Type': 'application/json' },
  });
  const endpoint = newEndpoint({
    method: (parsed.method as Endpoint['method']) || 'GET',
    path: parsed.path,
    scenarios: [scenario],
    active_scenario_id: scenario.id,
  });
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId ? { ...c, endpoints: [...c.endpoints, endpoint] } : c
    )
  );
  scheduleAutoSave();
  return endpoint;
}

export const collections = writable<Collection[]>([]);
export const activeCollectionId = writable<string | null>(null);
export const activeEndpointId = writable<string | null>(null);

export const activeCollection = derived(
  [collections, activeCollectionId],
  ([$cols, $id]) => $cols.find((c) => c.id === $id) ?? null
);

export const activeEndpoint = derived(
  [activeCollection, activeEndpointId],
  ([$col, $eid]) => $col?.endpoints.find((e) => e.id === $eid) ?? null
);

// Auto-save — debounced 500ms after any mutation
let saveTimer: ReturnType<typeof setTimeout>;
function scheduleAutoSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(persistData, 500);
}

async function persistData() {
  const data: AppData = {
    collections: get(collections),
    active_collection_id: get(activeCollectionId),
  };
  try {
    await invoke('save_data', { data });
  } catch (e) {
    console.error('Auto-save failed:', e);
  }
}

export async function loadCollections() {
  try {
    const data = await invoke<AppData>('load_data');
    collections.set(data.collections);
    if (data.active_collection_id) {
      activeCollectionId.set(data.active_collection_id);
    } else if (data.collections.length > 0) {
      activeCollectionId.set(data.collections[0].id);
    }
  } catch (e) {
    console.error('Load failed:', e);
  }
}

export function addCollection(name: string) {
  const col = newCollection(name);
  collections.update((c) => [...c, col]);
  activeCollectionId.set(col.id);
  scheduleAutoSave();
}

export async function deleteCollection(id: string) {
  const { stopServer, serverStatuses, serverErrors } = await import('./server');
  const status = get(serverStatuses)[id];
  if (status === 'running' || status === 'starting') {
    try { await stopServer(id); } catch { /* ignore */ }
  }
  serverStatuses.update((s) => { const n = { ...s }; delete n[id]; return n; });
  serverErrors.update((e) => { const n = { ...e }; delete n[id]; return n; });

  collections.update((c) => c.filter((col) => col.id !== id));
  activeCollectionId.update((current) => (current === id ? null : current));
  scheduleAutoSave();
}

export function renameCollection(id: string, name: string) {
  collections.update((cols) =>
    cols.map((c) => (c.id === id ? { ...c, name } : c))
  );
  scheduleAutoSave();
}

export function updateCollectionPort(id: string, port: number) {
  collections.update((cols) =>
    cols.map((c) => (c.id === id ? { ...c, port } : c))
  );
  scheduleAutoSave();
}

export function importCollections(imported: Collection[]) {
  collections.update((cols) => [...cols, ...imported]);
  if (imported.length > 0) {
    activeCollectionId.set(imported[imported.length - 1].id);
  }
  scheduleAutoSave();
}

export function updateCollectionProxy(id: string, enabled: boolean, url: string, insecure: boolean) {
  collections.update((cols) =>
    cols.map((c) =>
      c.id === id
        ? { ...c, proxy_enabled: enabled, proxy_url: url || undefined, proxy_insecure: insecure }
        : c
    )
  );
  scheduleAutoSave();
}

export function updateCollectionAlias(
  id: string,
  updates: Partial<Pick<Collection, 'alias_enabled' | 'alias_hostname' | 'alias_port' | 'alias_cert_pem' | 'alias_key_pem'>>
) {
  collections.update((cols) =>
    cols.map((c) => (c.id === id ? { ...c, ...updates } : c))
  );
  scheduleAutoSave();
}

export function addEndpoint(collectionId: string) {
  const ep = newEndpoint();
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId ? { ...c, endpoints: [...c.endpoints, ep] } : c
    )
  );
  activeEndpointId.set(ep.id);
  scheduleAutoSave();
}

export function duplicateEndpoint(collectionId: string, endpointId: string) {
  collections.update((cols) =>
    cols.map((c) => {
      if (c.id !== collectionId) return c;
      const original = c.endpoints.find((e) => e.id === endpointId);
      if (!original) return c;
      const newScenarios = original.scenarios.map((s) => ({ ...s, id: crypto.randomUUID() }));
      const copy: Endpoint = {
        ...original,
        id: crypto.randomUUID(),
        path: original.path + ' (copy)',
        scenarios: newScenarios,
        active_scenario_id: newScenarios[0]?.id ?? '',
      };
      return { ...c, endpoints: [...c.endpoints, copy] };
    })
  );
  scheduleAutoSave();
}

export function deleteEndpoint(collectionId: string, endpointId: string) {
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId
        ? { ...c, endpoints: c.endpoints.filter((e) => e.id !== endpointId) }
        : c
    )
  );
  activeEndpointId.update((current) => (current === endpointId ? null : current));
  scheduleAutoSave();
}

// ── OpenAPI / Swagger import helpers ─────────────────────────────────────────

function inferSchemaExample(schema: Record<string, unknown>): string {
  if (schema.example !== undefined) {
    return JSON.stringify(schema.example, null, 2);
  }
  if (schema.type === 'object' || schema.properties) {
    const props = (schema.properties as Record<string, Record<string, unknown>>) ?? {};
    const obj: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(props)) {
      obj[k] = JSON.parse(inferSchemaExample(v));
    }
    return JSON.stringify(obj, null, 2);
  }
  if (schema.type === 'array' && schema.items) {
    return JSON.stringify([JSON.parse(inferSchemaExample(schema.items as Record<string, unknown>))], null, 2);
  }
  const typeMap: Record<string, unknown> = { string: 'string', number: 0, integer: 0, boolean: false };
  return JSON.stringify(typeMap[schema.type as string] ?? null);
}

function getStatusText(status: number): string {
  const map: Record<number, string> = {
    200: 'OK', 201: 'Created', 202: 'Accepted', 204: 'No Content',
    400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden',
    404: 'Not Found', 409: 'Conflict', 422: 'Unprocessable Entity',
    500: 'Internal Server Error', 502: 'Bad Gateway', 503: 'Service Unavailable',
  };
  return map[status] ?? '';
}

export function addEndpointsFromOpenApi(collectionId: string, spec: unknown): number {
  const s = spec as Record<string, unknown>;
  const isSwagger2 = typeof (s as { swagger?: string }).swagger === 'string';
  const basePath: string = isSwagger2 ? ((s as { basePath?: string }).basePath ?? '') : '';
  const paths = (s as { paths?: Record<string, unknown> }).paths ?? {};
  const endpoints: Endpoint[] = [];

  for (const [pathKey, pathItem] of Object.entries(paths)) {
    const pi = pathItem as Record<string, unknown>;
    for (const method of ['get', 'post', 'put', 'delete', 'patch', 'head', 'options']) {
      if (!pi[method]) continue;
      const op = pi[method] as Record<string, unknown>;
      const responses = (op.responses as Record<string, unknown>) ?? {};
      const scenarios: ReturnType<typeof newScenario>[] = [];

      for (const [statusStr, resp] of Object.entries(responses)) {
        const status = parseInt(statusStr, 10);
        if (isNaN(status)) continue;
        const r = resp as Record<string, unknown>;

        let body = '';
        // OpenAPI 3.0
        const content = r.content as Record<string, unknown> | undefined;
        const jsonContent = content?.['application/json'] as Record<string, unknown> | undefined;
        if (jsonContent?.example !== undefined) {
          body = JSON.stringify(jsonContent.example, null, 2);
        } else if (jsonContent?.schema) {
          body = inferSchemaExample(jsonContent.schema as Record<string, unknown>);
        }
        // Swagger 2.0
        if (!body && r.examples) {
          const ex = r.examples as Record<string, unknown>;
          const first = ex['application/json'] ?? ex[Object.keys(ex)[0]];
          body = JSON.stringify(first, null, 2);
        }
        if (!body && r.schema) {
          body = inferSchemaExample(r.schema as Record<string, unknown>);
        }

        scenarios.push(newScenario({
          name: `${statusStr} ${getStatusText(status)}`.trim(),
          status,
          body: body || '',
        }));
      }

      if (scenarios.length === 0) {
        scenarios.push(newScenario({ name: 'Default 200', status: 200 }));
      }

      const fullPath = basePath + pathKey;
      const convertedPath = fullPath.replace(/\{([^}]+)\}/g, ':$1');

      endpoints.push(newEndpoint({
        method: method.toUpperCase() as Endpoint['method'],
        path: convertedPath,
        scenarios,
        active_scenario_id: scenarios[0].id,
      }));
    }
  }

  collections.update((cols) =>
    cols.map((c) => c.id === collectionId ? { ...c, endpoints: [...c.endpoints, ...endpoints] } : c)
  );
  scheduleAutoSave();
  return endpoints.length;
}

export function updateEndpoint(collectionId: string, endpoint: Endpoint) {
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId
        ? { ...c, endpoints: c.endpoints.map((e) => (e.id === endpoint.id ? endpoint : e)) }
        : c
    )
  );
  scheduleAutoSave();
}
