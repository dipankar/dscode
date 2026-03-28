import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

/**
 * STATE MACHINE: DirectoryNode
 *
 * Tracks the expand/collapse/loading state of a directory node in the
 * file explorer tree view.
 *
 * State Diagram:
 *
 *   Collapsed ──► Loading ──► Expanded ──► Refreshing ──► Expanded
 *       ▲             │           │             │
 *       │     (error) │           │     (error) │
 *       │             ▼           │             ▼
 *       │         Collapsed      │          Expanded (keep old children)
 *       │              ▲         │
 *       │              │         │
 *       └──────────────┴─────────┘
 *                  (collapse)
 *
 * Transitions:
 *   Collapsed  -> Loading    (expandDirectory() on node without cached children)
 *   Collapsed  -> Expanded   (expandDirectory() on node with cached children)
 *   Loading    -> Expanded   (children loaded successfully from backend)
 *   Loading    -> Collapsed  (load failed, error logged)
 *   Expanded   -> Collapsed  (collapseDirectory() called)
 *   Expanded   -> Refreshing (refreshDirectory() called, reloading children)
 *   Refreshing -> Expanded   (refresh completed, new children applied)
 *
 * Note: Only directory nodes have this state machine. File nodes have no
 * expand/collapse lifecycle.
 *
 * Concurrency:
 *   Svelte store updates are synchronous. The backend call to list
 *   directory contents is async. Between requesting and receiving
 *   children, the node is in Loading/Refreshing state and should
 *   show a loading indicator in the UI.
 */
export type DirectoryNodeState = 'Collapsed' | 'Loading' | 'Expanded' | 'Refreshing';

export function isNodeExpanded(node: FileNode): boolean {
  return node.dirState === 'Expanded' || node.dirState === 'Refreshing';
}

export function isNodeLoading(node: FileNode): boolean {
  return node.dirState === 'Loading' || node.dirState === 'Refreshing';
}

export function isNodeLoaded(node: FileNode): boolean {
  return node.dirState === 'Expanded' || node.dirState === 'Refreshing';
}

export interface FileNode {
  name: string;
  path: string;
  node_type: 'file' | 'directory';
  children?: FileNode[];
  dirState?: DirectoryNodeState;
  /** @deprecated Use dirState and isNodeExpanded() instead. Kept for backward compatibility. */
  isExpanded?: boolean;
  /** @deprecated Use dirState and isNodeLoaded() instead. Kept for backward compatibility. */
  isLoaded?: boolean;
  /** @deprecated Use dirState and isNodeLoading() instead. Kept for backward compatibility. */
  isLoading?: boolean;
}

/** Derives legacy boolean flags from dirState for backward compatibility. */
function withLegacyFlags(node: FileNode): FileNode {
  if (node.node_type !== 'directory') return node;
  return {
    ...node,
    isExpanded: isNodeExpanded(node),
    isLoaded: isNodeLoaded(node),
    isLoading: isNodeLoading(node),
  };
}

interface WorkspaceState {
  rootPath: string | null;
  fileTree: FileNode[];
  selectedFile: string | null;
}

