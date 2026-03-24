<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let direction: 'horizontal' | 'vertical' = 'horizontal';

  const dispatch = createEventDispatcher();

  let isResizing = false;
  let startPos = 0;

  function handleMouseDown(e: MouseEvent) {
    isResizing = true;
    startPos = direction === 'horizontal' ? e.clientX : e.clientY;

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;

    const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
    const delta = currentPos - startPos;

    dispatch('resize', { delta });
    startPos = currentPos;
  }

  function handleMouseUp() {
    isResizing = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }

  function handleKeyDown(e: KeyboardEvent) {
    const step = e.shiftKey ? 24 : 8;
    const isHorizontal = direction === 'horizontal';

    if ((isHorizontal && e.key === 'ArrowLeft') || (!isHorizontal && e.key === 'ArrowUp')) {
      e.preventDefault();
      dispatch('resize', { delta: -step });
    }

    if ((isHorizontal && e.key === 'ArrowRight') || (!isHorizontal && e.key === 'ArrowDown')) {
      e.preventDefault();
      dispatch('resize', { delta: step });
    }
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions a11y-no-noninteractive-tabindex -->
<div
  class="resize-handle"
  class:horizontal={direction === 'horizontal'}
  class:vertical={direction === 'vertical'}
  class:resizing={isResizing}
  on:mousedown={handleMouseDown}
  on:keydown={handleKeyDown}
  role="separator"
  aria-orientation={direction}
  tabindex="0"
  aria-label={direction === 'horizontal' ? 'Resize horizontally' : 'Resize vertically'}
/>

<style>
  .resize-handle {
    position: relative;
    user-select: none;
    border: none;
    padding: 0;
  }

  .resize-handle.horizontal {
    width: 4px;
    height: 100%;
    cursor: ew-resize;
    background-color: transparent;
    transition: background-color 0.2s;
  }

  .resize-handle.vertical {
    width: 100%;
    height: 4px;
    cursor: ns-resize;
    background-color: transparent;
    transition: background-color 0.2s;
  }

  .resize-handle:hover,
  .resize-handle.resizing {
    background-color: var(--color-accent);
  }

  .resize-handle.horizontal::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -2px;
    right: -2px;
  }

  .resize-handle.vertical::before {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    top: -2px;
    bottom: -2px;
  }
</style>
