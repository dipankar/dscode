import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface WorkspaceFolder {
  uri: string;
  name: string;
  index: number;
}

export interface WorkspaceConfiguration {
  section: string;
  scope?: string;
  values: Record<string, any>;
}

export interface FileDecoration {
  uri: string;
  badge?: string;
  tooltip?: string;
  color?: string;
  propagate: boolean;
}

export interface FileDecorationProvider {
  id: string;
  owner: string;
}

export interface FindFilesOptions {
  include?: string;
  exclude?: string;
  max_results?: number;
  follow_symlinks: boolean;
}

export interface TextSearchOptions {
  pattern: string;
  is_regex: boolean;
  is_case_sensitive: boolean;
  is_word_match: boolean;
  include?: string;
  exclude?: string;
  max_results?: number;
}

export interface TextSearchResult {
  uri: string;
  matches: TextSearchMatch[];
}

export interface TextSearchMatch {
  line: number;
  character: number;
  length: number;
  line_text: string;
}

/**
 * Workspace API manager
 */
export class WorkspaceManager {
  private folders: WorkspaceFolder[] = [];
  private configurations: Map<string, WorkspaceConfiguration> = new Map();
  private fileDecorations: Map<string, FileDecoration[]> = new Map();
  private onDidChangeFoldersCallbacks: Array<(folders: WorkspaceFolder[]) => void> = [];
  private onDidChangeConfigurationCallbacks: Array<(section: string, scope?: string) => void> = [];
  private onDidChangeFileDecorationsCallbacks: Array<(providerId: string) => void> = [];

  constructor() {}

  /**
   * Initialize workspace manager
   */
  async initialize(): Promise<void> {
    // Load initial folders
    this.folders = await this.getWorkspaceFolders();

    // Listen for workspace folder changes
    await listen<WorkspaceFolder[]>('workspace-folders-changed', (event) => {
      this.folders = event.payload;
      this.notifyFoldersChanged();
    });

    // Listen for configuration changes
    await listen<[string, string | null]>('configuration-changed', (event) => {
      const [section, scope] = event.payload;
      this.notifyConfigurationChanged(section, scope || undefined);
    });

    // Listen for file decoration changes
    await listen<string>('file-decorations-changed', (event) => {
      this.notifyFileDecorationsChanged(event.payload);
    });

    console.log('[Workspace] Initialized');
  }

  /**
   * Get workspace folders
   */
  async getWorkspaceFolders(): Promise<WorkspaceFolder[]> {
    return await invoke<WorkspaceFolder[]>('get_workspace_folders');
  }

  /**
   * Add workspace folder
   */
  async addWorkspaceFolder(folder: WorkspaceFolder): Promise<void> {
    await invoke('workspace_add_folder', { folder });
  }

  /**
   * Remove workspace folder
   */
  async removeWorkspaceFolder(uri: string): Promise<void> {
    await invoke('workspace_remove_folder', { uri });
  }

  /**
   * Get workspace configuration
   */
  async getConfiguration(section: string, scope?: string): Promise<WorkspaceConfiguration | null> {
    return await invoke<WorkspaceConfiguration | null>('get_workspace_configuration', {
      section,
      scope: scope || null,
    });
  }

  /**
   * Update workspace configuration
   */
  async updateConfiguration(
    section: string,
    key: string,
    value: any,
    scope?: string
  ): Promise<void> {
    await invoke('update_workspace_configuration', {
      section,
      scope: scope || null,
      key,
      value,
    });
  }

  /**
   * Register file decoration provider
   */
  async registerFileDecorationProvider(providerId: string, owner: string): Promise<string> {
    const provider: FileDecorationProvider = {
      id: providerId,
      owner,
    };

    return await invoke<string>('register_file_decoration_provider', { provider });
  }

  /**
   * Update file decorations
   */
  async updateFileDecorations(providerId: string, decorations: FileDecoration[]): Promise<void> {
    await invoke('update_file_decorations', { providerId, decorations });
  }

  /**
   * Get file decorations for a URI
   */
  async getFileDecorations(uri: string): Promise<FileDecoration[]> {
    return await invoke<FileDecoration[]>('get_file_decorations', { uri });
  }

  /**
   * Find files in workspace
   */
  async findFiles(options: FindFilesOptions): Promise<string[]> {
    return await invoke<string[]>('workspace_find_files', {
      options,
      folders: this.folders,
    });
  }

  /**
   * Search for text in workspace
   */
  async findText(options: TextSearchOptions): Promise<TextSearchResult[]> {
    return await invoke<TextSearchResult[]>('workspace_find_text', {
      options,
      folders: this.folders,
    });
  }

  /**
   * Clear workspace data for an owner
   */
  async clearWorkspaceData(owner: string): Promise<void> {
    await invoke('clear_workspace_data', { owner });
  }

  /**
   * Subscribe to workspace folder changes
   */
  onDidChangeFolders(callback: (folders: WorkspaceFolder[]) => void): () => void {
    this.onDidChangeFoldersCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.onDidChangeFoldersCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeFoldersCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to configuration changes
   */
  onDidChangeConfiguration(callback: (section: string, scope?: string) => void): () => void {
    this.onDidChangeConfigurationCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.onDidChangeConfigurationCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeConfigurationCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to file decoration changes
   */
  onDidChangeFileDecorations(callback: (providerId: string) => void): () => void {
    this.onDidChangeFileDecorationsCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.onDidChangeFileDecorationsCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeFileDecorationsCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Notify folder change listeners
   */
  private notifyFoldersChanged(): void {
    for (const callback of this.onDidChangeFoldersCallbacks) {
      try {
        callback(this.folders);
      } catch (error) {
        console.error('[Workspace] Error in folder change callback:', error);
      }
    }
  }

  /**
   * Notify configuration change listeners
   */
  private notifyConfigurationChanged(section: string, scope?: string): void {
    for (const callback of this.onDidChangeConfigurationCallbacks) {
      try {
        callback(section, scope);
      } catch (error) {
        console.error('[Workspace] Error in configuration change callback:', error);
      }
    }
  }

  /**
   * Notify file decoration change listeners
   */
  private notifyFileDecorationsChanged(providerId: string): void {
    for (const callback of this.onDidChangeFileDecorationsCallbacks) {
      try {
        callback(providerId);
      } catch (error) {
        console.error('[Workspace] Error in file decoration change callback:', error);
      }
    }
  }

  /**
   * Get current workspace folders (cached)
   */
  get workspaceFolders(): WorkspaceFolder[] {
    return [...this.folders];
  }
}

// Export singleton instance
export const workspaceManager = new WorkspaceManager();
