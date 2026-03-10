import { writable } from 'svelte/store';

interface LogPanelState {
  collapsed: boolean;
  height: number;
}

const DEFAULT_STATE: LogPanelState = {
  collapsed: false,
  height: 144, // h-36 = 144px
};

function createLogPanelStore() {
  let initial = DEFAULT_STATE;
  try {
    const stored = localStorage.getItem('tarasque-log-panel');
    if (stored) initial = { ...DEFAULT_STATE, ...JSON.parse(stored) };
  } catch {}
  
  const { subscribe, set, update } = writable<LogPanelState>(initial);

  return {
    subscribe,
    toggle: () => {
      update(s => {
        const newState = { ...s, collapsed: !s.collapsed };
        localStorage.setItem('tarasque-log-panel', JSON.stringify(newState));
        return newState;
      });
    },
    setHeight: (height: number) => {
      update(s => {
        const newState = { ...s, height: Math.max(40, Math.min(400, height)) };
        localStorage.setItem('tarasque-log-panel', JSON.stringify(newState));
        return newState;
      });
    },
  };
}

export const logPanel = createLogPanelStore();
