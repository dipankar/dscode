export function useEditorDecorations() {
  const decorationCache = new Map<string, Map<string, any[]>>();
  const decorationHandles = new Map<string, string[]>();

  function applyDecorationsForPath(
    editor: any,
    path: string,
    key: string,
    decorations: any[]
  ): string[] {
    if (!editor) return [];

    const pathCache = decorationCache.get(path) || new Map<string, any[]>();
    pathCache.set(key, decorations);
    decorationCache.set(path, pathCache);

    const allDecorations: any[] = [];
    for (const [, decos] of pathCache) {
      allDecorations.push(...decos);
    }

    const handles = editor.deltaDecorations(decorationHandles.get(path) || [], allDecorations);
    decorationHandles.set(path, handles);
    return handles;
  }

  function clearAllDecorations(editor: any) {
    for (const [path] of decorationHandles) {
      if (editor) {
        editor.deltaDecorations(decorationHandles.get(path) || [], []);
      }
    }
    decorationCache.clear();
    decorationHandles.clear();
  }

  function applyDecorationKey(editor: any, path: string, key: string, decorations: any[]) {
    applyDecorationsForPath(editor, path, key, decorations);
  }

  function getDecorationCache(): Map<string, Map<string, any[]>> {
    return decorationCache;
  }

  function getDecorationHandles(): Map<string, string[]> {
    return decorationHandles;
  }

  return {
    applyDecorationKey,
    applyDecorationsForPath,
    clearAllDecorations,
    getDecorationCache,
    getDecorationHandles,
  };
}
