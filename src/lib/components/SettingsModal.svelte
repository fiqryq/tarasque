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
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onclick={onClose}>
  <div
    class="w-[520px] max-h-[80vh] glass-card shadow-hig-dark overflow-hidden"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-5">
      <h2 class="text-headline text-ink">Settings</h2>
      <button
        onclick={onClose}
        class="w-7 h-7 flex items-center justify-center rounded-full bg-surface-alt text-ink-tertiary
               hover:text-ink hover:bg-[var(--color-hover)] transition-colors"
        aria-label="Close settings"
      >
        <X size={14} />
      </button>
    </div>

    <!-- Tabs -->
    <div class="flex px-6 gap-1">
      {#each [{ key: 'general', label: 'General' }, { key: 'appearance', label: 'Appearance' }, { key: 'about', label: 'About' }] as tab}
        <button
          onclick={() => activeTab = tab.key as typeof activeTab}
          class="px-4 py-2 text-body rounded-hig-sm transition-all duration-200
                 {activeTab === tab.key
                   ? 'bg-accent/12 text-accent font-medium'
                   : 'text-ink-secondary hover:text-ink hover:bg-[var(--color-hover)]'}"
        >
          {tab.label}
        </button>
      {/each}
    </div>

    <div class="hig-separator mt-3"></div>

    <!-- Content -->
    <div class="p-6 max-h-[400px] overflow-y-auto">
      {#if activeTab === 'general'}
        <div class="space-y-5">
          <div>
            <label for="settings-port" class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">Default Port</label>
            <input
              id="settings-port"
              type="number"
              bind:value={localSettings.defaultPort}
              min="1024"
              max="65535"
              class="w-full bg-surface-alt text-ink text-body font-mono px-4 py-2.5 rounded-hig-sm
                     focus:outline-none focus:ring-2 focus:ring-accent/40 transition-all"
            />
          </div>
          <div>
            <label for="settings-debounce" class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">Auto-save Debounce (ms)</label>
            <input
              id="settings-debounce"
              type="number"
              bind:value={localSettings.autoSaveDebounce}
              min="100" max="5000" step="100"
              class="w-full bg-surface-alt text-ink text-body font-mono px-4 py-2.5 rounded-hig-sm
                     focus:outline-none focus:ring-2 focus:ring-accent/40 transition-all"
            />
          </div>
          <div>
            <label for="settings-log-buffer" class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">Log Buffer Size</label>
            <input
              id="settings-log-buffer"
              type="number"
              bind:value={localSettings.logBufferSize}
              min="50" max="1000" step="50"
              class="w-full bg-surface-alt text-ink text-body font-mono px-4 py-2.5 rounded-hig-sm
                     focus:outline-none focus:ring-2 focus:ring-accent/40 transition-all"
            />
          </div>
        </div>

      {:else if activeTab === 'appearance'}
        <div class="space-y-5">
          <div>
            <label class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">Theme</label>
            <div class="flex gap-2">
              {#each ['dark', 'light', 'system'] as theme}
                <button
                  onclick={() => localSettings.theme = theme as AppSettings['theme']}
                  class="flex-1 px-4 py-2.5 text-body rounded-hig-sm transition-all duration-200
                         {localSettings.theme === theme
                           ? 'bg-accent/12 text-accent font-medium ring-1 ring-accent/30'
                           : 'bg-surface-alt text-ink-secondary hover:text-ink hover:bg-[var(--color-hover)]'}"
                >
                  {theme.charAt(0).toUpperCase() + theme.slice(1)}
                </button>
              {/each}
            </div>
          </div>
          <div>
            <label for="settings-code-font" class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">
              Code Font Size: {localSettings.codeFontSize}px
            </label>
            <input
              id="settings-code-font"
              type="range"
              bind:value={localSettings.codeFontSize}
              min="12" max="18" step="1"
              class="w-full accent-[#007AFF]"
            />
          </div>
          <div>
            <label for="settings-ui-font" class="block text-caption text-ink-tertiary uppercase tracking-widest mb-2">
              UI Font Size: {localSettings.uiFontSize}px
            </label>
            <input
              id="settings-ui-font"
              type="range"
              bind:value={localSettings.uiFontSize}
              min="12" max="16" step="1"
              class="w-full accent-[#007AFF]"
            />
          </div>
        </div>

      {:else if activeTab === 'about'}
        <div class="space-y-4 text-center py-6">
          <div class="text-headline text-ink">Tarasque</div>
          <div class="text-body text-ink-secondary">Version 1.0.0</div>
          <div class="text-caption text-ink-tertiary pt-4">
            Mock API Server for Developers
          </div>
          <div class="text-caption text-ink-tertiary">
            Built with Tauri + Rust + Svelte
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="hig-separator"></div>
    <div class="flex justify-end gap-3 px-6 py-4">
      <button onclick={onClose} class="btn-capsule btn-capsule-secondary">
        Cancel
      </button>
      <button onclick={save} class="btn-capsule btn-capsule-primary">
        Save
      </button>
    </div>
  </div>
</div>
