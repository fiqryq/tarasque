/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}', './index.html'],
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {
      fontFamily: {
        sf: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'SF Pro Display', 'Helvetica Neue', 'Inter', 'sans-serif'],
        // Aliases for backwards compatibility
        inter: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'SF Pro Display', 'Helvetica Neue', 'Inter', 'sans-serif'],
        krub: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'SF Pro Display', 'Helvetica Neue', 'Inter', 'sans-serif'],
        mono: ['SF Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
      },
      fontSize: {
        'headline': ['24px', { lineHeight: '1.2', fontWeight: '700' }],
        'title': ['17px', { lineHeight: '1.35', fontWeight: '600' }],
        'body': ['13px', { lineHeight: '1.5', fontWeight: '400' }],
        'caption': ['11px', { lineHeight: '1.4', fontWeight: '500' }],
      },
      colors: {
        // HIG Dark Mode
        'app':          'var(--color-app)',
        'surface':      'var(--color-surface)',
        'surface-alt':  'var(--color-surface-alt)',
        'hover':        'var(--color-hover)',
        'active':       'var(--color-active)',
        'line':         'var(--color-line)',
        'line-strong':  'var(--color-line-strong)',
        'ink':          'var(--color-ink)',
        'ink-secondary':'var(--color-ink-secondary)',
        'ink-tertiary': 'var(--color-ink-tertiary)',
        'ink-quaternary':'var(--color-ink-quaternary)',

        // Backwards-compatible aliases
        'ink-muted':    'var(--color-ink-secondary)',
        'ink-dim':      'var(--color-ink-tertiary)',
        'panel':        'var(--color-surface-alt)',

        // HIG System Colors
        'accent':     '#007AFF',
        'accent-dim': '#0058CC',
        'run':        '#30D158',
        'err':        '#FF453A',
        'warn':       '#FF9F0A',
        'purple':     '#BF5AF2',
        'teal':       '#64D2FF',
        'indigo':     '#5E5CE6',

        // HIG Grays
        'gray-1': '#8E8E93',
        'gray-2': '#636366',
        'gray-3': '#48484A',
        'gray-4': '#3A3A3C',
        'gray-5': '#2C2C2E',
        'gray-6': '#1C1C1E',
      },
      borderRadius: {
        'hig': '12px',
        'hig-sm': '8px',
        'hig-lg': '16px',
      },
      spacing: {
        'hig': '16px',
        'toolbar': '44px',
      },
      backdropBlur: {
        'glass': '20px',
      },
      boxShadow: {
        'hig': '0 0 0 0.5px rgba(0,0,0,0.12), 0 2px 8px rgba(0,0,0,0.08)',
        'hig-hover': '0 0 0 0.5px rgba(0,0,0,0.12), 0 4px 16px rgba(0,0,0,0.12)',
        'hig-dark': '0 0 0 0.5px rgba(255,255,255,0.06), 0 2px 8px rgba(0,0,0,0.3)',
        'hig-dark-hover': '0 0 0 0.5px rgba(255,255,255,0.08), 0 4px 16px rgba(0,0,0,0.4)',
      },
    },
  },
  plugins: [],
};
