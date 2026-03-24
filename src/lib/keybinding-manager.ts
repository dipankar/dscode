import { listen } from '@tauri-apps/api/event';
import { getCommandContext, initializeCommandContextTracking } from './command-context';
import { executeCommand } from './command-dispatcher';
import { registryCommands, systemCommands } from './contracts/commands';
import { evaluateWhenClause } from './when-clause';

interface Keybinding {
  command: string;
  key: string;
  when?: string;
  platform?: string;
  owner: string;
  args?: any;
}

interface KeySequence {
  keys: string[];
  timestamp: number;
}

export class KeybindingManager {
  private keybindings: Map<string, Keybinding[]> = new Map();
  private currentSequence: KeySequence | null = null;
  private sequenceTimeout = 1000; // 1 second to complete a sequence
  private platform: string = 'unknown';

  constructor() {
    this.init();
  }

  private async init() {
    initializeCommandContextTracking();

    // Get current platform
    this.platform = await systemCommands.getPlatform();

    // Load all keybindings
    await this.loadKeybindings();

    // Listen for keybinding updates
    await listen('keybinding-registered', () => {
      this.loadKeybindings();
    });

    await listen('keybindings-cleared', () => {
      this.loadKeybindings();
    });

    // Set up global keydown listener
    window.addEventListener('keydown', this.handleKeyDown.bind(this), true);
  }

  private async loadKeybindings() {
    try {
      const bindings = await registryCommands.getAllKeybindings<Keybinding>();

      // Clear existing
      this.keybindings.clear();

      // Group by normalized key
      for (const binding of bindings) {
        const normalizedKey = this.normalizeKey(binding.key);
        if (!this.keybindings.has(normalizedKey)) {
          this.keybindings.set(normalizedKey, []);
        }
        this.keybindings.get(normalizedKey)!.push(binding);
      }

      console.log(`[KeybindingManager] Loaded ${bindings.length} keybindings`);
    } catch (error) {
      console.error('[KeybindingManager] Failed to load keybindings:', error);
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    // Don't handle keybindings in input fields (unless it's Escape)
    if (this.isInputElement(event.target) && event.key !== 'Escape') {
      return;
    }

    const keyString = this.eventToKeyString(event);
    if (!keyString) {
      return;
    }

    // Handle key sequences (like "Ctrl+K Ctrl+S")
    const now = Date.now();

    if (this.currentSequence) {
      // Check if sequence timed out
      if (now - this.currentSequence.timestamp > this.sequenceTimeout) {
        this.currentSequence = null;
      }
    }

    // Build current key combination
    let fullKey: string;
    if (this.currentSequence) {
      fullKey = [...this.currentSequence.keys, keyString].join('+');
    } else {
      fullKey = keyString;
    }

    const normalizedKey = this.normalizeKey(fullKey);

    // Try to find matching keybindings
    const bindings = this.keybindings.get(normalizedKey);

    if (bindings && bindings.length > 0) {
      // Found a match - execute
      const binding = this.selectKeybinding(bindings);
      if (binding) {
        event.preventDefault();
        event.stopPropagation();
        this.executeCommand(binding.command, binding.args);
        this.currentSequence = null;
        return;
      }
    }

    // Check if this could be the start of a sequence
    const possibleSequence = this.couldBeSequenceStart(normalizedKey);
    if (possibleSequence) {
      // Start/continue sequence
      event.preventDefault();
      event.stopPropagation();

      if (this.currentSequence) {
        this.currentSequence.keys.push(keyString);
        this.currentSequence.timestamp = now;
      } else {
        this.currentSequence = {
          keys: [keyString],
          timestamp: now,
        };
      }
    } else {
      // Not a match and not a sequence start - reset
      this.currentSequence = null;
    }
  }

  private eventToKeyString(event: KeyboardEvent): string | null {
    const parts: string[] = [];

    // Add modifiers
    if (event.ctrlKey) {
      parts.push(this.platform === 'macos' ? 'Ctrl' : 'Ctrl');
    }
    if (event.altKey) {
      parts.push('Alt');
    }
    if (event.shiftKey && this.needsShiftInKey(event.key)) {
      parts.push('Shift');
    }
    if (event.metaKey) {
      parts.push(this.platform === 'macos' ? 'Cmd' : 'Meta');
    }

    // Add the actual key
    const key = this.normalizeEventKey(event.key);
    if (!key) {
      return null;
    }

    parts.push(key);

    return parts.join('+');
  }

  private needsShiftInKey(key: string): boolean {
    // Shift is only needed for non-printable keys
    // For letters, Shift is implied by uppercase (handled by event.key)
    return key.length > 1; // Multi-character keys like 'ArrowUp', 'F1', etc.
  }

  private normalizeEventKey(key: string): string | null {
    // Map event.key values to keybinding format
    const keyMap: Record<string, string> = {
      ' ': 'Space',
      'ArrowUp': 'Up',
      'ArrowDown': 'Down',
      'ArrowLeft': 'Left',
      'ArrowRight': 'Right',
      'Escape': 'Escape',
      'Enter': 'Enter',
      'Tab': 'Tab',
      'Backspace': 'Backspace',
      'Delete': 'Delete',
      'Insert': 'Insert',
      'Home': 'Home',
      'End': 'End',
      'PageUp': 'PageUp',
      'PageDown': 'PageDown',
    };

    if (keyMap[key]) {
      return keyMap[key];
    }

    // Function keys
    if (key.startsWith('F') && key.length <= 3) {
      return key; // F1-F12
    }

    // Single character keys
    if (key.length === 1) {
      return key.toUpperCase();
    }

    // Ignore modifier keys themselves
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) {
      return null;
    }

    return key;
  }

