import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { setStatusBarItems, type StatusBarItemState } from './statusbar';
import { toastStore } from '../lib/error-handler';
import { showWindowPrompt } from './windowPrompt';
import { showQuickPick } from './quickPick';
import { showInputBox } from './inputBox';
import { showStatusBarMessage, clearStatusBarMessage } from './statusBarMessage';
import { editorStore } from './editor';
import { TauriEventName, WindowEventName, dispatchWindowDetailEvent } from '../lib/contracts/events';
import { sessionCommands } from '../lib/contracts/commands';
import {
    registerOutputChannel,
    appendOutputChannel,
    clearOutputChannel,
    disposeOutputChannel,
    setOutputChannelVisibility,
} from './outputChannels';

export interface ExtensionInfo {
    id: string;
    name: string;
    version: string;
    publisher: string;
    description?: string;
    enabled: boolean;
    active: boolean;
    categories?: string[];
    dependencies?: string[];
    repository?: string | null;
    activation_events?: string[];
    commands?: string[];
}

export interface SessionState {
    workspace_folders: string[];
    active_extensions: ExtensionInfo[];
    installed_extensions: ExtensionInfo[];
    available_commands: string[];
    status_bar_items: StatusBarItemState[];
}

// Session state store
export const sessionState = writable<SessionState>({
    workspace_folders: [],
    active_extensions: [],
    installed_extensions: [],
    available_commands: [],
    status_bar_items: [],
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
        await sessionCommands.initialize();

        // Load initial state
        const state = await sessionCommands.getState<SessionState>();
        sessionState.set(state);
        setStatusBarItems(state.status_bar_items || []);

        console.log('[Session] Initial state loaded:', state);

        // Listen for session events
        await listen(TauriEventName.session, (event: any) => {
            const { type, data } = event.payload;

            console.log('[Session] Event received:', type, data);

            switch (type) {
                case 'StateChanged':
                    sessionState.set(data.state);
                    setStatusBarItems(data.state.status_bar_items || []);
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

                case 'CommandsChanged':
                    sessionState.update(s => ({
                        ...s,
                        available_commands: data.commands || [],
                    }));
                    console.log('[Session] Commands updated');
                    break;

                case 'StatusBarItems':
                    sessionState.update(s => ({
                        ...s,
                        status_bar_items: data.items || [],
                    }));
                    setStatusBarItems(data.items || []);
                    console.log('[Session] Status bar items updated');
                    break;

                case 'StatusBarMessageShown':
                    showStatusBarMessage(data.id, data.text);
                    break;

                case 'StatusBarMessageCleared':
                    clearStatusBarMessage(data.id);
                    break;

                case 'OutputChannelRegistered':
                    registerOutputChannel(data.channel);
                    break;

                case 'OutputChannelAppended':
                    appendOutputChannel(data.channel, data.value ?? '');
                    break;

                case 'OutputChannelCleared':
                    clearOutputChannel(data.channel);
                    break;

                case 'OutputChannelDisposed':
                    disposeOutputChannel(data.channel);
                    break;

                case 'OutputChannelVisibility':
                    setOutputChannelVisibility(data.channel, !!data.visible);
                    break;

                case 'TreeViewReveal':
                    if (typeof window !== 'undefined') {
                        dispatchWindowDetailEvent(WindowEventName.extensionTreeReveal, data);
                    }
                    break;

                case 'ConfigurationChanged':
                    console.log('[Session] Configuration changed:', data.section, data.key);
                    break;

                case 'DocumentChanged':
                    editorStore.updateContent(data.path, data.content);
                    editorStore.markClean(data.path);
                    break;

                case 'EditorDecorations':
                    if (typeof window !== 'undefined') {
                        dispatchWindowDetailEvent(WindowEventName.editorDecorations, data);
                    }
                    break;

                case 'WindowMessage':
                    {
                        const level = (data.level || 'info') as 'info' | 'warning' | 'error';
                        const actions = Array.isArray(data.actions) ? data.actions : [];
                        const message = actions.length
                            ? `${data.message} (Actions: ${actions.join(', ')})`
                            : data.message;

                        toastStore.show({
                            type: level,
                            message,
                            duration: actions.length ? 0 : 4000,
                        });

                        console.log('[Session] Window message:', data.message, actions);
                    }
                    break;

                case 'WindowActionRequest':
                    showWindowPrompt({
                        id: data.id,
                        level: (data.level || 'info') as 'info' | 'warning' | 'error',
                        message: data.message,
                        actions: data.actions || [],
                    });
                    break;

                case 'QuickPickRequest':
                    showQuickPick(data);
                    break;

                case 'InputBoxRequest':
                    showInputBox(data);
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

export const availableCommands = derived(
    sessionState,
    $state => $state.available_commands
);

// Extension operations
export async function loadExtension(id: string): Promise<void> {
    try {
        await sessionCommands.loadExtension(id);
        console.log('[Session] Extension load requested:', id);
    } catch (error) {
        console.error('[Session] Failed to load extension:', error);
        throw error;
    }
}

export async function unloadExtension(id: string): Promise<void> {
    try {
        await sessionCommands.unloadExtension(id);
        console.log('[Session] Extension unload requested:', id);
    } catch (error) {
        console.error('[Session] Failed to unload extension:', error);
        throw error;
    }
}

export async function deleteExtension(id: string): Promise<void> {
    try {
        await sessionCommands.deleteExtension(id);
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
        await sessionCommands.addWorkspaceFolder(path);
        console.log('[Session] Workspace folder add requested:', path);
    } catch (error) {
        console.error('[Session] Failed to add workspace folder:', error);
        throw error;
    }
}

export async function removeWorkspaceFolder(path: string): Promise<void> {
    try {
        await sessionCommands.removeWorkspaceFolder(path);
        console.log('[Session] Workspace folder remove requested:', path);
    } catch (error) {
        console.error('[Session] Failed to remove workspace folder:', error);
        throw error;
    }
}

// Query functions
export async function refreshSessionState(): Promise<void> {
    try {
        const state = await sessionCommands.getState<SessionState>();
        sessionState.set(state);
        console.log('[Session] State refreshed');
    } catch (error) {
        console.error('[Session] Failed to refresh state:', error);
        throw error;
    }
}
