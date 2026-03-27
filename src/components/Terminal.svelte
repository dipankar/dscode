<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { settingsStore } from '../lib/settings-store';
  import '@xterm/xterm/css/xterm.css';

  export let terminalId: string;
  export let onClose: (() => void) | undefined = undefined;
  export let visible: boolean = true;

  let terminalContainer: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let unlistenData: (() => void) | null = null;
  let unlistenClosed: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let closedByBackend = false;
  let settingsUnsubscribe: (() => void) | null = null;

  function getTerminalTheme(theme: string): Record<string, string> {
    switch (theme) {
      case 'light':
        return {
          background: '#ffffff',
          foreground: '#333333',
          cursor: '#333333',
          selectionBackground: '#add6ff',
        };
      case 'high-contrast':
        return {
          background: '#000000',
          foreground: '#ffffff',
          cursor: '#ffffff',
          selectionBackground: '#660000',
        };
      default:
        return {
          background: '#1e1e1e',
          foreground: '#cccccc',
          cursor: '#cccccc',
          selectionBackground: '#264f78',
        };
    }
  }

  $: if (visible && fitAddon && terminal) {
    setTimeout(() => {
      fitAddon?.fit();
      terminal?.focus();
    }, 0);
  }

  onMount(async () => {
    const currentSettings = $settingsStore;

    terminal = new Terminal({
      cursorBlink: true,
      fontSize: currentSettings.terminal.fontSize,
      fontFamily: currentSettings.terminal.fontFamily,
      theme: getTerminalTheme(currentSettings.theme.colorTheme),
      scrollback: 10000,
    });

    settingsUnsubscribe = settingsStore.subscribe((settings) => {
      if (!terminal) return;
      terminal.options.theme = getTerminalTheme(settings.theme.colorTheme);
      terminal.options.fontSize = settings.terminal.fontSize;
      terminal.options.fontFamily = settings.terminal.fontFamily;
    });

    // Add addons
    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon());

    // Listen for data from backend BEFORE opening terminal
    // This ensures we catch any initial output
    unlistenData = await listen(`terminal-data:${terminalId}`, (event: any) => {
      if (terminal && event.payload.data) {
        terminal.write(event.payload.data);
      }
    });

    // Open terminal in container
    terminal.open(terminalContainer);

    // Fit terminal to container with a small delay to ensure layout is complete
    setTimeout(() => {
      fitAddon?.fit();
      // Signal backend that we're ready to receive data
      invoke('terminal_ready', { terminalId }).catch((err) => {
        console.error('Failed to signal terminal ready:', err);
      });
    }, 10);

    unlistenClosed = await listen(`terminal-closed:${terminalId}`, () => {
      closedByBackend = true;
      if (onClose) {
        onClose();
      }
    });

    // Handle user input
    terminal.onData((data) => {
      // Send input to backend
      invoke('write_to_terminal', {
        terminalId: terminalId,
        data: data,
      }).catch((err) => {
        console.error('Failed to write to terminal:', err);
      });
    });

    // Handle terminal resize
    resizeObserver = new ResizeObserver(() => {
      if (fitAddon && terminal) {
        fitAddon.fit();

        // Notify backend of resize
        invoke('resize_terminal', {
          terminalId: terminalId,
          cols: terminal.cols,
          rows: terminal.rows,
        }).catch((err) => {
          console.error('Failed to resize terminal:', err);
        });
      }
    });

    resizeObserver.observe(terminalContainer);

    // Focus terminal
    terminal.focus();
  });

  onDestroy(() => {
    if (unlistenData) {
      unlistenData();
    }
    if (unlistenClosed) {
      unlistenClosed();
    }
    if (settingsUnsubscribe) {
      settingsUnsubscribe();
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
    }
    if (terminal) {
      terminal.dispose();
    }
    if (!closedByBackend) {
      invoke('close_terminal', { terminalId }).catch(() => {});
    }
  });
</script>

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalContainer}></div>
</div>

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg);
    overflow: hidden;
  }

  .terminal-container {
    width: 100%;
    height: 100%;
    padding: 8px;
  }

  /* Override xterm styles to fill container */
  :global(.terminal-container .xterm) {
    height: 100% !important;
  }

  :global(.terminal-container .xterm-viewport) {
    width: 100% !important;
  }
</style>
