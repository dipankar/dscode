import { get } from 'svelte/store';
import { debugStore } from '../../stores/debug';

export function useEditorBreakpoints() {
  let breakpointDecorations: string[] = [];

  function applyBreakpoints(editor: any, path: string) {
    if (!editor) return;
    const state = get(debugStore);
    const fileBreakpoints = state.breakpoints.get(path) || [];

    const newDecorations = fileBreakpoints.map((bp: any) => ({
      range: {
        startLineNumber: bp.line + 1,
        startColumn: 1,
        endLineNumber: bp.line + 1,
        endColumn: 1,
      },
      options: {
        isWholeLine: true,
        glyphMarginClassName: 'breakpoint-glyph',
        glyphMarginHoverMessage: { value: `Breakpoint at line ${bp.line + 1}` },
        stickiness: 1,
      },
    }));

    breakpointDecorations = editor.deltaDecorations(breakpointDecorations, newDecorations);
  }

  function toggleBreakpoint(editor: any, path: string, line: number) {
    if (!editor) return;
    const state = get(debugStore);
    const existing = (state.breakpoints.get(path) || []).find((bp: any) => bp.line === line);

    if (existing) {
      debugStore.toggleBreakpoint(path, line);
    } else {
      debugStore.toggleBreakpoint(path, line);
    }

    applyBreakpoints(editor, path);
  }

  function clearBreakpoints() {
    breakpointDecorations = [];
  }

  return {
    applyBreakpoints,
    toggleBreakpoint,
    clearBreakpoints,
    get breakpointDecorations() {
      return breakpointDecorations;
    },
  };
}
