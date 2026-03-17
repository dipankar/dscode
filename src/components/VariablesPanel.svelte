<script lang="ts">
  import { variables, scopes } from '../lib/stores';
  import type { Variable, Scope } from '../lib/types';

  let currentScopes: Scope[] = [];
  let currentVariables: Variable[] = [];
  let expandedItems = new Set<string>();

  scopes.subscribe((s) => {
    currentScopes = s;
  });

  variables.subscribe((v) => {
    currentVariables = v;
  });

  function toggleExpand(itemId: string) {
    if (expandedItems.has(itemId)) {
      expandedItems.delete(itemId);
    } else {
      expandedItems.add(itemId);
    }
    expandedItems = expandedItems; // Trigger reactivity
  }

  function getVariableIcon(variable: Variable): string {
    const type = variable.type?.toLowerCase() || '';
    if (type.includes('string')) return '📝';
    if (type.includes('number') || type.includes('int') || type.includes('float')) return '🔢';
    if (type.includes('bool')) return '✓';
    if (type.includes('array') || type.includes('[]')) return '📋';
    if (type.includes('object') || type.includes('{}')) return '📦';
    if (type.includes('function')) return '⚡';
    return '●';
  }
</script>

<div class="variables-panel">
  <div class="header">
    <span class="title">Variables</span>
  </div>

  <div class="content">
    {#if currentScopes.length === 0 && currentVariables.length === 0}
      <div class="empty-state">
        <span class="empty-icon">🔍</span>
        <p>No variables to display</p>
        <p class="hint">Start debugging to view variables</p>
      </div>
    {:else}
      {#each currentScopes as scope}
        <div class="scope-section">
          <button
            class="scope-header"
            on:click={() => toggleExpand(`scope-${scope.name}`)}
          >
            <span class="expand-icon" class:expanded={expandedItems.has(`scope-${scope.name}`)}>
              ▶
            </span>
            <span class="scope-name">{scope.name}</span>
          </button>

          {#if expandedItems.has(`scope-${scope.name}`)}
            <div class="scope-content">
              {#each currentVariables as variable}
                <div class="variable-item">
                  <div class="variable-row">
                    {#if variable.variablesReference > 0}
                      <button
                        class="expand-icon"
                        class:expanded={expandedItems.has(`var-${variable.name}`)}
                        on:click={() => toggleExpand(`var-${variable.name}`)}
                      >
                        ▶
                      </button>
                    {:else}
                      <span class="no-expand"></span>
                    {/if}
                    <span class="variable-icon">{getVariableIcon(variable)}</span>
                    <span class="variable-name">{variable.name}</span>
                    <span class="variable-separator">:</span>
                    <span class="variable-value">{variable.value}</span>
                    {#if variable.type}
                      <span class="variable-type">{variable.type}</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}

      {#if currentScopes.length === 0}
        {#each currentVariables as variable}
          <div class="variable-item">
            <div class="variable-row">
              {#if variable.variablesReference > 0}
                <button
                  class="expand-icon"
                  class:expanded={expandedItems.has(`var-${variable.name}`)}
                  on:click={() => toggleExpand(`var-${variable.name}`)}
                >
                  ▶
                </button>
              {:else}
                <span class="no-expand"></span>
              {/if}
              <span class="variable-icon">{getVariableIcon(variable)}</span>
              <span class="variable-name">{variable.name}</span>
              <span class="variable-separator">:</span>
              <span class="variable-value">{variable.value}</span>
              {#if variable.type}
                <span class="variable-type">{variable.type}</span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    {/if}
  </div>
</div>

<style>
  .variables-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .header {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    padding: 20px;
    text-align: center;
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 4px 0;
    font-size: 13px;
  }

  .hint {
    font-size: 11px;
    opacity: 0.7;
  }

  .scope-section {
    margin-bottom: 4px;
  }

  .scope-header {
    width: 100%;
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 600;
    text-align: left;
  }

  .scope-header:hover {
    background: var(--bg-hover);
  }

  .scope-name {
    color: var(--accent-color);
  }

  .scope-content {
    padding-left: 16px;
  }

  .variable-item {
    padding: 2px 8px;
  }

  .variable-item:hover {
    background: var(--bg-hover);
  }

  .variable-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-family: 'Fira Code', 'Consolas', monospace;
  }

  .expand-icon {
    width: 14px;
    height: 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    transition: transform 0.15s;
    font-size: 10px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .expand-icon.expanded {
    transform: rotate(90deg);
  }

  .no-expand {
    width: 14px;
    display: inline-block;
  }

  .variable-icon {
    font-size: 11px;
  }

  .variable-name {
    color: var(--text-primary);
    font-weight: 500;
  }

  .variable-separator {
    color: var(--text-tertiary);
  }

  .variable-value {
    color: #ce9178;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .variable-type {
    color: var(--text-tertiary);
    font-size: 10px;
    font-style: italic;
    margin-left: auto;
  }
</style>
