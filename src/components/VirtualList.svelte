<script lang="ts" context="module">
  export interface VirtualListOptions {
    itemHeight: number;
    overscan?: number;
  }
</script>

<script lang="ts">
  import { onMount } from 'svelte';

  export let items: any[] = [];
  export let options: VirtualListOptions = { itemHeight: 24, overscan: 5 };

  let containerEl: HTMLElement;
  let scrollTop = 0;
  let containerHeight = 0;
  let rafId: number | null = null;

  $: totalHeight = items.length * options.itemHeight;
  $: startIndex = Math.max(0, Math.floor(scrollTop / options.itemHeight) - (options.overscan ?? 5));
  $: visibleCount = Math.ceil(containerHeight / options.itemHeight) + (options.overscan ?? 5) * 2;
  $: endIndex = Math.min(items.length, startIndex + visibleCount);
  $: visibleItems = items.slice(startIndex, endIndex);
  $: offsetY = startIndex * options.itemHeight;

  function handleScroll() {
    if (rafId !== null) return;
    rafId = requestAnimationFrame(() => {
      if (containerEl) {
        scrollTop = containerEl.scrollTop;
      }
      rafId = null;
    });
  }

  onMount(() => {
    if (containerEl) {
      containerHeight = containerEl.clientHeight;
    }

    const resizeObserver = new ResizeObserver(() => {
      if (containerEl) {
        containerHeight = containerEl.clientHeight;
      }
    });

    resizeObserver.observe(containerEl);

    return () => {
      resizeObserver.disconnect();
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
    };
  });
</script>

<div bind:this={containerEl} class="virtual-list-container" on:scroll={handleScroll}>
  <div class="virtual-list-spacer" style="height: {totalHeight}px;">
    <div class="virtual-list-content" style="transform: translateY({offsetY}px);">
      {#each visibleItems as item, i (startIndex + i)}
        <slot {item} index={startIndex + i} />
      {/each}
    </div>
  </div>
</div>

<style>
  .virtual-list-container {
    height: 100%;
    overflow-y: auto;
    position: relative;
  }

  .virtual-list-spacer {
    position: relative;
  }

  .virtual-list-content {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }
</style>
