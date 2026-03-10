import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { collections } from './collections';
import type { Collection } from '$lib/types';

export type ProxyStatus = 'running' | 'stopped' | 'unknown';

export const proxyStatus = writable<ProxyStatus>('unknown');
export const proxyError = writable<string | null>(null);
export const proxyLoading = writable(false);

function enabledAliases(cols: Collection[]) {
  return cols
    .filter((c) => c.alias_enabled && c.alias_hostname && c.alias_cert_pem)
    .map((c) => ({ hostname: c.alias_hostname!, port: c.port }));
}

export async function refreshProxyStatus() {
  try {
    const status = await invoke<string>('get_proxy_status');
    proxyStatus.set(status as ProxyStatus);
  } catch {
    proxyStatus.set('unknown');
  }
}

export async function startProxy() {
  proxyLoading.set(true);
  proxyError.set(null);
  try {
    const aliases = enabledAliases(get(collections));
    await invoke('start_proxy', { aliases });
    proxyStatus.set('running'); // optimistic — avoids flicker while refresh resolves
    await refreshProxyStatus();
  } catch (err) {
    proxyError.set(String(err));
    proxyStatus.set('stopped');
  } finally {
    proxyLoading.set(false);
  }
}

export async function stopProxy() {
  proxyLoading.set(true);
  proxyError.set(null);
  try {
    await invoke('stop_proxy');
    proxyStatus.set('stopped');
  } catch (err) {
    proxyError.set(String(err));
  } finally {
    proxyLoading.set(false);
  }
}
