<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { workspaceStore } from '../stores/workspace';
  import { onMount } from 'svelte';

  interface GitCommit {
    id: string;
    author: string;
    email: string;
    message: string;
    timestamp: number;
    parent_ids: string[];
  }

  let commits: GitCommit[] = [];
  let loading = false;
  let error: string | null = null;
  let limit = 50;
  let selectedCommit: GitCommit | null = null;

  onMount(() => {
    loadHistory();
  });

  async function loadHistory() {
    const rootPath = $workspaceStore.rootPath;
    if (!rootPath) {
      error = 'No workspace folder opened';
      return;
    }

    try {
      loading = true;
      error = null;
      commits = await invoke<GitCommit[]>('git_log', {
        repoPath: rootPath,
        limit,
      });
    } catch (e) {
      error = e as string;
      commits = [];
    } finally {
      loading = false;
    }
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
      if (diffHours === 0) {
        const diffMinutes = Math.floor(diffMs / (1000 * 60));
        return `${diffMinutes} minute${diffMinutes !== 1 ? 's' : ''} ago`;
      }
      return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
    } else if (diffDays < 7) {
      return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  function getCommitShortId(id: string): string {
    return id.substring(0, 7);
  }

  function getFirstLine(message: string): string {
    return message.split('\n')[0];
  }

  function selectCommit(commit: GitCommit) {
    selectedCommit = selectedCommit?.id === commit.id ? null : commit;
  }

  async function loadMore() {
    limit += 50;
    await loadHistory();
  }
</script>

<div class="git-history-panel">
  <div class="panel-header">
    <h3>COMMIT HISTORY</h3>
    <button class="refresh-btn" on:click={loadHistory} title="Refresh">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path
          d="M13.65 2.35A8 8 0 102.35 13.65 8 8 0 1013.65 2.35zM8 14A6 6 0 118 2a6 6 0 010 12z"
        />
        <path d="M8 4v4l3 1.5" />
      </svg>
    </button>
  </div>

  <div class="panel-content">
    {#if loading && commits.length === 0}
      <div class="loading-state">
        <p>Loading commits...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <span class="error-icon">⚠</span>
        <p>{error}</p>
      </div>
    {:else if commits.length === 0}
      <div class="empty-state">
        <span class="empty-icon">📝</span>
        <p>No commits yet</p>
        <p class="hint">Make your first commit to see history</p>
      </div>
    {:else}
      <div class="commits-list">
        {#each commits as commit}
          <button
            class="commit-item"
            class:selected={selectedCommit?.id === commit.id}
            type="button"
            on:click={() => selectCommit(commit)}
          >
            <div class="commit-header">
              <span class="commit-id">{getCommitShortId(commit.id)}</span>
              <span class="commit-author">{commit.author}</span>
              <span class="commit-date">{formatDate(commit.timestamp)}</span>
            </div>
            <div class="commit-message">{getFirstLine(commit.message)}</div>
            {#if selectedCommit?.id === commit.id}
              <div class="commit-details">
                <div class="detail-row">
                  <span class="label">Author:</span>
                  <span class="value">{commit.author} &lt;{commit.email}&gt;</span>
                </div>
                <div class="detail-row">
                  <span class="label">Date:</span>
                  <span class="value">{new Date(commit.timestamp * 1000).toLocaleString()}</span>
                </div>
                <div class="detail-row">
                  <span class="label">SHA:</span>
                  <span class="value commit-sha">{commit.id}</span>
                </div>
                {#if commit.parent_ids.length > 0}
                  <div class="detail-row">
                    <span class="label">Parents:</span>
                    <span class="value"
                      >{commit.parent_ids.map((id) => getCommitShortId(id)).join(', ')}</span
                    >
                  </div>
                {/if}
                <div class="commit-full-message">
                  {commit.message}
                </div>
              </div>
            {/if}
          </button>
        {/each}
      </div>

      {#if commits.length >= limit}
        <div class="load-more">
          <button class="load-more-btn" on:click={loadMore} disabled={loading}>
            {loading ? 'Loading...' : 'Load More'}
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .git-history-panel {
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-header h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.5px;
  }

  .refresh-btn {
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .refresh-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--color-text-secondary);
    text-align: center;
    padding: 20px;
  }

  .error-icon,
  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .hint {
    font-size: 11px;
    opacity: 0.7;
    margin-top: 4px;
  }

  .commits-list {
    padding: 8px 0;
  }

  .commit-item {
    width: 100%;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    border-left: none;
    border-right: none;
    border-top: none;
    background: transparent;
    cursor: pointer;
    transition: background 0.1s;
    text-align: left;
    color: var(--color-text);
  }

  .commit-item:hover {
    background: var(--color-bg-tertiary);
  }

  .commit-item.selected {
    background: var(--color-bg-tertiary);
  }

  .commit-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    margin-bottom: 4px;
  }

  .commit-id {
    font-family: 'Fira Code', 'Consolas', monospace;
    color: var(--color-accent);
    font-weight: 600;
  }

  .commit-author {
    color: var(--color-text-secondary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .commit-date {
    color: var(--color-text-secondary);
    font-size: 10px;
  }

  .commit-message {
    font-size: 13px;
    color: var(--color-text);
    line-height: 1.4;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .commit-details {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
    font-size: 12px;
  }

  .detail-row {
    display: flex;
    gap: 8px;
    margin-bottom: 6px;
  }

  .detail-row .label {
    color: var(--color-text-secondary);
    min-width: 60px;
    font-weight: 600;
  }

  .detail-row .value {
    color: var(--color-text);
    flex: 1;
    word-break: break-all;
  }

  .commit-sha {
    font-family: 'Fira Code', 'Consolas', monospace;
    font-size: 11px;
  }

  .commit-full-message {
    margin-top: 12px;
    padding: 8px;
    background: var(--color-bg);
    border-radius: 3px;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    color: var(--color-text);
  }

  .load-more {
    padding: 16px;
    display: flex;
    justify-content: center;
  }

  .load-more-btn {
    padding: 8px 16px;
    background: var(--color-accent);
    color: var(--color-text-on-accent);
    border: none;
    border-radius: 3px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.1s;
  }

  .load-more-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .load-more-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
