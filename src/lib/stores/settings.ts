import { writable } from 'svelte/store';

export interface AppSettings {
  theme: 'dark' | 'light' | 'system';
  codeFontSize: number;
  uiFontSize: number;
  defaultPort: number;
  autoSaveDebounce: number;
  logBufferSize: number;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: 'dark',
  codeFontSize: 14,
  uiFontSize: 14,
  defaultPort: 3000,
  autoSaveDebounce: 500,
  logBufferSize: 200,
};

function createSettingsStore() {
  const { subscribe, set, update } = writable<AppSettings>(DEFAULT_SETTINGS);

  return {
    subscribe,
    set,
    update,
    load: () => {
      const stored = localStorage.getItem('tarasque-settings');
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          set({ ...DEFAULT_SETTINGS, ...parsed });
        } catch (e) {
          console.error('Failed to parse settings:', e);
        }
      }
    },
    save: (settings: AppSettings) => {
      localStorage.setItem('tarasque-settings', JSON.stringify(settings));
      set(settings);
    },
    reset: () => {
      localStorage.removeItem('tarasque-settings');
      set(DEFAULT_SETTINGS);
    },
  };
}

export const settings = createSettingsStore();

export function applyTheme(theme: 'dark' | 'light' | 'system') {
  let effectiveTheme = theme;
  if (theme === 'system') {
    effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  document.documentElement.setAttribute('data-theme', effectiveTheme);
}