  private normalizeKey(key: string): string {
    return key
      .toLowerCase()
      .replace(/\s+/g, '+')
      .replace(/command/g, 'cmd')
      .replace(/control/g, 'ctrl')
      .replace(/option/g, 'alt')
      .replace(/meta/g, this.platform === 'macos' ? 'cmd' : 'meta');
  }

  private couldBeSequenceStart(normalizedKey: string): boolean {
    // Check if any keybinding starts with this key
    for (const [key] of this.keybindings) {
      if (key.startsWith(normalizedKey + '+') && key !== normalizedKey) {
        return true;
      }
    }
    return false;
  }

  private selectKeybinding(bindings: Keybinding[]): Keybinding | null {
    const context = getCommandContext();

    const matchingBindings = bindings.filter((binding) =>
      evaluateWhenClause(binding.when, context),
    );

    if (matchingBindings.length === 0) {
      return null;
    }

    // If only one binding, use it
    if (matchingBindings.length === 1) {
      return matchingBindings[0];
    }

    const contextualBinding = matchingBindings.find((binding) => binding.when);
    if (contextualBinding) {
      return contextualBinding;
    }

    return matchingBindings.find((binding) => !binding.when) ?? matchingBindings[0];
  }

  private async executeCommand(command: string, args?: any) {
    try {
      console.log(`[KeybindingManager] Executing command: ${command}`, args);

      await executeCommand(command, args || []);
    } catch (error) {
      console.error(`[KeybindingManager] Failed to execute command ${command}:`, error);
    }
  }

  private isInputElement(target: EventTarget | null): boolean {
    if (!target || !(target instanceof HTMLElement)) {
      return false;
    }

    const tagName = target.tagName.toLowerCase();
    return (
      tagName === 'input' ||
      tagName === 'textarea' ||
      tagName === 'select' ||
      target.isContentEditable
    );
  }

  /**
   * Get the key string for a command (for display purposes)
   */
  async getKeybindingForCommand(command: string): Promise<string | null> {
    try {
      const bindings = await registryCommands.getKeybindingsForCommand<Keybinding>(command);

      if (bindings.length > 0) {
        // Return the first keybinding
        return this.formatKeyForDisplay(bindings[0].key);
      }

      return null;
    } catch (error) {
      console.error('[KeybindingManager] Failed to get keybinding:', error);
      return null;
    }
  }

  private formatKeyForDisplay(key: string): string {
    // Convert to platform-specific display format
    if (this.platform === 'macos') {
      return key
        .replace(/Ctrl/g, '⌃')
        .replace(/Alt/g, '⌥')
        .replace(/Shift/g, '⇧')
        .replace(/Cmd/g, '⌘');
    }

    return key;
  }
}

// Export singleton instance
export const keybindingManager = new KeybindingManager();
