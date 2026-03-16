<script lang="ts">
  export let sidebarVisible: boolean;

  const activities = [
    { id: 'explorer', icon: '📁', title: 'Explorer' },
    { id: 'search', icon: '🔍', title: 'Search' },
    { id: 'scm', icon: '🔀', title: 'Source Control' },
    { id: 'debug', icon: '🐛', title: 'Run and Debug' },
    { id: 'extensions', icon: '🧩', title: 'Extensions' },
  ];

  let activeItem = 'explorer';

  function handleClick(id: string) {
    if (activeItem === id) {
      sidebarVisible = !sidebarVisible;
    } else {
      activeItem = id;
      sidebarVisible = true;
    }
  }
</script>

<div class="activity-bar">
  {#each activities as activity}
    <button
      class="activity-item"
      class:active={activeItem === activity.id}
      on:click={() => handleClick(activity.id)}
      title={activity.title}
    >
      <span class="icon">{activity.icon}</span>
    </button>
  {/each}

  <div class="spacer"></div>

  <button class="activity-item" title="Settings">
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
