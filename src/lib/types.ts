export interface Scenario {
  id: string;
  name: string;
  status: number;
  delay_ms: number;
  headers: Record<string, string>;
  body: string;
}

export interface Endpoint {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  scenarios: Scenario[];
  active_scenario_id: string;
}

export interface Collection {
  id: string;
  name: string;
  port: number;
  endpoints: Endpoint[];
}

export interface AppData {
  collections: Collection[];
  active_collection_id: string | null;
}

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration: number;
}

export function newScenario(overrides: Partial<Scenario> = {}): Scenario {
  return {
    id: crypto.randomUUID(),
    name: 'Default',
    status: 200,
    delay_ms: 0,
    headers: { 'Content-Type': 'application/json' },
    body: '{\n  \n}',
    ...overrides,
  };
}

export function newEndpoint(overrides: Partial<Endpoint> = {}): Endpoint {
  const scenario = newScenario();
  return {
    id: crypto.randomUUID(),
    method: 'GET',
    path: '/new-endpoint',
    scenarios: [scenario],
    active_scenario_id: scenario.id,
    ...overrides,
  };
}

export function newCollection(name: string): Collection {
  return {
    id: crypto.randomUUID(),
    name,
    port: 3000,
    endpoints: [],
  };
}
