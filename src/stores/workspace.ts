import { writable } from 'svelte/store';

export interface FileNode {
  name: string;
  path: string;
  node_type: 'file' | 'directory';
  children?: FileNode[];
  isExpanded?: boolean;
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
        const toggleNode = (nodes: FileNode[]): FileNode[] => {
          return nodes.map((node) => {
            if (node.path === path && node.node_type === 'directory') {
              return { ...node, isExpanded: !node.isExpanded };
            }
            if (node.children) {
              return { ...node, children: toggleNode(node.children) };
            }
            return node;
          });
        };

        state.fileTree = toggleNode(state.fileTree);
        return state;
      });
    },
    selectFile: (path: string) => {
      update((state) => {
        state.selectedFile = path;
        return state;
      });
    },
  };
}

export const workspaceStore = createWorkspaceStore();
