import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCollectionId, collections } from './collections';
import type { AppData } from '$lib/types';

export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

export const serverStatus = writable<ServerStatus>('stopped');
export const serverError = writable<string | null>(null);

export async function startServer() {
  const collectionId = get(activeCollectionId);
  if (!collectionId) {
    serverError.set('No collection selected');
    serverStatus.set('error');
    return;
  }

  serverStatus.set('starting');
  serverError.set(null);

  const data: AppData = {
    collections: get(collections),
    active_collection_id: collectionId,
  };

  try {
    await invoke('start_server', { collectionId, data });
    serverStatus.set('running');
  } catch (e) {
    serverStatus.set('error');
    serverError.set(e as string);
  }
}

export async function stopServer() {
  try {
    await invoke('stop_server');
    serverStatus.set('stopped');
    serverError.set(null);
  } catch (e) {
    serverError.set(e as string);
  }
}