function createWorkspaceStore() {
  const { subscribe, set, update } = writable<WorkspaceState>({
    rootPath: null,
    fileTree: [],
    selectedFile: null,
  });

  function updateNodeInTree(
    nodes: FileNode[],
    path: string,
    updater: (node: FileNode) => FileNode
  ): FileNode[] {
    return nodes.map((node) => {
      if (node.path === path) {
        return updater(node);
      }
      if (node.children) {
        return { ...node, children: updateNodeInTree(node.children, path, updater) };
      }
      return node;
    });
  }

  function findNode(nodes: FileNode[], path: string): FileNode | null {
    for (const node of nodes) {
      if (node.path === path) return node;
      if (node.children) {
        const found = findNode(node.children, path);
        if (found) return found;
      }
    }
    return null;
  }

  return {
    subscribe,
    setRootPath: (path: string) => {
      update((state) => {
        state.rootPath = path;
        return state;
      });
    },
    setFileTree: (tree: FileNode[]) => {
      update((state) => {
        state.fileTree = tree;
        return state;
      });
    },
    toggleDirectory: (path: string) => {
      update((state) => {
        state.fileTree = updateNodeInTree(state.fileTree, path, (node) => {
          const expanded = isNodeExpanded(node);
          const newState: DirectoryNodeState = expanded ? 'Collapsed' : 'Expanded';
          return withLegacyFlags({ ...node, dirState: newState });
        });
        return state;
      });
    },
    selectFile: (path: string) => {
      update((state) => {
        state.selectedFile = path;
        return state;
      });
    },
    async expandDirectory(path: string) {
      let currentTree: FileNode[] = [];
      const unsub = subscribe((state) => {
        currentTree = state.fileTree;
      });
      unsub();

      const node = findNode(currentTree, path);

      if (!node || isNodeLoaded(node)) {
        // Toggle: if already loaded, just toggle expand/collapse
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) => {
            const expanded = isNodeExpanded(n);
            const newState: DirectoryNodeState = expanded ? 'Collapsed' : 'Expanded';
            return withLegacyFlags({ ...n, dirState: newState });
          });
          return state;
        });
        return;
      }

      // Transition to Loading
      update((state) => {
        state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
          withLegacyFlags({ ...n, dirState: 'Loading' })
        );
        return state;
      });

      try {
        const children = await invoke<FileNode[]>('read_directory', { path });

        // Transition to Expanded
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
            withLegacyFlags({
              ...n,
              children,
              dirState: 'Expanded',
            })
          );
          return state;
        });
      } catch (error) {
        console.error('Failed to load directory:', error);
        // Transition to Collapsed on error
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
            withLegacyFlags({ ...n, dirState: 'Collapsed' })
          );
          return state;
        });
      }
    },
    async expandPathToDir(path: string) {
      update((state) => {
        state.fileTree = expandPathInTree(state.fileTree, path);
        return state;
      });
    },
    collapseDirectory: (path: string) => {
      update((state) => {
        state.fileTree = updateNodeInTree(state.fileTree, path, (node) =>
          withLegacyFlags({ ...node, dirState: 'Collapsed' })
        );
        return state;
      });
    },
    refreshDirectory: async (path: string) => {
      // Transition to Refreshing
      update((state) => {
        state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
          withLegacyFlags({ ...n, dirState: 'Refreshing' })
        );
        return state;
      });

      try {
        const children = await invoke<FileNode[]>('read_directory', { path });
        // Transition to Expanded
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
            withLegacyFlags({
              ...n,
              children,
              dirState: 'Expanded',
            })
          );
          return state;
        });
      } catch (error) {
        console.error('Failed to refresh directory:', error);
        // Keep Expanded with old children on error
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) =>
            withLegacyFlags({ ...n, dirState: 'Expanded' })
          );
          return state;
        });
      }
    },
    async loadRootTree(path: string) {
      try {
        const tree = await invoke<FileNode[]>('read_directory', { path });
        update((state) => {
          state.rootPath = path;
          state.fileTree = tree.map((node) => {
            if (node.node_type === 'directory') {
              return withLegacyFlags({ ...node, dirState: 'Collapsed' });
            }
            return node;
          });
          return state;
        });
      } catch (error) {
        console.error('Failed to load workspace:', error);
      }
    },
    findNode,
  };
}

function expandPathInTree(nodes: FileNode[], targetPath: string): FileNode[] {
  return nodes.map((node) => {
    if (node.node_type !== 'directory') return node;

    if (targetPath.startsWith(node.path + '/') || targetPath.startsWith(node.path + '\\')) {
      const loaded = isNodeLoaded(node) || !!node.children;
      const result = withLegacyFlags({
        ...node,
        dirState: loaded ? 'Expanded' : (node.dirState ?? 'Collapsed'),
      });
      return result;
    }

    if (node.children) {
      return { ...node, children: expandPathInTree(node.children, targetPath) };
    }

    return node;
  });
}

export const workspaceStore = createWorkspaceStore();

export interface VirtualTreeRow {
  node: FileNode;
  depth: number;
  parentId: string | null;
}

export function flattenVisibleTree(
  nodes: FileNode[],
  expandedPaths: Set<string>,
  startDepth = 0
): VirtualTreeRow[] {
  const rows: VirtualTreeRow[] = [];

  function walk(items: FileNode[], depth: number) {
    for (const node of items) {
      rows.push({ node, depth, parentId: null });

      if (node.node_type === 'directory' && expandedPaths.has(node.path) && node.children) {
        walk(node.children, depth + 1);
      }
    }
  }

  walk(nodes, startDepth);
  return rows;
}
