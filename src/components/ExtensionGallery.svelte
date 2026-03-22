<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Download, Search, Trash2, Star, Users } from 'lucide-svelte';

  export let visible: boolean = false;
  export let onClose: () => void;

  interface MarketplaceExtension {
    publisher: string;
    name: string;
    display_name: string;
    version: string;
    description: string | null;
    install_count: number;
    rating: number;
    rating_count: number;
    icon_url: string | null;
  }

  interface InstalledExtension {
    id: string;
    name: string;
    version: string;
    publisher: string;
    description: string | null;
    path: string;
  }

  let activeTab: 'marketplace' | 'installed' = 'marketplace';
  let searchQuery = '';
  let marketplaceExtensions: MarketplaceExtension[] = [];
  let installedExtensions: InstalledExtension[] = [];
  let loading = false;
  let error: string | null = null;
  let installingExt: string | null = null;

  onMount(async () => {
    await loadInstalledExtensions();
  });

  async function searchMarketplace() {
    if (!searchQuery.trim()) return;

    loading = true;
    error = null;

    try {
      marketplaceExtensions = await invoke('search_marketplace', {
        query: searchQuery,
        page: 1,
        pageSize: 20
      });
    } catch (e) {
      error = `Search failed: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function loadInstalledExtensions() {
    // Retry up to 3 times with delay to allow extension host to connect
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        installedExtensions = await invoke('list_extensions');
        return; // Success
      } catch (e) {
        const errorStr = String(e);
        if (errorStr.includes('not connected') && attempt < 2) {
          // Extension host not ready yet, wait and retry
          console.log(`[ExtensionGallery] Extension host not ready, retrying in ${(attempt + 1) * 500}ms...`);
          await new Promise(resolve => setTimeout(resolve, (attempt + 1) * 500));
          continue;
        }
        // Final attempt failed or different error
        console.warn('Failed to load installed extensions:', e);
        return; // Exit gracefully
      }
    }
  }

  async function installExtension(ext: MarketplaceExtension) {
    const extId = `${ext.publisher}.${ext.name}`;
    installingExt = extId;

    try {
      await invoke('install_from_marketplace', {
        publisher: ext.publisher,
        name: ext.name,
        version: ext.version
      });

      await loadInstalledExtensions();
      error = null;
    } catch (e) {
      error = `Install failed: ${e}`;
      console.error(error);
    } finally {
      installingExt = null;
    }
  }

  async function uninstallExtension(extensionId: string) {
    try {
      await invoke('uninstall_extension', { extension_id: extensionId });
      await loadInstalledExtensions();
      error = null;
    } catch (e) {
      error = `Uninstall failed: ${e}`;
      console.error(error);
    }
  }

  function isInstalled(ext: MarketplaceExtension): boolean {
    const extId = `${ext.publisher}.${ext.name}`;
    return installedExtensions.some(installed => installed.id === extId);
  }

  function formatNumber(num: number): string {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      searchMarketplace();
    }
  }
</script>

{#if visible}
  <div class="gallery-overlay" on:click={onClose}>
    <div class="extension-gallery" on:click|stopPropagation>
      <!-- Header -->
      <div class="gallery-header">
    <h2>Extensions</h2>
    <div class="tabs">
      <button
        class:active={activeTab === 'marketplace'}
        on:click={() => activeTab = 'marketplace'}
      >
        Marketplace
      </button>
      <button
        class:active={activeTab === 'installed'}
        on:click={() => activeTab = 'installed'}
      >
        Installed ({installedExtensions.length})
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  <!-- Marketplace Tab -->
  {#if activeTab === 'marketplace'}
    <div class="search-box">
      <Search size={16} />
      <input
        type="text"
        placeholder="Search extensions..."
        bind:value={searchQuery}
        on:keypress={handleKeyPress}
      />
      <button on:click={searchMarketplace} disabled={loading}>
        {loading ? 'Searching...' : 'Search'}
      </button>
    </div>

    <div class="extensions-grid">
      {#each marketplaceExtensions as ext (ext.publisher + '.' + ext.name)}
        <div class="extension-card">
          {#if ext.icon_url}
            <img src={ext.icon_url} alt={ext.display_name} class="ext-icon" />
          {:else}
            <div class="ext-icon-placeholder">{ext.display_name[0]}</div>
          {/if}

          <div class="ext-info">
            <h3>{ext.display_name}</h3>
            <p class="ext-id">{ext.publisher}.{ext.name}</p>
            <p class="ext-description">{ext.description || 'No description'}</p>

            <div class="ext-meta">
              <span class="meta-item">
                <Star size={14} />
                {ext.rating.toFixed(1)} ({formatNumber(ext.rating_count)})
              </span>
              <span class="meta-item">
                <Users size={14} />
                {formatNumber(ext.install_count)} installs
              </span>
            </div>

            <div class="ext-actions">
              {#if isInstalled(ext)}
                <button class="btn-installed" disabled>Installed</button>
              {:else if installingExt === `${ext.publisher}.${ext.name}`}
                <button class="btn-installing" disabled>Installing...</button>
              {:else}
                <button class="btn-install" on:click={() => installExtension(ext)}>
                  <Download size={16} />
                  Install
                </button>
              {/if}
              <span class="ext-version">v{ext.version}</span>
            </div>
          </div>
        </div>
      {/each}

      {#if !loading && marketplaceExtensions.length === 0 && searchQuery}
        <div class="empty-state">
          <p>No extensions found. Try a different search.</p>
        </div>
      {/if}

      {#if !searchQuery}
        <div class="empty-state">
          <Search size={48} />
          <p>Search for extensions from the VS Code Marketplace</p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Installed Tab -->
  {#if activeTab === 'installed'}
    <div class="extensions-list">
      {#each installedExtensions as ext (ext.id)}
        <div class="installed-extension">
          <div class="ext-info">
            <h3>{ext.name}</h3>
            <p class="ext-id">{ext.id}</p>
            <p class="ext-description">{ext.description || 'No description'}</p>
            <span class="ext-version">v{ext.version}</span>
          </div>

          <button class="btn-uninstall" on:click={() => uninstallExtension(ext.id)}>
            <Trash2 size={16} />
            Uninstall
          </button>
        </div>
      {/each}

      {#if installedExtensions.length === 0}
        <div class="empty-state">
          <p>No extensions installed yet.</p>
          <p>Browse the marketplace to install extensions.</p>
        </div>
      {/if}
    </div>
  {/if}
    </div>
  </div>
{/if}

<style>
  .gallery-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--modal-overlay-bg);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }


  .extension-gallery {
    display: flex;
    flex-direction: column;
    width: 90vw;
    max-width: 1200px;
    height: 85vh;
    max-height: 800px;
    background: var(--modal-bg);
    color: var(--color-text);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .gallery-header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .tabs {
    display: flex;
    gap: 8px;
  }

  .tabs button {
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
  }

  .tabs button.active {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .error-banner {
    background: #f443361a;
    color: #f44336;
    padding: 12px 16px;
    border-bottom: 1px solid #f44336;
  }

  .search-box {
    padding: 16px;
    display: flex;
    gap: 8px;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
  }

  .search-box input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 14px;
  }

  .search-box button {
    padding: 8px 16px;
    background: var(--accent-color);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    font-size: 14px;
  }

  .search-box button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .extensions-grid {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
    align-content: start;
  }

  .extension-card {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    display: flex;
    gap: 12px;
  }

  .ext-icon {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    flex-shrink: 0;
  }

  .ext-icon-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    background: var(--accent-color);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    font-weight: bold;
    color: white;
    flex-shrink: 0;
  }

  .ext-info {
    flex: 1;
    min-width: 0;
  }

  .ext-info h3 {
    margin: 0 0 4px 0;
    font-size: 15px;
    font-weight: 600;
  }

  .ext-id {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .ext-description {
    margin: 0 0 12px 0;
    font-size: 13px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .ext-meta {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
  }

  .meta-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .ext-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-install, .btn-uninstall {
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
  }

  .btn-install {
    background: var(--accent-color);
    color: white;
  }

  .btn-installed {
    padding: 6px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: not-allowed;
  }

  .btn-installing {
    padding: 6px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .btn-uninstall {
    background: #f443361a;
    color: #f44336;
  }

  .ext-version {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .extensions-list {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .installed-extension {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .empty-state {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: var(--text-secondary);
    text-align: center;
    gap: 16px;
  }

  .empty-state p {
    margin: 0;
  }
</style>
