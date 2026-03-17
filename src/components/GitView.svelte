<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { workspaceStore } from '../stores/workspace';
  import { onMount } from 'svelte';

  interface GitChange {
    path: string;
    status: string;
    staged: boolean;
  }

  interface GitStatus {
    branch: string;
    changes: GitChange[];
    ahead: number;
    behind: number;
  }

  let gitStatus: GitStatus | null = null;
  let commitMessage = '';
  let loading = false;
  let error: string | null = null;

  $: stagedChanges = gitStatus?.changes.filter(c => c.staged) || [];
  $: unstagedChanges = gitStatus?.changes.filter(c => !c.staged) || [];

  onMount(() => {
    loadGitStatus();
    // Refresh every 5 seconds
    const interval = setInterval(loadGitStatus, 5000);
    return () => clearInterval(interval);
  });

  async function loadGitStatus() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) {
      error = 'No workspace folder opened';
      return;
    }

    try {
      loading = true;
      error = null;
      gitStatus = await invoke<GitStatus>('git_status', { repoPath: rootPath });
    } catch (e) {
      error = e as string;
      gitStatus = null;
    } finally {
      loading = false;
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'added': return '✚';
      case 'modified': return '●';
      case 'deleted': return '✖';
      case 'renamed': return '➜';
      case 'conflicted': return '⚠';
      default: return '?';
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'added': return '#89d185';
      case 'modified': return '#f9c74f';
      case 'deleted': return '#f48771';
      case 'renamed': return '#90caf9';
      case 'conflicted': return '#ff6b6b';
      default: return 'var(--text-secondary)';
    }
  }

  async function handleCommit() {
    if (!commitMessage.trim() || stagedChanges.length === 0) return;
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      const commitId = await invoke<string>('git_commit', {
        repoPath: rootPath,
        message: commitMessage
      });
      console.log('[Git] Commit created:', commitId);
      commitMessage = '';
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Commit failed:', e);
      error = `Commit failed: ${e}`;
    }
  }

  async function stageFile(filePath: string) {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_stage_file', {
        repoPath: rootPath,
        filePath
      });
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Stage failed:', e);
      error = `Stage failed: ${e}`;
    }
  }

  async function unstageFile(filePath: string) {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_unstage_file', {
        repoPath: rootPath,
        filePath
      });
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Unstage failed:', e);
      error = `Unstage failed: ${e}`;
    }
  }

  async function stageAll() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      await invoke('git_stage_all', {
        repoPath: rootPath
      });
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Stage all failed:', e);
      error = `Stage all failed: ${e}`;
    }
  }

  async function handlePush() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      const result = await invoke<string>('git_push', {
        repoPath: rootPath
      });
      console.log('[Git] Push:', result);
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Push failed:', e);
      error = `Push failed: ${e}`;
    }
  }

  async function handlePull() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) return;

    try {
      const result = await invoke<string>('git_pull', {
        repoPath: rootPath
      });
      console.log('[Git] Pull:', result);
      await loadGitStatus();
    } catch (e) {
      console.error('[Git] Pull failed:', e);
      error = `Pull failed: ${e}`;
    }
  }

  async function handleSync() {
    // Pull then push
    await handlePull();
    await handlePush();
  }
</script>

