/**
 * Startup State Machine
 *
 * Unified startup state management for DSCode.
 *
 * Replaces the scattered splash-screen logic across:
 *   - index.html
 *   - src/main.ts
 *   - src/App.svelte
 *   - src/stores/session.ts
 *   - src/lib/app-shell/controller.ts
 *
 * All startup decisions are derived from a single `StartupPhase` state machine
 * exposed via the Svelte store in `src/stores/startup.ts`.
 *
 * Usage:
 *   import { startupPhase } from './stores/startup';
 *   import { startupController } from './startup/controller';
 *   import { updateSplashVisibility } from './startup/splash';
 *
 *   $: updateSplashVisibility($startupPhase);
 *
 *   onMount(() => {
 *     startupController.markShellReady();
 *     startupController.initializeSession();
 *   });
 */

export { startupPhase } from '../../stores/startup';
export { startupController } from './controller';
export { updateSplashVisibility, forceHideSplash } from './splash';
export {
  type StartupPhase,
  canTransitionTo,
  isSplashVisible,
  isUiVisible,
  isReady,
  isError,
  isSessionLoading,
  STARTUP_TRANSITIONS,
} from './state';
