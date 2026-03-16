/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        'vscode-bg': '#1e1e1e',
        'vscode-bg-secondary': '#252526',
        'vscode-border': '#3e3e42',
        'vscode-text': '#cccccc',
        'vscode-accent': '#007acc',
      },
    },
  },
  plugins: [],
};
