import { writable, derived } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { setStatusBarItems, type StatusBarItemState } from './statusbar';
import { toastStore } from '../lib/error-handler';
import { showWindowPrompt } from './windowPrompt';
import { showQuickPick } from './quickPick';
import { showInputBox } from './inputBox';
import { showStatusBarMessage, clearStatusBarMessage } from './statusBarMessage';
import { editorStore } from './editor';
import {
  TauriEventName,
  WindowEventName,
  dispatchWindowDetailEvent,
} from '../lib/contracts/events';
import { sessionCommands } from '../lib/contracts/commands';
import { executeCommand } from '../lib/command-dispatcher';
import {
  registerOutputChannel,
  appendOutputChannel,
  clearOutputChannel,
  disposeOutputChannel,
  setOutputChannelVisibility,
} from './outputChannels';
import { languageFeaturesManager } from '../lib/language-features';

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
  contributes?: any;
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

// Stored unlisten function for cleanup
let sessionUnlisten: UnlistenFn | null = null;

// Initialize session and listen for events
export async function initializeSession() {
  try {
    sessionLoading.set(true);
    sessionError.set(null);

    // Clean up previous listener if any
    if (sessionUnlisten) {
      sessionUnlisten();
      sessionUnlisten = null;
    }

    // Initialize backend session
    await sessionCommands.initialize();

    // Load initial state
    const state = await sessionCommands.getState<SessionState>();
    sessionState.set(state);
    setStatusBarItems(state.status_bar_items || []);

    console.log('[Session] Initial state loaded:', state);

    // Listen for session events
    sessionUnlisten = await listen(TauriEventName.session, (event: any) => {
      const { type, data } = event.payload;

      console.log('[Session] Event received:', type, data);

      switch (type) {
        case 'StateChanged':
          sessionState.set(data.state);
          setStatusBarItems(data.state.status_bar_items || []);
          console.log('[Session] State updated:', data.state);
          break;

        case 'ExtensionsChanged':
          sessionState.update((s) => ({
            ...s,
            installed_extensions: data.extensions,
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
          sessionState.update((s) => ({
            ...s,
            available_commands: data.commands || [],
          }));
          console.log('[Session] Commands updated');
          break;

        case 'StatusBarItems':
          sessionState.update((s) => ({
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

        case 'ExecuteCommandRequest':
          {
            (async () => {
              try {
                await executeCommand(data.command, data.args || []);
                await invoke('window_execute_command_result', {
                  requestId: data.id,
                  result: { success: true },
                });
              } catch (err: any) {
                await invoke('window_execute_command_result', {
                  requestId: data.id,
                  result: { success: false, error: err?.message || String(err) },
                });
              }
            })();
          }
          break;

        case 'WorkspaceFolderAdded':
          console.log('[Session] Workspace folder added:', data.path);
          break;

        case 'WorkspaceFolderRemoved':
          console.log('[Session] Workspace folder removed:', data.path);
          break;

        case 'ProviderRegistered':
          handleProviderRegistered(data);
          break;

        case 'LanguageConfigurationChanged':
          handleLanguageConfigurationChanged(data);
          break;

        case 'DiagnosticsUpdated':
          languageFeaturesManager.publishDiagnostics(data.uri, data.diagnostics || []);
          break;

        case 'DiagnosticsCleared':
          if (data.uri) {
            languageFeaturesManager.publishDiagnostics(data.uri, []);
          }
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

export function cleanupSession(): void {
  if (sessionUnlisten) {
    sessionUnlisten();
    sessionUnlisten = null;
  }
}

// Derived stores for convenience
export const installedExtensions = derived(sessionState, ($state) => $state.installed_extensions);

export const activeExtensions = derived(sessionState, ($state) => $state.active_extensions);

export const workspaceFolders = derived(sessionState, ($state) => $state.workspace_folders);

export const availableCommands = derived(sessionState, ($state) => $state.available_commands);

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

async function handleProviderRegistered(data: any): Promise<void> {
  const {
    provider_type,
    provider_id,
    owner,
    selector: selectorData,
    trigger_characters,
    metadata,
  } = data;

  if (!provider_type || !provider_id || !selectorData) {
    console.warn('[Session] Incomplete provider registration data:', data);
    return;
  }

  const selector = normalizeSelector(selectorData);

  try {
    switch (provider_type) {
      case 'completion':
        await languageFeaturesManager.registerCompletionProvider(
          selector,
          provider_id,
          owner,
          trigger_characters || []
        );
        break;
      case 'hover':
        await languageFeaturesManager.registerHoverProvider(selector, provider_id, owner);
        break;
      case 'definition':
        await languageFeaturesManager.registerDefinitionProvider(selector, provider_id, owner);
        break;
      case 'references':
        await languageFeaturesManager.registerReferencesProvider(selector, provider_id, owner);
        break;
      case 'codeAction':
        console.log('[Session] Code action provider registered:', provider_id);
        break;
      case 'documentSymbol':
        await languageFeaturesManager.registerDocumentSymbolsProvider(selector, provider_id, owner);
        break;
      case 'formatting':
        await languageFeaturesManager.registerDocumentFormattingProvider(
          selector,
          provider_id,
          owner
        );
        break;
      case 'rename':
        await languageFeaturesManager.registerRenameProvider(selector, provider_id, owner, false);
        break;
      case 'signatureHelp':
        await languageFeaturesManager.registerSignatureHelpProvider(
          selector,
          provider_id,
          owner,
          trigger_characters || []
        );
        break;
      case 'codeLens':
        await languageFeaturesManager.registerCodeLensProvider(selector, provider_id, owner);
        break;
      case 'documentHighlight':
        await languageFeaturesManager.registerDocumentHighlightProvider(
          selector,
          provider_id,
          owner
        );
        break;
      case 'foldingRange':
        await languageFeaturesManager.registerFoldingRangeProvider(selector, provider_id, owner);
        break;
      case 'selectionRange':
        await languageFeaturesManager.registerSelectionRangeProvider(selector, provider_id, owner);
        break;
      case 'color':
        await languageFeaturesManager.registerColorProvider(selector, provider_id, owner);
        break;
      case 'rangeFormatting':
        await languageFeaturesManager.registerRangeFormattingProvider(selector, provider_id, owner);
        break;
      case 'onTypeFormatting':
        await languageFeaturesManager.registerOnTypeFormattingProvider(
          selector,
          provider_id,
          owner,
          trigger_characters || []
        );
        break;
      case 'semanticTokens':
        await languageFeaturesManager.registerSemanticTokensProvider(
          selector,
          provider_id,
          owner,
          metadata?.tokenTypes
            ? { token_types: metadata.tokenTypes, token_modifiers: metadata.tokenModifiers || [] }
            : { token_types: [], token_modifiers: [] }
        );
        break;
      case 'workspaceSymbol':
        await languageFeaturesManager.registerWorkspaceSymbolsProvider(provider_id, owner);
        break;
      default:
        console.log('[Session] Unhandled provider type:', provider_type);
    }
  } catch (error) {
    console.error(`[Session] Failed to register ${provider_type} provider:`, error);
  }
}

function normalizeSelector(selectorData: any): {
  filters: Array<{ language?: string; scheme?: string; pattern?: string }>;
} {
  if (selectorData && typeof selectorData === 'object' && 'filters' in selectorData) {
    return selectorData;
  }
  if (Array.isArray(selectorData)) {
    return {
      filters: selectorData.map((item: any) => {
        if (typeof item === 'string') {
          return { language: item };
        }
        return {
          language: item.language,
          scheme: item.scheme,
          pattern: item.pattern,
        };
      }),
    };
  }
  if (typeof selectorData === 'string') {
    return { filters: [{ language: selectorData }] };
  }
  return { filters: [] };
}

async function handleLanguageConfigurationChanged(data: any): Promise<void> {
  const { language, configuration } = data;
  if (!language) return;

  try {
    const monacoModule = await import('monaco-editor');
    if (configuration.comments) {
      monacoModule.languages.setLanguageConfiguration(language, {
        comments: configuration.comments,
      });
    }
    if (configuration.brackets) {
      monacoModule.languages.setLanguageConfiguration(language, {
        brackets: configuration.brackets,
      });
    }
    if (configuration.wordPattern) {
      const pattern =
        typeof configuration.wordPattern === 'string'
          ? new RegExp(configuration.wordPattern)
          : configuration.wordPattern;
      monacoModule.languages.setLanguageConfiguration(language, { wordPattern: pattern });
    }
    if (configuration.indentationRules) {
      monacoModule.languages.setLanguageConfiguration(language, {
        indentationRules: configuration.indentationRules,
      });
    }
  } catch (error) {
    console.error(`[Session] Failed to set language configuration for ${language}:`, error);
  }
}
