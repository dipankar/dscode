import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { flattenVisibleTree, type FileNode, type VirtualTreeRow } from './workspace';

function createWorkspaceStore() {
  const { subscribe, set, update } = (() => {
    let state: {
      rootPath: string | null;
      fileTree: FileNode[];
      selectedFile: string | null;
    } = { rootPath: null, fileTree: [], selectedFile: null };

    const subscribers: Array<(v: typeof state) => void> = [];

    return {
      subscribe(run: (v: typeof state) => void) {
        subscribers.push(run);
        run(state);
        return () => {
          const idx = subscribers.indexOf(run);
          if (idx > -1) subscribers.splice(idx, 1);
        };
      },
      set(v: typeof state) {
        state = v;
        subscribers.forEach((fn) => fn(state));
      },
      update(fn: (v: typeof state) => typeof state) {
        state = fn(state);
        subscribers.forEach((fn) => fn(state));
      },
    };
  })();

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
  };
}

describe('createWorkspaceStore', () => {
  it('initializes with default state', () => {
    const store = createWorkspaceStore();
    const state = get(store);
    expect(state.rootPath).toBeNull();
    expect(state.fileTree).toEqual([]);
    expect(state.selectedFile).toBeNull();
  });

  it('setRootPath updates rootPath', () => {
    const store = createWorkspaceStore();
    store.setRootPath('/home/user/project');
    expect(get(store).rootPath).toBe('/home/user/project');
  });

  it('setFileTree updates fileTree', () => {
    const store = createWorkspaceStore();
    const tree: FileNode[] = [
      { name: 'src', path: '/src', node_type: 'directory' },
      { name: 'index.ts', path: '/src/index.ts', node_type: 'file' },
    ];
    store.setFileTree(tree);
    expect(get(store).fileTree).toEqual(tree);
  });

  it('selectFile updates selectedFile', () => {
    const store = createWorkspaceStore();
    store.selectFile('/src/index.ts');
    expect(get(store).selectedFile).toBe('/src/index.ts');
  });

  it('toggleDirectory toggles isExpanded', () => {
    const store = createWorkspaceStore();
    store.setFileTree([{ name: 'src', path: '/src', node_type: 'directory', isExpanded: false }]);

    store.toggleDirectory('/src');
    expect(get(store).fileTree[0].isExpanded).toBe(true);

    store.toggleDirectory('/src');
    expect(get(store).fileTree[0].isExpanded).toBe(false);
  });

  it('toggleDirectory works on nested directories', () => {
    const store = createWorkspaceStore();
    store.setFileTree([
      {
        name: 'src',
        path: '/src',
        node_type: 'directory',
        isExpanded: false,
        children: [
          { name: 'components', path: '/src/components', node_type: 'directory', isExpanded: true },
        ],
      },
    ]);

    store.toggleDirectory('/src/components');
    expect(get(store).fileTree[0].children![0].isExpanded).toBe(false);

    store.toggleDirectory('/src/components');
    expect(get(store).fileTree[0].children![0].isExpanded).toBe(true);
  });
});

describe('flattenVisibleTree', () => {
  const tree: FileNode[] = [
    {
      name: 'src',
      path: '/src',
      node_type: 'directory',
      isExpanded: true,
      children: [
        { name: 'index.ts', path: '/src/index.ts', node_type: 'file' },
        {
          name: 'utils',
          path: '/src/utils',
          node_type: 'directory',
          isExpanded: false,
          children: [{ name: 'helpers.ts', path: '/src/utils/helpers.ts', node_type: 'file' }],
        },
      ],
    },
    { name: 'package.json', path: '/package.json', node_type: 'file' },
  ];

  it('returns only top-level nodes when no directories are expanded', () => {
    const expanded = new Set<string>();
    const rows = flattenVisibleTree(tree, expanded);
    expect(rows).toHaveLength(2);
    expect(rows[0].node.path).toBe('/src');
    expect(rows[1].node.path).toBe('/package.json');
    expect(rows[0].depth).toBe(0);
    expect(rows[1].depth).toBe(0);
  });

  it('includes children of expanded directories', () => {
    const expanded = new Set(['/src']);
    const rows = flattenVisibleTree(tree, expanded);
    expect(rows).toHaveLength(4);
    expect(rows.map((r) => r.node.path)).toEqual([
      '/src',
      '/src/index.ts',
      '/src/utils',
      '/package.json',
    ]);
    expect(rows[0].depth).toBe(0);
    expect(rows[1].depth).toBe(1);
    expect(rows[2].depth).toBe(1);
  });

  it('includes nested expanded directories', () => {
    const expanded = new Set(['/src', '/src/utils']);
    const rows = flattenVisibleTree(tree, expanded);
    expect(rows).toHaveLength(5);
    expect(rows.map((r) => r.node.path)).toEqual([
      '/src',
      '/src/index.ts',
      '/src/utils',
      '/src/utils/helpers.ts',
      '/package.json',
    ]);
    expect(rows[3].depth).toBe(2);
  });

  it('respects startDepth parameter', () => {
    const expanded = new Set(['/src']);
    const rows = flattenVisibleTree(tree, expanded, 3);
    expect(rows[0].depth).toBe(3);
    expect(rows[1].depth).toBe(4);
  });

  it('handles empty tree', () => {
    const rows = flattenVisibleTree([], new Set());
    expect(rows).toEqual([]);
  });
});
