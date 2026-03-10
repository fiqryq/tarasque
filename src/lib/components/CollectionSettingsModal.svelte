<script lang="ts">
  import { X, Globe, Shield, ShieldCheck, ShieldAlert, Copy, Download } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Collection } from '$lib/types';
  import { updateCollectionPort, updateCollectionProxy, updateCollectionAlias } from '$lib/stores/collections';
  import { toasts } from '$lib/stores/toasts';

  interface Props {
    collection: Collection;
    onClose: () => void;
  }

  let { collection, onClose }: Props = $props();

  // Local editable state
  let port           = $state(collection.port);
  let proxyEnabled   = $state(collection.proxy_enabled ?? false);
  let proxyUrl       = $state(collection.proxy_url ?? '');
  let proxyInsecure  = $state(collection.proxy_insecure ?? false);
  let aliasEnabled   = $state(collection.alias_enabled ?? false);
  let aliasHostname  = $state(collection.alias_hostname ?? '');
  let aliasPort      = $state(collection.alias_port ?? 8443);
  let aliasCertPem   = $state(collection.alias_cert_pem ?? '');
  let aliasKeyPem    = $state(collection.alias_key_pem ?? '');
  let generatingCert  = $state(false);
  let certMode = $state<'auto' | 'manual'>('auto');

  let hasCert = $derived(!!aliasCertPem && !!aliasKeyPem);

  const certFilePath = $derived(
    aliasHostname ? `$HOME/Library/Application Support/com.tarasque.app/proxy/cert/${aliasHostname}/cert.pem` : ''
  );
  const keychainCmd = $derived(
    `security add-trusted-cert -k "$HOME/Library/Keychains/login.keychain-db" "${certFilePath}"`
  );
  const hostsCmd = $derived(
    `sudo sh -c 'echo "127.0.0.1 ${aliasHostname || '<hostname>'}" >> /etc/hosts'`
  );

  function save() {
    updateCollectionPort(collection.id, port);
    updateCollectionProxy(collection.id, proxyEnabled, proxyUrl, proxyInsecure);
    updateCollectionAlias(collection.id, {
      alias_enabled: aliasEnabled,
      alias_hostname: aliasHostname || undefined,
      alias_port: aliasPort,
      alias_cert_pem: aliasCertPem || undefined,
      alias_key_pem: aliasKeyPem || undefined,
    });
    toasts.show('success', 'Collection settings saved');
    onClose();
  }

  async function generateCert() {
    if (!aliasHostname.trim()) {
      toasts.show('error', 'Enter a hostname before generating a certificate');
      return;
    }
    generatingCert = true;
    try {
      const result = await invoke<{ cert_pem: string; key_pem: string }>('generate_alias_cert', {
        hostname: aliasHostname.trim(),
      });
      aliasCertPem = result.cert_pem;
      aliasKeyPem  = result.key_pem;
      toasts.show('success', `Certificate generated for ${aliasHostname}`);
    } catch (err) {
      toasts.show('error', `Failed to generate certificate: ${err}`);
    } finally {
      generatingCert = false;
    }
  }

  function downloadCert() {
    const blob = new Blob([aliasCertPem], { type: 'application/x-pem-file' });
    const url  = URL.createObjectURL(blob);
    const a    = document.createElement('a');
    a.href     = url;
    a.download = `${aliasHostname || 'alias'}.crt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function copyCmd(cmd: string) {
    navigator.clipboard.writeText(cmd);
    toasts.show('info', 'Copied to clipboard');
  }

  async function generateAndSetupAuto() {
    if (!aliasHostname.trim()) {
      toasts.show('error', 'Enter a hostname first');
      return;
    }
    generatingCert = true;
    try {
      const result = await invoke<{ cert_pem: string; key_pem: string }>('generate_alias_cert', {
        hostname: aliasHostname.trim(),
      });
      aliasCertPem = result.cert_pem;
      aliasKeyPem  = result.key_pem;
      await invoke('trust_cert_macos', { certPem: aliasCertPem });
      await invoke('add_hosts_entry', { hostname: aliasHostname.trim() });
      // Persist cert PEM to collection so hasCert stays true after modal re-open
      updateCollectionAlias(collection.id, {
        alias_enabled: aliasEnabled,
        alias_hostname: aliasHostname || undefined,
        alias_port: aliasPort,
        alias_cert_pem: aliasCertPem || undefined,
        alias_key_pem: aliasKeyPem || undefined,
      });
      toasts.show('success', `HTTPS alias set up for ${aliasHostname} — restart your browser`);
    } catch (err) {
      const msg = String(err);
      if (msg.includes('cancelled') || msg.includes('User canceled')) {
        toasts.show('info', 'Setup cancelled');
      } else {
        toasts.show('error', `Setup failed: ${msg}`);
      }
    } finally {
      generatingCert = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }

</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
  onclick={onClose}
  role="dialog"
  aria-modal="true"
  onkeydown={() => {}}
>
  <div
    class="w-[520px] max-h-[88vh] bg-surface border border-line rounded-xl shadow-2xl flex flex-col overflow-hidden"
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
    role="document"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-line shrink-0">
      <div>
        <h2 class="text-sm font-krub font-semibold text-ink">{collection.name}</h2>
        <p class="text-[11px] text-ink-dim mt-0.5">Collection Settings</p>
      </div>
      <button onclick={onClose} class="text-ink-dim hover:text-ink transition-colors" aria-label="Close">
        <X size={16} />
      </button>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-5 space-y-6">

      <!-- Port -->
      <section>
        <label for="col-port" class="block text-[10px] font-krub text-ink-muted uppercase tracking-widest mb-2">HTTP Port</label>
        <input
          id="col-port"
          type="number"
          bind:value={port}
          min="1024"
          max="65535"
          class="w-28 bg-app text-ink text-sm font-mono px-3 py-2 border border-line-strong rounded
                 focus:outline-none focus:border-accent transition-colors"
        />
        <p class="text-[10px] text-ink-dim mt-1.5">Mock server listens on <code class="text-accent">http://localhost:{port}</code></p>
      </section>

      <!-- Proxy Forwarding -->
      <section class="border-t border-line pt-5">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Globe size={13} class="text-ink-muted" />
            <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Proxy Forwarding</span>
          </div>
          <button
            onclick={() => proxyEnabled = !proxyEnabled}
            class="relative w-9 h-5 rounded-full transition-colors duration-200 {proxyEnabled ? 'bg-accent' : 'bg-line-strong'}"
            role="switch"
            aria-checked={proxyEnabled}
            aria-label="Toggle proxy forwarding"
          >
            <span class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200
              {proxyEnabled ? 'translate-x-[18px]' : 'translate-x-0.5'}"></span>
          </button>
        </div>
        <p class="text-[11px] text-ink-dim mb-3">Requests that don't match any endpoint are forwarded to this URL.</p>

        {#if proxyEnabled}
          <div class="space-y-3 pl-4 border-l-2 border-line">
            <div>
              <label for="proxy-url" class="block text-[10px] text-ink-muted mb-1.5">Upstream URL</label>
              <input
                id="proxy-url"
                type="text"
                bind:value={proxyUrl}
                placeholder="https://api.example.com"
                class="w-full bg-app text-ink text-xs font-mono px-3 py-2 border border-line-strong rounded
                       focus:outline-none focus:border-accent transition-colors placeholder:text-ink-dim"
              />
            </div>
            <div class="flex items-center gap-3">
              <button
                onclick={() => proxyInsecure = !proxyInsecure}
                class="relative w-9 h-5 rounded-full transition-colors duration-200 {proxyInsecure ? 'bg-warn' : 'bg-line-strong'}"
                role="switch"
                aria-checked={proxyInsecure}
                aria-label="Toggle accept invalid TLS certificates"
              >
                <span class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200
                  {proxyInsecure ? 'translate-x-[18px]' : 'translate-x-0.5'}"></span>
              </button>
              <div>
                <p class="text-[11px] text-ink-muted">Accept invalid / self-signed TLS certificates</p>
                {#if proxyInsecure}
                  <p class="text-[10px] text-warn mt-0.5">⚠ Only use for local development targets</p>
                {/if}
              </div>
            </div>
          </div>
        {/if}
      </section>

      <!-- HTTPS Alias -->
      <section class="border-t border-line pt-5">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            {#if hasCert}
              <ShieldCheck size={13} class="text-run" />
            {:else}
              <Shield size={13} class="text-ink-muted" />
            {/if}
            <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Local HTTPS Alias</span>
          </div>
          <button
            onclick={() => aliasEnabled = !aliasEnabled}
            class="relative w-9 h-5 rounded-full transition-colors duration-200 {aliasEnabled ? 'bg-accent' : 'bg-line-strong'}"
            role="switch"
            aria-checked={aliasEnabled}
            aria-label="Toggle HTTPS alias"
          >
            <span class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200
              {aliasEnabled ? 'translate-x-[18px]' : 'translate-x-0.5'}"></span>
          </button>
        </div>
        <p class="text-[11px] text-ink-dim mb-3">
          Serve the same mock via HTTPS on a custom hostname — e.g. <code class="text-accent">https://api.local:8443</code>
        </p>

        {#if aliasEnabled}
          <div class="space-y-4 pl-4 border-l-2 border-line">
            <!-- Hostname + Port -->
            <div class="flex gap-3">
              <div class="flex-1">
                <label for="alias-host" class="block text-[10px] text-ink-muted mb-1.5">Hostname</label>
                <input
                  id="alias-host"
                  type="text"
                  bind:value={aliasHostname}
                  placeholder="api.local"
                  class="w-full bg-app text-ink text-xs font-mono px-3 py-2 border border-line-strong rounded
                         focus:outline-none focus:border-accent transition-colors placeholder:text-ink-dim"
                />
              </div>
              <div class="w-24">
                <label for="alias-port" class="block text-[10px] text-ink-muted mb-1.5">HTTPS Port</label>
                <input
                  id="alias-port"
                  type="number"
                  bind:value={aliasPort}
                  min="1024"
                  max="65535"
                  class="w-full bg-app text-ink text-xs font-mono px-3 py-2 border border-line-strong rounded
                         focus:outline-none focus:border-accent transition-colors"
                />
              </div>
            </div>

            <!-- Certificate Mode -->
            <div>
              <div class="flex items-center gap-4 mb-3">
                <span class="text-[10px] text-ink-muted">Certificate Mode</span>
                <label class="flex items-center gap-1.5 cursor-pointer">
                  <input type="radio" bind:group={certMode} value="auto" class="accent-[var(--color-accent)]" />
                  <span class="text-[11px] text-ink-muted">Auto</span>
                </label>
                <label class="flex items-center gap-1.5 cursor-pointer">
                  <input type="radio" bind:group={certMode} value="manual" class="accent-[var(--color-accent)]" />
                  <span class="text-[11px] text-ink-muted">Manual</span>
                </label>
              </div>

              <div class="flex items-center justify-between mb-2">
                <span class="text-[10px] text-ink-muted">TLS Certificate</span>
                {#if hasCert}
                  <span class="text-[10px] text-run flex items-center gap-1">
                    <ShieldCheck size={10} /> Generated
                  </span>
                {:else}
                  <span class="text-[10px] text-warn flex items-center gap-1">
                    <ShieldAlert size={10} /> Not generated
                  </span>
                {/if}
              </div>

              {#if certMode === 'auto'}
                <button
                  onclick={generateAndSetupAuto}
                  disabled={generatingCert || !aliasHostname.trim()}
                  class="w-full px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
                         text-ink-muted hover:text-ink hover:border-accent transition-colors
                         disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  {generatingCert ? 'Setting up…' : hasCert ? 'Regenerate & Re-trust' : 'Generate, Trust & Add to hosts'}
                </button>
                {#if hasCert}
                  <p class="text-[10px] text-ink-dim mt-1.5">
                    Certificate trusted in macOS Keychain. Restart your browser once.
                  </p>
                {/if}
              {:else}
                <div class="flex gap-2 mb-3">
                  <button
                    onclick={generateCert}
                    disabled={generatingCert || !aliasHostname.trim()}
                    class="flex-1 px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
                           text-ink-muted hover:text-ink hover:border-accent transition-colors
                           disabled:opacity-40 disabled:cursor-not-allowed"
                  >
                    {generatingCert ? 'Generating…' : hasCert ? 'Regenerate Certificate' : 'Generate Certificate'}
                  </button>
                  {#if hasCert}
                    <button
                      onclick={downloadCert}
                      class="px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
                             text-ink-muted hover:text-ink hover:border-accent transition-colors"
                      title="Download certificate (.crt)"
                      aria-label="Download certificate"
                    >
                      <Download size={12} />
                    </button>
                  {/if}
                </div>

                {#if hasCert && aliasHostname}
                  <div class="bg-app rounded-lg border border-line p-3 space-y-3 text-[11px]">
                    <p class="text-ink-muted font-krub mb-1">Run these commands in your terminal:</p>

                    <div>
                      <p class="text-ink-dim mb-1">① Trust certificate (macOS Keychain)</p>
                      <div class="flex items-center gap-2 bg-surface rounded px-2.5 py-2 font-mono text-[10px] text-ink border border-line">
                        <span class="flex-1 select-all break-all">{keychainCmd}</span>
                        <button
                          onclick={() => copyCmd(keychainCmd)}
                          class="text-ink-dim hover:text-accent shrink-0"
                          title="Copy"
                        ><Copy size={11} /></button>
                      </div>
                    </div>

                    <div>
                      <p class="text-ink-dim mb-1">② Add to /etc/hosts</p>
                      <div class="flex items-center gap-2 bg-surface rounded px-2.5 py-2 font-mono text-[10px] text-ink border border-line">
                        <span class="flex-1 select-all break-all">{hostsCmd}</span>
                        <button
                          onclick={() => copyCmd(hostsCmd)}
                          class="text-ink-dim hover:text-accent shrink-0"
                          title="Copy"
                        ><Copy size={11} /></button>
                      </div>
                    </div>
                  </div>
                {/if}
              {/if}
            </div>
          </div>
        {/if}
      </section>
    </div>

    <!-- Footer -->
    <div class="flex justify-end gap-2 px-5 py-4 border-t border-line shrink-0">
      <button
        onclick={onClose}
        class="px-4 py-2 text-sm font-krub text-ink-muted hover:text-ink transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={save}
        class="px-4 py-2 text-sm font-krub bg-accent text-white rounded hover:bg-accent/90 transition-colors"
      >
        Save
      </button>
    </div>
  </div>
</div>
