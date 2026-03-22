/**
 * Activation Events System
 *
 * Handles lazy activation of extensions based on VS Code activation events.
 * Supports all standard VS Code activation events.
 */

import { EventEmitter } from '../api/events';

/**
 * Supported activation event types
 */
export type ActivationEventType =
  | '*'                           // Activate immediately
  | 'onStartupFinished'           // After startup
  | 'onLanguage'                  // When a file of language is opened
  | 'onCommand'                   // When a command is invoked
  | 'onDebug'                     // When debugging starts
  | 'onDebugInitialConfigurations'
  | 'onDebugDynamicConfigurations'
  | 'onDebugResolve'
  | 'onView'                      // When a view is visible
  | 'onUri'                       // When a URI is opened
  | 'onWebviewPanel'              // When a webview panel is created
  | 'onCustomEditor'              // When a custom editor is opened
  | 'onNotebook'                  // When a notebook is opened
  | 'onAuthenticationRequest'     // When auth is requested
  | 'onFileSystem'                // When a file system scheme is used
  | 'onEditSession'               // When edit session identity is needed
  | 'onSearch'                    // When search is initiated
  | 'onTerminalProfile'           // When terminal profile is needed
  | 'workspaceContains'           // When workspace contains pattern
  | 'onWalkthrough';              // When walkthrough is shown

/**
 * Parsed activation event
 */
export interface ParsedActivationEvent {
  type: ActivationEventType;
  argument?: string;  // e.g., 'typescript' for 'onLanguage:typescript'
}

/**
 * Activation trigger callback
 */
export type ActivationCallback = (extensionId: string, event: ParsedActivationEvent) => Promise<void>;

/**
 * Manages activation events for extensions
 */
export class ActivationEventManager {
  // Map from event pattern to extension IDs waiting to be activated
  private pendingActivations = new Map<string, Set<string>>();

  // Extensions that should activate on '*' (immediate)
  private immediateActivations = new Set<string>();

  // Extensions that should activate on startup finished
  private startupFinishedActivations = new Set<string>();

  // Track which extensions have been activated
  private activatedExtensions = new Set<string>();

  // Callback to invoke when activation is needed
  private activationCallback: ActivationCallback | null = null;

  // Event emitters for various activation events
  private _onWillActivate = new EventEmitter<{ extensionId: string; event: ParsedActivationEvent }>();
  private _onDidActivate = new EventEmitter<{ extensionId: string; event: ParsedActivationEvent }>();

  readonly onWillActivate = this._onWillActivate.event;
  readonly onDidActivate = this._onDidActivate.event;

  /**
   * Set the activation callback
   */
  setActivationCallback(callback: ActivationCallback): void {
    this.activationCallback = callback;
  }

  /**
   * Parse an activation event string
   */
  parseActivationEvent(event: string): ParsedActivationEvent {
    if (event === '*') {
      return { type: '*' };
    }

    if (event === 'onStartupFinished') {
      return { type: 'onStartupFinished' };
    }

    // Parse events with arguments like 'onLanguage:typescript'
    const colonIndex = event.indexOf(':');
    if (colonIndex === -1) {
      return { type: event as ActivationEventType };
    }

    const type = event.substring(0, colonIndex) as ActivationEventType;
    const argument = event.substring(colonIndex + 1);

    return { type, argument };
  }

  /**
   * Register an extension's activation events
   */
  registerExtension(extensionId: string, activationEvents: string[]): void {
    console.log(`[Activation] Registering ${extensionId} with events:`, activationEvents);

    for (const event of activationEvents) {
      const parsed = this.parseActivationEvent(event);

      if (parsed.type === '*') {
        this.immediateActivations.add(extensionId);
      } else if (parsed.type === 'onStartupFinished') {
        this.startupFinishedActivations.add(extensionId);
      } else {
        // Create a key for this activation event
        const key = this.getEventKey(parsed);
        if (!this.pendingActivations.has(key)) {
          this.pendingActivations.set(key, new Set());
        }
        this.pendingActivations.get(key)!.add(extensionId);
      }
    }
  }

  /**
   * Unregister an extension
   */
  unregisterExtension(extensionId: string): void {
    this.immediateActivations.delete(extensionId);
    this.startupFinishedActivations.delete(extensionId);
    this.activatedExtensions.delete(extensionId);

    for (const extensions of this.pendingActivations.values()) {
      extensions.delete(extensionId);
    }
  }

  /**
   * Get extensions that need immediate activation
   */
  getImmediateActivations(): string[] {
    return Array.from(this.immediateActivations);
  }

  /**
   * Trigger startup finished activations
   */
  async triggerStartupFinished(): Promise<void> {
    console.log('[Activation] Triggering onStartupFinished');
    const extensions = Array.from(this.startupFinishedActivations);

    for (const extensionId of extensions) {
      await this.activateExtension(extensionId, { type: 'onStartupFinished' });
    }
  }

  /**
   * Trigger activation for a language being opened
   */
  async triggerOnLanguage(languageId: string): Promise<void> {
    console.log(`[Activation] Triggering onLanguage:${languageId}`);
    await this.triggerEvent({ type: 'onLanguage', argument: languageId });
  }

  /**
   * Trigger activation for a command being executed
   */
  async triggerOnCommand(commandId: string): Promise<void> {
    console.log(`[Activation] Triggering onCommand:${commandId}`);
    await this.triggerEvent({ type: 'onCommand', argument: commandId });
  }

