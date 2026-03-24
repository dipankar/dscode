<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { workspaceStore } from '../stores/workspace';
  import { onMount, tick } from 'svelte';
  import { showConfirmPrompt } from '../stores/windowPrompt';

  export let visible = false;

  interface GitBranch {
    name: string;
    is_head: boolean;
    is_remote: boolean;
  }

  let branches: GitBranch[] = [];
  let loading = false;
  let error: string | null = null;
  let searchQuery = '';
  let newBranchName = '';
  let showCreateForm = false;
  let searchInput: HTMLInputElement | null = null;
  let branchNameInput: HTMLInputElement | null = null;

  $: filteredBranches = branches.filter((b) =>
    b.name.toLowerCase().includes(searchQuery.toLowerCase())
  );
  $: localBranches = filteredBranches.filter((b) => !b.is_remote);
  $: remoteBranches = filteredBranches.filter((b) => b.is_remote);

  onMount(() => {
    if (visible) {
      loadBranches();
    }
  });

  $: if (visible) {
    loadBranches();
  }

  async function loadBranches() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      loading = true;
      error = null;
      branches = await invoke<GitBranch[]>('git_list_branches', {
        repoPath: rootPath,
      });
    } catch (e) {
      error = e as string;
      branches = [];
    } finally {
      loading = false;
    }
  }

  async function switchBranch(branchName: string) {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_checkout_branch', {
        repoPath: rootPath,
        branchName,
      });
      await loadBranches();
      close();
    } catch (e) {
      error = `Failed to switch branch: ${e}`;
    }
  }

  async function createBranch() {
    if (!newBranchName.trim()) return;

    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_create_branch', {
        repoPath: rootPath,
        branchName: newBranchName.trim(),
      });
      await invoke('git_checkout_branch', {
        repoPath: rootPath,
        branchName: newBranchName.trim(),
      });
      newBranchName = '';
      showCreateForm = false;
      await loadBranches();
      close();
    } catch (e) {
      error = `Failed to create branch: ${e}`;
    }
  }

  async function deleteBranch(branchName: string) {
    const confirmed = await showConfirmPrompt(`Delete branch '${branchName}'?`, {
      level: 'warning',
      confirmLabel: 'Delete',
      cancelLabel: 'Cancel',
    });
    if (!confirmed) return;

    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_delete_branch', {
        repoPath: rootPath,
        branchName,
      });
      await loadBranches();
    } catch (e) {
      error = `Failed to delete branch: ${e}`;
    }
  }

  function close() {
    visible = false;
    searchQuery = '';
    showCreateForm = false;
    newBranchName = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  $: if (visible) {
    tick().then(() => {
      if (showCreateForm) {
        branchNameInput?.focus();
      } else {
        searchInput?.focus();
      }
    });
  }
</script>

{#if visible}
  <div
    class="modal-overlay"
    on:click={handleOverlayClick}
    on:keydown={handleKeydown}
    role="presentation"
    tabindex="-1"
  >
    <div class="modal-content" role="dialog" aria-modal="true" aria-label="Switch branch dialog">
      <div class="modal-header">
        <h2>Switch Branch</h2>
        <button class="close-btn" on:click={close}>×</button>
      </div>

      <div class="modal-body">
        {#if !showCreateForm}
          <div class="search-box">
            <input
              bind:this={searchInput}
              type="text"
              bind:value={searchQuery}
              placeholder="Search branches..."
              class="search-input"
            />
          </div>
        {/if}

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        {#if showCreateForm}
          <div class="create-branch-form">
            <input
              bind:this={branchNameInput}
              type="text"
              bind:value={newBranchName}
              placeholder="New branch name..."
              class="branch-name-input"
              on:keydown={(e) => e.key === 'Enter' && createBranch()}
            />
            <div class="form-actions">
              <button class="btn btn-primary" on:click={createBranch}>Create</button>
              <button class="btn btn-secondary" on:click={() => (showCreateForm = false)}
                >Cancel</button
              >
            </div>
          </div>
        {:else}
          <div class="create-branch-action">
            <button class="btn btn-create" on:click={() => (showCreateForm = true)}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path
                  d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"
                />
              </svg>
              Create New Branch
            </button>
          </div>

          {#if loading}
            <div class="loading">Loading branches...</div>
          {:else}
            <div class="branches-container">
              {#if localBranches.length > 0}
                <div class="branch-section">
                  <h3 class="section-title">Local Branches</h3>
                  <div class="branch-list">
                    {#each localBranches as branch}
                      <div class="branch-item" class:current={branch.is_head}>
                        <button
                          class="branch-button"
                          on:click={() => switchBranch(branch.name)}
                          disabled={branch.is_head}
                        >
                          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                            <path
                              d="M11.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 113 2.122V6A2.5 2.5 0 0110 8.5H6a1 1 0 00-1 1v1.128a2.251 2.251 0 11-1.5 0V5.372a2.25 2.25 0 111.5 0v1.836A2.492 2.492 0 016 7h4a1 1 0 001-1v-.628A2.25 2.25 0 019.5 3.25zM4.25 12a.75.75 0 100 1.5.75.75 0 000-1.5zM3.5 3.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0z"
                            />
                          </svg>
                          <span class="branch-name">{branch.name}</span>
                          {#if branch.is_head}
                            <span class="current-badge">Current</span>
                          {/if}
                        </button>
                        {#if !branch.is_head}
                          <button
                            class="delete-btn"
                            on:click|stopPropagation={() => deleteBranch(branch.name)}
                            title="Delete branch"
                          >
                            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                              <path
                                d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75zM4.496 6.675l.66 6.6a.25.25 0 0 0 .249.225h5.19a.25.25 0 0 0 .249-.225l.66-6.6a.75.75 0 0 1 1.492.149l-.66 6.6A1.748 1.748 0 0 1 10.595 15h-5.19a1.75 1.75 0 0 1-1.741-1.575l-.66-6.6a.75.75 0 1 1 1.492-.15zM6.5 1.75V3h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25z"
                              />
                            </svg>
                          </button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if remoteBranches.length > 0}
                <div class="branch-section">
                  <h3 class="section-title">Remote Branches</h3>
                  <div class="branch-list">
                    {#each remoteBranches as branch}
                      <div class="branch-item remote">
                        <button class="branch-button" on:click={() => switchBranch(branch.name)}>
                          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                            <path
                              d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.249.249 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z"
                            />
                          </svg>
                          <span class="branch-name">{branch.name}</span>
                        </button>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if branches.length === 0 && !loading}
                <div class="empty-state">No branches found</div>
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--modal-overlay-bg);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    z-index: 1000;
    padding-top: 10vh;
  }

  .modal-content {
    background: var(--modal-bg);
    border: 1px solid var(--modal-border);
    border-radius: 6px;
    width: 600px;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transition: all 0.1s;
  }

  .close-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text);
  }

  .modal-body {
    padding: 16px 20px;
    overflow-y: auto;
    flex: 1;
  }

  .search-box {
    margin-bottom: 16px;
  }

  .search-input,
  .branch-name-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text);
    font-size: 13px;
    font-family: inherit;
  }

  .search-input:focus,
  .branch-name-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .error-message {
    padding: 8px 12px;
    background: rgba(244, 135, 113, 0.1);
    border: 1px solid #f48771;
    border-radius: 4px;
    color: #f48771;
    font-size: 12px;
    margin-bottom: 12px;
  }

  .create-branch-form {
    margin-bottom: 16px;
  }

  .form-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.1s;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-secondary {
    background: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .btn-secondary:hover {
    background: var(--color-bg-tertiary);
  }

  .create-branch-action {
    margin-bottom: 16px;
  }

  .btn-create {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px;
    background: transparent;
    border: 1px dashed var(--color-border);
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.1s;
  }

  .btn-create:hover {
    background: var(--color-bg-tertiary);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .loading,
  .empty-state {
    text-align: center;
    padding: 40px;
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  .branches-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .branch-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .branch-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .branch-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .branch-item.current {
    background: rgba(0, 122, 204, 0.1);
    border-radius: 4px;
  }

  .branch-button {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }

  .branch-button:not(:disabled):hover {
    background: var(--color-bg-tertiary);
  }

  .branch-button:disabled {
    cursor: default;
  }

  .branch-name {
    flex: 1;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .current-badge {
    padding: 2px 8px;
    background: var(--color-accent);
    color: white;
    font-size: 10px;
    font-weight: 600;
    border-radius: 3px;
    text-transform: uppercase;
  }

  .delete-btn {
    padding: 4px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    color: var(--color-text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s;
  }

  .delete-btn:hover {
    background: #f48771;
    border-color: #f48771;
    color: white;
  }
</style>
