<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { activeActivity, type ActivityId } from '../lib/activity-store';
  import { registryCommands } from '../lib/contracts/commands';
  import { WindowEventName, dispatchWindowEvent } from '../lib/contracts/events';

  export let sidebarVisible: boolean;

  interface ActivityBarItem {
    id: string;
    owner: string;
    title: string;
    icon?: string;
    icon_path?: string;
    priority: number;
    badge_count?: number;
    badge_text?: string;
    visible: boolean;
  }

  let activities: ActivityBarItem[] = [];
  let currentActivity: ActivityId = 'explorer';
  let unlistenActivityBarChanged: (() => void) | null = null;

  const unsubscribeActiveActivity = activeActivity.subscribe((activity) => {
    currentActivity = activity;
  });

  async function loadActivityBarItems() {
    try {
      const items = await registryCommands.getActivityBarItems<ActivityBarItem>();
      activities = items;
      console.log('[ActivityBar] Loaded items:', items.length);
    } catch (error) {
      console.error('[ActivityBar] Failed to load items:', error);
    }
  }

  onMount(async () => {
    // Load initial activity bar items
    await loadActivityBarItems();

    // Listen for activity bar changes
    unlistenActivityBarChanged = await listen<ActivityBarItem[]>(
      'activity-bar-items-changed',
      (event) => {
        activities = event.payload;
        console.log('[ActivityBar] Items updated:', event.payload.length);
      }
    );
  });

  onDestroy(() => {
    unsubscribeActiveActivity();
    if (unlistenActivityBarChanged) {
      unlistenActivityBarChanged();
    }
  });

  function handleClick(id: string) {
    // Extensions opens as modal/overlay
    if (id === 'extensions') {
      dispatchWindowEvent(WindowEventName.openExtensions);
      return;
    }

    if (currentActivity === id) {
      sidebarVisible = !sidebarVisible;
    } else {
      activeActivity.set(id as ActivityId);
      sidebarVisible = true;
    }
  }

  function handleSettingsClick() {
    dispatchWindowEvent(WindowEventName.openSettings);
  }
</script>

<div class="activity-bar" role="navigation" aria-label="Activity bar">
  {#each activities as activity}
    <button
      class="activity-item"
      class:active={currentActivity === activity.id}
      on:click={() => handleClick(activity.id)}
      title={activity.title}
      aria-label={activity.title}
      aria-pressed={currentActivity === activity.id}
    >
      <span class="icon">{activity.icon || '📦'}</span>
      {#if activity.badge_count || activity.badge_text}
        <span class="badge">
          {activity.badge_text || activity.badge_count}
        </span>
      {/if}
    </button>
  {/each}

  <div class="spacer"></div>

  <button
    class="activity-item"
    title="Settings"
    on:click={handleSettingsClick}
    aria-label="Settings"
  >
    <span class="icon">⚙️</span>
  </button>
</div>

<style>
  .activity-bar {
    width: 48px;
    background-color: var(--activity-bar-bg);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 4px 0;
  }

  .activity-item {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    position: relative;
  }

  .activity-item:hover {
    background-color: var(--activity-bar-item-hover);
  }

  .activity-item.active {
    color: var(--color-text);
  }

  .activity-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 2px;
    background-color: var(--color-accent);
  }

  .icon {
    font-size: 24px;
  }

  .badge {
    position: absolute;
    top: 8px;
    right: 8px;
    background-color: var(--color-accent);
    color: white;
    font-size: 10px;
    font-weight: bold;
    min-width: 16px;
    height: 16px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .spacer {
    flex: 1;
  }
</style>
