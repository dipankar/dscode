<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { settingsStore, type Settings } from '../lib/settings-store';
  import { showConfirmPrompt } from '../stores/windowPrompt';
  import { focusTrap } from '../lib/focus-trap';

  export let visible = false;
  export let onClose: () => void;

  let settings: Settings;
  let activeTab: 'editor' | 'theme' | 'terminal' | 'git' = 'editor';
  let settingsUnsubscribe: (() => void) | null = null;
  let modalContainer: HTMLDivElement;

  const tabKeys: Array<'editor' | 'theme' | 'terminal' | 'git'> = ['editor', 'theme', 'terminal', 'git'];

  onMount(() => {
    settingsUnsubscribe = settingsStore.subscribe((s) => {
      settings = JSON.parse(JSON.stringify(s));
    });
  });

  onDestroy(() => {
    if (settingsUnsubscribe) {
      settingsUnsubscribe();
      settingsUnsubscribe = null;
    }
  });

  function handleSave() {
    settingsStore.set(settings);
    onClose();
  }

  async function handleReset() {
    const confirmed = await showConfirmPrompt('Reset all settings to defaults?', {
      level: 'warning',
      confirmLabel: 'Reset',
      cancelLabel: 'Cancel',
    });

    if (confirmed) {
      settingsStore.reset();
    }
  }

  function handleTabKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
      e.preventDefault();
      const currentIdx = tabKeys.indexOf(activeTab);
      const nextIdx = (currentIdx + 1) % tabKeys.length;
      activeTab = tabKeys[nextIdx];
      focusActiveTab();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
      e.preventDefault();
      const currentIdx = tabKeys.indexOf(activeTab);
      const prevIdx = (currentIdx - 1 + tabKeys.length) % tabKeys.length;
      activeTab = tabKeys[prevIdx];
      focusActiveTab();
    } else if (e.key === 'Home') {
      e.preventDefault();
      activeTab = tabKeys[0];
      focusActiveTab();
    } else if (e.key === 'End') {
      e.preventDefault();
      activeTab = tabKeys[tabKeys.length - 1];
      focusActiveTab();
    }
  }

  function focusActiveTab() {
    if (!modalContainer) return;
    const tabs = modalContainer.querySelectorAll<HTMLButtonElement>('.tab-btn');
    const idx = tabKeys.indexOf(activeTab);
    if (tabs[idx]) {
      tabs[idx].focus();
    }
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="settings-overlay" on:click={onClose} use:focusTrap={{ onEscape: onClose }}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div bind:this={modalContainer} class="settings-modal" on:click|stopPropagation role="dialog" aria-modal="true" aria-label="Settings">
      <div class="settings-header">
        <h2>Settings</h2>
        <button class="close-btn" on:click={onClose} aria-label="Close settings">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path
              d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z"
            />
          </svg>
        </button>
      </div>

      <div class="settings-content">
        <div class="settings-sidebar" role="tablist" aria-label="Settings categories" tabindex="0" on:keydown={handleTabKeyDown}>
          <button
            class="tab-btn"
            class:active={activeTab === 'editor'}
            role="tab"
            aria-selected={activeTab === 'editor'}
            tabindex={activeTab === 'editor' ? 0 : -1}
            on:click={() => (activeTab = 'editor')}
          >
            Editor
          </button>
          <button
            class="tab-btn"
            class:active={activeTab === 'theme'}
            role="tab"
            aria-selected={activeTab === 'theme'}
            tabindex={activeTab === 'theme' ? 0 : -1}
            on:click={() => (activeTab = 'theme')}
          >
            Theme
          </button>
          <button
            class="tab-btn"
            class:active={activeTab === 'terminal'}
            role="tab"
            aria-selected={activeTab === 'terminal'}
            tabindex={activeTab === 'terminal' ? 0 : -1}
            on:click={() => (activeTab = 'terminal')}
          >
            Terminal
          </button>
          <button
            class="tab-btn"
            class:active={activeTab === 'git'}
            role="tab"
            aria-selected={activeTab === 'git'}
            tabindex={activeTab === 'git' ? 0 : -1}
            on:click={() => (activeTab = 'git')}
          >
            Git
          </button>
        </div>

        <div class="settings-panel" role="tabpanel" aria-label="{activeTab} settings">
          {#if activeTab === 'editor'}
            <h3>Editor Settings</h3>

            <div class="setting-group">
              <label>
                <span class="label">Font Size</span>
                <input type="number" bind:value={settings.editor.fontSize} min="10" max="24" />
              </label>
            </div>

            <div class="setting-group">
              <label>
                <span class="label">Font Family</span>
                <input type="text" bind:value={settings.editor.fontFamily} />
              </label>
            </div>

            <div class="setting-group">
              <label>
                <span class="label">Tab Size</span>
                <input type="number" bind:value={settings.editor.tabSize} min="1" max="8" />
              </label>
            </div>

            <div class="setting-group">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.editor.insertSpaces} />
                <span>Insert Spaces (instead of tabs)</span>
              </label>
            </div>

            <div class="setting-group">
              <label>
                <span class="label">Word Wrap</span>
                <select bind:value={settings.editor.wordWrap}>
                  <option value="off">Off</option>
                  <option value="on">On</option>
                  <option value="wordWrapColumn">Word Wrap Column</option>
                </select>
              </label>
            </div>

            <div class="setting-group">
              <label>
                <span class="label">Line Numbers</span>
                <select bind:value={settings.editor.lineNumbers}>
                  <option value="on">On</option>
                  <option value="off">Off</option>
                  <option value="relative">Relative</option>
                </select>
              </label>
            </div>

            <div class="setting-group">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.editor.minimap} />
                <span>Show Minimap</span>
              </label>
            </div>
          {:else if activeTab === 'theme'}
            <h3>Theme Settings</h3>

            <div class="setting-group">
              <label>
                <span class="label">Color Theme</span>
                <select bind:value={settings.theme.colorTheme}>
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                  <option value="high-contrast">High Contrast</option>
                </select>
              </label>
            </div>
          {:else if activeTab === 'terminal'}
            <h3>Terminal Settings</h3>

            <div class="setting-group">
              <label>
                <span class="label">Font Size</span>
                <input type="number" bind:value={settings.terminal.fontSize} min="10" max="24" />
              </label>
            </div>

            <div class="setting-group">
              <label>
                <span class="label">Font Family</span>
                <input type="text" bind:value={settings.terminal.fontFamily} />
              </label>
            </div>
          {:else if activeTab === 'git'}
            <h3>Git Settings</h3>

            <div class="setting-group">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.git.autoFetch} />
                <span>Auto Fetch</span>
              </label>
            </div>

            <div class="setting-group">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.git.confirmSync} />
                <span>Confirm before Sync</span>
              </label>
            </div>
          {/if}
        </div>
      </div>

      <div class="settings-footer">
        <button class="reset-btn" on:click={handleReset}>Reset to Defaults</button>
        <div class="footer-actions">
          <button class="cancel-btn" on:click={onClose}>Cancel</button>
          <button class="save-btn" on:click={handleSave}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--modal-overlay-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .settings-modal {
    background: var(--modal-bg);
    border: 1px solid var(--modal-border);
    border-radius: 6px;
    width: 800px;
    max-width: 90vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .settings-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 3px;
    display: flex;
    align-items: center;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .settings-sidebar {
    width: 200px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    padding: 8px 0;
  }

  .tab-btn {
    width: 100%;
    text-align: left;
    padding: 8px 16px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.1s;
    border-left: 3px solid transparent;
  }

  .tab-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab-btn.active {
    background: var(--bg-active);
    color: var(--text-primary);
    border-left-color: var(--accent-color);
  }

  .settings-panel {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }

  .settings-panel h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 20px 0;
  }

  .setting-group {
    margin-bottom: 16px;
  }

  .setting-group label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  input[type='text'],
  input[type='number'],
  select {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    padding: 6px 8px;
    font-size: 13px;
    font-family: inherit;
  }

  input[type='text']:focus,
  input[type='number']:focus,
  select:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .checkbox-label {
    flex-direction: row !important;
    align-items: center;
    gap: 8px !important;
  }

  input[type='checkbox'] {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .settings-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-top: 1px solid var(--border-color);
  }

  .footer-actions {
    display: flex;
    gap: 8px;
  }

  .reset-btn,
  .cancel-btn,
  .save-btn {
    padding: 6px 16px;
    border-radius: 3px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.1s;
    border: none;
  }

  .reset-btn {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
  }

  .reset-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .cancel-btn {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
  }

  .save-btn {
    background: var(--accent-color);
    color: var(--color-text-on-accent);
  }

  .save-btn:hover {
    opacity: 0.9;
  }
</style>
