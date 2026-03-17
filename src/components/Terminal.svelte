<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
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

  // Re-fit and focus when visibility changes
  $: if (visible && fitAddon && terminal) {
    setTimeout(() => {
      fitAddon?.fit();
      terminal?.focus();
    }, 0);
  }

  onMount(async () => {
    // Create terminal instance
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#cccccc',
        cursor: '#cccccc',
      },
      scrollback: 10000,
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
      invoke('terminal_ready', { terminalId }).catch(err => {
        console.error('Failed to signal terminal ready:', err);
      });
    }, 10);

    // Listen for terminal closure
    unlistenClosed = await listen(`terminal-closed:${terminalId}`, () => {
      console.log(`Terminal ${terminalId} closed by backend`);
      if (onClose) {
        onClose();
      }
    });

    // Handle user input
    terminal.onData((data) => {
      // Send input to backend
      invoke('write_to_terminal', {
        terminalId: terminalId,
        data: data
      }).catch(err => {
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
          rows: terminal.rows
        }).catch(err => {
          console.error('Failed to resize terminal:', err);
        });
      }
    });

    resizeObserver.observe(terminalContainer);

    // Focus terminal
    terminal.focus();
  });

  onDestroy(() => {
    // Clean up event listeners
    if (unlistenData) {
      unlistenData();
    }
    if (unlistenClosed) {
      unlistenClosed();
    }

    // Clean up resize observer
    if (resizeObserver) {
      resizeObserver.disconnect();
    }

    // Dispose terminal
    if (terminal) {
      terminal.dispose();
    }

    // Close terminal in backend
    invoke('close_terminal', { terminalId: terminalId }).catch(err => {
      console.error('Failed to close terminal:', err);
    });
  });
</script>

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalContainer}></div>
</div>

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
    background-color: #1e1e1e;
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
