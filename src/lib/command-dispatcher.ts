import { get } from 'svelte/store';
import { activeActivity } from './activity-store';
import { editorStore } from '../stores/editor';
import { workspaceStore } from '../stores/workspace';
import { showConfirmPrompt } from '../stores/windowPrompt';
import { extensionCommands, systemCommands } from './contracts/commands';
import { WindowEventName, dispatchWindowEvent } from './contracts/events';

type CommandHandler = (args?: any[]) => void | Promise<void>;
type ExplorerResourceType = 'file' | 'directory';

function activeEditorState() {
  return get(editorStore);
}

function activeWorkspaceState() {
  return get(workspaceStore);
}

async function saveActiveFile() {
  const state = activeEditorState();
  const activeTab = state.tabs.find((tab) => tab.id === state.activeTabId);

  if (!activeTab || !state.monacoInstance) {
    return;
  }

  await systemCommands.writeFile(activeTab.path, state.monacoInstance.getValue());
  editorStore.markClean(activeTab.path);
}

async function saveAllFiles() {
  await editorStore.saveAll(async (path, content) => {
    await systemCommands.writeFile(path, content);
  });
}

function closeActiveFile() {
  const state = activeEditorState();
  if (state.activeTabId) {
    editorStore.closeTab(state.activeTabId);
  }
}

function closeAllFiles() {
  const state = activeEditorState();
  for (const tab of state.tabs) {
    editorStore.closeTab(tab.id);
  }
}

function runEditorAction(actionId: string) {
  activeEditorState().monacoInstance?.getAction(actionId)?.run();
}

function focusActivity(activity: 'explorer' | 'search' | 'scm') {
  activeActivity.set(activity);

  if (activity === 'explorer') {
    dispatchWindowEvent(WindowEventName.focusExplorer);
    return;
  }

  if (activity === 'search') {
    dispatchWindowEvent(WindowEventName.focusSearch);
    return;
  }

  dispatchWindowEvent(WindowEventName.focusGit);
}

function getResourceTarget(args: any[] = []): {
  path?: string;
  resourceType?: ExplorerResourceType;
} {
  const [path, details] = args;
  const resourceType =
    details && typeof details === 'object' && 'resourceType' in details
      ? (details.resourceType as ExplorerResourceType | undefined)
      : undefined;

  return {
    path: typeof path === 'string' ? path : undefined,
    resourceType,
  };
}

function resolveDirectoryTarget(path: string, resourceType?: ExplorerResourceType) {
  if (resourceType === 'directory') {
    return path;
  }

  const lastSlash = path.lastIndexOf('/');
  return lastSlash >= 0 ? path.slice(0, lastSlash) : path;
}

function joinPath(parent: string, name: string) {
  return parent.endsWith('/') ? `${parent}${name}` : `${parent}/${name}`;
}

function basename(path: string) {
  const lastSlash = path.lastIndexOf('/');
  return lastSlash >= 0 ? path.slice(lastSlash + 1) : path;
}

function refreshWorkspace() {
  dispatchWindowEvent(WindowEventName.refreshWorkspace);
}

async function createExplorerFile(args: any[] = []) {
  const { path, resourceType } = getResourceTarget(args);
  if (!path) {
    return;
  }

  const fileName = window.prompt('Enter file name:');
  if (!fileName?.trim()) {
    return;
  }

  const directory = resolveDirectoryTarget(path, resourceType);
  await systemCommands.createFile(joinPath(directory, fileName.trim()));
  refreshWorkspace();
}

async function createExplorerFolder(args: any[] = []) {
  const { path, resourceType } = getResourceTarget(args);
  if (!path) {
    return;
  }

  const folderName = window.prompt('Enter folder name:');
  if (!folderName?.trim()) {
    return;
  }

  const directory = resolveDirectoryTarget(path, resourceType);
  await systemCommands.createFolder(joinPath(directory, folderName.trim()));
  refreshWorkspace();
}

