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

// Strategy 5: Persistent module state (like V8 snapshot)
interface ModuleState {
  timestamp: number;
  exports: any;
  signature: string;
}

export async function loadOrCacheModule(
  modulePath: string,
  loader: () => Promise<any>
): Promise<any> {
  const cacheKey = `module_state_${modulePath}`;

  try {
    // Try IndexedDB for persistent caching
    if ('indexedDB' in window) {
      const cached = await getFromIndexedDB<ModuleState>(cacheKey);
      if (cached && Date.now() - cached.timestamp < 86400000) { // 24hr
        console.log(`[CodeCache] Restored ${modulePath} from persistent cache`);
        return cached.exports;
      }
    }
  } catch (e) {
    console.warn('Failed to load from IndexedDB:', e);
  }

  // Load fresh
  const module = await loader();

  // Cache for next time
  try {
    await saveToIndexedDB(cacheKey, {
      timestamp: Date.now(),
      exports: module,
      signature: JSON.stringify(module),
    });
  } catch (e) {
    console.warn('Failed to save to IndexedDB:', e);
  }

  return module;
}

// IndexedDB helpers
async function getFromIndexedDB<T>(key: string): Promise<T | null> {
  return new Promise((resolve) => {
    const request = indexedDB.open('DSCodeCache', 1);

    request.onerror = () => resolve(null);

    request.onsuccess = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      const transaction = db.transaction(['modules'], 'readonly');
      const store = transaction.objectStore('modules');
      const getRequest = store.get(key);

      getRequest.onsuccess = () => resolve(getRequest.result);
      getRequest.onerror = () => resolve(null);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains('modules')) {
        db.createObjectStore('modules');
      }
    };
  });
}

async function saveToIndexedDB(key: string, value: any): Promise<void> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('DSCodeCache', 1);

    request.onerror = () => reject(request.error);

    request.onsuccess = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      const transaction = db.transaction(['modules'], 'readwrite');
      const store = transaction.objectStore('modules');
      const putRequest = store.put(value, key);

      putRequest.onsuccess = () => resolve();
      putRequest.onerror = () => reject(putRequest.error);
    };
  });
}
