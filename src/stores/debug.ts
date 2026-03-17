import { writable } from 'svelte/store';

export interface Breakpoint {
  id: string;
  filePath: string;
  line: number;
  enabled: boolean;
  condition?: string;
}

interface DebugState {
  breakpoints: Map<string, Breakpoint[]>; // keyed by file path
  activeSessionId: string | null;
}

function createDebugStore() {
  const { subscribe, set, update } = writable<DebugState>({
    breakpoints: new Map(),
    activeSessionId: null,
  });

  return {
    subscribe,

    toggleBreakpoint: (filePath: string, line: number) => {
      update((state) => {
        const fileBreakpoints = state.breakpoints.get(filePath) || [];
        const existingIndex = fileBreakpoints.findIndex(bp => bp.line === line);

        if (existingIndex >= 0) {
          // Remove breakpoint
          fileBreakpoints.splice(existingIndex, 1);
          if (fileBreakpoints.length === 0) {
            state.breakpoints.delete(filePath);
          } else {
            state.breakpoints.set(filePath, fileBreakpoints);
          }
        } else {
          // Add breakpoint
          const newBreakpoint: Breakpoint = {
            id: `${filePath}:${line}`,
            filePath,
            line,
            enabled: true,
          };
          fileBreakpoints.push(newBreakpoint);
          state.breakpoints.set(filePath, fileBreakpoints);
        }

        return state;
      });
    },

    setBreakpoints: (filePath: string, breakpoints: Breakpoint[]) => {
      update((state) => {
        if (breakpoints.length === 0) {
          state.breakpoints.delete(filePath);
        } else {
          state.breakpoints.set(filePath, breakpoints);
        }
        return state;
      });
    },

    clearBreakpoints: (filePath?: string) => {
      update((state) => {
        if (filePath) {
          state.breakpoints.delete(filePath);
        } else {
          state.breakpoints.clear();
        }
        return state;
      });
    },

    enableBreakpoint: (filePath: string, line: number, enabled: boolean) => {
      update((state) => {
        const fileBreakpoints = state.breakpoints.get(filePath);
        if (fileBreakpoints) {
          const bp = fileBreakpoints.find(bp => bp.line === line);
          if (bp) {
            bp.enabled = enabled;
          }
        }
        return state;
      });
    },

    setActiveSession: (sessionId: string | null) => {
      update((state) => {
        state.activeSessionId = sessionId;
        return state;
      });
    },

    getBreakpoints: (filePath: string): Breakpoint[] => {
      let breakpoints: Breakpoint[] = [];
      const unsubscribe = subscribe((state) => {
        breakpoints = state.breakpoints.get(filePath) || [];
      });
      unsubscribe();
      return breakpoints;
    },
  };
}

export const debugStore = createDebugStore();
