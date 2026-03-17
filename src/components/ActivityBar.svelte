<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { activeActivity, type ActivityId } from '../lib/activity-store';

  export let sidebarVisible: boolean;

  interface Activity {
    id: string;
    icon: string;
    title: string;
    isExtension?: boolean;
  }

  const builtInActivities: Activity[] = [
    { id: 'explorer', icon: '📁', title: 'Explorer' },
    { id: 'search', icon: '🔍', title: 'Search' },
    { id: 'scm', icon: '🔀', title: 'Source Control' },
    { id: 'debug', icon: '🐛', title: 'Run and Debug' },
    { id: 'extensions', icon: '🧩', title: 'Extensions' },
  ];

  let activities: Activity[] = builtInActivities;
  let currentActivity: ActivityId = 'explorer';

  activeActivity.subscribe((activity) => {
    currentActivity = activity;
  });

  onMount(async () => {
    try {
      const contributions = await invoke('get_extension_contributions');
      const extensionActivities: Activity[] = [];

      // Parse viewsContainers.activitybar from each extension
      for (const contrib of contributions as any[]) {
        if (contrib.contributes?.viewsContainers?.activitybar) {
          for (const container of contrib.contributes.viewsContainers.activitybar) {
            extensionActivities.push({
              id: container.id,
              icon: container.icon ? '🔷' : '📦', // Use generic icon for now
              title: container.title || contrib.extension_name,
              isExtension: true,
            });
          }
        }
      }

      // Add extension activities before settings (after built-in)
      activities = [...builtInActivities, ...extensionActivities];
      console.log('Loaded extension activities:', extensionActivities);
    } catch (error) {
      console.error('Failed to load extension contributions:', error);
    }
  });

  function handleClick(id: string) {
    // Extensions opens as modal/overlay
    if (id === 'extensions') {
      window.dispatchEvent(new CustomEvent('openExtensions'));
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
    window.dispatchEvent(new CustomEvent('openSettings'));
  }
</script>

<div class="activity-bar">
  {#each activities as activity}
    <button
      class="activity-item"
      class:active={currentActivity === activity.id}
      on:click={() => handleClick(activity.id)}
      title={activity.title}
    >
      <span class="icon">{activity.icon}</span>
    </button>
  {/each}

  <div class="spacer"></div>

  <button class="activity-item" title="Settings" on:click={handleSettingsClick}>
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

  .spacer {
    flex: 1;
  }
</style>
