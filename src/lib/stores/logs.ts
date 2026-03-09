import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { LogEntry } from '$lib/types';

const MAX_LOGS = 200;

export const logs = writable<LogEntry[]>([]);

export function clearLogs() {
  logs.set([]);
}

export async function initLogListener() {
  await listen<LogEntry>('server:log', (event) => {
    logs.update((current) => {
      const next = [...current, event.payload];
      return next.length > MAX_LOGS ? next.slice(-MAX_LOGS) : next;
    });
  });
}
