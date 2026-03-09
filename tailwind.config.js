/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}', './index.html'],
  theme: {
    extend: {
      fontFamily: {
        krub: ['Krub', 'sans-serif'],
        inter: ['Inter', 'sans-serif'],
      },
      colors: {
        app:     '#181818',
        surface: '#1e1e1e',
        panel:   '#222222',
        hover:   '#282828',
        active:  '#2e2e2e',
        line:    '#2a2a2a',
        'line-strong': '#383838',
        ink:        '#e4e4e4',
        'ink-muted': '#909090',
        'ink-dim':   '#484848',
        accent:      '#818cf8',
        'accent-dim':'#3730a3',
        run:  '#4ade80',
        err:  '#f87171',
        warn: '#fbbf24',
      },
    },
  },
  plugins: [],
};
