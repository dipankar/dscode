import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { startupPhase } from '../../stores/startup';
import { sessionCommands } from '../contracts/commands';
import { TauriEventName } from '../contracts/events';

/**
 * Startup Controller
 *
 * Drives the startup state machine by sequencing the phases:
 *   Booting -> Mounted -> ShellReady -> SessionConnecting -> ... -> Ready
 *
 * This is the only place that calls `startupPhase.transitionTo()`.
 * All other code should call methods on this controller rather than
 * transitioning the store directly.
 */

let sessionReadyUnlisten: UnlistenFn | null = null;

async function waitForSessionReadyEvent(): Promise<void> {
  if (sessionReadyUnlisten) {
    sessionReadyUnlisten();
    sessionReadyUnlisten = null;
  }

  let resolvePromise: (() => void) | null = null;

  function doResolve() {
    if (resolvePromise) {
      resolvePromise();
      resolvePromise = null;
    }
    if (sessionReadyUnlisten) {
      sessionReadyUnlisten();
      sessionReadyUnlisten = null;
    }
  }

  sessionReadyUnlisten = await listen(TauriEventName.session, (event: any) => {
    const { type, data } = event.payload;
    if (type === 'StateChanged') {
      const lifecycle = data.state?.lifecycle;
      if (lifecycle === 'Ready') {
        doResolve();
      }
    }
  });

  // Safety timeout: if no StateChanged event arrives in 35s, resolve anyway
  // to prevent the startup from hanging forever.
  return new Promise((resolve) => {
    resolvePromise = resolve;
    setTimeout(() => {
      if (resolvePromise) {
        console.warn('[Startup] Timeout waiting for StateChanged event; proceeding to SessionReady');
        doResolve();
      }
    }, 35000);
  });
}

export const startupController = {
  /** Mark that main.ts has started executing. */
  markBooting() {
    startupPhase.transitionTo('Booting');
  },

  /** Mark that the Svelte app has been mounted to the DOM. */
  markMounted() {
    startupPhase.transitionTo('Mounted');
  },

  /** Mark that the AppShellController has finished initializing the layout. */
  markShellReady() {
    startupPhase.transitionTo('ShellReady');
  },

  /** Mark a fatal error during boot. */
  markError(message: string) {
    startupPhase.setError(message);
    startupPhase.transitionTo('Error');
  },

  /**
   * Initialize the backend session.
   *
   * Sequence:
   *   1. Query backend lifecycle.
   *   2. If already Ready, transition directly to SessionReady.
   *   3. If Initializing, wait for StateChanged event.
   *   4. If Uninitialized / Error, call initialize_session.
   *   5. Load extensions.
   */
  async initializeSession() {
    startupPhase.transitionTo('SessionConnecting');

    try {
      const lifecycle = await sessionCommands.getLifecycle();

      if (lifecycle === 'Ready') {
        startupPhase.transitionTo('SessionReady');
        await this.loadExtensions();
        return;
      }

      if (lifecycle === 'Initializing') {
        startupPhase.transitionTo('SessionInitializing');
        await waitForSessionReadyEvent();
        startupPhase.transitionTo('SessionReady');
        await this.loadExtensions();
        return;
      }

      // Uninitialized, Error, or unknown — trigger initialization
      startupPhase.transitionTo('SessionInitializing');
      await sessionCommands.initialize();
      startupPhase.transitionTo('SessionReady');
      await this.loadExtensions();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('[Startup] Session initialization failed:', message);
      startupPhase.setError(message);
      startupPhase.transitionTo('Error');
    }
  },

  /**
   * Load extensions in the background.
   *
   * The backend already handles extension scanning and auto-loading.
   * The frontend just transitions through ExtensionsLoading and
   * listens for StateChanged events to know when everything is up.
   */
  async loadExtensions() {
    startupPhase.transitionTo('ExtensionsLoading');

    // Give the backend a moment to finish extension scanning
    // before declaring Ready. In the future this should be gated
    // on a specific event from the backend.
    await new Promise((resolve) => setTimeout(resolve, 500));

    startupPhase.transitionTo('Ready');
  },

  /**
   * Retry initialization after an error.
   */
  retry() {
    startupPhase.setError(null);
    startupPhase.transitionTo('SessionConnecting');
    this.initializeSession();
  },
};
