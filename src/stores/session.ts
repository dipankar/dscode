import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface ExtensionInfo {
    id: string;
    name: string;
    version: string;
    publisher: string;
    description?: string;
    enabled: boolean;
    active: boolean;
}

export interface SessionState {
    workspace_folders: string[];
    active_extensions: ExtensionInfo[];
    installed_extensions: ExtensionInfo[];
}

// Session state store
export const sessionState = writable<SessionState>({
    workspace_folders: [],
    active_extensions: [],
    installed_extensions: [],
});

// Loading state
export const sessionLoading = writable<boolean>(false);

// Error state
export const sessionError = writable<string | null>(null);

// Initialize session and listen for events
export async function initializeSession() {
    try {
        sessionLoading.set(true);
        sessionError.set(null);

        // Initialize backend session
        await invoke('initialize_session');

        // Load initial state
        const state = await invoke<SessionState>('get_session_state');
        sessionState.set(state);

        console.log('[Session] Initial state loaded:', state);

        // Listen for session events
        await listen('session-event', (event: any) => {
            const { type, data } = event.payload;

            console.log('[Session] Event received:', type, data);

            switch (type) {
                case 'StateChanged':
                    sessionState.set(data.state);
                    console.log('[Session] State updated:', data.state);
                    break;

                case 'ExtensionsChanged':
                    sessionState.update(s => ({
                        ...s,
                        installed_extensions: data.extensions
                    }));
                    console.log('[Session] Extensions list updated');
                    break;

                case 'ExtensionDeleted':
                    console.log('[Session] Extension deleted:', data.extension_id);
                    // State will be updated via StateChanged event
                    break;

                case 'ExtensionLoaded':
                    console.log('[Session] Extension loaded:', data.extension_id);
                    break;

                case 'ExtensionUnloaded':
                    console.log('[Session] Extension unloaded:', data.extension_id);
                    break;

                case 'ExtensionInstalled':
                    console.log('[Session] Extension installed:', data.extension_id);
                    break;

                case 'WorkspaceFolderAdded':
                    console.log('[Session] Workspace folder added:', data.path);
                    break;

                case 'WorkspaceFolderRemoved':
                    console.log('[Session] Workspace folder removed:', data.path);
                    break;

                default:
                    console.warn('[Session] Unknown event type:', type);
            }
        });

        console.log('[Session] Session initialized and listening for events');
    } catch (error) {
        console.error('[Session] Failed to initialize:', error);
        sessionError.set(error instanceof Error ? error.message : String(error));
    } finally {
        sessionLoading.set(false);
    }
}

// Derived stores for convenience
export const installedExtensions = derived(
    sessionState,
    $state => $state.installed_extensions
);

export const activeExtensions = derived(
    sessionState,
    $state => $state.active_extensions
);

export const workspaceFolders = derived(
    sessionState,
    $state => $state.workspace_folders
);

// Extension operations
export async function loadExtension(id: string): Promise<void> {
    try {
        await invoke('session_load_extension', { extensionId: id });
        console.log('[Session] Extension load requested:', id);
    } catch (error) {
        console.error('[Session] Failed to load extension:', error);
        throw error;
    }
}

export async function unloadExtension(id: string): Promise<void> {
    try {
        await invoke('session_unload_extension', { extensionId: id });
        console.log('[Session] Extension unload requested:', id);
    } catch (error) {
        console.error('[Session] Failed to unload extension:', error);
        throw error;
    }
}

export async function deleteExtension(id: string): Promise<void> {
    try {
        await invoke('session_delete_extension', { extensionId: id });
        console.log('[Session] Extension delete requested:', id);
        // UI will auto-update via session events
    } catch (error) {
        console.error('[Session] Failed to delete extension:', error);
        throw error;
    }
}

// Workspace operations
export async function addWorkspaceFolder(path: string): Promise<void> {
    try {
        await invoke('add_workspace_folder', { path });
        console.log('[Session] Workspace folder add requested:', path);
    } catch (error) {
        console.error('[Session] Failed to add workspace folder:', error);
        throw error;
    }
}

export async function removeWorkspaceFolder(path: string): Promise<void> {
    try {
        await invoke('remove_workspace_folder', { path });
        console.log('[Session] Workspace folder remove requested:', path);
    } catch (error) {
        console.error('[Session] Failed to remove workspace folder:', error);
        throw error;
    }
}

// Query functions
export async function refreshSessionState(): Promise<void> {
    try {
        const state = await invoke<SessionState>('get_session_state');
        sessionState.set(state);
        console.log('[Session] State refreshed');
    } catch (error) {
        console.error('[Session] Failed to refresh state:', error);
        throw error;
    }
}
