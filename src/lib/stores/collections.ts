import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Collection, Endpoint, AppData } from '$lib/types';
import { newCollection, newEndpoint } from '$lib/types';

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

export function deleteCollection(id: string) {
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
