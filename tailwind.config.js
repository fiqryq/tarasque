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
        // Dark theme (Apple Shark)
        'app': '#1d1d1f',
        'surface': '#2c2c2e',
        'panel': '#3a3a3c',
        'hover': '#3d3d3f',
        'active': '#48484a',
        'line': '#3d3d3f',
        'line-strong': '#545458',
        'ink': '#f5f5f7',
        'ink-muted': '#aeaeb2',
        'ink-dim': '#96969b',
        
        // Apple Science Blue
        'accent': '#0066cc',
        'accent-dim': '#004999',
        
        'run': '#34c759',
        'err': '#ff3b30',
        'warn': '#ff9500',
      },
    },
  },
  plugins: [],
};
