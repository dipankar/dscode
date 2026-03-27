import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface FileNode {
  name: string;
  path: string;
  node_type: 'file' | 'directory';
  children?: FileNode[];
  isExpanded?: boolean;
  isLoaded?: boolean;
  isLoading?: boolean;
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
        state.fileTree = updateNodeInTree(state.fileTree, path, (node) => ({
          ...node,
          isExpanded: !node.isExpanded,
        }));
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

      if (!node || node.isLoaded) {
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) => ({
            ...n,
            isExpanded: !n.isExpanded,
          }));
          return state;
        });
        return;
      }

      update((state) => {
        state.fileTree = updateNodeInTree(state.fileTree, path, (n) => ({
          ...n,
          isLoading: true,
          isExpanded: true,
        }));
        return state;
      });

      try {
        const children = await invoke<FileNode[]>('read_directory', { path });

        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) => ({
            ...n,
            children,
            isLoaded: true,
            isLoading: false,
            isExpanded: true,
          }));
          return state;
        });
      } catch (error) {
        console.error('Failed to load directory:', error);
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) => ({
            ...n,
            isLoading: false,
            isExpanded: false,
          }));
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
        state.fileTree = updateNodeInTree(state.fileTree, path, (node) => ({
          ...node,
          isExpanded: false,
        }));
        return state;
      });
    },
    refreshDirectory: async (path: string) => {
      try {
        const children = await invoke<FileNode[]>('read_directory', { path });
        update((state) => {
          state.fileTree = updateNodeInTree(state.fileTree, path, (n) => ({
            ...n,
            children,
            isLoaded: true,
            isExpanded: true,
          }));
          return state;
        });
      } catch (error) {
        console.error('Failed to refresh directory:', error);
      }
    },
    async loadRootTree(path: string) {
      try {
        const tree = await invoke<FileNode[]>('read_directory', { path });
        update((state) => {
          state.rootPath = path;
          state.fileTree = tree.map((node) => ({
            ...node,
            isLoaded: node.node_type === 'directory' ? false : undefined,
          }));
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
      return { ...node, isExpanded: true, isLoaded: node.isLoaded || !!node.children };
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
