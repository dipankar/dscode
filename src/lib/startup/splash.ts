import { isSplashVisible, type StartupPhase } from './state';

/**
 * Splash Screen Visibility Logic
 *
 * Pure functions that decide when to show / hide the splash screen
 * based on the current startup phase.
 *
 * The splash is a static HTML element in index.html. It is hidden
 * by adding a CSS class that triggers an opacity transition, then
 * removing it from the DOM after the transition completes.
 */

const HIDDEN_CLASS = 'loading-screen--hidden';

/**
 * Update the splash screen visibility to match the current startup phase.
 *
 * @param phase — Current startup phase from the startup store.
 */
export function updateSplashVisibility(phase: StartupPhase) {
  const el = document.querySelector('.loading-screen');
  if (!el) return;

  const shouldShow = isSplashVisible(phase);
  const isHidden = el.classList.contains(HIDDEN_CLASS);

  if (!shouldShow && !isHidden) {
    // Hide the splash screen
    el.classList.add(HIDDEN_CLASS);

    // Remove from DOM after the CSS transition completes (700ms)
    setTimeout(() => {
      if (el.parentNode) {
        el.remove();
      }
    }, 700);
  }

  // If shouldShow && isHidden, we don't re-show it — hiding is one-way.
}

/**
 * Force-hide the splash screen immediately, bypassing the transition.
 * Used as an emergency fallback when the startup controller may not
 * have loaded.
 */
export function forceHideSplash() {
  const el = document.querySelector('.loading-screen');
  if (!el) return;
  el.classList.add(HIDDEN_CLASS);
  setTimeout(() => {
    if (el.parentNode) {
      el.remove();
    }
  }, 700);
}
