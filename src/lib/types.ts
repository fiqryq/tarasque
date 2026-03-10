export interface Scenario {
  id: string;
  name: string;
  status: number;
  delay_ms: number;
  headers: Record<string, string>;
  query_params: Record<string, string>;
  body: string;
}

export interface Endpoint {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';
  path: string;
  scenarios: Scenario[];
  active_scenario_id: string;
  description?: string;
}

export interface Collection {
  id: string;
  name: string;
  port: number;
  endpoints: Endpoint[];
  // Proxy forwarding
  proxy_url?: string;
  proxy_enabled: boolean;
  proxy_insecure: boolean;
  // Local HTTPS alias
  alias_enabled: boolean;
  alias_hostname?: string;
  alias_port?: number;
  alias_cert_pem?: string;
  alias_key_pem?: string;
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
  query: Record<string, string>;
  headers: Record<string, string>;
  body: string;
  response_body?: string;
  proxied?: boolean;
}

export interface TestAssertion {
  id: string;
  type: 'status' | 'body';
  path?: string;
  operator: 'eq' | 'contains' | 'exists' | 'not_exists';
  expected: string | number | boolean;
}

export interface TestCase {
  id: string;
  name: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';
  path: string;
  headers?: Record<string, string>;
  body?: string;
  assertions: TestAssertion[];
}

export interface TestSuite {
  id: string;
  name: string;
  collection_id: string;
  test_cases: TestCase[];
}

export interface TestResult {
  test_case_id: string;
  passed: boolean;
  duration: number;
  actual_status?: number;
  actual_body?: string;
  error?: string;
  assertion_results: {
    assertion: TestAssertion;
    passed: boolean;
    actual?: string;
  }[];
}

export interface TestRunResult {
  suite_id: string;
  timestamp: string;
  total: number;
  passed: number;
  failed: number;
  results: TestResult[];
}

export function newScenario(overrides: Partial<Scenario> = {}): Scenario {
  return {
    id: crypto.randomUUID(),
    name: 'Default',
    status: 200,
    delay_ms: 0,
    headers: { 'Content-Type': 'application/json' },
    query_params: {},
    body: '{\n  \n}',
    ...overrides,
  };
}

const METHOD_DEFAULT_BODIES: Record<string, { status: number; body: string }> = {
  GET: {
    status: 200,
    body: JSON.stringify({ id: '{{uuid}}', name: 'Example', created_at: '{{iso_timestamp}}' }, null, 2),
  },
  POST: {
    status: 201,
    body: JSON.stringify({ id: '{{uuid}}', message: 'Created successfully', created_at: '{{iso_timestamp}}' }, null, 2),
  },
  PUT: {
    status: 200,
    body: JSON.stringify({ id: '{{request.params.id}}', message: 'Updated successfully', updated_at: '{{iso_timestamp}}' }, null, 2),
  },
  PATCH: {
    status: 200,
    body: JSON.stringify({ id: '{{request.params.id}}', message: 'Updated successfully', updated_at: '{{iso_timestamp}}' }, null, 2),
  },
  DELETE: {
    status: 204,
    body: '',
  },
};

export function newEndpoint(overrides: Partial<Endpoint> = {}): Endpoint {
  const method = (overrides.method ?? 'GET') as string;
  const defaults = METHOD_DEFAULT_BODIES[method] ?? METHOD_DEFAULT_BODIES.GET;
  const scenario = newScenario({ status: defaults.status, body: defaults.body });
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
    proxy_enabled: false,
    proxy_insecure: false,
    alias_enabled: false,
  };
}

export function newTestSuite(name: string, collectionId: string): TestSuite {
  return {
    id: crypto.randomUUID(),
    name,
    collection_id: collectionId,
    test_cases: [],
  };
}

export function newTestCase(overrides: Partial<TestCase> = {}): TestCase {
  return {
    id: crypto.randomUUID(),
    name: 'New Test',
    method: 'GET',
    path: '/',
    assertions: [],
    ...overrides,
  };
}

export function newTestAssertion(overrides: Partial<TestAssertion> = {}): TestAssertion {
  return {
    id: crypto.randomUUID(),
    type: 'status',
    operator: 'eq',
    expected: 200,
    ...overrides,
  };
}
