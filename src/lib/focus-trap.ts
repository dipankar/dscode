// Focus trap action for Svelte
// Traps focus within an element when applied (use:focusTrap)
// Handles Tab/Shift+Tab cycling and Escape to close

export interface FocusTrapOptions {
  onEscape?: () => void;
}

export function focusTrap(node: HTMLElement, options?: FocusTrapOptions) {
  const focusableSelector =
    'a[href], button, input, textarea, select, [tabindex]:not([tabindex="-1"])';

  let currentOptions = options;

  function getFocusableElements(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(focusableSelector)).filter(
      (el) => el.offsetParent !== null
    );
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Tab') {
      const focusable = getFocusableElements();
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
    } else if (event.key === 'Escape') {
      event.preventDefault();
      currentOptions?.onEscape?.();
    }
  }

  // Store previously focused element to restore on destroy
  const previouslyFocused = document.activeElement as HTMLElement;

  node.addEventListener('keydown', handleKeyDown);

  // Focus the first focusable element
  const firstFocusable = getFocusableElements()[0];
  if (firstFocusable) {
    firstFocusable.focus();
  }

  return {
    update(newOptions?: FocusTrapOptions) {
      currentOptions = newOptions;
    },
    destroy() {
      node.removeEventListener('keydown', handleKeyDown);
      // Restore focus to previously focused element
      if (previouslyFocused) {
        previouslyFocused.focus();
      }
    }
  };
}