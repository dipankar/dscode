// Focus trap action for Svelte
// Traps focus within an element when applied (use:focusTrap)

export function focusTrap(node: HTMLElement) {
  const focusableSelector = 'a[href], button, input, textarea, select, [tabindex]:not([tabindex="-1"])';

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    const focusable = Array.from(node.querySelectorAll(focusableSelector)) as HTMLElement[];
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (event.shiftKey) {
      if (document.activeElement === first) {
        event.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  }

  // Store previously focused element to restore on destroy
  const previouslyFocused = document.activeElement as HTMLElement;

  node.addEventListener('keydown', handleKeyDown);

  // Focus the first focusable element
  const firstFocusable = node.querySelector(focusableSelector) as HTMLElement;
  if (firstFocusable) {
    firstFocusable.focus();
  }

  return {
    destroy() {
      node.removeEventListener('keydown', handleKeyDown);
      // Restore focus to previously focused element
      if (previouslyFocused) {
        previouslyFocused.focus();
      }
    }
  };
}