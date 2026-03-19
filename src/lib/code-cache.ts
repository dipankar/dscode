/**
 * Exploit browser's native code caching mechanisms
 * Chromium caches compiled bytecode after 2+ loads with same script
 */

// Strategy 1: Use stable script URLs for better caching
export function enableStableScriptUrls() {
  // Vite uses hashed filenames - this ensures cache hits
  // Already handled by Vite's build.modulePreload configuration
}

// Strategy 2: Warm up V8's JIT compiler
export async function warmupJIT() {
  // Run heavy operations once to trigger JIT compilation
  // Subsequent calls use optimized machine code

  const startTime = performance.now();

  // Trigger array operations (common in Monaco)
  const warmup = Array.from({ length: 10000 }, (_, i) => i)
    .map(x => x * 2)
    .filter(x => x % 2 === 0)
    .reduce((a, b) => a + b, 0);

  // Trigger string operations
  const text = 'a'.repeat(10000);
  text.split('').reverse().join('');

  // Trigger object creation patterns
  const objects = Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    name: `item-${i}`,
    value: Math.random(),
  }));

  objects.sort((a, b) => a.value - b.value);

  console.log(`[CodeCache] JIT warmup completed in ${performance.now() - startTime}ms`);
}

// Strategy 3: Module instantiation caching
const moduleCache = new Map<string, any>();

export async function cachedImport<T>(modulePath: string): Promise<T> {
  if (moduleCache.has(modulePath)) {
    return moduleCache.get(modulePath);
  }

  const module = await import(/* @vite-ignore */ modulePath);
  moduleCache.set(modulePath, module);
  return module;
}

// Strategy 4: Compile-on-idle for non-critical modules
export function scheduleBackgroundCompilation() {
  if ('requestIdleCallback' in window) {
    requestIdleCallback(async () => {
      // Pre-compile heavy modules during idle
      await cachedImport('./components/CommandPalette.svelte');
      await cachedImport('./components/QuickOpen.svelte');
      console.log('[CodeCache] Background compilation complete');
    }, { timeout: 2000 });
  }
}

// Strategy 5: Persistent module state is intentionally simplified
// We rely on the runtime module cache instead of serialising exports to IndexedDB
export async function loadOrCacheModule(
  modulePath: string,
  loader: () => Promise<any>
): Promise<any> {
  if (moduleCache.has(modulePath)) {
    return moduleCache.get(modulePath);
  }

  const module = await loader();
  moduleCache.set(modulePath, module);
  return module;
}
