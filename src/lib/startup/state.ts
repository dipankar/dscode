/**
 * STATE MACHINE: StartupPhase
 *
 * Tracks the entire application startup sequence from HTML render through
 * full interactivity. This is the single source of truth for startup state.
 *
 * State Diagram:
 *
 *   Idle ──► Booting ──► Mounted ──► ShellReady ──► SessionConnecting
 *     │        │          │            │                │
 *     │        │          │            │                ├──► SessionReady
 *     │        │          │            │                │      │
 *     │        │          │            │                │      ▼
 *     │        │          │            │                │  ExtensionsLoading
 *     │        │          │            │                │      │
 *     │        │          │            │                │      ▼
 *     │        │          │            │                │     Ready
 *     │        │          │            │                │
 *     │        │          │            │                └──► SessionInitializing
 *     │        │          │            │                       │
 *     │        │          │            │                       ▼
 *     │        │          │            │                   SessionReady
 *     │        │          │            │
 *     └────────┴──────────┴────────────┴───────────────────────┘
 *                             │
 *                             ▼
 *                           Error ──► SessionConnecting (retry)
 *
 * Transitions:
 *   Idle -> Booting            (main.ts starts executing)
 *   Booting -> Mounted          (Svelte app mounted to #app)
 *   Booting -> Error            (module-level import failure)
 *   Mounted -> ShellReady        (AppShellController initialized, layout painted)
 *   Mounted -> Error             (shell init failure)
 *   ShellReady -> SessionConnecting  (begin session initialization)
 *   SessionConnecting -> SessionReady      (backend already ready)
 *   SessionConnecting -> SessionInitializing (backend needs init)
 *   SessionConnecting -> Error             (backend unreachable)
 *   SessionInitializing -> SessionReady    (backend init succeeded)
 *   SessionInitializing -> Error          (backend init failed / timeout)
 *   SessionReady -> ExtensionsLoading      (extensions scanning/loading)
 *   SessionReady -> Ready                   (no extensions to load)
 *   ExtensionsLoading -> Ready               (all extensions loaded)
 *   ExtensionsLoading -> Error             (extension load failure)
 *   Ready -> SessionConnecting              (reconnect after error)
 *   Error -> SessionConnecting              (retry initialization)
 *
 * Concurrency:
 *   Svelte store updates are synchronous. Async backend calls (session init,
 *   extension loading) happen between state transitions. The store itself
 *   is single-threaded via Svelte's reactive model.
 *
 * Splash Visibility:
 *   The splash screen is visible during: Idle, Booting, Mounted, ShellReady,
 *   SessionConnecting, SessionInitializing.
 *   The splash is hidden during: SessionReady, ExtensionsLoading, Ready, Error.
 *
 * Interruption Table:
 * ┌─────────────────────┬─────────────────────────────────────────────────────┐
 * │ State               │ What happens on crash / reload                      │
 * ├─────────────────────┼─────────────────────────────────────────────────────┤
 * │ Idle                │ Safe. No JS executing yet.                          │
 * │ Booting             │ If module import fails -> Error. Splash may stick.│
 * │                     │ 3s inline fallback in index.html handles this.      │
 * │ Mounted             │ App mounted but shell not ready. Reload safe.       │
 * │ ShellReady          │ Layout visible. Session not started. Reload safe.   │
 * │ SessionConnecting   │ Waiting for backend. If backend dead -> Error.    │
 * │ SessionInitializing │ Backend init in progress. 30s timeout -> Error.   │
 * │ SessionReady        │ Core ready. Extensions may still load. Safe.        │
 * │ ExtensionsLoading   │ Extensions loading. Safe to reload.                │
 * │ Ready               │ Fully operational. Safe.                           │
 * │ Error               │ Error state shown. User can retry.                   │
 * └─────────────────────┴─────────────────────────────────────────────────────┘
 */

export type StartupPhase =
  | 'Idle'
  | 'Booting'
  | 'Mounted'
  | 'ShellReady'
  | 'SessionConnecting'
  | 'SessionInitializing'
  | 'SessionReady'
  | 'ExtensionsLoading'
  | 'Ready'
  | 'Error';

/** Valid transitions from each state. */
export const STARTUP_TRANSITIONS: Record<StartupPhase, StartupPhase[]> = {
  Idle: ['Booting'],
  Booting: ['Mounted', 'Error'],
  Mounted: ['ShellReady', 'Error'],
  ShellReady: ['SessionConnecting'],
  SessionConnecting: ['SessionInitializing', 'SessionReady', 'Error'],
  SessionInitializing: ['SessionReady', 'Error'],
  SessionReady: ['ExtensionsLoading', 'Ready'],
  ExtensionsLoading: ['Ready', 'Error'],
  Ready: ['SessionConnecting'],
  Error: ['SessionConnecting'],
};

/**
 * Validates whether a transition from `from` to `to` is allowed.
 * Invalid transitions are rejected (logged as a warning) to prevent
 * race conditions where an async callback arrives after the state
 * has already moved forward.
 */
export function canTransitionTo(from: StartupPhase, to: StartupPhase): boolean {
  return STARTUP_TRANSITIONS[from].includes(to);
}

/**
 * Returns true if the splash screen should be visible in this phase.
 * The splash is shown while the app framework or core services are
 * still initializing. It is hidden as soon as the backend session
 * is ready, even if extensions are still loading in the background.
 */
export function isSplashVisible(phase: StartupPhase): boolean {
  return (
    phase === 'Idle' ||
    phase === 'Booting' ||
    phase === 'Mounted' ||
    phase === 'ShellReady' ||
    phase === 'SessionConnecting' ||
    phase === 'SessionInitializing'
  );
}

/**
 * Returns true if the phase represents a state where the app UI
 * should be visible underneath any loading indicators.
 */
export function isUiVisible(phase: StartupPhase): boolean {
  return (
    phase === 'Mounted' ||
    phase === 'ShellReady' ||
    phase === 'SessionConnecting' ||
    phase === 'SessionInitializing' ||
    phase === 'SessionReady' ||
    phase === 'ExtensionsLoading' ||
    phase === 'Ready' ||
    phase === 'Error'
  );
}

/**
 * Returns true if the app is fully ready for user interaction.
 */
export function isReady(phase: StartupPhase): boolean {
  return phase === 'Ready';
}

/**
 * Returns true if the startup sequence is in an error state
 * and requires user action (retry) to proceed.
 */
export function isError(phase: StartupPhase): boolean {
  return phase === 'Error';
}

/**
 * Returns true if the session is actively being initialized
 * (connecting or initializing). Useful for showing a
 * non-blocking loading indicator in the UI.
 */
export function isSessionLoading(phase: StartupPhase): boolean {
  return phase === 'SessionConnecting' || phase === 'SessionInitializing';
}
