<script lang="ts">
  import { settings, applyTheme, type AppSettings } from '$lib/stores/settings';
  import { X } from 'lucide-svelte';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let activeTab = $state<'general' | 'appearance' | 'about'>('general');

  let localSettings = $state<AppSettings>({ ...$settings });

  function save() {
    settings.save(localSettings);
    applyTheme(localSettings.theme);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
  <div 
    class="w-[500px] max-h-[80vh] bg-surface border border-line rounded-xl shadow-2xl overflow-hidden"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-line">
      <h2 class="text-lg font-krub font-semibold text-ink">Settings</h2>
      <button onclick={onClose} class="text-ink-dim hover:text-ink transition-colors">
        <X size={18} />
      </button>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-line">
      <button
        onclick={() => activeTab = 'general'}
        class="flex-1 px-4 py-3 text-sm font-krub transition-colors border-b-2 {activeTab === 'general' ? 'text-ink border-accent' : 'text-ink-muted border-transparent hover:text-ink'}"
      >
        General
      </button>
      <button
        onclick={() => activeTab = 'appearance'}
        class="flex-1 px-4 py-3 text-sm font-krub transition-colors border-b-2 {activeTab === 'appearance' ? 'text-ink border-accent' : 'text-ink-muted border-transparent hover:text-ink'}"
      >
        Appearance
      </button>
      <button
        onclick={() => activeTab = 'about'}
        class="flex-1 px-4 py-3 text-sm font-krub transition-colors border-b-2 {activeTab === 'about' ? 'text-ink border-accent' : 'text-ink-muted border-transparent hover:text-ink'}"
      >
        About
      </button>
    </div>

    <!-- Content -->
    <div class="p-5 max-h-[400px] overflow-y-auto">
      {#if activeTab === 'general'}
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">Default Port</label>
            <input
              type="number"
              bind:value={localSettings.defaultPort}
              min="1024"
              max="65535"
              class="w-full bg-surface text-ink text-sm font-mono px-3 py-2 border border-line-strong rounded focus:outline-none focus:border-accent"
            />
          </div>
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">Auto-save Debounce (ms)</label>
            <input
              type="number"
              bind:value={localSettings.autoSaveDebounce}
              min="100"
              max="5000"
              step="100"
              class="w-full bg-surface text-ink text-sm font-mono px-3 py-2 border border-line-strong rounded focus:outline-none focus:border-accent"
            />
          </div>
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">Log Buffer Size</label>
            <input
              type="number"
              bind:value={localSettings.logBufferSize}
              min="50"
              max="1000"
              step="50"
              class="w-full bg-surface text-ink text-sm font-mono px-3 py-2 border border-line-strong rounded focus:outline-none focus:border-accent"
            />
          </div>
        </div>

      {:else if activeTab === 'appearance'}
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">Theme</label>
            <div class="flex gap-2">
              {#each ['dark', 'light', 'system'] as theme}
                <button
                  onclick={() => localSettings.theme = theme as AppSettings['theme']}
                  class="flex-1 px-3 py-2 text-sm font-krub border rounded transition-colors {localSettings.theme === theme ? 'bg-accent/10 border-accent text-ink' : 'bg-surface border-line-strong text-ink-muted hover:text-ink'}"
                >
                  {theme.charAt(0).toUpperCase() + theme.slice(1)}
                </button>
              {/each}
            </div>
          </div>
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">Code Font Size: {localSettings.codeFontSize}px</label>
            <input
              type="range"
              bind:value={localSettings.codeFontSize}
              min="12"
              max="18"
              step="1"
              class="w-full accent-accent"
            />
          </div>
          <div>
            <label class="block text-xs font-krub text-ink-muted uppercase tracking-widest mb-2">UI Font Size: {localSettings.uiFontSize}px</label>
            <input
              type="range"
              bind:value={localSettings.uiFontSize}
              min="12"
              max="16"
              step="1"
              class="w-full accent-accent"
            />
          </div>
        </div>

      {:else if activeTab === 'about'}
        <div class="space-y-4 text-center py-4">
          <div class="text-2xl font-bold text-ink font-krub">Tarasque</div>
          <div class="text-sm text-ink-muted">Version 1.0.0</div>
          <div class="text-xs text-ink-dim pt-4">
            Mock API Server for Developers
          </div>
          <div class="text-xs text-ink-dim">
            Built with Tauri + Rust + Svelte
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex justify-end gap-2 px-5 py-4 border-t border-line">
      <button
        onclick={onClose}
        class="px-4 py-2 text-sm font-krub text-ink-muted hover:text-ink transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={save}
        class="px-4 py-2 text-sm font-krub bg-accent text-[#1a1a1a] rounded hover:bg-accent/90 transition-colors"
      >
        Save
      </button>
    </div>
  </div>
</div>
