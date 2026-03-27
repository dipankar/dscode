import { writable, derived, get } from 'svelte/store';
import type { FileNode } from '../../stores/workspace';

export interface VirtualTreeRow {
  id: string;
  node: FileNode;
  depth: number;
}

export function useVirtualTree(fileTreeStore: import('svelte/store').Readable<FileNode[]>) {
  const expandedPaths = writable<Set<string>>(new Set());

  const flatRows = derived(
    [fileTreeStore, expandedPaths],
    ([$fileTree, $expandedPaths]): VirtualTreeRow[] => {
      const rows: VirtualTreeRow[] = [];

      function walk(nodes: FileNode[], depth: number) {
        for (const node of nodes) {
          rows.push({
            id: node.path,
            node,
            depth,
          });

          if (node.node_type === 'directory' && $expandedPaths.has(node.path) && node.children) {
            walk(node.children, depth + 1);
          }
        }
      }

      walk($fileTree, 0);
      return rows;
    }
  );

  function toggleExpand(path: string) {
    expandedPaths.update((paths) => {
      const next = new Set(paths);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  }

  function expandPath(path: string) {
    expandedPaths.update((paths) => {
      const next = new Set(paths);
      next.add(path);
      return next;
    });
  }

  function collapsePath(path: string) {
    expandedPaths.update((paths) => {
      const next = new Set(paths);
      next.delete(path);
      return next;
    });
  }

  function expandAllPaths(paths: string[]) {
    expandedPaths.update((existing) => {
      const next = new Set(existing);
      for (const p of paths) {
        next.add(p);
      }
      return next;
    });
  }

  function isExpanded(path: string): boolean {
    return get(expandedPaths).has(path);
  }

  return {
    flatRows,
    expandedPaths,
    toggleExpand,
    expandPath,
    collapsePath,
    expandAllPaths,
    isExpanded,
  };
}