async function renameExplorerPath(args: any[] = []) {
  const { path } = getResourceTarget(args);
  if (!path) {
    return;
  }

  const currentName = basename(path);
  const nextName = window.prompt('Enter new name:', currentName);
  if (!nextName?.trim() || nextName.trim() === currentName) {
    return;
  }

  const lastSlash = path.lastIndexOf('/');
  const parent = lastSlash >= 0 ? path.slice(0, lastSlash) : '';
  const newPath = parent ? joinPath(parent, nextName.trim()) : nextName.trim();

  await systemCommands.renamePath(path, newPath);
  refreshWorkspace();
}

async function deleteExplorerPath(args: any[] = []) {
  const { path, resourceType } = getResourceTarget(args);
  if (!path || !resourceType) {
    return;
  }

  const itemName = basename(path);
  const message =
    resourceType === 'directory'
      ? `Delete folder "${itemName}" and all its contents?`
      : `Delete file "${itemName}"?`;

  if (
    !(await showConfirmPrompt(message, {
      level: 'warning',
      confirmLabel: 'Delete',
      cancelLabel: 'Cancel',
    }))
  ) {
    return;
  }

  if (resourceType === 'directory') {
    await systemCommands.deleteFolder(path);
  } else {
    await systemCommands.deleteFile(path);
  }

  refreshWorkspace();
}

async function copyFilePath(args: any[] = []) {
  const { path } = getResourceTarget(args);
  if (!path) {
    return;
  }

  await navigator.clipboard.writeText(path);
}

async function copyRelativeFilePath(args: any[] = []) {
  const { path } = getResourceTarget(args);
  if (!path) {
    return;
  }

  const workspace = activeWorkspaceState();
  const rootPath = workspace.rootPath;
  const relativePath =
    rootPath && path.startsWith(`${rootPath}/`) ? path.slice(rootPath.length + 1) : basename(path);

  await navigator.clipboard.writeText(relativePath);
}

const builtinCommandHandlers: Record<string, CommandHandler> = {
  'explorer.newFile': (args) => createExplorerFile(args),
  'explorer.newFolder': (args) => createExplorerFolder(args),
  'file.save': () => saveActiveFile(),
  'file.saveAll': () => saveAllFiles(),
  'file.rename': (args) => renameExplorerPath(args),
  'file.delete': (args) => deleteExplorerPath(args),
  'file.close': () => closeActiveFile(),
  'file.closeAll': () => closeAllFiles(),
  copyFilePath: (args) => copyFilePath(args),
  copyRelativeFilePath: (args) => copyRelativeFilePath(args),
  'view.toggleSidebar': () => dispatchWindowEvent(WindowEventName.toggleSidebar),
  'view.togglePanel': () => dispatchWindowEvent(WindowEventName.togglePanel),
  'editor.action.formatDocument': () => runEditorAction('editor.action.formatDocument'),
  'editor.action.commentLine': () => runEditorAction('editor.action.commentLine'),
  'editor.action.startFindReplaceAction': () =>
    runEditorAction('editor.action.startFindReplaceAction'),
  'actions.find': () => runEditorAction('actions.find'),
  'workbench.action.showCommands': () => dispatchWindowEvent(WindowEventName.showCommandPalette),
  'workbench.action.quickOpen': () => dispatchWindowEvent(WindowEventName.showQuickOpen),
  'workbench.action.showAllSymbols': () => dispatchWindowEvent(WindowEventName.showSymbolSearch),
  'workbench.view.explorer': () => focusActivity('explorer'),
  'workbench.action.findInFiles': () => focusActivity('search'),
  'workbench.view.scm': () => focusActivity('scm'),
  'workbench.view.extensions': () => dispatchWindowEvent(WindowEventName.openExtensions),
  'workbench.action.openSettings': () => dispatchWindowEvent(WindowEventName.openSettings),
  'workbench.action.reloadWindow': () => window.location.reload(),
  'workbench.action.navigateBack': () => window.history.back(),
  'workbench.action.navigateForward': () => window.history.forward(),
  'dscode.toggleResourceMetrics': () => dispatchWindowEvent(WindowEventName.toggleResourceMetrics),
};

export async function executeCommand(commandId: string, args: any[] = []) {
  const handler = builtinCommandHandlers[commandId];
  if (handler) {
    await handler(args);
    return;
  }

  await extensionCommands.executeCommand(commandId, args);
}
