let wasmInitialized = false;
type FastTokenizerModule = {
  default: () => Promise<void>;
  FastTokenizer: new (language: string) => {
    tokenize(text: string): Uint32Array;
  };
};

let tokenizer: InstanceType<FastTokenizerModule['FastTokenizer']> | null = null;
let wasmModulePromise: Promise<FastTokenizerModule> | null = null;

async function loadWasmModule(): Promise<FastTokenizerModule> {
  if (!wasmModulePromise) {
    const modulePath = '../../monaco-wasm/pkg/monaco_wasm.js';
    wasmModulePromise = import(/* @vite-ignore */ modulePath) as Promise<FastTokenizerModule>;
  }
  return wasmModulePromise;
}

/**
 * Initialize the WASM tokenizer
 * Call this once during app startup
 */
export async function initializeWasmTokenizer(language: string = 'javascript'): Promise<void> {
  if (wasmInitialized) return;

  try {
    const wasmModule = await loadWasmModule();
    await wasmModule.default();

    // Create tokenizer instance
    tokenizer = new wasmModule.FastTokenizer(language);
    wasmInitialized = true;

    console.log('[WASM] Tokenizer initialized for', language);
  } catch (error) {
    console.error('[WASM] Failed to initialize tokenizer:', error);
    throw error;
  }
}

/**
 * Tokenize text using WASM
 * Returns array of tokens: [start, length, type, start, length, type, ...]
 *
 * Token types:
 * 0 = Keyword
 * 1 = Identifier
 * 2 = String
 * 3 = Number
 * 4 = Comment
 * 5 = Operator
 * 6 = Whitespace
 */
export function tokenizeText(text: string): Uint32Array | null {
  if (!wasmInitialized || !tokenizer) {
    console.warn('[WASM] Tokenizer not initialized');
    return null;
  }

  try {
    return tokenizer.tokenize(text);
  } catch (error) {
    console.error('[WASM] Tokenization failed:', error);
    return null;
  }
}

/**
 * Change the language of the tokenizer
 */
export function setTokenizerLanguage(language: string): void {
  if (!wasmInitialized) return;

  if (!wasmModulePromise) {
    return;
  }

  void loadWasmModule()
    .then((wasmModule) => {
      tokenizer = new wasmModule.FastTokenizer(language);
      console.log('[WASM] Tokenizer language changed to', language);
    })
    .catch((error) => {
      console.error('[WASM] Failed to change tokenizer language:', error);
    });
}

/**
 * Convert WASM token array to Monaco-compatible format
 */
export function convertToMonacoTokens(tokens: Uint32Array, text: string) {
  const result = [];

  for (let i = 0; i < tokens.length; i += 3) {
    const start = tokens[i];
    const length = tokens[i + 1];
    const type = tokens[i + 2];

    const tokenType =
      ['keyword', 'identifier', 'string', 'number', 'comment', 'operator', 'whitespace'][type] ||
      'identifier';

    result.push({
      startIndex: start,
      length: length,
      type: tokenType,
    });
  }

  return result;
}
