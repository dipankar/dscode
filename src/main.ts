import './app.css';
import App from './App.svelte';
import { warmupJIT, scheduleBackgroundCompilation } from './lib/code-cache';
import { initializeWasmTokenizer } from './lib/wasm-tokenizer';
import { startupController } from './lib/startup/controller';

const appEl = document.getElementById('app')!;

// Mark that JS is executing — this is the first observable phase
startupController.markBooting();

// Warm up V8 JIT compiler immediately
warmupJIT();

// Schedule background compilation for non-critical modules
scheduleBackgroundCompilation();

// Initialize WASM tokenizer in background
const idleCallback = typeof requestIdleCallback !== 'undefined'
  ? requestIdleCallback
  : (cb: () => void) => setTimeout(cb, 100);

idleCallback(() => {
  initializeWasmTokenizer('javascript').catch(err => {
    console.warn('[WASM] Tokenizer initialization failed:', err);
  });
});

// Lazy load Monaco Editor and workers only when needed
// This significantly improves initial load time
export async function initializeMonaco() {
  const [
    { default: editorWorker },
    { default: jsonWorker },
    { default: cssWorker },
    { default: htmlWorker },
    { default: tsWorker }
  ] = await Promise.all([
    import('monaco-editor/esm/vs/editor/editor.worker?worker'),
    import('monaco-editor/esm/vs/language/json/json.worker?worker'),
    import('monaco-editor/esm/vs/language/css/css.worker?worker'),
    import('monaco-editor/esm/vs/language/html/html.worker?worker'),
    import('monaco-editor/esm/vs/language/typescript/ts.worker?worker')
  ]);

  self.MonacoEnvironment = {
    getWorker(_: any, label: string) {
      if (label === 'json') {
        return new jsonWorker();
      }
      if (label === 'css' || label === 'scss' || label === 'less') {
        return new cssWorker();
      }
      if (label === 'html' || label === 'handlebars' || label === 'razor') {
        return new htmlWorker();
      }
      if (label === 'typescript' || label === 'javascript') {
        return new tsWorker();
      }
      return new editorWorker();
    },
  };

  // Load Monaco CSS
  await import('monaco-editor/min/vs/editor/editor.main.css');

  // Register WASM-powered tokenizers for 5-10x faster syntax highlighting
  try {
    const { registerCommonWasmTokenizers } = await import('./lib/monaco-wasm-provider');
    registerCommonWasmTokenizers();
  } catch (error) {
    console.warn('[WASM] Could not register WASM tokenizers, using default:', error);
  }
}

// Mount the Svelte application
let app: App | undefined;
try {
  app = new App({ target: appEl });
} catch (error) {
  console.error('[main] Failed to mount App:', error);
  startupController.markError(
    error instanceof Error ? error.message : String(error)
  );
}

// After mount, the framework is ready — transition the startup state machine
startupController.markMounted();

// Expose the startup controller globally for emergency fallbacks
(window as any).startupController = startupController;

export default app;
