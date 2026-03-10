import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCollectionId, collections } from './collections';
import type { AppData } from '$lib/types';

export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

// Map of collectionId → status / error
export const serverStatuses = writable<Record<string, ServerStatus>>({});
export const serverErrors   = writable<Record<string, string | null>>({});

/** Reactive status for the currently active collection */
export const activeServerStatus = derived(
  [serverStatuses, activeCollectionId],
  ([$s, $id]) => ($id ? ($s[$id] ?? 'stopped') : 'stopped')
);

export const activeServerError = derived(
  [serverErrors, activeCollectionId],
  ([$e, $id]) => ($id ? ($e[$id] ?? null) : null)
);

function setStatus(id: string, status: ServerStatus) {
  serverStatuses.update((s) => ({ ...s, [id]: status }));
}

function setError(id: string, err: string | null) {
  serverErrors.update((e) => ({ ...e, [id]: err }));
}

export async function startServer(collectionId: string) {
  const cols = get(collections);
  const data: AppData = {
    collections: cols,
    active_collection_id: collectionId,
  };

  setStatus(collectionId, 'starting');
  setError(collectionId, null);

  const timeout = new Promise<never>((_, reject) =>
    setTimeout(() => reject('Server start timed out after 15s'), 15_000)
  );

  try {
    await Promise.race([
      invoke('start_server', { collectionId, data }),
      timeout,
    ]);
    setStatus(collectionId, 'running');
  } catch (e) {
    setStatus(collectionId, 'error');
    setError(collectionId, String(e));
  }
}

export async function stopServer(collectionId: string) {
  try {
    await invoke('stop_server', { collectionId });
    setStatus(collectionId, 'stopped');
    setError(collectionId, null);
  } catch (e) {
    setError(collectionId, String(e));
  }
}

/** Sync running state from Rust on app load */
export async function syncRunningServers() {
  try {
    const ids = await invoke<string[]>('get_running_collections');
    serverStatuses.update((s) => {
      const next = { ...s };
      for (const id of ids) next[id] = 'running';
      return next;
    });
  } catch {
    // non-fatal
  }
}

export async function findFreePort(): Promise<number> {
  return invoke<number>('find_free_port');
}
