import { writable } from 'svelte/store';
import { canTransitionTo, isSplashVisible, type StartupPhase } from '../lib/startup/state';

/**
 * Startup Store
 *
 * The single source of truth for the application startup phase.
 * All startup decisions (splash visibility, UI readiness, error display)
 * are derived from this store.
 *
 * Usage:
 *   import { startupPhase, startupError } from './stores/startup';
 *   $startupPhase // 'Idle', 'Booting', 'Mounted', ..., 'Ready', 'Error'
 */

function createStartupStore() {
  const { subscribe, set, update } = writable<StartupPhase>('Idle');
  const errorStore = writable<string | null>(null);

  /**
   * Transition to a new startup phase.
   * Invalid transitions are silently rejected and logged as a warning.
   * This prevents race conditions from async callbacks that arrive
   * after the state has already moved forward.
   */
  function transitionTo(next: StartupPhase) {
    update((current) => {
      if (current === next) {
        return current;
      }
      if (!canTransitionTo(current, next)) {
        console.warn(`[Startup] Invalid transition rejected: ${current} -> ${next}`);
        return current;
      }
      console.log(`[Startup] ${current} -> ${next}`);
      return next;
    });
  }

  /**
   * Set or clear the startup error message.
   */
  function setError(msg: string | null) {
    errorStore.set(msg);
  }

  return {
    subscribe,
    transitionTo,
    setError,
    /** Readable derived store for the current error message. */
    error: { subscribe: errorStore.subscribe },
  };
}

export const startupPhase = createStartupStore();
