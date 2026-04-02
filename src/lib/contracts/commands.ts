import { invoke } from '@tauri-apps/api/core';

export const TauriCommandName = {
  initializeSession: 'initialize_session',
  getSessionState: 'get_session_state',
  getSessionLifecycle: 'get_session_lifecycle',
  sessionLoadExtension: 'session_load_extension',
  sessionUnloadExtension: 'session_unload_extension',
  sessionDeleteExtension: 'session_delete_extension',
  addWorkspaceFolder: 'add_workspace_folder',
  removeWorkspaceFolder: 'remove_workspace_folder',
  getAllCommands: 'get_all_commands',
  getAllKeybindings: 'get_all_keybindings',
  getKeybindingsForCommand: 'get_keybindings_for_command',
  getActivityBarItems: 'get_activity_bar_items',
  getMenuItems: 'get_menu_items',
  getMenuItemsFiltered: 'get_menu_items_filtered',
  getPlatform: 'get_platform',
  gitStatus: 'git_status',
  writeFile: 'write_file',
  createFile: 'create_file',
  createFolder: 'create_folder',
  deleteFile: 'delete_file',
  deleteFolder: 'delete_folder',
  renamePath: 'rename_path',
  listExtensions: 'list_extensions',
  searchMarketplace: 'search_marketplace',
  installFromMarketplace: 'install_from_marketplace',
  uninstallExtension: 'uninstall_extension',
  extensionTreeNotifyEvent: 'extension_tree_notify_event',
  extensionTreeGetChildren: 'extension_tree_get_children',
  extensionTreeGetItem: 'extension_tree_get_item',
  extensionExecuteCommand: 'extension_execute_command',
} as const;

export const sessionCommands = {
  initialize() {
    return invoke<void>(TauriCommandName.initializeSession);
  },
  getState<T>() {
    return invoke<T>(TauriCommandName.getSessionState);
  },
  getLifecycle() {
    return invoke<string>(TauriCommandName.getSessionLifecycle);
  },
  loadExtension(extensionId: string) {
    return invoke<void>(TauriCommandName.sessionLoadExtension, {
      extension_id: extensionId,
    });
  },
  unloadExtension(extensionId: string) {
    return invoke<void>(TauriCommandName.sessionUnloadExtension, {
      extension_id: extensionId,
    });
  },
  deleteExtension(extensionId: string) {
    return invoke<void>(TauriCommandName.sessionDeleteExtension, {
      extension_id: extensionId,
    });
  },
  addWorkspaceFolder(path: string) {
    return invoke<void>(TauriCommandName.addWorkspaceFolder, { path });
  },
  removeWorkspaceFolder(path: string) {
    return invoke<void>(TauriCommandName.removeWorkspaceFolder, { path });
  },
} as const;

export const extensionCommands = {
  list<T>() {
    return invoke<T[]>(TauriCommandName.listExtensions);
  },
  searchMarketplace<T>(query: string, page = 1, pageSize = 20) {
    return invoke<T[]>(TauriCommandName.searchMarketplace, {
      query,
      page,
      pageSize,
    });
  },
  installFromMarketplace<T>(publisher: string, name: string, version: string) {
    return invoke<T>(TauriCommandName.installFromMarketplace, {
      publisher,
      name,
      version,
    });
  },
  uninstall(extensionId: string) {
    return invoke<void>(TauriCommandName.uninstallExtension, {
      extension_id: extensionId,
    });
  },
  executeCommand<T = unknown>(command: string, args: any[] = []) {
    return invoke<T>(TauriCommandName.extensionExecuteCommand, {
      command,
      args,
    });
  },
  treeGetChildren<T>(viewId: string, element: unknown) {
    return invoke<T[]>(TauriCommandName.extensionTreeGetChildren, {
      viewId,
      element,
    });
  },
  treeGetItem<T>(viewId: string, element: unknown) {
    return invoke<T>(TauriCommandName.extensionTreeGetItem, {
      viewId,
      element,
    });
  },
  treeNotifyEvent(
    viewId: string,
    event: string,
    details: {
      element?: unknown;
      selection?: unknown[];
      visible?: boolean;
    } = {},
  ) {
    return invoke<void>(TauriCommandName.extensionTreeNotifyEvent, {
      viewId,
      event,
      element: details.element,
      selection: details.selection,
      visible: details.visible,
    });
  },
} as const;

export const registryCommands = {
  getAllCommands<T>() {
    return invoke<T[]>(TauriCommandName.getAllCommands);
  },
  getAllKeybindings<T>() {
    return invoke<T[]>(TauriCommandName.getAllKeybindings);
  },
  getKeybindingsForCommand<T>(command: string) {
    return invoke<T[]>(TauriCommandName.getKeybindingsForCommand, { command });
  },
  getActivityBarItems<T>() {
    return invoke<T[]>(TauriCommandName.getActivityBarItems);
  },
  getMenuItems<T>(location: string) {
    return invoke<T[]>(TauriCommandName.getMenuItems, { location });
  },
  getMenuItemsFiltered<T>(location: string, context: Record<string, any>) {
    return invoke<T[]>(TauriCommandName.getMenuItemsFiltered, {
      location,
      context,
    });
  },
} as const;

export const systemCommands = {
  getPlatform() {
    return invoke<string>(TauriCommandName.getPlatform);
  },
  getGitStatus<T>(repoPath: string) {
    return invoke<T>(TauriCommandName.gitStatus, { repoPath });
  },
  writeFile(path: string, content: string) {
    return invoke<void>(TauriCommandName.writeFile, { path, content });
  },
  createFile(path: string) {
    return invoke<void>(TauriCommandName.createFile, { path });
  },
  createFolder(path: string) {
    return invoke<void>(TauriCommandName.createFolder, { path });
  },
  deleteFile(path: string) {
    return invoke<void>(TauriCommandName.deleteFile, { path });
  },
  deleteFolder(path: string) {
    return invoke<void>(TauriCommandName.deleteFolder, { path });
  },
  renamePath(oldPath: string, newPath: string) {
    return invoke<void>(TauriCommandName.renamePath, {
      oldPath,
      newPath,
    });
  },
} as const;