  /**
   * Trigger activation for a view becoming visible
   */
  async triggerOnView(viewId: string): Promise<void> {
    console.log(`[Activation] Triggering onView:${viewId}`);
    await this.triggerEvent({ type: 'onView', argument: viewId });
  }

  /**
   * Trigger activation for debugging
   */
  async triggerOnDebug(debugType?: string): Promise<void> {
    console.log(`[Activation] Triggering onDebug${debugType ? ':' + debugType : ''}`);

    // Trigger general onDebug
    await this.triggerEvent({ type: 'onDebug' });

    // Trigger specific debug type if provided
    if (debugType) {
      await this.triggerEvent({ type: 'onDebug', argument: debugType });
    }
  }

  /**
   * Trigger activation for a URI scheme
   */
  async triggerOnUri(scheme?: string): Promise<void> {
    console.log(`[Activation] Triggering onUri${scheme ? ':' + scheme : ''}`);
    await this.triggerEvent({ type: 'onUri', argument: scheme });
  }

  /**
   * Trigger activation for file system scheme
   */
  async triggerOnFileSystem(scheme: string): Promise<void> {
    console.log(`[Activation] Triggering onFileSystem:${scheme}`);
    await this.triggerEvent({ type: 'onFileSystem', argument: scheme });
  }

  /**
   * Trigger activation for authentication request
   */
  async triggerOnAuthenticationRequest(providerId: string): Promise<void> {
    console.log(`[Activation] Triggering onAuthenticationRequest:${providerId}`);
    await this.triggerEvent({ type: 'onAuthenticationRequest', argument: providerId });
  }

  /**
   * Trigger activation for webview panel
   */
  async triggerOnWebviewPanel(viewType: string): Promise<void> {
    console.log(`[Activation] Triggering onWebviewPanel:${viewType}`);
    await this.triggerEvent({ type: 'onWebviewPanel', argument: viewType });
  }

  /**
   * Trigger activation for custom editor
   */
  async triggerOnCustomEditor(viewType: string): Promise<void> {
    console.log(`[Activation] Triggering onCustomEditor:${viewType}`);
    await this.triggerEvent({ type: 'onCustomEditor', argument: viewType });
  }

  /**
   * Trigger activation for notebook
   */
  async triggerOnNotebook(notebookType: string): Promise<void> {
    console.log(`[Activation] Triggering onNotebook:${notebookType}`);
    await this.triggerEvent({ type: 'onNotebook', argument: notebookType });
  }

  /**
   * Trigger activation for terminal profile
   */
  async triggerOnTerminalProfile(profileId: string): Promise<void> {
    console.log(`[Activation] Triggering onTerminalProfile:${profileId}`);
    await this.triggerEvent({ type: 'onTerminalProfile', argument: profileId });
  }

  /**
   * Trigger activation for workspace contains pattern
   */
  async triggerWorkspaceContains(pattern: string): Promise<void> {
    console.log(`[Activation] Triggering workspaceContains:${pattern}`);
    await this.triggerEvent({ type: 'workspaceContains', argument: pattern });
  }

  /**
   * Check if an extension is activated
   */
  isActivated(extensionId: string): boolean {
    return this.activatedExtensions.has(extensionId);
  }

  /**
   * Mark an extension as activated
   */
  markActivated(extensionId: string): void {
    this.activatedExtensions.add(extensionId);
  }

  /**
   * Get all pending activation events for debugging
   */
  getPendingActivations(): Map<string, string[]> {
    const result = new Map<string, string[]>();
    for (const [key, extensions] of this.pendingActivations) {
      result.set(key, Array.from(extensions));
    }
    return result;
  }

  /**
   * Internal: Get the key for an activation event
   */
  private getEventKey(event: ParsedActivationEvent): string {
    if (event.argument) {
      return `${event.type}:${event.argument}`;
    }
    return event.type;
  }

  /**
   * Internal: Trigger activation for an event
   */
  private async triggerEvent(event: ParsedActivationEvent): Promise<void> {
    const key = this.getEventKey(event);
    const extensions = this.pendingActivations.get(key);

    if (!extensions || extensions.size === 0) {
      return;
    }

    // Copy the set since we'll be modifying it
    const extensionsToActivate = Array.from(extensions);

    for (const extensionId of extensionsToActivate) {
      await this.activateExtension(extensionId, event);
    }
  }

  /**
   * Internal: Activate an extension
   */
  private async activateExtension(extensionId: string, event: ParsedActivationEvent): Promise<void> {
    if (this.activatedExtensions.has(extensionId)) {
      return; // Already activated
    }

    if (!this.activationCallback) {
      console.error('[Activation] No activation callback set');
      return;
    }

    console.log(`[Activation] Activating ${extensionId} due to ${this.getEventKey(event)}`);

    this._onWillActivate.fire({ extensionId, event });

    try {
      await this.activationCallback(extensionId, event);
      this.activatedExtensions.add(extensionId);

      // Remove from pending activations
      for (const extensions of this.pendingActivations.values()) {
        extensions.delete(extensionId);
      }

      this._onDidActivate.fire({ extensionId, event });
    } catch (error) {
      console.error(`[Activation] Failed to activate ${extensionId}:`, error);
    }
  }
}

/**
 * Global activation event manager instance
 */
export const activationManager = new ActivationEventManager();