<div class="git-view">
  <div class="git-header">
    <h3>SOURCE CONTROL</h3>
    <button class="refresh-btn" on:click={loadGitStatus} title="Refresh">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M13.65 2.35A8 8 0 102.35 13.65 8 8 0 1013.65 2.35zM8 14A6 6 0 118 2a6 6 0 010 12z"/>
        <path d="M8 4v4l3 1.5"/>
      </svg>
    </button>
  </div>

  <div class="git-content">
    {#if loading}
      <div class="loading-state">
        <p>Loading...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <span class="error-icon">⚠</span>
        <p>{error}</p>
        <p class="hint">Open a git repository to use source control</p>
      </div>
    {:else if gitStatus}
      <!-- Branch Info -->
      <div class="branch-info">
        <div class="branch-name">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M11.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 113 2.122V6A2.5 2.5 0 0110 8.5H6a1 1 0 00-1 1v1.128a2.251 2.251 0 11-1.5 0V5.372a2.25 2.25 0 111.5 0v1.836A2.492 2.492 0 016 7h4a1 1 0 001-1v-.628A2.25 2.25 0 019.5 3.25zM4.25 12a.75.75 0 100 1.5.75.75 0 000-1.5zM3.5 3.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0z"/>
          </svg>
          {gitStatus.branch}
        </div>
        <div class="branch-actions">
          {#if gitStatus.ahead > 0 || gitStatus.behind > 0}
            <div class="sync-status">
              {#if gitStatus.ahead > 0}
                <span class="ahead">↑{gitStatus.ahead}</span>
              {/if}
              {#if gitStatus.behind > 0}
                <span class="behind">↓{gitStatus.behind}</span>
              {/if}
            </div>
          {/if}
          <button class="action-btn" on:click={handleSync} title="Sync (Pull & Push)">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 2.5a5.5 5.5 0 0 1 5.5 5.5.75.75 0 0 0 1.5 0 7 7 0 1 0-7 7 .75.75 0 0 0 0-1.5A5.5 5.5 0 1 1 8 2.5z"/>
              <path d="M13.25 8.75a.75.75 0 0 0 0-1.5h-3.5a.75.75 0 0 0-.75.75v3.5a.75.75 0 0 0 1.5 0V9.56l2.22 2.22a.75.75 0 1 0 1.06-1.06l-2.22-2.22h1.69z"/>
            </svg>
          </button>
          <button class="action-btn" on:click={handlePull} title="Pull">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 12.5a.75.75 0 0 1-.75-.75V3.56L5.03 5.78a.75.75 0 0 1-1.06-1.06l3.5-3.5a.75.75 0 0 1 1.06 0l3.5 3.5a.75.75 0 0 1-1.06 1.06L8.75 3.56v8.19a.75.75 0 0 1-.75.75z"/>
              <path d="M3.5 9.75a.75.75 0 0 1 .75.75v1.75c0 .138.112.25.25.25h6.5a.25.25 0 0 0 .25-.25V10.5a.75.75 0 0 1 1.5 0v1.75A1.75 1.75 0 0 1 11 14H4.5A1.75 1.75 0 0 1 2.75 12.25V10.5a.75.75 0 0 1 .75-.75z"/>
            </svg>
          </button>
          <button class="action-btn" on:click={handlePush} title="Push">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 1a.75.75 0 0 1 .75.75v8.19l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 0 1 1.06-1.06l2.22 2.22V1.75A.75.75 0 0 1 8 1z"/>
              <path d="M3.5 9.75a.75.75 0 0 1 .75.75v1.75c0 .138.112.25.25.25h6.5a.25.25 0 0 0 .25-.25V10.5a.75.75 0 0 1 1.5 0v1.75A1.75 1.75 0 0 1 11 14H4.5A1.75 1.75 0 0 1 2.75 12.25V10.5a.75.75 0 0 1 .75-.75z"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Commit Message -->
      {#if stagedChanges.length > 0}
        <div class="commit-section">
          <textarea
            bind:value={commitMessage}
            placeholder="Commit message (Ctrl+Enter to commit)"
            class="commit-message"
            rows="3"
            on:keydown={(e) => {
              if (e.ctrlKey && e.key === 'Enter') {
                handleCommit();
              }
            }}
          />
          <button
            class="commit-btn"
            disabled={!commitMessage.trim()}
            on:click={handleCommit}
          >
            Commit ({stagedChanges.length})
          </button>
        </div>
      {/if}

      <!-- Changes -->
      <div class="changes-container">
        {#if stagedChanges.length > 0}
          <div class="changes-section">
            <div class="section-header">
              <span class="section-title">Staged Changes ({stagedChanges.length})</span>
            </div>
            <div class="changes-list">
              {#each stagedChanges as change}
                <div class="change-item">
                  <span class="status-icon" style="color: {getStatusColor(change.status)}">
                    {getStatusIcon(change.status)}
                  </span>
                  <span class="change-path" title={change.path}>{change.path}</span>
                  <button
                    class="file-action-btn"
                    on:click={() => unstageFile(change.path)}
                    title="Unstage"
                  >
                    −
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if unstagedChanges.length > 0}
          <div class="changes-section">
            <div class="section-header">
              <span class="section-title">Changes ({unstagedChanges.length})</span>
              <button class="stage-all-btn" on:click={stageAll} title="Stage All">
                <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
                </svg>
              </button>
            </div>
            <div class="changes-list">
              {#each unstagedChanges as change}
                <div class="change-item">
                  <span class="status-icon" style="color: {getStatusColor(change.status)}">
                    {getStatusIcon(change.status)}
                  </span>
                  <span class="change-path" title={change.path}>{change.path}</span>
                  <button
                    class="file-action-btn"
                    on:click={() => stageFile(change.path)}
                    title="Stage"
                  >
                    +
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if gitStatus.changes.length === 0}
          <div class="no-changes">
            <span class="no-changes-icon">✓</span>
            <p>No changes</p>
            <p class="hint">Your working tree is clean</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .git-view {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
  }

  .git-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .git-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .refresh-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .refresh-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .git-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .loading-state,
  .error-state,
  .no-changes {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-tertiary);
    text-align: center;
    padding: 20px;
  }

  .error-icon,
  .no-changes-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .error-state p,
  .no-changes p {
    margin: 4px 0;
    font-size: 13px;
  }

  .hint {
    font-size: 11px;
    opacity: 0.7;
  }

  .branch-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .branch-name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .branch-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .action-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 4px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--text-tertiary);
  }

  .sync-status {
    display: flex;
    gap: 8px;
    font-size: 11px;
    font-weight: 600;
  }

  .ahead {
    color: #89d185;
  }

  .behind {
    color: #f9c74f;
  }

  .commit-section {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .commit-message {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    padding: 8px;
    font-family: inherit;
    font-size: 12px;
    resize: vertical;
    min-height: 60px;
  }

  .commit-message:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .commit-btn {
    width: 100%;
    margin-top: 8px;
    padding: 6px 12px;
    background: var(--accent-color);
    color: white;
    border: none;
    border-radius: 3px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.1s;
  }

  .commit-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .commit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .changes-container {
    padding: 4px 0;
  }

  .changes-section {
    margin-bottom: 12px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stage-all-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 3px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .stage-all-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--text-tertiary);
  }

  .changes-list {
    padding: 0;
  }

  .change-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    font-size: 12px;
    transition: background 0.1s;
    position: relative;
  }

  .change-item:hover {
    background: var(--bg-hover);
  }

  .change-item:hover .file-action-btn {
    opacity: 1;
  }

  .status-icon {
    font-size: 14px;
    font-weight: bold;
    min-width: 16px;
    text-align: center;
  }

  .change-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: 'Fira Code', 'Consolas', monospace;
    color: var(--text-primary);
    cursor: default;
  }

  .file-action-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 2px 6px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 14px;
    font-weight: bold;
    line-height: 1;
    transition: all 0.1s;
    opacity: 0;
    min-width: 24px;
  }

  .file-action-btn:hover {
    background: var(--accent-color);
    color: white;
    border-color: var(--accent-color);
  }
</style>
